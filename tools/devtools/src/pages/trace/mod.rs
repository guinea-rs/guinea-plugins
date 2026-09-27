//! What the application did: the chains of records that set each other off,
//! by what started them, or every record in turn; and one record, whole.

use std::sync::{Arc, Mutex, PoisonError};

use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_core::feature::Dispatch;
use guinea_devtools_model::chains::Stream;
use guinea_devtools_model::sessions::Session;
use guinea_devtools_model::trace::{self, Consequence, Origins, Query, Record, Shown};
use guinea_devtools_model::words::{Kind, Target};

mod lines;
mod streams;

use lines::{Go, Line, row, row_height, sentence};
use streams::Open;

use crate::components;
use crate::components::source;
use crate::features::editor::contracts::{Editor, EditorChoice};
use crate::features::focus::contracts::{Focus, Show};
use crate::features::trace::TraceFeature;
use crate::features::trace::contracts::{Filter, Freeze, OpenStream, Select, Toggle, TraceState};
use crate::features::sessions::contracts::Live;
use crate::routes::Route;
use crate::theme;

#[derive(Default)]
pub struct Traces;

impl Page for Traces {
    type Params = crate::routes::TracesParams;
    type Installs = TraceFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, show) = cx.state::<Focus, _>();
        let (view, dispatch) = cx.state::<TraceState, _>();
        let (editor, _) = cx.state::<EditorChoice, _>();
        let nav = cx.navigate::<Route>();
        let ui = cx.ui();

        let query = view.query();
        let records = view.stream == Stream::Records;
        let mut go = None;

        let kept = {
            let sessions = live.read();
            let Some(session) = sessions.get(focus.app) else {
                return;
            };
            let log = &session.trace;

            egui::Panel::top("trace-tools")
                .resizable(false)
                .frame(components::bare(ui))
                .show(ui, |ui| toolbar(ui, &view, &dispatch, log.len(), log.dropped));

            egui::Panel::left("trace-streams")
                .resizable(true)
                .size_range(160.0..=360.0)
                .default_size(220.0)
                .frame(components::side())
                .show(ui, |ui| streams::rail(ui, session, &view.stream, &mut go));

            let record = view
                .selected
                .and_then(|id| trace::record(session.reading(), id, &query));
            if let Some(record) = &record {
                let closed = egui::Panel::right("trace-detail")
                    .resizable(true)
                    .size_range(320.0..=720.0)
                    .default_size(460.0)
                    .frame(components::side())
                    .show(ui, |ui| detail(ui, record, editor.0, &mut go))
                    .inner;
                if closed {
                    go = Some(Go::Close);
                }
            }

            egui::CentralPanel::default().frame(components::bare(ui)).show(ui, |ui| {
                if records {
                    components::block(ui, |ui| list(ui, session, &view, &query, &mut go));
                } else {
                    streams::stream(
                        ui,
                        session,
                        &view.stream,
                        &view.query,
                        view.selected,
                        editor.0,
                        &mut go,
                    );
                }
            });

            log.len()
        };

        let app = focus.app;
        match go {
            None => {}
            Some(Go::Close) => dispatch.emit(Select(None)),
            Some(Go::Select(id)) => dispatch.emit(Select(Some(id))),
            Some(Go::Record(id)) => {
                dispatch.emit(Select(Some(id)));
                if records {
                    dispatch.emit(Freeze(Some(kept)));
                    ui.data_mut(|data| data.insert_temp(jump_id(), id));
                }
            }
            Some(Go::Open(stream)) => dispatch.emit(OpenStream(stream)),
            Some(Go::Toggle(key)) => ui.data_mut(|data| {
                let open = data.get_temp_mut_or_default::<Open>(streams::open_id());
                if !open.remove(&key) {
                    open.insert(key);
                }
            }),
            Some(Go::Link(Target::Stream(id))) => {
                if let Ok(stream) = id.parse() {
                    dispatch.emit(OpenStream(stream));
                }
            }
            Some(Go::Link(Target::Records(name))) => {
                dispatch.emit(OpenStream(Stream::Records));
                dispatch.emit(Filter(name));
            }
            Some(Go::Link(target @ (Target::Actor(_) | Target::Reducer(_)))) => {
                show.emit(Show(target));
                nav.to(Route::Elements { app });
            }
            Some(Go::Link(target @ Target::Key(_))) => {
                show.emit(Show(target));
                nav.to(Route::Application { app });
            }
        }
    }
}

fn jump_id() -> egui::Id {
    egui::Id::new("trace-jump")
}

fn toolbar(ui: &mut egui::Ui, view: &TraceState, dispatch: &Dispatch, kept: usize, dropped: u64) {
    components::block(ui, |ui| tools(ui, view, dispatch, kept, dropped));
}

fn tools(ui: &mut egui::Ui, view: &TraceState, dispatch: &Dispatch, kept: usize, dropped: u64) {
    ui.horizontal_wrapped(|ui| {
        let mut query = view.query.clone();
        if components::search(ui, &mut query, "filter", 220.0).changed() {
            dispatch.emit(Filter(query));
        }

        if dropped > 0 {
            ui.colored_label(
                crate::theme::CLAY,
                format!("{dropped} dropped by the application"),
            );
        }

        if view.stream != Stream::Records {
            return;
        }

        components::rule(ui);

        for kind in Kind::ALL {
            let on = view.shows(kind);
            let text = egui::RichText::new(kind.name()).color(if on {
                theme::kind_color(kind)
            } else {
                ui.visuals().weak_text_color()
            });
            if ui.selectable_label(on, text).clicked() {
                dispatch.emit(Toggle(kind));
            }
        }

        components::rule(ui);

        let frozen = view.frozen.is_some();
        if ui
            .selectable_label(frozen, if frozen { "▶ follow" } else { "⏸ freeze" })
            .clicked()
        {
            dispatch.emit(Freeze(if frozen { None } else { Some(kept) }));
        }
    });
}

