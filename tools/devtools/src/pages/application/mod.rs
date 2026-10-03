//! Whatever backends and plugins contribute: the panels and their sections
//! on the left, the open section as a tree, and on the right whatever was
//! picked in it. Both trees are drawn by [`components::tree`].

use std::collections::HashSet;

use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_devtools_model::panels::{self, Entry, Line, Listed, Place};
use guinea_devtools_protocol::Node;

use crate::components;
use crate::components::tree;
use crate::features::focus::contracts::Focus;
use crate::features::panels::PanelsFeature;
use crate::features::panels::contracts::{Open, PanelsState, Select};
use crate::features::sessions::contracts::Live;

/// What is folded in the trees and which language messages are read in: this
/// page's own business. Which section is open and what is picked in it is the
/// feature's, which remembers them.
#[derive(Default)]
pub struct Application {
    /// The rows of the open section a click closed; every other row is open.
    closed: HashSet<Place>,
    /// The panels and folders of the side list a click closed.
    folded: HashSet<Entry>,
    /// The language picked; `None` reads messages in the one the application
    /// shows.
    language: Option<String>,
    /// The last ask from a link that this page answered.
    seen: Option<u64>,
}

impl Page for Application {
    type Params = crate::routes::ApplicationParams;
    type Installs = PanelsFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.read::<Live, _>();
        let (focus, _) = cx.read::<Focus, _>();
        let (view, dispatch) = cx.read::<PanelsState, _>();
        let ui = cx.ui();

        let listed = {
            let sessions = live.read();
            let Some(session) = sessions.get(focus.app) else {
                return;
            };
            components::memo(ui.ctx(), egui::Id::new(("panels", session.id)), session.revision, || {
                panels::listed(&session.snapshot)
            })
        };

        if listed.is_empty() {
            components::block(ui, |ui| {
                ui.label(components::dim(
                    "nothing contributed a panel - a plugin adds one for what it holds",
                ))
            });
            return;
        }

        let wanted = focus
            .key()
            .and_then(|path| Place::holding(&listed, "path", path))
            .filter(|_| super::fresh(&focus, &mut self.seen));
        if let Some(place) = &wanted {
            dispatch.emit(Open(place.panel.clone()));
            dispatch.emit(Select(place.path.clone()));
        }

        let Some(open) = view
            .panel
            .as_ref()
            .and_then(|key| listed.iter().find(|listed| listed.key == *key))
            .or(listed.first())
        else {
            return;
        };
        if view.panel.as_ref() != Some(&open.key) {
            dispatch.emit(Open(open.key.clone()));
        }

        let path = match &wanted {
            Some(place) => place.path.clone(),
            None if view.panel.as_ref() == Some(&open.key) => view.node.clone(),
            None => Vec::new(),
        };
        let at = path.first().copied().filter(|at| *at < open.panel.nodes.len()).unwrap_or(0);
        let Some(section) = open.panel.nodes.get(at) else {
            return;
        };
        let section_place = Place {
            panel: open.key.clone(),
            path: vec![at],
        };
        let picked = (path.len() > 1).then(|| Place {
            panel: open.key.clone(),
            path: path.clone(),
        });

        let chosen = egui::Panel::left("application-sections")
            .resizable(true)
            .size_range(160.0..=360.0)
            .default_size(220.0)
            .frame(components::side())
            .show(ui, |ui| sections(ui, &listed, &section_place, &mut self.folded))
            .inner;
        if let Some(place) = chosen {
            dispatch.emit(Open(place.panel));
            dispatch.emit(Select(place.path));
        }

        if let Some(node) = picked.as_ref().and_then(|place| place.node(&listed)) {
            let closed = egui::Panel::right("application-details")
                .resizable(true)
                .size_range(260.0..=620.0)
                .default_size(400.0)
                .frame(components::side())
                .show(ui, |ui| details(ui, node))
                .inner;
            if closed {
                dispatch.emit(Select(vec![at]));
            }
        }

