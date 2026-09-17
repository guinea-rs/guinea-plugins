//! Whatever backends and plugins contribute, drawn as trees.

use std::sync::Arc;

use guinea::eframe::{Page, PageCx};
use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::{Bound, Dispatch};
use guinea_core::messages;
use guinea_core::scope::Reducer;
use guinea_devtools_model::panels;
use guinea_devtools_protocol::{Node, Panel};
use guinea_macros::installs;

use crate::focus::Focus;
use crate::memory::Remembered;
use crate::sessions::contracts::Live;
use crate::style;

messages! {
    Open(String),
    Select(Vec<usize>),
}

/// Which panel is open, and the path to the selected node in it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PanelsView {
    pub panel: Option<String>,
    pub node: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum Picked {
    Panel(String),
    Node(Vec<usize>),
}

impl Reducer for PanelsView {
    type Update = Picked;

    fn reduce(&mut self, picked: Picked) {
        match picked {
            Picked::Panel(id) => {
                if self.panel.as_ref() != Some(&id) {
                    self.node.clear();
                }

                self.panel = Some(id);
            }
            Picked::Node(path) => self.node = path,
        }
    }
}

pub struct PanelsViewFeature {
    _view: Bound<PanelsView>,
}

#[installs]
impl Feature for PanelsViewFeature {
    type Exports = (PanelsView,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let remembered: Arc<Remembered> = cx.require()?;
        let panel = remembered.panel().get();

        let view = cx
            .state::<PanelsView>()
            .seed(PanelsView {
                node: vec![remembered.section().get()],
                panel: (!panel.is_empty()).then_some(panel),
            })
            .plain();

        let port = view.clone();
        let memory = remembered.clone();
        cx.answers(move |Open(id)| {
            let _ = memory.panel().set(id.clone());
            port.push(Picked::Panel(id));
        });

        let port = view.clone();
        cx.answers(move |Select(path)| {
            if let [section] = path.as_slice() {
                let _ = remembered.section().set(*section);
            }
            port.push(Picked::Node(path));
        });

        Ok(Self { _view: view })
    }
}

pub struct Panels;

impl Page for Panels {
    type Params = crate::routes::PanelsParams;
    type Installs = PanelsViewFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn render(cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, _) = cx.state::<Focus, _>();
        let (view, dispatch) = cx.state::<PanelsView, _>();
        let ui = cx.ui();

        let listed = {
            let sessions = live.read();
            let Some(session) = sessions.get(focus.app) else {
                return;
            };
            panels::listed(&session.snapshot)
        };
        let listed: Vec<(String, String, &Panel)> = listed
            .iter()
            .map(|listed| (listed.key.clone(), listed.title.clone(), &listed.panel))
            .collect();

        if listed.is_empty() {
            style::block(ui, |ui| {
                ui.label(style::dim(
                    "nothing contributed a panel - a plugin adds one for what it holds",
                ))
            });
            return;
        }

        let wanted = focus
            .key()
            .filter(|_| focus.fresh(ui, "panels"))
            .and_then(|path| {
                listed.iter().find_map(|(key, _, panel)| {
                    panels::holding(&panel.nodes, "path", path).map(|at| (key.clone(), at))
                })
            });
        if let Some((key, path)) = wanted {
            dispatch.emit(Open(key));
            dispatch.emit(Select(path.clone()));
            ui.data_mut(|data| data.insert_temp(reveal_id(), path));
            return;
        }

        let open = view
            .panel
            .as_ref()
            .and_then(|id| listed.iter().find(|(key, _, _)| key == id))
            .or(listed.first());

        let Some((key, _, panel)) = open else { return };
        if view.panel.as_ref() != Some(key) {
            dispatch.emit(Open(key.clone()));
        }

        let mut selected = view
            .panel
            .as_ref()
            .filter(|id| *id == key)
            .map(|_| view.node.clone())
            .unwrap_or_default();
        if selected.is_empty() {
            selected.push(0);
        }
        let section_at = selected[0];

