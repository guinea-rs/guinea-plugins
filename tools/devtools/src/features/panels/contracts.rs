use guinea_core::scope::Reducer;

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

impl Reducer for PanelsState {
    type Update = Picked;

    fn reduce(&mut self, picked: Picked) {
        match picked {
            Picked::Panel(id) => {
                if self.panel.as_ref() != Some(&id) {
                    self.node.clear();
                }

                self.panel = Some(id);
            }
            Picked::Node(path) => self.node = path,
        }
    }
}
