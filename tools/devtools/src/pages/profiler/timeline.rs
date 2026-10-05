//! One second on one time axis: every frame in it, what the application
//! recorded towards them, and their layout passes, each pass under the one
//! it ran inside.
//!
//! Dragging selects a stretch and sums what ran in it, ctrl and the wheel
//! zoom, a right or middle drag and shift with the wheel pan, a double click
//! fits the second again.

use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, pos2, vec2};
use guinea_devtools_model::native::NativeTree;
use guinea_devtools_model::profile::{BUDGET_US, Frame, Profile};
use guinea_devtools_model::samples::{self, Origin};
use guinea_devtools_model::protocol::Span;
use guinea_devtools_model::protocol::native::Pass;
use guinea_devtools_model::timers::Timers;
use guinea_devtools_model::words;

use super::flame::{self, NARROWEST_US, View, last_segment};
use super::{ms, shade, short};
use crate::components;
use crate::theme;

const LANE_TALL: f32 = 22.0;
const AXIS_TALL: f32 = 20.0;
const NAMES_WIDE: f32 = 64.0;
const GROUP_GAP: f32 = 8.0;
/// How many levels of nested layout passes the timeline shows.
const DEPTHS: usize = 40;
/// The least of what came before a frame brought into view; it shows the
/// records towards the frame, as far back as twice its length.
const LEAD_US: i64 = 3_000;
/// How often the tap samples the UI thread's stack.
const SAMPLE_PERIOD_US: i64 = 1_000;
/// How many levels of a sampled stack the timeline shows.
const STACK_DEPTHS: usize = 80;
/// How many of the functions sampled in a selection are listed.
const HEAVIEST: usize = 12;

/// How the timeline is looked at; each part names the second it was set on.
#[derive(Default)]
pub struct Lens {
    /// The stretch shown.
    view: Option<(i64, View)>,
    /// The stretch dragged over.
    selected: Option<(i64, View)>,
    /// Where the drag selecting it began.
    anchor: Option<i64>,
    /// The frame to bring into view when the timeline is drawn next.
    pub zoom_to: Option<i64>,
    /// Whose calls the stacks show.
    pub shown: Shown,
}

/// Whose calls the sampled stacks show, besides the application's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown {
    pub rust: bool,
    pub winui: bool,
    pub windows: bool,
    pub unknown: bool,
    /// Samples taken while the thread waited in the system.
    pub waits: bool,
}

impl Default for Shown {
    fn default() -> Self {
        Self {
            rust: true,
            winui: true,
            windows: false,
            unknown: false,
            waits: false,
        }
    }
}

impl Shown {
    pub fn shows(self, origin: Origin) -> bool {
        match origin {
            Origin::App => true,
            Origin::Rust => self.rust,
            Origin::WinUi => self.winui,
            Origin::Windows => self.windows,
            Origin::Unknown => self.unknown,
        }
    }

    /// The checkboxes that set it.
    pub fn edit(&mut self, ui: &mut egui::Ui) {
        for (on, name, about) in [
            (&mut self.rust, "Rust std", "std, core and alloc"),
            (&mut self.winui, "WinUI", "the Windows App SDK"),
            (&mut self.windows, "Windows", "modules under the Windows folder"),
            (&mut self.unknown, "no module", "addresses in no module the tap knew"),
            (
                &mut self.waits,
                "waiting",
                "samples taken while the thread waited for a message, an object or time",
            ),
        ] {
            ui.checkbox(on, name).on_hover_text(about);
        }
    }
}

