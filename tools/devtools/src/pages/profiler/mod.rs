//! Where a second's time went: every second the inspector's ring still
//! holds, the frames of the one picked side by side, all of them on one
//! timeline beside what the application recorded towards them, and for the
//! frame picked, its records and costliest layout passes.
//!
//! While the page is open it reads the ring again every few seconds; the ring
//! records all along, so nothing has to be started before the stutter.

mod flame;
mod timeline;

use std::time::{Duration, Instant};

use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, pos2, vec2};
use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_devtools_model::native::NativeTree;
use guinea_devtools_model::profile::{BUDGET_US, Frame, Profile, Second};
use guinea_devtools_model::protocol::Span;
use guinea_devtools_model::sessions::{Session, Sessions};
use guinea_devtools_model::timers::Timers;
use guinea_devtools_model::words;
use guinea_devtools_protocol::{Capability, Command};

use crate::components;
use crate::features::focus::contracts::Focus;
use crate::features::sessions::contracts::Live;
use crate::theme;
use timeline::{Lens, timeline};

/// How often the ring is read again while the page is open.
const EVERY: Duration = Duration::from_secs(2);
/// Twice the budget: a frame past it lost two vsyncs.
const TWICE_US: u64 = 2 * BUDGET_US;
const SECONDS_TALL: f32 = 34.0;
const FRAMES_TALL: f32 = 52.0;

#[derive(Default)]
pub struct Profiler {
    second: Option<i64>,
    frame: Option<i64>,
    lens: Lens,
    paused: bool,
    /// Whether the inspector was asked to sample the UI thread's stack.
    sampling: bool,
    read_at: Option<Instant>,
    attach_error: Option<String>,
}

impl Page for Profiler {
    type Params = crate::routes::ProfilerParams;
    type Installs = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.read::<Live>();
        let (focus, _) = cx.read::<Focus>();
        let ui = cx.ui();

        let sessions = live.read();
        let Some(app) = sessions.get(focus.app) else {
            return;
        };
        let inspector = sessions.native_for(app.id);
        let traced = sessions.clocked(app.id).unwrap_or(app);

        if let Some(inspector) =
            inspector.filter(|inspector| inspector.info.can(Capability::NativePerf))
        {
            let due = self.read_at.is_none_or(|at| at.elapsed() >= EVERY);
            if !self.paused && due {
                let _ = live
                    .hub
                    .send_in(&sessions, inspector.id, Command::NativePerfCapture);
                self.read_at = Some(Instant::now());
            }
            if !self.paused {
                ui.ctx().request_repaint_after(EVERY);
            }
        }

        let asked = egui::Panel::top("profiler-tools")
            .resizable(false)
            .exact_size(30.0)
            .frame(components::side())
            .show(ui, |ui| self.toolbar(ui, &sessions, app, inspector))
            .inner;
        if let (Some(command), Some(inspector)) = (asked, inspector) {
            let _ = live.hub.send_in(&sessions, inspector.id, command);
        }

        let profile = match sessions.profile(app.id) {
            Ok(profile) if !profile.frames.is_empty() => profile,
            Ok(_) => {
                components::block(ui, |ui| {
                    ui.label(components::dim("no frames in the ring yet"))
                });
                return;
            }
            Err(why) => {
                components::block(ui, |ui| ui.label(components::dim(why)));
                return;
            }
        };
        let tree = inspector.map(|inspector| &inspector.inspection.tree);

        let seconds = profile.seconds();
        let second = self
            .second
            .filter(|t| seconds.iter().any(|second| second.t == *t));

        components::heading(ui, "seconds");
        if let Some(picked) =
            components::block(ui, |ui| seconds_track(ui, &profile, &seconds, second))
        {
            self.second = Some(picked);
            self.frame = None;
        }

        let Some(second) = second else {
            return;
        };
        let frames: Vec<&Frame> = profile.frames_in(second).collect();
        let frame = self
            .frame
            .filter(|at| frames.iter().any(|frame| frame.at_us == *at));

        components::heading(
            ui,
            &format!("{} · {} frames", profile.second_named(second), frames.len()),
        );
        if let Some(picked) = components::block(ui, |ui| frame_strip(ui, &profile, &frames, frame))
        {
            self.frame = Some(picked);
            self.lens.zoom_to = Some(picked);
        }

