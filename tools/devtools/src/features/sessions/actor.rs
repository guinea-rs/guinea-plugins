use std::sync::Arc;

use guinea::prelude::In;
use guinea::prelude::Push;
use guinea::{actor, handler};
use guinea_devtools_hub::Hub;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use super::contracts::{Change, Listen, Live};

#[derive(Debug, Clone)]
pub struct Changed;

#[derive(Debug)]
pub struct SessionsActor {
    push: Push<Live>,
    changes: Option<Arc<Mutex<UnboundedReceiver<()>>>>,
}

impl SessionsActor {
    pub fn new(push: Push<Live>) -> Self {
        Self {
            push,
            changes: None,
        }
    }
}

actor! {
    SessionsActor {
        handlers {
            Listen => { task Changed },
            Changed => { task Changed loop },
        }
    }
}

#[handler]
fn listen(In(this, _, cx): In<SessionsActor, Listen>) {
    if this.changes.is_some() {
        return;
    }

    let hub = Hub::start();
    match guinea_devtools_hub::serve(hub.clone()) {
        Ok(access) => tracing::info!(url = access.url, "the HTTP API is up"),
        Err(error) => tracing::warn!(%error, "no HTTP API"),
    }

    let (out, changes) = unbounded_channel();
    hub.on_change(move || {
        let _ = out.send(());
    });
    this.push.send(Change::Started(hub));

    let changes = Arc::new(Mutex::new(changes));
    this.changes = Some(changes.clone());
    cx.task().spawn(wait(changes));
}

#[handler]
fn changed(In(this, _, cx): In<SessionsActor, Changed>) {
    this.push.send(Change::Changed);

    if let Some(changes) = this.changes.clone() {
        cx.task().spawn(wait(changes));
    }
}

/// The next change, with every other one already waiting folded into it.
async fn wait(changes: Arc<Mutex<UnboundedReceiver<()>>>) -> Changed {
    let mut changes = changes.lock().await;
    if changes.recv().await.is_none() {
        std::future::pending::<()>().await;
    }

    while changes.try_recv().is_ok() {}
    Changed
}
