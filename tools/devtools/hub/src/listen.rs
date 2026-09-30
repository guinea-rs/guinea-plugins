//! The endpoint, on a thread and a runtime of its own.
//!
//! A session's remote end lives here too: capnp clients belong to the runtime
//! that made them, so a command from anywhere else is queued to this thread
//! and sent from it - each session from a queue of its own, so one that stops
//! answering holds up nobody else.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use guinea_devtools_model::sessions::Incoming;
use guinea_devtools_protocol::devtools_capnp::peer;
use guinea_devtools_protocol::{Command, Report, key, wire};
use ogurpchik::rpc::accept_session;

/// Where commands for the applications go in.
pub type Commands = UnboundedSender<(u64, Command)>;

/// Each connected session's own queue.
type Queues = Rc<RefCell<HashMap<u64, UnboundedSender<Command>>>>;

/// How long a session has to take a command before it is given up on.
const SEND_TIMEOUT: Duration = Duration::from_secs(10);

pub fn spawn(out: Sender<Incoming>) -> Commands {
    let (commands, queued) = unbounded();

    std::thread::Builder::new()
        .name("devtools-listen".into())
        .spawn(move || match compio::runtime::Runtime::new() {
            Ok(runtime) => runtime.block_on(listen(out, queued)),
            Err(error) => {
                let _ = out.send(Incoming::Failed(format!("no runtime: {error}")));
            }
        })
        .expect("spawning the listener thread");

    commands
}

/// One application's side of the conversation.
struct Session {
    id: u64,
    out: Sender<Incoming>,
}

impl peer::Server for Session {
    async fn send(
        self: capnp::capability::Rc<Self>,
        params: peer::SendParams,
        _results: peer::SendResults,
    ) -> Result<(), capnp::Error> {
        let report: Report = wire::received(&params)?;

        self.out
            .send(Incoming::Report(self.id, Box::new(report)))
            .map_err(|_| capnp::Error::disconnected("devtools are closing".into()))
    }
}

/// Hands each queued command to its session's queue.
async fn deliver(queues: Queues, mut queued: UnboundedReceiver<(u64, Command)>) {
    while let Some((id, command)) = queued.next().await {
        let Some(queue) = queues.borrow().get(&id).cloned() else {
            tracing::debug!(id, ?command, "no such session to send to");
            continue;
        };

        if queue.unbounded_send(command).is_err() {
            tracing::debug!(id, "the session closed before its command went");
        }
    }
}

/// Sends one session its commands, one at a time and in order.
async fn send_each(id: u64, remote: peer::Client, mut commands: UnboundedReceiver<Command>) {
    while let Some(command) = commands.next().await {
        match compio::time::timeout(SEND_TIMEOUT, wire::send(&remote, &command)).await {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => tracing::debug!(id, ?error, "a command did not reach its session"),
            Err(_) => tracing::warn!(id, ?command, "a session did not take a command in time"),
        }
    }
}

async fn listen(out: Sender<Incoming>, queued: UnboundedReceiver<(u64, Command)>) {
    let queues = Queues::default();
    compio::runtime::spawn(deliver(queues.clone(), queued)).detach();

    let endpoint = guinea_devtools_protocol::endpoint();
    let listener = match endpoint.listen().await {
        Ok(listener) => listener,
        Err(report) => {
            let _ = out.send(Incoming::Failed(format!("{endpoint}: {report:?}")));
            return;
        }
    };

    let secret = match key::create() {
        Ok(secret) => secret,
        Err(error) => {
            let _ = out.send(Incoming::Failed(format!("cannot write the key: {error}")));
            return;
        }
    };

    let mode = key::handshake(secret);
    let _ = out.send(Incoming::Listening(endpoint.to_string()));

    let mut next = 1;

    loop {
        let id = next;
        next += 1;

        let session = Session {
            id,
            out: out.clone(),
        };
        let accepted = accept_session::<peer::Client, _>(
            &listener,
            &mode,
            guinea_devtools_protocol::PROTOCOL,
            session,
        )
        .await;
        let session = match accepted {
            Ok(session) => session,
            Err(report) => {
                tracing::warn!(?report, "a connection was turned away");
                continue;
            }
        };

        if out.send(Incoming::Opened(id)).is_err() {
            return;
        }

        let (queue, commands) = unbounded();
        queues.borrow_mut().insert(id, queue);
        compio::runtime::spawn(send_each(id, session.remote().clone(), commands)).detach();

        let out = out.clone();
        let queues = queues.clone();
        compio::runtime::spawn(async move {
            if let Err(report) = session.wait().await {
                tracing::debug!(id, ?report, "session ended");
            }

            queues.borrow_mut().remove(&id);
            let _ = out.send(Incoming::Closed(id));
        })
        .detach();
    }
}