/// Draws the second's frames; the frame clicked, if any.
#[allow(clippy::too_many_arguments)]
pub fn timeline(
    ui: &mut egui::Ui,
    profile: &Profile,
    second: i64,
    frames: &[&Frame],
    picked: Option<i64>,
    work: &[&Span],
    timers: &Timers,
    tree: Option<&NativeTree>,
    lens: &mut Lens,
) -> Option<i64> {
    let (Some(earliest), Some(latest)) = (frames.first(), frames.last()) else {
        return None;
    };
    let origin = profile.second_starts(second);

    let records: Vec<(i64, i64)> = work
        .iter()
        .map(|span| {
            let at = span.at as i64;
            (at, at + span.took.unwrap_or(0).max(1) as i64)
        })
        .collect();
    let mut passes: Vec<(i64, i64)> = Vec::new();
    let mut owned: Vec<(&Frame, &Pass)> = Vec::new();
    for frame in frames {
        for pass in &frame.passes {
            let at = frame.at_us + pass.at_us as i64;
            passes.push((at, at + pass.took_us.max(1) as i64));
            owned.push((frame, pass));
        }
    }

    let fitted = View {
        from: origin.min(earliest.at_us),
        to: (origin + 1_000_000).max(latest.end_us()),
    };
    let margin = (fitted.length() / 100).max(NARROWEST_US);
    let within = View {
        from: records.iter().map(|(at, _)| *at).fold(fitted.from, i64::min) - margin,
        to: records.iter().map(|(_, to)| *to).fold(fitted.to, i64::max) + margin,
    };
    let mut view = lens
        .view
        .filter(|(t, _)| *t == second)
        .map_or(fitted, |(_, view)| view);
    if let Some(at) = lens.zoom_to.take()
        && let Some(frame) = frames.iter().find(|frame| frame.at_us == at)
    {
        view = around(frame, frames, &records, within);
    }
    let mut selected = lens
        .selected
        .filter(|(t, _)| *t == second)
        .map(|(_, selected)| selected);

    let record_row = flame::rows(&records);
    let record_rows = record_row.iter().max().map_or(1, |row| row + 1);
    let depth = flame::nesting(&passes);
    let pass_rows = depth
        .iter()
        .max()
        .map_or(0, |deepest| (deepest + 1).min(DEPTHS));
    let frame_lane = record_rows;
    let shows = lens.shown;
    let sampled = flame::shown(
        &profile.samples_between(within.from, within.to),
        |function| shows.shows(profile.origin(function)),
        |function| !shows.waits && samples::waits(profile.function(function)),
    );
    let sampled: Vec<(i64, &[u32])> = sampled
        .iter()
        .map(|(at, stack)| (*at, stack.as_slice()))
        .collect();
    let runs = flame::runs(&sampled, SAMPLE_PERIOD_US);
    let stack_rows = runs
        .iter()
        .map(|run| run.depth + 1)
        .max()
        .unwrap_or(0)
        .min(STACK_DEPTHS);
    let stack_lane = frame_lane + 1 + pass_rows;
    let lanes = stack_lane + stack_rows;
    let gaps = 2 + usize::from(stack_rows > 0);

    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(
        vec2(
            width,
            AXIS_TALL + lanes as f32 * LANE_TALL + gaps as f32 * GROUP_GAP + 4.0,
        ),
        Sense::click_and_drag(),
    );
    let track = Rect::from_min_max(
        pos2(rect.left() + NAMES_WIDE, rect.top() + AXIS_TALL),
        rect.right_bottom(),
    );

    let per_px = view.length() as f32 / track.width().max(1.0);
    let shown = view.from;
    let moment = move |x: f32| {
        (shown + ((x - track.left()) * per_px) as i64).clamp(within.from, within.to)
    };
    let selecting = response.dragged_by(egui::PointerButton::Primary);
    if response.double_clicked() {
        view = fitted;
        selected = None;
    } else if response.clicked() {
        selected = None;
    } else if response.drag_started_by(egui::PointerButton::Primary) {
        lens.anchor = response
            .interact_pointer_pos()
            .map(|pointer| moment(pointer.x));
    } else if selecting {
        if let (Some(anchor), Some(pointer)) = (lens.anchor, response.interact_pointer_pos()) {
            let at = moment(pointer.x);
            selected = (at != anchor).then(|| View {
                from: anchor.min(at),
                to: anchor.max(at),
            });
        }
    } else if response.dragged() {
        view = view.panned((-response.drag_delta().x * per_px) as i64, within);
    }
    if let Some(pointer) = response.hover_pos() {
        let (zoom, scroll) = ui.input(|input| (input.zoom_delta(), input.smooth_scroll_delta));
        if zoom != 1.0 {
            view = view.zoomed(moment(pointer.x.max(track.left())), zoom, within);
        }
        if scroll.x != 0.0 {
            view = view.panned((-scroll.x * per_px) as i64, within);
        }
        ui.ctx().set_cursor_icon(if response.dragged() && !selecting {
            egui::CursorIcon::Grabbing
        } else {
            egui::CursorIcon::Crosshair
        });
    }
    lens.view = Some((second, view));
    lens.selected = selected.map(|selected| (second, selected));

    let painter = ui.painter_at(rect);
    let font = FontId::proportional(12.0);
    let length = view.length().max(1) as f32;
    let x = |at: i64| track.left() + (at - view.from) as f32 / length * track.width();

    let top_of = |index: usize| {
        let gaps = usize::from(index >= frame_lane)
            + usize::from(index > frame_lane)
            + usize::from(stack_rows > 0 && index >= stack_lane);
        track.top() + index as f32 * LANE_TALL + gaps as f32 * GROUP_GAP
    };
    let lane = |index: usize| {
        Rect::from_min_size(
            pos2(track.left(), top_of(index) + 1.0),
            vec2(track.width(), LANE_TALL - 2.0),
        )
    };
    let groups = [
        (0, record_rows, "guinea"),
        (frame_lane, frame_lane + 1, "frame"),
        (frame_lane + 1, stack_lane, "layout"),
        (stack_lane, lanes, "stacks"),
    ];
    for (index, (first, past, name)) in groups.into_iter().enumerate() {
        if first >= past {
            continue;
        }
        let band = Rect::from_x_y_ranges(
            rect.x_range(),
            top_of(first)..=top_of(past - 1) + LANE_TALL,
        );
        if index % 2 == 0 {
            painter.rect_filled(band, 0.0, theme::FIELD.gamma_multiply(0.6));
        }
        if index > 0 {
            painter.hline(
                rect.x_range(),
                band.top() - GROUP_GAP / 2.0,
                Stroke::new(1.0, theme::DIVIDER),
            );
        }
        painter.text(
            pos2(rect.left() + 4.0, lane(first).center().y),
            Align2::LEFT_CENTER,
            name,
            font.clone(),
            theme::MUTED,
        );
    }

    for frame in frames {
        let left = x(frame.at_us).max(track.left());
        let right = x(frame.end_us()).min(track.right());
        if right <= left {
            continue;
        }
        painter.rect_filled(
            Rect::from_x_y_ranges(left..=right, track.y_range()),
            0.0,
            theme::TEXT.gamma_multiply(0.05),
        );
        for edge in [x(frame.at_us), x(frame.end_us())] {
            if (track.left()..=track.right()).contains(&edge) {
                painter.vline(
                    edge,
                    track.y_range(),
                    Stroke::new(1.0, theme::MUTED.gamma_multiply(0.7)),
                );
            }
        }
    }

    let step = nice_step(length / 1000.0) as f64;
    let mut tick = ((view.from - origin) as f64 / 1000.0 / step).ceil() * step;
    while origin as f64 + tick * 1000.0 <= view.to as f64 {
        let at_x = x(origin + (tick * 1000.0).round() as i64);
        let shown = if tick.abs() < step / 2.0 { 0.0 } else { tick };
        painter.vline(at_x, track.y_range(), Stroke::new(1.0, theme::DIVIDER));
        painter.text(
            pos2(at_x, rect.top() + 2.0),
            Align2::CENTER_TOP,
            if step < 1.0 {
                format!("{shown:.2} ms")
            } else {
                format!("{shown:.0} ms")
            },
            font.clone(),
            theme::MUTED,
        );
        tick += step;
    }

    let mut hits: Vec<(Rect, String, Option<i64>)> = Vec::new();
    let mut drawn = vec![f32::MIN; lanes];
    let mut bar = |index: usize,
                   (from, to): (i64, i64),
                   color: Color32,
                   label: &str,
                   about: String,
                   frame: Option<i64>| {
        if x(from) >= track.right() || x(to) <= track.left() {
            return;
        }
        let left = x(from).max(track.left());
        let right = x(to).min(track.right()).max(left + 1.0);
        if right - left < 1.5 && left < drawn[index] + 1.0 {
            return;
        }
        drawn[index] = right;
        let shape = Rect::from_x_y_ranges(left..=right, lane(index).y_range());
        painter.rect_filled(shape, 2.0, color);
        if shape.width() > 24.0 {
            painter.with_clip_rect(shape.shrink2(vec2(4.0, 0.0))).text(
                pos2(shape.left() + 4.0, shape.center().y),
                Align2::LEFT_CENTER,
                label,
                font.clone(),
                theme::BACKGROUND,
            );
        }
        if index == frame_lane && frame.is_some_and(|at| picked == Some(at)) {
            painter.rect_stroke(
                shape.expand(1.0),
                2.0,
                Stroke::new(1.5, theme::TEXT),
                egui::StrokeKind::Outside,
            );
        }
        hits.push((shape, about, frame));
    };

    for ((span, interval), row) in work.iter().zip(&records).zip(&record_row) {
        let said = words::text(&words::sentence(&span.point, timers));
        let took = span
            .took
            .map(|took| format!(" · {:.2} ms", ms(took)))
            .unwrap_or_default();
        bar(
            *row,
            *interval,
            theme::kind_color(words::Kind::of(&span.point)),
            &said,
            format!("{said}{took}\nat {}", profile.when(interval.0)),
            None,
        );
    }

    for frame in frames {
        let label = format!("{:.1} ms", ms(frame.took_us));
        bar(
            frame_lane,
            (frame.at_us, frame.end_us()),
            shade(frame.took_us),
            &label,
            format!(
                "{} · {label}\nmeasure {:.1} ms · arrange {:.1} ms · the rest {:.1} ms · {} passes",
                profile.when(frame.at_us),
                ms(frame.measure_us),
                ms(frame.arrange_us),
                ms(frame
                    .took_us
                    .saturating_sub(frame.measure_us + frame.arrange_us)),
                frame.passes.len()
            ),
            Some(frame.at_us),
        );
    }

    for (((frame, pass), interval), level) in owned.iter().zip(&passes).zip(&depth) {
        if *level >= DEPTHS {
            continue;
        }
        let element = tree
            .and_then(|tree| tree.get(pass.element))
            .map(|element| short(&element.kind));
        let label = format!(
            "{} {:.2} ms",
            element.unwrap_or(&pass.kind),
            ms(pass.took_us)
        );
        let color = if pass.kind == "measure" {
            theme::SKY
        } else {
            theme::VIOLET
        };
        bar(
            frame_lane + 1 + level,
            *interval,
            color,
            &label,
            format!(
                "{} · {} · {:.2} ms\nstarted {:+.2} ms into its frame",
                pass.kind,
                element.unwrap_or("element not in the tree"),
                ms(pass.took_us),
                ms(pass.at_us)
            ),
            Some(frame.at_us),
        );
    }

    for run in &runs {
        if run.depth >= STACK_DEPTHS {
            continue;
        }
        let name = profile.function(run.function);
        let module = profile
            .module(run.function)
            .map(|path| format!("\n{path}"))
            .unwrap_or_default();
        bar(
            stack_lane + run.depth,
            (run.from, run.to),
            tint(run.function),
            last_segment(name),
            format!(
                "{name}{module}\n{:.1} ms · {} samples",
                ms((run.to - run.from) as u64),
                run.samples
            ),
            None,
        );
    }

    for frame in frames.iter().filter(|frame| frame.over()) {
        let budget = x(frame.at_us + BUDGET_US as i64);
        if (track.left()..=track.right()).contains(&budget) {
            painter.vline(budget, track.y_range(), Stroke::new(1.5, theme::RED));
        }
    }

    if let Some(selected) = selected {
        let left = x(selected.from).max(track.left());
        let right = x(selected.to).min(track.right());
        if right > left {
            let band = Rect::from_x_y_ranges(left..=right, track.y_range());
            painter.rect_filled(band, 0.0, theme::TEXT.gamma_multiply(0.12));
            for edge in [left, right] {
                painter.vline(edge, track.y_range(), Stroke::new(1.0, theme::TEXT));
            }
            let label = painter.layout_no_wrap(
                format!("{:.2} ms", ms(selected.length() as u64)),
                font.clone(),
                theme::BACKGROUND,
            );
            let at = pos2(band.center().x - label.size().x / 2.0, rect.top() + 2.0);
            painter.rect_filled(
                Rect::from_min_size(at, label.size()).expand2(vec2(4.0, 1.0)),
                2.0,
                theme::TEXT,
            );
            painter.galley(at, label, theme::BACKGROUND);
        }
    }

    let clicked = response
        .clicked()
        .then(|| response.interact_pointer_pos())
        .flatten()
        .and_then(|pointer| {
            hits.iter()
                .rev()
                .find(|(shape, _, _)| shape.contains(pointer))
                .and_then(|(_, _, frame)| *frame)
        });

    if !response.dragged()
        && let Some(pointer) = response.hover_pos()
    {
        match hits.iter().rev().find(|(shape, _, _)| shape.contains(pointer)) {
            Some((_, about, _)) => response.on_hover_text(about.as_str()),
            None => response.on_hover_text(
                "drag selects · ctrl+wheel zooms · right drag or shift+wheel pans · double click fits the second",
            ),
        };
    }

    if let Some(selected) = selected {
        let of_kind = |kind: &str| -> Vec<(i64, i64)> {
            owned
                .iter()
                .zip(&passes)
                .filter(|((_, pass), _)| pass.kind == kind)
                .map(|(_, interval)| *interval)
                .collect()
        };
        let framed: Vec<(i64, i64)> = frames
            .iter()
            .map(|frame| (frame.at_us, frame.end_us()))
            .collect();
        let measure = flame::busy(selected, &of_kind("measure"));
        let arrange = flame::busy(selected, &of_kind("arrange"));
        let framed = flame::busy(selected, &framed);
        let recorded = flame::busy(selected, &records);
        ui.label(components::mono(format!(
            "{:.2} ms · {} frames {:.2} ms · measure {:.2} ms in {} passes · arrange {:.2} ms in {} passes · {} records",
            ms(selected.length() as u64),
            framed.count,
            ms(framed.us as u64),
            ms(measure.us as u64),
            measure.count,
            ms(arrange.us as u64),
            arrange.count,
            recorded.count
        )));

        let inside: Vec<(i64, &[u32])> = sampled
            .iter()
            .filter(|(at, _)| (selected.from..selected.to).contains(at))
            .copied()
            .collect();
        let sample_ms = |samples: usize| ms(samples as u64 * SAMPLE_PERIOD_US as u64);
        for weight in flame::heaviest(&inside).iter().take(HEAVIEST) {
            ui.label(components::mono(format!(
                "{:>8.1} ms own {:>8.1} ms in all  {}",
                sample_ms(weight.own),
                sample_ms(weight.total),
                profile.function(weight.function)
            )));
        }
    }

    clicked
}

