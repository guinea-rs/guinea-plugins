//! Whatever backends and plugins contribute, drawn as trees.

use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_core::feature::Dispatch;
use guinea_devtools_model::panels;
use guinea_devtools_protocol::{Node, Panel};

use crate::components;
use crate::features::focus::contracts::Focus;
use crate::features::panels::PanelsFeature;
use crate::features::panels::contracts::{Open, PanelsState, Select};
use crate::features::sessions::contracts::Live;

#[derive(Default)]
pub struct Panels {
    /// The last ask from a link that this page answered.
    seen: Option<u64>,
}

impl Page for Panels {
    type Params = crate::routes::PanelsParams;
    type Installs = PanelsFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, _) = cx.state::<Focus, _>();
        let (view, dispatch) = cx.state::<PanelsState, _>();
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
            components::block(ui, |ui| {
                ui.label(components::dim(
                    "nothing contributed a panel - a plugin adds one for what it holds",
                ))
            });
            return;
        }

        let wanted = focus
            .key()
            .filter(|_| super::fresh(&focus, &mut self.seen))
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
            .frame(components::side())
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| components::block(ui, |ui| {
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
                .frame(components::side())
                .show(ui, |ui| {
                    let closed = components::head(ui, |ui| title(ui, node));
                    components::rule(ui);
                    properties(ui, node);
                    closed
                })
                .inner;
            if closed {
                dispatch.emit(Select(vec![section_at]));
            }
        }

        let pane = components::bare(ui);
        let reveal = ui.data(|data| data.get_temp::<Vec<usize>>(reveal_id()));
        egui::CentralPanel::default().frame(pane).show(ui, |ui| {
            if whole {
                components::block(ui, |ui| title(ui, section));
                components::rule(ui);
                properties(ui, section);
                return;
            }

            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| components::block(ui, |ui| {
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
    let kind = components::dim(&node.kind);
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

    let id = egui::Id::new(("panel node", shown.panel, path.clone()));
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
    ui.label(components::dim(&node.kind));
}

fn properties(ui: &mut egui::Ui, node: &Node) {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| components::block(ui, |ui| components::fields(ui, &node.properties)));
}
