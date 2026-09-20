use guinea_core::actor::Context;
use guinea_core::feature::Push;
use guinea_macros::{actor, handler};

use super::contracts::{Change, Focus, Show};

#[derive(Debug)]
pub struct FocusActor {
    push: Push<Focus>,
}

impl FocusActor {
    pub fn new(push: Push<Focus>) -> Self {
        Self { push }
    }
}

actor! {
    FocusActor {
        handlers { Show }
    }
}

#[handler]
fn show(this: &mut FocusActor, ctx: Context<FocusActor, Show>) {
    this.push.send(Change::Show(ctx.msg.0));
}
