//! What the application did: the chains of records that set each other off,
//! by what started them, or every record in turn; and one record, whole.

use std::sync::{Arc, Mutex, PoisonError};

use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_core::feature::Dispatch;
use guinea_devtools_model::chains::Stream;
use guinea_devtools_model::sessions::Session;
use guinea_devtools_model::trace::{self, Consequence, Origins, Query, Record, Shown};
use guinea_devtools_model::words::Target;

use crate::focus::{Focus, Show};
use crate::memory::EditorChoice;
use crate::pages::lines::{Go, Line, row, row_height, sentence};
use crate::pages::streams::{self, Open};
use crate::routes::Route;
use crate::sessions::contracts::Live;
use crate::style;
use crate::trace_view::{
    Filter, Freeze, KINDS, OpenStream, Select, Toggle, TraceView, TraceViewFeature,
};

pub struct Traces;

impl Page for Traces {
    type Params = crate::routes::TracesParams;
    type Installs = TraceViewFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn render(cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, show) = cx.state::<Focus, _>();
        let (view, dispatch) = cx.state::<TraceView, _>();
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
                .frame(style::bare(ui))
                .show(ui, |ui| toolbar(ui, &view, &dispatch, log.len(), log.dropped));

            egui::Panel::left("trace-streams")
                .resizable(true)
                .size_range(160.0..=360.0)
                .default_size(220.0)
                .frame(style::side())
                .show(ui, |ui| streams::rail(ui, session, &view.stream, &mut go));

            let record = view
                .selected
                .and_then(|id| trace::record(session.reading(), id, &query));
            if let Some(record) = &record {
                let closed = egui::Panel::right("trace-detail")
                    .resizable(true)
                    .size_range(320.0..=720.0)
                    .default_size(460.0)
                    .frame(style::side())
                    .show(ui, |ui| detail(ui, record, &mut go))
                    .inner;
                if closed {
                    go = Some(Go::Close);
                }
            }

            egui::CentralPanel::default().frame(style::bare(ui)).show(ui, |ui| {
                if records {
                    style::block(ui, |ui| list(ui, session, &view, &query, &mut go));
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
                nav.to(Route::Panels { app });
            }
        }
    }
}

fn jump_id() -> egui::Id {
    egui::Id::new("trace-jump")
}

fn toolbar(ui: &mut egui::Ui, view: &TraceView, dispatch: &Dispatch, kept: usize, dropped: u64) {
    style::block(ui, |ui| tools(ui, view, dispatch, kept, dropped));
}

fn tools(ui: &mut egui::Ui, view: &TraceView, dispatch: &Dispatch, kept: usize, dropped: u64) {
    ui.horizontal_wrapped(|ui| {
        let mut query = view.query.clone();
        if ui
            .add(
                egui::TextEdit::singleline(&mut query)
                    .hint_text("filter")
                    .desired_width(200.0),
            )
            .changed()
        {
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

        style::rule(ui);

        for kind in KINDS {
            let on = view.shows(kind);
            let text = egui::RichText::new(kind).color(if on {
                style::kind_color(kind)
            } else {
                ui.visuals().weak_text_color()
            });
            if ui.selectable_label(on, text).clicked() {
                dispatch.emit(Toggle(kind));
            }
        }

        style::rule(ui);

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
fn list(ui: &mut egui::Ui, session: &Session, view: &TraceView, query: &Query, go: &mut Option<Go>) {
    let log = &session.trace;
    let reading = session.reading();

    let kept: Arc<Mutex<Shown>> = ui.data_mut(|data| {
        data.get_temp_mut_or_default::<Arc<Mutex<Shown>>>(egui::Id::new(("trace-shown", session.id)))
            .clone()
    });
    let mut shown = kept.lock().unwrap_or_else(PoisonError::into_inner);
    shown.refresh(reading, query);

    ui.horizontal(|ui| {
        ui.label(style::dim(format!("{} of {}", shown.len(), log.len())));
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
fn detail(ui: &mut egui::Ui, record: &Record, go: &mut Option<Go>) -> bool {
    let closed = style::head(ui, |ui| {
        ui.horizontal_wrapped(|ui| sentence(ui, &record.row.words, go));

        ui.horizontal(|ui| {
            ui.colored_label(style::kind_color(&record.row.kind), &record.row.kind);
            ui.label(style::dim(&record.row.when));
            if let Some(took) = &record.row.took {
                ui.label(style::dim(format!("took {took}")));
            }
        });
    });

    let height = row_height(ui);

    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        style::heading(ui, "Where it came from");
        style::block(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            origins(ui, &record.origins, record.row.id, height, go);
        });

        style::heading(ui, "What it set off");
        style::block(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            if record.set_off.is_empty() {
                ui.label(style::dim("nothing observed"));
            }

            consequences(ui, &record.set_off, height, go);

            if record.cut {
                ui.label(style::dim("… more than devtools draw at once"));
            }
        });
    });

    closed
}

/// How far a chain of causes steps to the right before it goes straight down.
const MAX_INDENT: usize = 8;

fn origins(ui: &mut egui::Ui, origins: &Origins, selected: u64, height: f32, go: &mut Option<Go>) {
    if let Some(note) = &origins.note {
        ui.label(style::dim(note));
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
                ui.label(style::dim(format!("… and {} more in between", step.more)));
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
                ui.make_persistent_id(("set off", record.id)),
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
