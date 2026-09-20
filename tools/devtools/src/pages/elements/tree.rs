//! The reactor's element tree. What the rows are is the model's; how they
//! are drawn is [`crate::components::tree`]'s.

use std::collections::HashSet;

use guinea_devtools_model::elements::{self, Element};
use guinea_devtools_model::protocol::Snapshot;

use crate::components;
use crate::components::tree;

/// Draws the tree, folding what `closed` holds; what was picked, if anything
/// was.
pub fn show(
    ui: &mut egui::Ui,
    snapshot: &Snapshot,
    picked: Option<&Element>,
    reveal: Option<&Element>,
    closed: &mut HashSet<Element>,
) -> Option<Element> {
    let lines = elements::lines(snapshot, closed, reveal);

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
        if !closed.remove(element) {
            closed.insert(element.clone());
        }
    }

    answer.clicked.map(|index| lines[index].element.clone())
}
