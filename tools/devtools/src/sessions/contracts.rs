use std::sync::{Arc, RwLockReadGuard};

use guinea_core::messages;
use guinea_core::scope::Reducer;
use guinea_devtools_hub::Hub;
use guinea_devtools_model::sessions::Sessions;

messages! {
    Listen,
}

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

impl Reducer for Live {
    type Update = Change;

    fn reduce(&mut self, change: Change) {
        match change {
            Change::Started(hub) => self.hub = hub,
            Change::Changed => {}
        }

        self.revision += 1;
    }
}
