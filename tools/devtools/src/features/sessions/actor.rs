use std::sync::Arc;

use guinea_core::actor::Cx;
use guinea_core::feature::Push;
use guinea_devtools_hub::Hub;
use guinea_macros::{actor, handler};
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
            Listen => { bg Changed },
            Changed => { bg Changed loop },
        }
    }
}

#[handler]
fn listen(this: &mut SessionsActor, _: Listen, cx: Cx) {
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
    cx.spawn_bg::<Changed, _>(wait(changes));
}

#[handler]
fn changed(this: &mut SessionsActor, _: Changed, cx: Cx) {
    this.push.send(Change::Changed);

    if let Some(changes) = this.changes.clone() {
        cx.spawn_bg::<Changed, _>(wait(changes));
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
