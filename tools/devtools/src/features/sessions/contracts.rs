use std::sync::{Arc, RwLockReadGuard};

use guinea_devtools_hub::Hub;
use guinea_devtools_model::sessions::Sessions;

#[derive(Debug, Clone)]
pub struct Listen;

/// The hub every page reads, and a revision that moves whenever it changed so
/// that pages draw again.
#[derive(Clone, Default)]
pub struct Live {
    pub hub: Arc<Hub>,
    pub revision: u64,
}

impl std::fmt::Debug for Live {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Live")
            .field("revision", &self.revision)
            .finish_non_exhaustive()
    }
}

impl Live {
    pub fn read(&self) -> RwLockReadGuard<'_, Sessions> {
        self.hub.read()
    }
}

#[derive(Clone)]
pub enum Change {
    Started(Arc<Hub>),
    Changed,
}

impl std::fmt::Debug for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Change::Started(_) => "Started",
            Change::Changed => "Changed",
        })
    }
}

#[guinea::reducer]
fn reduce_live(this: &mut Live, change: Change) {
    match change {
        Change::Started(hub) => this.hub = hub,
        Change::Changed => {}
    }

    this.revision += 1;
}
