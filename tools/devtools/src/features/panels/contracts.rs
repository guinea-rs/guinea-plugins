#[derive(Debug, Clone)]
pub struct Open(pub String);

#[derive(Debug, Clone)]
pub struct Select(pub Vec<usize>);

/// Which panel is open, and the path to the selected node in it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PanelsState {
    pub panel: Option<String>,
    pub node: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum Picked {
    Panel(String),
    Node(Vec<usize>),
}

#[guinea::reducer]
fn reduce_panelsstate(this: &mut PanelsState, picked: Picked) {
    match picked {
        Picked::Panel(id) => {
            if this.panel.as_ref() != Some(&id) {
                this.node.clear();
            }

            this.panel = Some(id);
        }
        Picked::Node(path) => this.node = path,
    }
}