/// A sampled function's colour: the same function, the same colour.
fn tint(function: u32) -> Color32 {
    const PALETTE: [Color32; 4] = [theme::PEACH, theme::PINK, theme::CLAY, theme::LILAC];
    let pick = (function as usize).wrapping_mul(2_654_435_761) >> 7;
    PALETTE[pick % PALETTE.len()].gamma_multiply(0.85)
}

/// `frame` with the records towards it before it, as far back as twice its
/// length.
fn around(frame: &Frame, frames: &[&Frame], records: &[(i64, i64)], within: View) -> View {
    let since = frames
        .iter()
        .take_while(|earlier| earlier.at_us < frame.at_us)
        .last()
        .map_or(i64::MIN, |earlier| earlier.end_us());
    let first = records
        .iter()
        .map(|(at, _)| *at)
        .filter(|at| *at > since && *at < frame.at_us)
        .min()
        .unwrap_or(frame.at_us);
    let took = frame.took_us as i64;
    let lead = (frame.at_us - first).clamp(LEAD_US, 2 * took.max(10_000));
    let margin = (took / 20).max(NARROWEST_US);
    View {
        from: frame.at_us - lead - margin,
        to: frame.end_us() + margin,
    }
    .panned(0, within)
}

fn nice_step(length_ms: f32) -> f32 {
    [
        0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0,
        1000.0,
    ]
    .into_iter()
    .find(|step| length_ms / step <= 10.0)
    .unwrap_or(2000.0)
}
