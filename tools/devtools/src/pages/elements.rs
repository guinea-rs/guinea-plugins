//! The application as one tree, and on the right whatever was picked in it.

use guinea::eframe::{Page, PageCx};
use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_core::messages;
use guinea_core::scope::Reducer;
use guinea_devtools_model::elements::{self, Details, Element};
use guinea_macros::installs;

use crate::editor::{self, Editor};
use crate::focus::Focus;
use crate::memory::EditorChoice;
use crate::pages::element_tree;
use crate::sessions::contracts::Live;
use crate::style;

messages! {
    Pick(Element),
    Unpick,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picked(pub Option<Element>);

impl Reducer for Picked {
    type Update = Option<Element>;

    fn reduce(&mut self, element: Option<Element>) {
        self.0 = element;
    }
}

pub struct PickedFeature {
    _picked: Bound<Picked>,
}

#[installs]
impl Feature for PickedFeature {
    type Exports = (Picked,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let picked = cx.state::<Picked>().plain();

        let port = picked.clone();
        cx.answers(move |Pick(element)| port.push(Some(element)));

        let port = picked.clone();
        cx.answers(move |Unpick| port.push(None));

        Ok(Self { _picked: picked })
    }
}

pub struct Elements;

impl Page for Elements {
    type Params = crate::routes::ElementsParams;
    type Installs = PickedFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn render(cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, _) = cx.state::<Focus, _>();
        let (picked, dispatch) = cx.state::<Picked, _>();
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
        let wanted = named.filter(|_| focus.fresh(ui, "elements"));
        if let Some(element) = &wanted {
            dispatch.emit(Pick(element.clone()));
        }

        let picked = wanted.clone().or_else(|| picked.0.clone());
        let details = picked
            .as_ref()
            .and_then(|element| elements::describe(session, element));

        if let Some(details) = &details {
            let closed = egui::Panel::right("element-details")
                .resizable(true)
                .size_range(280.0..=620.0)
                .default_size(400.0)
                .frame(style::side())
                .show(ui, |ui| show(ui, details, editor))
                .inner;
            if closed {
                dispatch.emit(Unpick);
            }
        }

        egui::CentralPanel::default()
            .frame(style::bare(ui))
            .show(ui, |ui| {
                element_tree::show(ui, snapshot, picked.as_ref(), wanted.as_ref(), &dispatch)
            });
    }
}

/// What is known about the picked element; whether it was closed.
fn show(ui: &mut egui::Ui, details: &Details, editor: Editor) -> bool {
    let closed = style::head(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(&details.kind).heading().weak());
            ui.heading(&details.title);
        });

        if let Some(declared) = &details.declared {
            editor::link(ui, declared, editor);
        }
    });

    style::rule(ui);

    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        style::block(ui, |ui| style::fields(ui, &details.rows));

        if !details.handlers.is_empty() {
            section(ui, "Handlers", |ui| {
                for handler in &details.handlers {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(style::mono(&handler.message).strong());

                        if !handler.flows.is_empty() {
                            ui.label(style::dim(format!("→ {}", handler.flows)));
                        }
                        if let Some(declared) = &handler.declared {
                            editor::link(ui, declared, editor);
                        }
                    });
                }
            });
        }

        if let Some(body) = &details.body {
            section(ui, "State", |ui| {
                ui.label(style::mono(body));
            });
        }
    });

    closed
}

fn section(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    style::heading(ui, title);
    style::block(ui, add);
}
