//! Which application the pages below are about, and what a link asked them
//! to bring into view.

use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_core::messages;
use guinea_core::scope::Reducer;
pub use guinea_devtools_model::words::Target;
use guinea_macros::installs;

messages! {
    Show(Target),
}

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

    /// Whether `page` has yet to bring the current target into view; it
    /// counts as brought after this call.
    pub fn fresh(&self, ui: &egui::Ui, page: &str) -> bool {
        let id = egui::Id::new(("shown", page));
        ui.data_mut(|data| {
            let seen = data.get_temp::<u64>(id);
            data.insert_temp(id, self.asked);
            seen != Some(self.asked)
        })
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

pub struct FocusFeature {
    _focus: Bound<Focus>,
}

#[installs]
impl Feature for FocusFeature {
    type Exports = (Focus,);

    fn install(cx: &FeatureInitContext, app: &u64) -> anyhow::Result<Self> {
        let focus = cx
            .state::<Focus>()
            .seed(Focus {
                app: *app,
                ..Focus::default()
            })
            .plain();
        focus.push(Change::App(*app));

        let port = focus.clone();
        cx.answers(move |Show(target)| port.push(Change::Show(target)));

        Ok(Self { _focus: focus })
    }
}
