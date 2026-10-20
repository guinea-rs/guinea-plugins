use guinea::prelude::In;
use guinea::prelude::Push;
use guinea::{actor, handler};

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
fn show(In(this, Show(target), _): In<FocusActor, Show>) {
    this.push.send(Change::Show(target));
}
