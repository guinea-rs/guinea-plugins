use guinea_core::feature::Push;
use guinea_macros::{actor, handler};

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
fn select(this: &mut NativeActor, Select(selected): Select) {
    this.push.send(Changed::Selected(selected));
}

#[handler]
fn picking(this: &mut NativeActor, Picking(on): Picking) {
    this.push.send(Changed::Picking(on));
}
