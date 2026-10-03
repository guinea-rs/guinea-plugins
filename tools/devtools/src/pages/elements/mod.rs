//! The application as one tree - its actors, its windows, each window's
//! segments with what they hold and what they made, down to the backend's
//! own elements - and on the right whatever was picked in it.

use std::collections::HashSet;

use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_devtools_model::elements::{self, Details, Element};
use guinea_devtools_protocol::{Capability, Command};

mod native;
mod properties;
mod tree;

use crate::components;
use crate::components::source;
use crate::features::editor::contracts::{Editor, EditorChoice};
use crate::features::focus::contracts::Focus;
use crate::features::native::NativeFeature;
use crate::features::native::contracts::{NativeState, Select};
use crate::features::sessions::contracts::Live;

/// What is picked in the tree and what is folded in it: this page's own
/// business. The native element picked is the feature's too, since a
/// property that names another element, a pick in the window and a layout
/// pass all pick one.
#[derive(Default)]
pub struct Elements {
    picked: Option<Element>,
    /// The rows a click flipped from how they start.
    flipped: HashSet<Element>,
    /// The last ask from a link that this page answered.
    seen: Option<u64>,
    /// The native element last picked, by whatever picked it.
    native_seen: Option<u64>,
    inspecting: native::Inspecting,
}

impl Page for Elements {
    type Params = crate::routes::ElementsParams;
    type Installs = NativeFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.read::<Live, _>();
        let (focus, _) = cx.read::<Focus, _>();
        let (editor, _) = cx.read::<EditorChoice, _>();
        let (view, dispatch) = cx.read::<NativeState, _>();
        let editor = editor.0;
        let ui = cx.ui();

        let sessions = live.read();
        let Some(session) = sessions.get(focus.app) else {
            return;
        };
        let snapshot = &session.snapshot;
        let inspector = sessions.native_for(focus.app);
        let native = inspector.map(|inspector| &inspector.inspection.tree);
        let send = |command: Command| {
            if let Some(inspector) = inspector {
                let _ = live.hub.send_in(&sessions, inspector.id, command);
            }
        };

        let named = if let Some(name) = focus.actor() {
            elements::actor_named(snapshot, name)
        } else if let Some(name) = focus.reducer() {
            elements::state_named(snapshot, name)
        } else {
            None
        };
        let mut wanted = named.filter(|_| super::fresh(&focus, &mut self.seen));
        if view.selected != self.native_seen {
            self.native_seen = view.selected;
            wanted = view.selected.map(Element::Native).or(wanted);
        }
        if let Some(element) = &wanted {
            self.picked = Some(element.clone());
        }

        match inspector {
            Some(inspector) if view.picking => self.inspecting.pick(ui, session, inspector, &dispatch, &send),
            _ => self.inspecting.picker_off(),
        }

        egui::Panel::top("elements-tools")
            .resizable(false)
            .exact_size(30.0)
            .frame(components::side())
            .show(ui, |ui| match inspector {
                Some(inspector) => self.inspecting.toolbar(ui, inspector, &view, &dispatch, &send),
                None => self.inspecting.attach(ui, &sessions, session),
            });

        if let Some(inspector) = inspector.filter(|inspector| !inspector.inspection.frames.is_empty()) {
            egui::Panel::bottom("native-frames")
                .resizable(true)
                .size_range(120.0..=420.0)
                .frame(components::side())
                .show(ui, |ui| {
                    let inspection = &inspector.inspection;
                    self.inspecting.frames(ui, &inspection.tree, &inspection.frames, &dispatch)
                });
        }

        let picked = self.picked.clone();
        let closed = match (&picked, inspector) {
            (Some(Element::Native(handle)), Some(inspector)) if inspector.inspection.tree.get(*handle).is_some() => {
                self.inspecting.ask_for(*handle, &send);
                let inspection = &inspector.inspection;
                egui::Panel::right("native-properties")
                    .resizable(true)
                    .default_size(420.0)
                    .size_range(300.0..=720.0)
                    .frame(components::side())
                    .show(ui, |ui| {
                        let properties = inspection
                            .properties
                            .as_ref()
                            .filter(|(element, _)| element == handle)
                            .map(|(_, properties)| properties.as_slice());
                        let context = properties::Context {
                            tree: &inspection.tree,
                            enums: &inspection.enums,
                            edits: inspector.info.can(Capability::NativeEdit),
                            send: &send,
                            dispatch: &dispatch,
                        };
                        properties::show(ui, *handle, properties, &context);
                    });
                false
            }
            _ => match picked.as_ref().and_then(|element| elements::describe(session, element)) {
                Some(details) => egui::Panel::right("element-details")
                    .resizable(true)
                    .size_range(280.0..=620.0)
                    .default_size(400.0)
                    .frame(components::side())
                    .show(ui, |ui| show(ui, &details, editor))
                    .inner,
                None => false,
            },
        };
        if closed {
            self.picked = None;
        }

        let answer = egui::CentralPanel::default()
            .frame(components::bare(ui))
            .show(ui, |ui| {
                tree::show(
                    ui,
                    (session.id, session.revision, inspector.map(|inspector| inspector.revision)),
                    snapshot,
                    native,
                    picked.as_ref(),
                    wanted.as_ref(),
                    &mut self.flipped,
                )
            })
            .inner;

        if !view.picking {
            let under = match answer.hovered {
                Some(Element::Native(handle)) => Some(handle),
                _ => None,
            };
            self.inspecting.highlight(under, &send);
        }

        if let Some(element) = answer.clicked {
            if let Element::Native(handle) = element {
                self.native_seen = Some(handle);
                dispatch.emit(Select(handle));
            }
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
