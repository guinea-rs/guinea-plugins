use guinea_core::actor::Context;
use guinea_core::feature::Push;
use guinea_macros::{actor, handler};

use super::contracts::{LastTab, Opened};
use super::settings::TabSettings;

#[derive(Debug)]
pub struct LastTabActor {
    push: Push<LastTab>,
    settings: TabSettings,
}

impl LastTabActor {
    pub fn new(push: Push<LastTab>, settings: TabSettings) -> Self {
        Self { push, settings }
    }
}

actor! {
    LastTabActor {
        handlers { Opened }
    }
}

#[handler]
fn opened(this: &mut LastTabActor, ctx: Context<LastTabActor, Opened>) {
    let title = ctx.msg.0.to_string();
    let _ = this.settings.tab().set(title.clone());
    this.push.send(title);
}