        let work: Vec<&Span> = frames
            .iter()
            .flat_map(|frame| profile.work(frame, &traced.trace))
            .collect();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if let Some(clicked) = components::block(ui, |ui| {
                    timeline(
                        ui,
                        &profile,
                        second,
                        &frames,
                        frame,
                        &work,
                        &traced.timers,
                        tree,
                        &mut self.lens,
                    )
                }) {
                    self.frame = Some(clicked);
                }

                let Some(frame) = frame.and_then(|at| profile.frame(at)) else {
                    return;
                };
                let own = profile.work(frame, &traced.trace);
                components::heading(
                    ui,
                    &format!(
                        "{} · {:.1} ms · measure {:.1} ms · arrange {:.1} ms · {} passes · {} records",
                        profile.when(frame.at_us),
                        ms(frame.took_us),
                        ms(frame.measure_us),
                        ms(frame.arrange_us),
                        frame.passes.len(),
                        own.len()
                    ),
                );
                components::block(ui, |ui| detail(ui, frame, &own, &traced.timers, tree));
            });
    }
}

impl Profiler {
    fn toolbar(
        &mut self,
        ui: &mut egui::Ui,
        sessions: &Sessions,
        app: &Session,
        inspector: Option<&Session>,
    ) -> Option<Command> {
        let mut asked = None;
        ui.horizontal_centered(|ui| {
            let Some(inspector) = inspector else {
                ui.label(components::dim(
                    "frames come from the native inspector, and none is attached",
                ));
                #[cfg(windows)]
                if app.connected && ui.button("attach the XAML inspector").clicked() {
                    self.attach_error =
                        guinea_devtools_hub::attach_native_in(sessions, app.id).err();
                }
                #[cfg(not(windows))]
                let _ = (sessions, app);
                if let Some(error) = &self.attach_error {
                    ui.label(egui::RichText::new(error).color(ui.visuals().warn_fg_color));
                }
                return;
            };

            let label = if self.paused {
                "paused"
            } else {
                "reading the ring"
            };
            if ui
                .add(egui::Button::selectable(!self.paused, label))
                .clicked()
            {
                self.paused = !self.paused;
            }
            if self.paused && ui.button("read it now").clicked() {
                self.read_at = None;
                self.paused = false;
            }
            if inspector.info.can(Capability::NativeSamples)
                && ui
                    .add(egui::Button::selectable(self.sampling, "sampling stacks"))
                    .on_hover_text(
                        "every thread's stack while it runs, and the UI thread's even while it waits, a thousand times a second",
                    )
                    .clicked()
            {
                self.sampling = !self.sampling;
                asked = Some(Command::NativeSampling { on: self.sampling });
            }
            if !inspector.inspection.sampled.samples.is_empty() {
                let threads = sessions
                    .profile(app.id)
                    .map(|profile| profile.threads())
                    .unwrap_or_default();
                ui.menu_button("shown", |ui| self.lens.edit(ui, &threads))
                    .response
                    .on_hover_text(
                        "whose calls the stacks show, besides the application's, and on which threads",
                    );
            }
            ui.separator();
            ui.label(components::dim(format!(
                "{} frames kept · {} in the last read · {} stack samples · {}",
                inspector.inspection.kept.len(),
                inspector.inspection.frames.len(),
                inspector.inspection.sampled.samples.len(),
                inspector.info.backend
            )));
            if let Some((command, reason)) = &inspector.inspection.refused {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("{command}: {reason}"))
                        .color(ui.visuals().warn_fg_color),
                );
            }
        });
        asked
    }
}

fn ms(us: u64) -> f64 {
    us as f64 / 1000.0
}

fn shade(took_us: u64) -> Color32 {
    if took_us > TWICE_US {
        theme::RED
    } else if took_us > BUDGET_US {
        theme::PEACH
    } else {
        theme::GREEN.gamma_multiply(0.55)
    }
}

