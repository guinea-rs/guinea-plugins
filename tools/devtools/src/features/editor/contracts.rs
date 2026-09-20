use guinea_core::scope::Reducer;

/// An editor source files can be opened in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Editor {
    Configured,
    RustRover,
    VsCode,
    Cursor,
    Zed,
    Clion,
    Idea,
    System,
}

impl Editor {
    pub const ALL: [Editor; 8] = [
        Editor::Configured,
        Editor::RustRover,
        Editor::VsCode,
        Editor::Cursor,
        Editor::Zed,
        Editor::Clion,
        Editor::Idea,
        Editor::System,
    ];

    /// How the choice is remembered.
    pub fn key(self) -> &'static str {
        match self {
            Editor::Configured => "configured",
            Editor::RustRover => "rustrover",
            Editor::VsCode => "vscode",
            Editor::Cursor => "cursor",
            Editor::Zed => "zed",
            Editor::Clion => "clion",
            Editor::Idea => "idea",
            Editor::System => "system",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Editor::Configured => "GUINEA_EDITOR",
            Editor::RustRover => "RustRover",
            Editor::VsCode => "VS Code",
            Editor::Cursor => "Cursor",
            Editor::Zed => "Zed",
            Editor::Clion => "CLion",
            Editor::Idea => "IntelliJ IDEA",
            Editor::System => "System default",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PickEditor(pub Editor);

/// The editor source links open in.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorChoice(pub Editor);

impl Default for EditorChoice {
    fn default() -> Self {
        Self(Editor::System)
    }
}

impl Reducer for EditorChoice {
    type Update = Editor;

    fn reduce(&mut self, editor: Editor) {
        self.0 = editor;
    }
}