        egui::Panel::left("panel-list")
            .resizable(true)
            .size_range(160.0..=320.0)
            .frame(style::side())
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| style::block(ui, |ui| {
                    for (listed_key, title, listed_panel) in &listed {
                        let current = (listed_key == key).then_some(section_at);
                        if let Some(index) = sections(ui, listed_key, title, listed_panel, current) {
                            dispatch.emit(Open(listed_key.clone()));
                            dispatch.emit(Select(vec![index]));
                        }
                    }
                }));
            });

        let Some(section) = panel.nodes.get(section_at) else {
            return;
        };
        let whole = section.children.is_empty();

        let picked = (!whole && selected.len() > 1)
            .then(|| panels::find(&panel.nodes, &selected))
            .flatten();
        if let Some(node) = picked {
            let closed = egui::Panel::right("node-properties")
                .resizable(true)
                .size_range(260.0..=520.0)
                .frame(style::side())
                .show(ui, |ui| {
                    let closed = style::head(ui, |ui| title(ui, node));
                    style::rule(ui);
                    properties(ui, node);
                    closed
                })
                .inner;
            if closed {
                dispatch.emit(Select(vec![section_at]));
            }
        }

        let pane = style::bare(ui);
        let reveal = ui.data(|data| data.get_temp::<Vec<usize>>(reveal_id()));
        egui::CentralPanel::default().frame(pane).show(ui, |ui| {
            if whole {
                style::block(ui, |ui| title(ui, section));
                style::rule(ui);
                properties(ui, section);
                return;
            }

            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| style::block(ui, |ui| {
                let shown = Shown {
                    panel: key,
                    selected: &selected,
                    reveal: reveal.as_deref(),
                    dispatch: &dispatch,
                    flat: section.children.iter().all(|child| child.children.is_empty()),
                };

                let mut path = vec![section_at];
                for (index, node) in section.children.iter().enumerate() {
                    path.push(index);
                    tree(ui, node, &mut path, &shown);
                    path.pop();
                }
            }));
        });
    }
}

/// A panel in the list, open to its sections; the section clicked, if any.
fn sections(
    ui: &mut egui::Ui,
    key: &str,
    title: &str,
    panel: &Panel,
    open: Option<usize>,
) -> Option<usize> {
    let mut clicked = None;

    egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        ui.make_persistent_id(("panel", key)),
        true,
    )
    .show_header(ui, |ui| {
        ui.label(egui::RichText::new(title).strong());
    })
    .body(|ui| {
        for (index, section) in panel.nodes.iter().enumerate() {
            if ui
                .selectable_label(open == Some(index), &section.label)
                .clicked()
            {
                clicked = Some(index);
            }
        }
    });

    clicked
}

fn reveal_id() -> egui::Id {
    egui::Id::new("panels-reveal")
}

struct Shown<'a> {
    panel: &'a str,
    selected: &'a [usize],
    /// The node a link led to, brought into view this frame.
    reveal: Option<&'a [usize]>,
    dispatch: &'a Dispatch,
    /// Whether nothing in the section opens, so no row needs room for an arrow.
    flat: bool,
}

fn tree(ui: &mut egui::Ui, node: &Node, path: &mut Vec<usize>, shown: &Shown) {
    let text = egui::RichText::new(&node.label).strong();
    let kind = style::dim(&node.kind);
    let on = path.as_slice() == shown.selected;
    let reveal = shown.reveal == Some(path.as_slice());

    let label = |ui: &mut egui::Ui| {
        let response = ui.selectable_label(on, text);
        if response.clicked() {
            shown.dispatch.emit(Select(path.clone()));
        }

        if reveal {
            response.scroll_to_me(Some(egui::Align::Center));
            ui.data_mut(|data| data.remove::<Vec<usize>>(reveal_id()));
        }

        ui.label(kind);
    };

    if node.children.is_empty() {
        ui.horizontal(|ui| {
            if !shown.flat {
                ui.add_space(18.0);
            }
            label(ui);
        });
        return;
    }

    let id = ui.make_persistent_id((shown.panel, path.clone()));
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, path.len() < 8);
    if shown
        .reveal
        .is_some_and(|target| target.len() > path.len() && target.starts_with(path))
    {
        state.set_open(true);
    }

    state.show_header(ui, label).body(|ui| {
        for (index, child) in node.children.iter().enumerate() {
            path.push(index);
            tree(ui, child, path, shown);
            path.pop();
        }
    });
}

fn title(ui: &mut egui::Ui, node: &Node) {
    ui.heading(&node.label);
    ui.label(style::dim(&node.kind));
}

fn properties(ui: &mut egui::Ui, node: &Node) {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| style::block(ui, |ui| style::fields(ui, &node.properties)));
}
