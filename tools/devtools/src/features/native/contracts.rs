use guinea_core::scope::Reducer;

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

impl Reducer for NativeState {
    type Update = Changed;

    fn reduce(&mut self, changed: Changed) {
        match changed {
            Changed::Selected(handle) => self.selected = Some(handle),
            Changed::Picking(on) => self.picking = on,
        }
    }
}
