//! What devtools remember between runs, kept by amethystate through the store
//! plugin.

use std::sync::Arc;

use amethystate::amethystate;
use guinea::app::{Plugin, PluginBuilder};
use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_core::messages;
use guinea_core::scope::Reducer;
use guinea_macros::installs;

use crate::editor::Editor;

#[amethystate(prefix = "devtools")]
pub struct Remembered {
    /// The tab open last, by its title.
    #[amestate(default = "Elements".to_string())]
    pub tab: String,

    /// Trace kinds switched off.
    #[amestate(default = vec!["tick".to_string()])]
    pub hidden: Vec<String>,

    /// The trace filter.
    #[amestate(default = String::new())]
    pub query: String,

    /// The panel open last, by its key.
    #[amestate(default = String::new())]
    pub panel: String,

    /// Which of its sections.
    #[amestate(default = 0usize)]
    pub section: usize,

    /// The editor source links open in, by its key.
    #[amestate(default = String::new())]
    pub editor: String,

    /// The trace stream open last, by its id.
    #[amestate(default = "all".to_string())]
    pub stream: String,
}

/// Opens [`Remembered`] on the store and provides it; install after the store
/// plugin.
pub struct MemoryPlugin;

impl Plugin for MemoryPlugin {
    const ID: &'static str = "guinea.devtools.memory";

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        let store = app.require::<guinea_plugin_store::Store>()?;
        let remembered = Remembered::new_with(&store)
            .map_err(|error| anyhow::anyhow!("opening what devtools remember: {error:?}"))?;

        app.provide(remembered);
        Ok(())
    }
}

messages! {
    Opened(&'static str),
    PickEditor(Editor),
}

/// The editor source links open in.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorChoice(pub Editor);

impl Default for EditorChoice {
    fn default() -> Self {
        Self(Editor::remembered(""))
    }
}

impl Reducer for EditorChoice {
    type Update = Editor;

    fn reduce(&mut self, editor: Editor) {
        self.0 = editor;
    }
}

/// The tab open last, by its title: where a newly connected application opens.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LastTab(pub String);

impl Reducer for LastTab {
    type Update = String;

    fn reduce(&mut self, tab: String) {
        self.0 = tab;
    }
}

pub struct MemoryFeature {
    _tab: Bound<LastTab>,
    _editor: Bound<EditorChoice>,
}

#[installs]
impl Feature for MemoryFeature {
    type Exports = (LastTab, EditorChoice);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let remembered: Arc<Remembered> = cx.require()?;

        let tab = cx
            .state::<LastTab>()
            .seed(LastTab(remembered.tab().get()))
            .plain();
        let editor = cx
            .state::<EditorChoice>()
            .seed(EditorChoice(Editor::remembered(&remembered.editor().get())))
            .plain();

        let port = tab.clone();
        let memory = remembered.clone();
        cx.answers(move |Opened(title)| {
            let _ = memory.tab().set(title.to_string());
            port.push(title.to_string());
        });

        let port = editor.clone();
        cx.answers(move |PickEditor(picked)| {
            let _ = remembered.editor().set(picked.key().to_string());
            port.push(picked);
        });

        Ok(Self {
            _tab: tab,
            _editor: editor,
        })
    }
}
