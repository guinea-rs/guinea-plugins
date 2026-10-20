use guinea::prelude::In;
use guinea::prelude::Push;
use guinea::{actor, handler};

use super::contracts::{Changed, NativeState, Picking, Select};

#[derive(Debug)]
pub struct NativeActor {
    push: Push<NativeState>,
}

impl NativeActor {
    pub fn new(push: Push<NativeState>) -> Self {
        Self { push }
    }
}

actor! {
    NativeActor {
        handlers { Select, Picking }
    }
}

#[handler]
fn select(In(this, Select(selected), _): In<NativeActor, Select>) {
    this.push.send(Changed::Selected(selected));
}

#[handler]
fn picking(In(this, Picking(on), _): In<NativeActor, Picking>) {
    this.push.send(Changed::Picking(on));
}
