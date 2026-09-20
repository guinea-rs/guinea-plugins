//! Where something was declared, as a link that opens it.

use std::path::Path;

use guinea_devtools_protocol::Declared;

use crate::features::editor::contracts::Editor;
use crate::features::editor::launch;

use super::text::{dim, mono};

pub fn link(ui: &mut egui::Ui, declared: &Declared, editor: Editor) {
    let file = Path::new(&declared.file);
    let name = file
        .file_name()
        .map_or_else(|| declared.file.clone(), |name| name.to_string_lossy().into_owned());
    let shown = format!("{name}:{}", declared.line);

    if !declared.found {
        ui.label(dim(shown)).on_hover_text(&declared.file);
        return;
    }

    let link = ui
        .add(egui::Link::new(mono(shown).underline()))
        .on_hover_text(format!("{}\nopen in {}", declared.file, editor.title()));
    if link.clicked() {
        launch::open(editor, declared);
    }
}
