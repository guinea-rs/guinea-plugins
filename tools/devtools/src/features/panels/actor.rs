use guinea::prelude::In;
use guinea::prelude::Push;
use guinea::{actor, handler};

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
fn open(In(this, Open(id), _): In<PanelsActor, Open>) {
    this.settings.panel().set(id.clone());
    this.push.send(Picked::Panel(id));
}

#[handler]
fn select(In(this, Select(path), _): In<PanelsActor, Select>) {
    if let Some(section) = path.first() {
        this.settings.section().set(*section);
    }
    this.push.send(Picked::Node(path));
}
