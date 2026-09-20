use guinea_core::scope::Reducer;

#[derive(Debug, Clone)]
pub struct Opened(pub &'static str);

/// The tab open last, by its title: where a newly connected application opens.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LastTab(pub String);

impl Reducer for LastTab {
    type Update = String;

    fn reduce(&mut self, tab: String) {
        self.0 = tab;
    }
}
