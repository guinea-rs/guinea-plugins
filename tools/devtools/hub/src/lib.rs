//! Where applications report to devtools, kept in one place that the window
//! and the HTTP API both read.

mod http;
mod listen;

use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

use guinea_devtools_model::sessions::{Incoming, Sessions};

pub use http::serve;

type Watcher = Box<dyn Fn() + Send + Sync>;

/// What every application reported, as one [`Sessions`].
#[derive(Default)]
pub struct Hub {
    sessions: RwLock<Sessions>,
    watchers: Mutex<Vec<Watcher>>,
}

/// How many reports are applied under one write before readers get a turn.
const BATCH: usize = 512;

impl Hub {
    /// Starts listening for applications.
    pub fn start() -> Arc<Hub> {
        let hub = Arc::new(Hub::default());
        let (out, inbox) = channel();
        listen::spawn(out);

        let applying = hub.clone();
        std::thread::Builder::new()
            .name("devtools-apply".into())
            .spawn(move || applying.apply_all(inbox))
            .expect("spawning the applying thread");

        hub
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Sessions> {
        self.sessions.read().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Calls `watcher` on the applying thread after every batch of reports.
    pub fn on_change(&self, watcher: impl Fn() + Send + Sync + 'static) {
        self.watchers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(Box::new(watcher));
    }

    fn apply_all(&self, inbox: Receiver<Incoming>) {
        while let Ok(first) = inbox.recv() {
            {
                let mut sessions = self.sessions.write().unwrap_or_else(|poisoned| poisoned.into_inner());
                sessions.apply(first);

                for next in inbox.try_iter().take(BATCH) {
                    sessions.apply(next);
                }
            }

            for watcher in self.watchers.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).iter() {
                watcher();
            }
        }
    }
}