/// A cell per second, oldest at the left; the one clicked, if any.
fn seconds_track(
    ui: &mut egui::Ui,
    profile: &Profile,
    seconds: &[Second],
    picked: Option<i64>,
) -> Option<i64> {
    let (Some(first), Some(last)) = (seconds.first(), seconds.last()) else {
        return None;
    };
    let span = (last.t - first.t + 1).max(1) as usize;
    let width = ui.available_width();
    let cell = (width / span as f32).clamp(4.0, 28.0);

    let mut clicked = None;
    egui::ScrollArea::horizontal()
        .id_salt("profiler-seconds")
        .stick_to_right(true)
        .show(ui, |ui| {
            let (rect, response) = ui.allocate_exact_size(
                vec2(cell * span as f32, SECONDS_TALL + 14.0),
                Sense::click(),
            );
            let painter = ui.painter_at(rect);
            let font = FontId::proportional(10.0);

            for offset in 0..span {
                let left = rect.left() + offset as f32 * cell;
                let cell_rect = Rect::from_min_size(
                    pos2(left + 1.0, rect.top()),
                    vec2(cell - 2.0, SECONDS_TALL),
                );
                painter.rect_stroke(
                    cell_rect,
                    2.0,
                    Stroke::new(1.0, theme::DIVIDER),
                    egui::StrokeKind::Inside,
                );
            }
            for second in seconds {
                let offset = (second.t - first.t) as f32;
                let left = rect.left() + offset * cell;
                let cell_rect = Rect::from_min_size(
                    pos2(left + 1.0, rect.top()),
                    vec2(cell - 2.0, SECONDS_TALL),
                );
                painter.rect_filled(cell_rect, 2.0, shade(second.worst_us));
                if second.over > 0 && cell >= 12.0 {
                    painter.text(
                        cell_rect.center_bottom() - vec2(0.0, 3.0),
                        Align2::CENTER_BOTTOM,
                        second.over.to_string(),
                        font.clone(),
                        theme::BACKGROUND,
                    );
                }
                if picked == Some(second.t) {
                    painter.rect_stroke(
                        cell_rect.expand(1.0),
                        2.0,
                        Stroke::new(1.5, theme::TEXT),
                        egui::StrokeKind::Outside,
                    );
                }
            }
            painter.text(
                rect.left_bottom(),
                Align2::LEFT_BOTTOM,
                profile.second_named(first.t),
                font.clone(),
                theme::MUTED,
            );
            if rect.width() > 140.0 {
                painter.text(
                    rect.right_bottom(),
                    Align2::RIGHT_BOTTOM,
                    profile.second_named(last.t),
                    font,
                    theme::MUTED,
                );
            }

            if let Some(pointer) = response.hover_pos() {
                let t = first.t + ((pointer.x - rect.left()) / cell) as i64;
                if let Some(second) = seconds.iter().find(|second| second.t == t) {
                    response.clone().on_hover_text(format!(
                        "{} · {} frames · {} over 16.7 ms · the worst {:.1} ms",
                        profile.second_named(t),
                        second.frames,
                        second.over,
                        ms(second.worst_us)
                    ));
                }
            }
            if response.clicked()
                && let Some(pointer) = response.interact_pointer_pos()
            {
                let t = first.t + ((pointer.x - rect.left()) / cell) as i64;
                clicked = seconds.iter().any(|second| second.t == t).then_some(t);
            }
        });
    clicked
}

