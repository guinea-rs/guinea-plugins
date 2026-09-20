//! The application as one tree, and on the right whatever was picked in it.

use std::collections::HashSet;

use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_devtools_model::elements::{self, Details, Element};

mod tree;

use crate::components;
use crate::components::source;
use crate::features::editor::contracts::{Editor, EditorChoice};
use crate::features::focus::contracts::Focus;
use crate::features::sessions::contracts::Live;

/// What is picked in the tree and what is folded in it: this page's own
/// business, and nobody else reads it.
#[derive(Default)]
pub struct Elements {
    picked: Option<Element>,
    /// The rows a click closed; every other row is open.
    closed: HashSet<Element>,
    /// The last ask from a link that this page answered.
    seen: Option<u64>,
}

impl Page for Elements {
    type Params = crate::routes::ElementsParams;
    type Installs = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, _) = cx.state::<Focus, _>();
        let (editor, _) = cx.state::<EditorChoice, _>();
        let editor = editor.0;
        let ui = cx.ui();

        let sessions = live.read();
        let Some(session) = sessions.get(focus.app) else {
            return;
        };
        let snapshot = &session.snapshot;

        let named = if let Some(name) = focus.actor() {
            elements::actor_named(snapshot, name)
        } else if let Some(name) = focus.reducer() {
            elements::state_named(snapshot, name)
        } else {
            None
        };
        let wanted = named.filter(|_| super::fresh(&focus, &mut self.seen));
        if let Some(element) = &wanted {
            self.picked = Some(element.clone());
        }

        let picked = self.picked.clone();
        let details = picked
            .as_ref()
            .and_then(|element| elements::describe(session, element));

        if let Some(details) = &details {
            let closed = egui::Panel::right("element-details")
                .resizable(true)
                .size_range(280.0..=620.0)
                .default_size(400.0)
                .frame(components::side())
                .show(ui, |ui| show(ui, details, editor))
                .inner;
            if closed {
                self.picked = None;
            }
        }

        let clicked = egui::CentralPanel::default()
            .frame(components::bare(ui))
            .show(ui, |ui| {
                tree::show(ui, snapshot, picked.as_ref(), wanted.as_ref(), &mut self.closed)
            })
            .inner;

        if let Some(element) = clicked {
            self.picked = Some(element);
        }
    }
}

/// What is known about the picked element; whether it was closed.
fn show(ui: &mut egui::Ui, details: &Details, editor: Editor) -> bool {
    let closed = components::head(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(&details.kind).heading().weak());
            ui.heading(&details.title);
        });

        ui.horizontal_wrapped(|ui| {
            if let Some(declared) = &details.declared {
                source::link(ui, declared, editor);
            }
            if let Some(routed) = &details.routed {
                ui.label(components::dim("· routed at"));
                source::link(ui, routed, editor);
            }
        });
    });

    components::rule(ui);

    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        components::block(ui, |ui| components::fields(ui, &details.rows));

        if !details.handlers.is_empty() {
            section(ui, "Handlers", |ui| {
                for handler in &details.handlers {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(components::mono(&handler.message).strong());

                        if !handler.flows.is_empty() {
                            ui.label(components::dim(format!("→ {}", handler.flows)));
                        }
                        if let Some(declared) = &handler.declared {
                            source::link(ui, declared, editor);
                        }
                    });
                }
            });
        }

        if let Some(body) = &details.body {
            section(ui, "State", |ui| {
                ui.label(components::mono(body));
            });
        }
    });

    closed
}

fn section(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    components::heading(ui, title);
    components::block(ui, add);
}