        let clicked = egui::CentralPanel::default()
            .frame(components::bare(ui))
            .show(ui, |ui| {
                if section.children.is_empty() {
                    components::block(ui, |ui| title(ui, section));
                    components::rule(ui);
                    properties(ui, section);
                    return None;
                }

                let (offered, showing) = panels::languages(section);
                if !offered.is_empty() {
                    components::block(ui, |ui| {
                        language(ui, &offered, showing.as_deref(), &mut self.language)
                    });
                    components::rule(ui);
                }

                let read_in = self.language.as_deref().or(showing.as_deref());
                let lines = panels::lines(open, at, &self.closed, wanted.as_ref(), read_in);
                show(ui, &lines, picked.as_ref(), wanted.as_ref(), &mut self.closed)
            })
            .inner;

        if let Some(place) = clicked {
            dispatch.emit(Select(place.path));
        }
    }
}

/// The panels with their sections, in folders where their names are paths;
/// the section clicked, if any. A click on a panel or a folder folds it.
fn sections(
    ui: &mut egui::Ui,
    listed: &[Listed],
    open: &Place,
    folded: &mut HashSet<Entry>,
) -> Option<Place> {
    let rows = panels::sections(listed, folded, Some(open));
    let answer = tree::show(ui, rows.len(), None, |ui, index| {
        let row = &rows[index];
        tree::Line {
            depth: row.depth,
            branch: row.branch,
            open: row.open,
            selected: matches!(&row.entry, Entry::Section(place) if place == open),
            text: components::line(ui, &row.words),
        }
    });

    let clicked = answer.toggled.or(answer.clicked)?;
    match &rows[clicked].entry {
        Entry::Section(place) => Some(place.clone()),
        entry => {
            if !folded.remove(entry) {
                folded.insert(entry.clone());
            }
            None
        }
    }
}

/// The languages messages can be read in; `picked` becomes the one chosen.
fn language(ui: &mut egui::Ui, offered: &[String], showing: Option<&str>, picked: &mut Option<String>) {
    ui.horizontal(|ui| {
        ui.label(components::dim("messages in"));

        let current = picked.as_deref().or(showing).unwrap_or_default().to_string();
        components::select(ui, "application-language", current.as_str(), |ui| {
            for tag in offered {
                let label = if Some(tag.as_str()) == showing {
                    format!("{tag} (shown)")
                } else {
                    tag.clone()
                };
                if ui.selectable_label(*tag == current, label).clicked() {
                    *picked = (Some(tag.as_str()) != showing).then(|| tag.clone());
                }
            }
        });
    });
}

/// Draws `lines`, folding and unfolding what `closed` holds; the row
/// clicked, if any.
fn show(
    ui: &mut egui::Ui,
    lines: &[Line],
    picked: Option<&Place>,
    reveal: Option<&Place>,
    closed: &mut HashSet<Place>,
) -> Option<Place> {
    let wanted = reveal.and_then(|wanted| lines.iter().position(|line| &line.place == wanted));
    let answer = tree::show(ui, lines.len(), wanted, |ui, index| {
        let line = &lines[index];
        tree::Line {
            depth: line.depth,
            branch: line.branch,
            open: line.open,
            selected: picked == Some(&line.place),
            text: components::line(ui, &line.words),
        }
    });

    if let Some(index) = answer.toggled {
        let place = &lines[index].place;
        if !closed.remove(place) {
            closed.insert(place.clone());
        }
    }

    answer.clicked.map(|index| lines[index].place.clone())
}

/// What the picked node holds; whether it was closed.
fn details(ui: &mut egui::Ui, node: &Node) -> bool {
    let closed = components::head(ui, |ui| title(ui, node));
    components::rule(ui);
    properties(ui, node);

    closed
}

fn title(ui: &mut egui::Ui, node: &Node) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(&node.kind).heading().weak());
        ui.heading(&node.label);
    });
}

fn properties(ui: &mut egui::Ui, node: &Node) {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| components::block(ui, |ui| components::fields(ui, &node.properties)));
}