/// The second's frames side by side, as wide as they took; the one clicked.
fn frame_strip(
    ui: &mut egui::Ui,
    profile: &Profile,
    frames: &[&Frame],
    picked: Option<i64>,
) -> Option<i64> {
    const GAP: f32 = 3.0;
    let width = ui.available_width();
    let total: u64 = frames.iter().map(|frame| frame.took_us.max(1)).sum();
    let room = (width - GAP * frames.len().saturating_sub(1) as f32).max(1.0);
    let scale = room / total.max(1) as f32;

    let (rect, response) = ui.allocate_exact_size(vec2(width, FRAMES_TALL + 14.0), Sense::click());
    let painter = ui.painter_at(rect);
    let font = FontId::proportional(10.0);
    let mut placed: Vec<(Rect, i64)> = Vec::new();

    let mut left = rect.left();
    for frame in frames {
        let wide = (frame.took_us.max(1) as f32 * scale).max(1.0);
        let bar = Rect::from_min_size(pos2(left, rect.top()), vec2(wide, FRAMES_TALL));
        let laid = frame.measure_us + frame.arrange_us;
        let other = frame.took_us.saturating_sub(laid);
        let mut x = bar.left();
        for (part, color) in [
            (other, theme::MUTED),
            (frame.measure_us, theme::SKY),
            (frame.arrange_us, theme::VIOLET),
        ] {
            let part_wide = part as f32 * scale;
            if part_wide > 0.0 {
                painter.rect_filled(
                    Rect::from_min_size(pos2(x, bar.top()), vec2(part_wide, FRAMES_TALL)),
                    0.0,
                    color,
                );
                x += part_wide;
            }
        }
        if frame.over() {
            painter.rect_filled(
                Rect::from_min_size(pos2(bar.left(), bar.bottom() - 3.0), vec2(wide, 3.0)),
                0.0,
                shade(frame.took_us),
            );
            painter.text(
                pos2(bar.center().x, rect.bottom()),
                Align2::CENTER_BOTTOM,
                format!("{:.1}", ms(frame.took_us)),
                font.clone(),
                shade(frame.took_us),
            );
        }
        if picked == Some(frame.at_us) {
            painter.rect_stroke(
                bar.expand(1.0),
                0.0,
                Stroke::new(1.5, theme::TEXT),
                egui::StrokeKind::Outside,
            );
        }
        placed.push((bar, frame.at_us));
        left += wide + GAP;
    }

    if let Some(pointer) = response.hover_pos()
        && let Some((_, at)) = placed
            .iter()
            .find(|(bar, _)| bar.x_range().contains(pointer.x))
        && let Some(frame) = profile.frame(*at)
    {
        response.clone().on_hover_text(format!(
            "{} · {:.1} ms\nmeasure {:.1} ms · arrange {:.1} ms · the rest {:.1} ms",
            profile.when(frame.at_us),
            ms(frame.took_us),
            ms(frame.measure_us),
            ms(frame.arrange_us),
            ms(frame
                .took_us
                .saturating_sub(frame.measure_us + frame.arrange_us))
        ));
    }
    if response.clicked()
        && let Some(pointer) = response.interact_pointer_pos()
    {
        return placed
            .iter()
            .find(|(bar, _)| bar.x_range().contains(pointer.x))
            .map(|(_, at)| *at);
    }
    None
}

/// What led to the frame, as the Trace page says it, and its costliest
/// layout passes.
fn detail(
    ui: &mut egui::Ui,
    frame: &Frame,
    work: &[&Span],
    timers: &Timers,
    tree: Option<&NativeTree>,
) {
    ui.label(egui::RichText::new("what the application recorded towards it").strong());
    if work.is_empty() {
        ui.label(components::dim(
            "nothing: the frame came without the application asking",
        ));
    }
    for span in work {
        let offset = span.at as i64 - frame.at_us;
        ui.horizontal(|ui| {
            ui.label(components::mono(format!(
                "{:>+8.2} ms",
                offset as f64 / 1000.0
            )));
            ui.label(components::line(ui, &words::sentence(&span.point, timers)));
            if let Some(took) = span.took {
                ui.label(components::dim(format!("{:.2} ms", ms(took))));
            }
        });
    }

    ui.add_space(8.0);
    ui.label(egui::RichText::new("costliest layout passes").strong());
    for pass in frame.passes.iter().take(20) {
        let kind = tree
            .and_then(|tree| tree.get(pass.element))
            .map_or("(not in the tree)", |element| short(&element.kind));
        ui.label(components::mono(format!(
            "{:>8.2} ms  {:<8} {kind}  at +{:.2} ms",
            ms(pass.took_us),
            pass.kind,
            ms(pass.at_us)
        )));
    }
    if frame.passes.len() > 20 {
        ui.label(components::dim(format!(
            "and {} more",
            frame.passes.len() - 20
        )));
    }
}

fn short(kind: &str) -> &str {
    kind.rsplit(['.', ':']).next().unwrap_or(kind)
}
