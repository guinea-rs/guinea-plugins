#[derive(Debug, Clone)]
pub struct Select(pub u64);

#[derive(Debug, Clone)]
pub struct Picking(pub bool);

/// Which element is selected, and whether the pointer is picking one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeState {
    pub selected: Option<u64>,
    pub picking: bool,
}

#[derive(Clone, Debug)]
pub enum Changed {
    Selected(u64),
    Picking(bool),
}

#[guinea::reducer]
fn reduce_nativestate(this: &mut NativeState, changed: Changed) {
    match changed {
        Changed::Selected(handle) => this.selected = Some(handle),
        Changed::Picking(on) => this.picking = on,
    }
}