/// Every record the query lets through, one per line.
fn list(ui: &mut egui::Ui, session: &Session, view: &TraceState, query: &Query, go: &mut Option<Go>) {
    let log = &session.trace;
    let reading = session.reading();

    let kept: Arc<Mutex<Shown>> = ui.data_mut(|data| {
        data.get_temp_mut_or_default::<Arc<Mutex<Shown>>>(egui::Id::new(("trace-shown", session.id)))
            .clone()
    });
    let mut shown = kept.lock().unwrap_or_else(PoisonError::into_inner);
    shown.refresh(reading, query);

    ui.horizontal(|ui| {
        ui.label(components::dim(format!("{} of {}", shown.len(), log.len())));
        if !view.query.is_empty() && ui.link("show everything").clicked() {
            *go = Some(Go::Link(Target::Records(String::new())));
        }
    });

    let height = row_height(ui);
    ui.spacing_mut().item_spacing.y = 0.0;

    let mut area = egui::ScrollArea::vertical()
        .auto_shrink(false)
        .stick_to_bottom(view.frozen.is_none());
    let jump = ui.data_mut(|data| data.remove_temp::<u64>(jump_id()));
    if let Some(index) = jump.and_then(|id| shown.position(id)) {
        area = area.vertical_scroll_offset((index as f32 * height - 4.0 * height).max(0.0));
    }

    area.show_rows(ui, height, shown.len(), |ui, range| {
        ui.spacing_mut().item_spacing.y = 0.0;

        for span in shown.spans(log, range) {
            let line = Line::record(ui, &trace::row(reading, span));
            row(ui, height, view.selected == Some(span.id), go, Go::Select(span.id), line);
        }
    });
}

/// One record, whole; whether it was closed.
fn detail(ui: &mut egui::Ui, record: &Record, editor: Editor, go: &mut Option<Go>) -> bool {
    let closed = components::head(ui, |ui| {
        ui.horizontal_wrapped(|ui| sentence(ui, &record.row.words, go));

        ui.horizontal(|ui| {
            ui.colored_label(theme::kind_color(record.row.kind), record.row.kind.name());
            ui.label(components::dim(&record.row.when));
            if let Some(took) = &record.row.took {
                ui.label(components::dim(format!("took {took}")));
            }
        });

        if let Some(written) = &record.written {
            ui.horizontal(|ui| {
                ui.label(components::dim("written at"));
                source::link(ui, written, editor);
            });
        }
    });

    let height = row_height(ui);

    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        components::heading(ui, "Where it came from");
        components::block(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            origins(ui, &record.origins, record.row.id, height, go);
        });

        components::heading(ui, "What it set off");
        components::block(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            if record.set_off.is_empty() {
                ui.label(components::dim("nothing observed"));
            }

            consequences(ui, &record.set_off, height, go);

            if record.cut {
                ui.label(components::dim("… more than devtools draw at once"));
            }
        });
    });

    closed
}

/// How far a chain of causes steps to the right before it goes straight down.
const MAX_INDENT: usize = 8;

fn origins(ui: &mut egui::Ui, origins: &Origins, selected: u64, height: f32, go: &mut Option<Go>) {
    if let Some(note) = &origins.note {
        ui.label(components::dim(note));
    }

    for (at, step) in origins.steps.iter().enumerate() {
        let depth = at.min(MAX_INDENT);

        let mut line = Line::new(ui);
        line.indent(depth as f32 * 12.0);
        line.weak(if at == 0 { "  " } else { "└ " });
        line.words(&step.row.words);
        row(ui, height, step.row.id == selected, go, Go::Record(step.row.id), line);

        let indent = (depth + 1) as f32 * 12.0 + 12.0;
        for between in &step.between {
            ui.scope(|ui| {
                ui.set_opacity(0.55);

                let mut line = Line::new(ui);
                line.indent(indent);
                line.weak("· ");
                line.words(&between.words);
                row(ui, height, false, go, Go::Record(between.id), line);
            });
        }

        if step.more > 0 {
            ui.horizontal(|ui| {
                ui.add_space(indent);
                ui.label(components::dim(format!("… and {} more in between", step.more)));
            });
        }
    }
}

fn consequences(ui: &mut egui::Ui, set_off: &[Consequence], height: f32, go: &mut Option<Go>) {
    for consequence in set_off {
        let record = &consequence.row;
        let line = |ui: &egui::Ui| {
            let mut line = Line::new(ui);
            line.words(&record.words);
            line.took(record.took.as_ref());

            line
        };

        if consequence.children.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                let line = line(ui);
                row(ui, height, false, go, Go::Record(record.id), line);
            });
        } else {
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                egui::Id::new(("set off", record.id)),
                true,
            )
            .show_header(ui, |ui| {
                let line = line(ui);
                row(ui, height, false, go, Go::Record(record.id), line)
            })
            .body(|ui| consequences(ui, &consequence.children, height, go));
        }
    }
}
