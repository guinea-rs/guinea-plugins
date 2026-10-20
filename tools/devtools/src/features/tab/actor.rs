use guinea::prelude::In;
use guinea::prelude::Push;
use guinea::{actor, handler};

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
fn opened(In(this, Opened(title), _): In<LastTabActor, Opened>) {
    let title = title.to_string();
    this.settings.tab().set(title.clone());
    this.push.send(title);
}
