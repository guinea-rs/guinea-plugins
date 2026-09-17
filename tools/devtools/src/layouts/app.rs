use guinea::eframe::{Layout, LayoutCx};
use guinea::feature::FeatureInitContext;
use guinea_devtools_model::sessions::{Listening, Session, Sessions};

use crate::focus::{Focus, FocusFeature};
use crate::editor::{self, Editor};
use crate::memory::{EditorChoice, LastTab, Opened, PickEditor};
use crate::pages::elements::Elements;
use crate::pages::graph::Graphs;
use crate::pages::panels::Panels;
use crate::pages::trace::Traces;
use crate::routes::Route;
use crate::sessions::contracts::Live;
use crate::style;

pub struct App;

const TABS: [&str; 4] = ["Elements", "Graph", "Trace", "Panels"];

/// The tab titled `title` for `app`, or the first one for a title no tab has.
pub fn tab_named(title: &str, app: u64) -> Route {
    tab(TABS.iter().position(|tab| *tab == title).unwrap_or(0), app)
}

/// The page at `index` of [`TABS`], for `app`.
fn tab(index: usize, app: u64) -> Route {
    match index {
        0 => Route::Elements { app },
        1 => Route::Graphs { app },
        2 => Route::Traces { app },
        _ => Route::Panels { app },
    }
}

impl Layout for App {
    type Params = crate::routes::AppParams;
    type Installs = FocusFeature;

    fn install(ctx: &FeatureInitContext, params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&params.app)
    }

    fn render(cx: &mut LayoutCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, _) = cx.state::<Focus, _>();
        let (last, remember) = cx.state::<LastTab, _>();
        let (editor, _) = cx.state::<EditorChoice, _>();
        let nav = cx.navigate::<Route>();

        let current = [
            cx.child_is::<Elements>(),
            cx.child_is::<Graphs>(),
            cx.child_is::<Traces>(),
            cx.child_is::<Panels>(),
        ];
        let open = current.iter().position(|on| *on).unwrap_or(0);
        if current[open] && last.0 != TABS[open] {
            remember.emit(Opened(TABS[open]));
        }

        let page = cx.outlet();
        let ui = cx.ui();

        let (known, mut go) = {
            let sessions = live.read();
            (
                sessions.get(focus.app).is_some(),
                sessions.successor(focus.app).map(|id| tab(open, id)),
            )
        };

        egui::Panel::top("tabs")
            .resizable(false)
            .show_separator_line(false)
            .exact_size(30.0)
            .frame(style::side())
            .show(ui, |ui| {
                if let Some(index) = tabs(ui, open) {
                    go = Some(tab(index, focus.app));
                }
            });

        egui::Panel::bottom("status")
            .resizable(false)
            .exact_size(24.0)
            .frame(
                style::side().inner_margin(egui::Margin::symmetric(style::PADDING, 0)),
            )
            .show(ui, |ui| {
                let (app, picked) = status(ui, &live.read(), focus.app, editor.0);
                if let Some(id) = app {
                    go = Some(tab(open, id));
                }

                if let Some(picked) = picked {
                    remember.emit(PickEditor(picked));
                }
            });

        let pane = style::bare(ui);
        egui::CentralPanel::default().frame(pane).show(ui, |ui| {
            if known {
                page.draw(ui);
            } else {
                style::block(ui, |ui| ui.label(style::dim("this connection is gone")));
            }
        });

        if let Some(route) = go {
            nav.to(route);
        }
    }
}

/// A row of tabs, like a browser's devtools; the one clicked, if another.
fn tabs(ui: &mut egui::Ui, open: usize) -> Option<usize> {
    let mut clicked = None;
    let bar = ui.max_rect();
    let height = bar.height();

    let mut edge = ui.painter().clone();
    edge.set_clip_rect(bar);
    let line = ui.visuals().widgets.noninteractive.bg_stroke;
    edge.hline(bar.x_range(), bar.bottom() - line.width / 2.0, line);

    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

        for (index, title) in TABS.into_iter().enumerate() {
            let on = index == open;
            let font = egui::TextStyle::Button.resolve(ui.style());
            let galley =
                ui.painter()
                    .layout_no_wrap(title.to_string(), font, egui::Color32::PLACEHOLDER);

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(galley.size().x + 24.0, height),
                egui::Sense::click(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Button, true, on, title)
            });

            let visuals = ui.visuals();
            if response.hovered() && !on {
                ui.painter()
                    .rect_filled(rect, 0, visuals.widgets.hovered.weak_bg_fill);
            }
            let color = if on {
                visuals.strong_text_color()
            } else if response.hovered() {
                visuals.text_color()
            } else {
                visuals.weak_text_color()
            };

            if on {
                let underline = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.bottom() - 2.0),
                    rect.right_bottom(),
                );
                edge.rect_filled(underline, 0, style::ACCENT);
            }
            ui.painter()
                .galley(rect.center() - galley.size() / 2.0, galley, color);

            if response.clicked() && !on {
                clicked = Some(index);
            }
        }
    });

    clicked
}

/// Which application is shown and how it is connected, and the editor source
/// links open in; the application and the editor picked instead, if any.
fn status(
    ui: &mut egui::Ui,
    sessions: &Sessions,
    app: u64,
    editor: Editor,
) -> (Option<u64>, Option<Editor>) {
    let session = sessions.get(app);
    let mut picked = None;
    let mut chosen = None;

    ui.horizontal_centered(|ui| {
        let connected = session.is_some_and(|session| session.connected);
        let (dot, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().circle_filled(
            dot.center(),
            4.0,
            if connected { style::LIVE } else { style::GONE },
        );

        egui::ComboBox::from_id_salt("application")
            .selected_text(session.map(Session::name).unwrap_or_default())
            .show_ui(ui, |ui| {
                for other in sessions.by_id.values().rev() {
                    let label = if other.connected {
                        other.name()
                    } else {
                        format!("{} (gone)", other.name())
                    };

                    let on = session.is_some_and(|session| session.id == other.id);
                    if ui.selectable_label(on, label).clicked() && !on {
                        picked = Some(other.id);
                    }
                }
            });

        if let Some(session) = session {
            let info = &session.info;
            ui.label(style::dim(format!(
                "{} · pid {} · {}",
                info.backend, info.pid, info.version
            )));

            if !connected {
                ui.colored_label(style::GONE, "disconnected");
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(style::dim(match &sessions.listening {
                Listening::On(addr) => addr.clone(),
                other => other.describe(),
            }));
            ui.add_space(f32::from(style::PADDING));

            egui::ComboBox::from_id_salt("editor")
                .selected_text(editor.title())
                .show_ui(ui, |ui| {
                    for other in editor::installed() {
                        let on = *other == editor;
                        if ui.selectable_label(on, other.title()).clicked() && !on {
                            chosen = Some(*other);
                        }
                    }
                })
                .response
                .on_hover_text("where source links open");
            ui.label(style::dim("open in"));
        });
    });

    (picked, chosen)
}
