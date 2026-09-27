use guinea_core::feature::Push;
use guinea_macros::{actor, handler};

use super::contracts::{Open, PanelsState, Picked, Select};
use super::settings::PanelsSettings;

#[derive(Debug)]
pub struct PanelsActor {
    push: Push<PanelsState>,
    settings: PanelsSettings,
}

impl PanelsActor {
    pub fn new(push: Push<PanelsState>, settings: PanelsSettings) -> Self {
        Self { push, settings }
    }
}

actor! {
    PanelsActor {
        handlers { Open, Select }
    }
}

#[handler]
fn open(this: &mut PanelsActor, Open(id): Open) {
    let _ = this.settings.panel().set(id.clone());
    this.push.send(Picked::Panel(id));
}

#[handler]
fn select(this: &mut PanelsActor, Select(path): Select) {
    if let Some(section) = path.first() {
        let _ = this.settings.section().set(*section);
    }
    this.push.send(Picked::Node(path));
}
