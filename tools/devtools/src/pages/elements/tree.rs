//! The application's tree, with what each segment made in the backend's own
//! tree under it. What the rows are is the model's; how they are drawn is
//! [`crate::components::tree`]'s.

use std::collections::HashSet;

use guinea_devtools_model::elements::{self, Element};
use guinea_devtools_model::native::NativeTree;
use guinea_devtools_model::protocol::Snapshot;

use crate::components;
use crate::components::tree;

/// What the pointer did to the tree.
pub struct Answer {
    pub clicked: Option<Element>,
    pub hovered: Option<Element>,
}

/// Draws the tree, flipping a row open or closed in `flipped` when its arrow
/// is clicked. Its rows are built again only when `revision` - the session,
/// its revision and its inspector's - or what is open moved.
#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut egui::Ui,
    revision: (u64, u64, Option<u64>),
    snapshot: &Snapshot,
    native: Option<&NativeTree>,
    picked: Option<&Element>,
    reveal: Option<&Element>,
    flipped: &mut HashSet<Element>,
) -> Answer {
    let key = (revision, flipped.clone(), reveal.cloned());
    let lines = components::memo(
        ui.ctx(),
        egui::Id::new(("elements", revision.0)),
        key,
        || elements::lines(snapshot, native, flipped, reveal),
    );

    let wanted = reveal.and_then(|wanted| lines.iter().position(|line| &line.element == wanted));
    let answer = tree::show(ui, lines.len(), wanted, |ui, index| {
        let line = &lines[index];
        tree::Line {
            depth: line.depth,
            branch: line.branch,
            open: line.open,
            selected: picked == Some(&line.element),
            text: components::line(ui, &line.words),
        }
    });

    if let Some(index) = answer.toggled {
        let element = &lines[index].element;
        if !flipped.remove(element) {
            flipped.insert(element.clone());
        }
    }

    Answer {
        clicked: answer.clicked.map(|index| lines[index].element.clone()),
        hovered: answer.hovered.map(|index| lines[index].element.clone()),
    }
}
