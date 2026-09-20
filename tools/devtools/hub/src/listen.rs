//! The endpoint, on a thread and a runtime of its own.
//!
//! A session's remote end lives here too: capnp clients belong to the runtime
//! that made them, so a command from anywhere else is queued to this thread
//! and sent from it.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use guinea_devtools_model::sessions::Incoming;
use guinea_devtools_protocol::devtools_capnp::peer;
use guinea_devtools_protocol::{Command, Report, key, wire};
use ogurpchik::rpc::accept_session;

/// Where commands for the applications go in.
pub type Commands = UnboundedSender<(u64, Command)>;

type Remotes = Rc<RefCell<HashMap<u64, peer::Client>>>;

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

/// Sends each queued command to its session, one at a time and in order.
async fn deliver(remotes: Remotes, mut queued: UnboundedReceiver<(u64, Command)>) {
    while let Some((id, command)) = queued.next().await {
        let Some(remote) = remotes.borrow().get(&id).cloned() else {
            tracing::debug!(id, ?command, "no such session to send to");
            continue;
        };

        if let Err(error) = wire::send(&remote, &command).await {
            tracing::debug!(id, ?error, "a command did not reach its session");
        }
    }
}

async fn listen(out: Sender<Incoming>, queued: UnboundedReceiver<(u64, Command)>) {
    let remotes = Remotes::default();
    compio::runtime::spawn(deliver(remotes.clone(), queued)).detach();

    let secret = match key::create() {
        Ok(secret) => secret,
        Err(error) => {
            let _ = out.send(Incoming::Failed(format!("cannot write the key: {error}")));
            return;
        }
    };

    let mode = key::handshake(secret);

    let endpoint = guinea_devtools_protocol::endpoint();
    let listener = match endpoint.listen().await {
        Ok(listener) => listener,
        Err(report) => {
            let _ = out.send(Incoming::Failed(format!("{endpoint}: {report:?}")));
            return;
        }
    };
    let _ = out.send(Incoming::Listening(endpoint.to_string()));

    let mut next = 1;

    loop {
        let id = next;
        next += 1;

        let session = Session {
            id,
            out: out.clone(),
        };
        let accepted = accept_session::<peer::Client, _>(&listener, &mode, session).await;
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

        remotes.borrow_mut().insert(id, session.remote().clone());

        let out = out.clone();
        let remotes = remotes.clone();
        compio::runtime::spawn(async move {
            if let Err(report) = session.wait().await {
                tracing::debug!(id, ?report, "session ended");
            }

            remotes.borrow_mut().remove(&id);
            let _ = out.send(Incoming::Closed(id));
        })
        .detach();
    }
}
