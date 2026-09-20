use guinea_core::actor::Context;
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
fn open(this: &mut PanelsActor, ctx: Context<PanelsActor, Open>) {
    let id = ctx.msg.0;
    let _ = this.settings.panel().set(id.clone());
    this.push.send(Picked::Panel(id));
}

#[handler]
fn select(this: &mut PanelsActor, ctx: Context<PanelsActor, Select>) {
    let path = ctx.msg.0;
    if let [section] = path.as_slice() {
        let _ = this.settings.section().set(*section);
    }
    this.push.send(Picked::Node(path));
}
