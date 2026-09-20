use guinea_core::scope::Reducer;
use guinea_devtools_model::words::Target;

#[derive(Debug, Clone)]
pub struct Show(pub Target);

/// Which application the pages below are about, and what a link asked them
/// to bring into view.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Focus {
    pub app: u64,
    /// The last target a link asked for.
    pub target: Option<Target>,
    /// Changes with every click, so a page brings the target into view once.
    pub asked: u64,
}

impl Focus {
    pub fn actor(&self) -> Option<&str> {
        match &self.target {
            Some(Target::Actor(name)) => Some(name),
            _ => None,
        }
    }

    pub fn reducer(&self) -> Option<&str> {
        match &self.target {
            Some(Target::Reducer(name)) => Some(name),
            _ => None,
        }
    }

    /// A store key, by its path.
    pub fn key(&self) -> Option<&str> {
        match &self.target {
            Some(Target::Key(path)) => Some(path),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Change {
    App(u64),
    Show(Target),
}

impl Reducer for Focus {
    type Update = Change;

    fn reduce(&mut self, change: Change) {
        match change {
            Change::App(app) => self.app = app,
            Change::Show(target) => {
                self.target = Some(target);
                self.asked += 1;
            }
        }
    }
}
