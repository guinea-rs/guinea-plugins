//! Whatever backends and plugins contribute, as one tree, and on the right
//! whatever was picked in it.

use std::collections::HashSet;

use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_devtools_model::panels::{self, Listed, Place};
use guinea_devtools_protocol::Node;

use crate::components;
use crate::components::tree;
use crate::features::focus::contracts::Focus;
use crate::features::sessions::contracts::Live;

/// What is picked in the tree, what is folded in it and which language its
/// messages are read in: this page's own business.
#[derive(Default)]
pub struct Application {
    picked: Option<Place>,
    /// The rows a click closed; every other row is open.
    closed: HashSet<Place>,
    /// The language picked; `None` reads messages in the one the application
    /// shows.
    language: Option<String>,
    /// The last ask from a link that this page answered.
    seen: Option<u64>,
}

impl Page for Application {
    type Params = crate::routes::ApplicationParams;
    type Installs = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, _) = cx.state::<Focus, _>();
        let ui = cx.ui();

        let sessions = live.read();
        let Some(session) = sessions.get(focus.app) else {
            return;
        };
        let listed = panels::listed(&session.snapshot);

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
            self.picked = Some(place.clone());
        }

        let (offered, showing) = panels::languages(&listed);
        if !offered.is_empty() {
            egui::Panel::top("application-tools")
                .resizable(false)
                .frame(components::bare(ui))
                .show(ui, |ui| {
                    components::block(ui, |ui| {
                        language(ui, &offered, showing.as_deref(), &mut self.language)
                    })
                });
        }

        let picked = self.picked.clone();
        if let Some(node) = picked.as_ref().and_then(|place| place.node(&listed)) {
            let closed = egui::Panel::right("application-details")
                .resizable(true)
                .size_range(260.0..=620.0)
                .default_size(400.0)
                .frame(components::side())
                .show(ui, |ui| details(ui, node))
                .inner;
            if closed {
                self.picked = None;
            }
        }

        let read_in = self.language.as_deref().or(showing.as_deref());
        let clicked = egui::CentralPanel::default()
            .frame(components::bare(ui))
            .show(ui, |ui| {
                show(ui, &listed, picked.as_ref(), wanted.as_ref(), &mut self.closed, read_in)
            })
            .inner;

        if let Some(place) = clicked {
            self.picked = Some(place);
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

/// Draws the tree, folding what `closed` holds; what was picked, if anything
/// was.
fn show(
    ui: &mut egui::Ui,
    listed: &[Listed],
    picked: Option<&Place>,
    reveal: Option<&Place>,
    closed: &mut HashSet<Place>,
    language: Option<&str>,
) -> Option<Place> {
    let lines = panels::lines(listed, closed, reveal, language);

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
    let closed = components::head(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(&node.kind).heading().weak());
            ui.heading(&node.label);
        });
    });

    components::rule(ui);

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| components::block(ui, |ui| components::fields(ui, &node.properties)));

    closed
}
