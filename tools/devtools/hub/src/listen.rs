//! The endpoint, on a thread and a runtime of its own.

use std::sync::mpsc::Sender;

use guinea_devtools_model::sessions::Incoming;
use guinea_devtools_protocol::devtools_capnp::peer;
use guinea_devtools_protocol::{key, wire};
use ogurpchik::rpc::accept_session;

pub fn spawn(out: Sender<Incoming>) {
    std::thread::Builder::new()
        .name("devtools-listen".into())
        .spawn(move || match compio::runtime::Runtime::new() {
            Ok(runtime) => runtime.block_on(listen(out)),
            Err(error) => {
                let _ = out.send(Incoming::Failed(format!("no runtime: {error}")));
            }
        })
        .expect("spawning the listener thread");
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
        let report = wire::received(&params)?;

        self.out
            .send(Incoming::Report(self.id, Box::new(report)))
            .map_err(|_| capnp::Error::disconnected("devtools are closing".into()))
    }
}

async fn listen(out: Sender<Incoming>) {
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

        let out = out.clone();
        compio::runtime::spawn(async move {
            if let Err(report) = session.wait().await {
                tracing::debug!(id, ?report, "session ended");
            }

            let _ = out.send(Incoming::Closed(id));
        })
        .detach();
    }
}
