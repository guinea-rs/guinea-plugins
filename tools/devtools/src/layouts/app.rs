use guinea::eframe::{Layout, LayoutCx};
use guinea::feature::FeatureInitContext;
use guinea_devtools_model::sessions::{Listening, Session, Sessions};

use crate::components;
use crate::features::editor::contracts::{Editor, EditorChoice, PickEditor};
use crate::features::editor::detect;
use crate::features::focus::FocusFeature;
use crate::features::focus::contracts::Focus;
use crate::features::tab::contracts::{LastTab, Opened};
use crate::pages::application::Application;
use crate::pages::elements::Elements;
use crate::pages::graph::Graphs;
use crate::pages::trace::Traces;
use crate::routes::Route;
use crate::features::sessions::contracts::Live;
use crate::theme;

#[derive(Default)]
pub struct App;

const TABS: [&str; 4] = ["Elements", "Graph", "Trace", "Application"];

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
        _ => Route::Application { app },
    }
}

impl Layout for App {
    type Params = crate::routes::AppParams;
    type Installs = FocusFeature;

    fn install(ctx: &FeatureInitContext, params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&params.app)
    }

    fn render(&mut self, cx: &mut LayoutCx<'_, Self>) {
        let (live, _) = cx.read::<Live>();
        let (focus, _) = cx.read::<Focus>();
        let (last, remember) = cx.read::<LastTab>();
        let (editor, _) = cx.read::<EditorChoice>();
        let nav = cx.navigate::<Route>();

        let current = [
            cx.child_is::<Elements>(),
            cx.child_is::<Graphs>(),
            cx.child_is::<Traces>(),
            cx.child_is::<Application>(),
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
            .exact_size(components::TABS_HEIGHT)
            .frame(components::side())
            .show(ui, |ui| {
                if let Some(index) = components::tabs(ui, "app-tabs", &TABS, open) {
                    go = Some(tab(index, focus.app));
                }
            });

        egui::Panel::bottom("status")
            .resizable(false)
            .exact_size(24.0)
            .frame(
                components::side().inner_margin(egui::Margin::symmetric(components::PADDING, 0)),
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

        let pane = components::bare(ui);
        egui::CentralPanel::default().frame(pane).show(ui, |ui| {
            if known {
                page.draw(ui);
            } else {
                components::block(ui, |ui| ui.label(components::dim("this connection is gone")));
            }
        });

        if let Some(route) = go {
            nav.to(route);
        }
    }
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
            if connected { theme::LIVE } else { theme::GONE },
        );

        let name = session.map(Session::name).unwrap_or_default();
        components::select(ui, "application", name, |ui| {
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
            ui.label(components::dim(format!(
                "{} · pid {} · {}",
                info.backend, info.pid, info.version
            )));

            if !connected {
                ui.colored_label(theme::GONE, "disconnected");
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(components::dim(match &sessions.listening {
                Listening::On(addr) => addr.clone(),
                other => other.describe(),
            }));
            ui.add_space(f32::from(components::PADDING));

            components::select(ui, "editor", editor.title(), |ui| {
                for other in detect::installed() {
                    let on = *other == editor;
                    if ui.selectable_label(on, other.title()).clicked() && !on {
                        chosen = Some(*other);
                    }
                }
            })
            .on_hover_text("where source links open");
            ui.label(components::dim("open in"));
        });
    });

    (picked, chosen)
}
