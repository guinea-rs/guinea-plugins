#[derive(Debug, Clone)]
pub struct Opened(pub &'static str);

/// The tab open last, by its title: where a newly connected application opens.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LastTab(pub String);

#[guinea::reducer]
fn reduce_lasttab(this: &mut LastTab, tab: String) {
    this.0 = tab;
}
