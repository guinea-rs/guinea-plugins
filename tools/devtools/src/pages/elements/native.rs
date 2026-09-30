//! The backend's own element tree as the Elements tab reaches it: attaching
//! its inspector, picking an element in the window, highlighting the one
//! under the pointer, and the frames a capture recorded.
//!
//! What the tree holds depends on the backend's inspector. For WinUI it is
//! the live XAML tree, from a tap loaded into the process on demand.

#[cfg(windows)]
use std::time::{Duration, Instant};

use guinea_core::feature::Dispatch;
use guinea_devtools_model::native::NativeTree;
use guinea_devtools_model::sessions::{Session, Sessions};
use guinea_devtools_protocol::native::Frame;
use guinea_devtools_protocol::{Capability, Command};

use crate::components;
use crate::features::native::contracts::{NativeState, Picking, Select};

#[cfg(windows)]
const HIT_EVERY: Duration = Duration::from_millis(40);

/// The pick in progress, what was last asked of the inspector, and how the
/// frame strip is being looked at.
///
/// The `Picker` lives here. It is a live mouse hook rather than a value,
/// which is why it belongs to the page and to nothing else.
#[derive(Default)]
pub struct Inspecting {
    #[cfg(windows)]
    picker: Option<guinea_xaml_tap::pick::Picker>,
    hit_at: Option<(i32, i32)>,
    #[cfg(windows)]
    hit_when: Option<Instant>,
    highlighted: Option<u64>,
    asked_for: Option<u64>,
    attach_error: Option<String>,
    /// The frame whose layout passes are listed.
    frame: Option<usize>,
    /// How wide a frame's bar is drawn; `None` fits them all.
    frame_width: Option<f32>,
}

impl Inspecting {
    /// The strip over the tree while no inspector is attached: a button that
    /// attaches one.
    pub fn attach(&mut self, ui: &mut egui::Ui, sessions: &Sessions, app: &Session) {
        ui.horizontal_centered(|ui| {
            ui.label(components::dim("no native inspector"));
            #[cfg(windows)]
            if app.connected && ui.button("attach the XAML inspector").clicked() {
                self.attach_error = guinea_devtools_hub::attach_native_in(sessions, app.id).err();
            }
            #[cfg(not(windows))]
            let _ = (sessions, app);

            if let Some(error) = &self.attach_error {
                ui.label(egui::RichText::new(error).color(ui.visuals().warn_fg_color));
            }
        });
    }

    /// The strip over the tree with an inspector attached: pick an element,
    /// capture frames.
    pub fn toolbar(
        &mut self,
        ui: &mut egui::Ui,
        inspector: &Session,
        view: &NativeState,
        dispatch: &Dispatch,
        send: &dyn Fn(Command),
    ) {
        ui.horizontal_centered(|ui| {
            let picks = inspector.info.can(Capability::NativeHitTest);
            let label = if view.picking { "picking… click an element" } else { "pick an element" };
            if ui
                .add_enabled(picks, egui::Button::selectable(view.picking, label))
                .clicked()
            {
                dispatch.emit(Picking(!view.picking));
            }

            if inspector.info.can(Capability::NativePerf) && ui.button("capture frames").clicked() {
                send(Command::NativePerfCapture);
                self.frame = None;
            }

            ui.separator();
            ui.label(components::dim(format!(
                "{} · {} elements",
                inspector.info.backend,
                inspector.inspection.tree.len()
            )));

            if let Some((command, reason)) = &inspector.inspection.refused {
                ui.separator();
                ui.label(egui::RichText::new(format!("{command}: {reason}")).color(ui.visuals().warn_fg_color));
            }
        });
    }

    /// Asks for `element`'s properties, once each time it becomes the one.
    pub fn ask_for(&mut self, element: u64, send: &dyn Fn(Command)) {
        if self.asked_for.replace(element) != Some(element) {
            send(Command::NativeProperties { element });
        }
    }

    /// Lights up `element` in the window, or nothing.
    pub fn highlight(&mut self, element: Option<u64>, send: &dyn Fn(Command)) {
        if self.highlighted != element {
            self.highlighted = element;
            send(Command::NativeHighlight { element });
        }
    }

    pub fn picker_off(&mut self) {
        #[cfg(windows)]
        {
            self.picker = None;
        }
        self.hit_at = None;
    }

    /// Follows the pointer in the application's window; a click there picks
    /// the innermost element under it.
    #[cfg(windows)]
    pub fn pick(
        &mut self,
        ui: &egui::Ui,
        app: &Session,
        inspector: &Session,
        dispatch: &Dispatch,
        send: &dyn Fn(Command),
    ) {
        ui.ctx().request_repaint_after(HIT_EVERY);

        let picker = self
            .picker
            .get_or_insert_with(|| guinea_xaml_tap::pick::Picker::start(app.info.pid));
        let (at, clicked) = (picker.at(), picker.take_click());

        if let Some((x, y)) = at {
            let moved = self.hit_at != Some((x, y));
            let rested = self.hit_when.is_none_or(|when| when.elapsed() >= HIT_EVERY);

            if moved && rested {
                self.hit_at = Some((x, y));
                self.hit_when = Some(Instant::now());
                send(Command::NativeHitTest { x, y });
            }
        }

        let innermost = inspector
            .inspection
            .picked
            .as_ref()
            .and_then(|picked| picked.chain.first().copied());

        if innermost.is_some() {
            self.highlight(innermost, send);
        }

        if clicked {
            if let Some(innermost) = innermost {
                dispatch.emit(Select(innermost));
            }
            dispatch.emit(Picking(false));
        }
    }

    #[cfg(not(windows))]
    pub fn pick(
        &mut self,
        _ui: &egui::Ui,
        _app: &Session,
        _inspector: &Session,
        dispatch: &Dispatch,
        _send: &dyn Fn(Command),
    ) {
        dispatch.emit(Picking(false));
    }
}

fn short(kind: &str) -> &str {
    kind.rsplit('.').next().unwrap_or(kind)
}

/// A frame at 60 Hz, the line a bar crosses when the frame ran late.
const BUDGET_US: f32 = 16_667.0;
/// How narrow and how wide a frame's bar can be zoomed.
const MIN_BAR: f32 = 1.0;
const MAX_BAR: f32 = 48.0;

impl Inspecting {
    /// Every captured frame as a bar, and the layout passes of the one
    /// clicked.
    pub fn frames(&mut self, ui: &mut egui::Ui, tree: &NativeTree, frames: &[Frame], dispatch: &Dispatch) {
        let chosen = self.frame;
        let slowest = frames.iter().map(|frame| frame.took_us).max().unwrap_or(0) as f32;

        let fit = (ui.available_width() / frames.len().max(1) as f32).max(MIN_BAR);
        let mut width = self.frame_width.unwrap_or(fit);

        components::block(ui, |ui| {
            ui.horizontal(|ui| {
                let late = frames.iter().filter(|frame| frame.took_us as f32 > BUDGET_US).count();
                ui.label(components::dim(format!(
                    "{} frames over {:.1} s · {} over 16.7 ms · slowest {:.1} ms",
                    frames.len(),
                    frames.last().map_or(0.0, |last| last.at_us as f32 / 1e6),
                    late,
                    slowest / 1000.0
                )));

                ui.separator();
                let zoomed = ui.add(egui::Slider::new(&mut width, MIN_BAR..=MAX_BAR).logarithmic(true).text("px per frame"));
                let fitted = ui.button("fit").clicked();

                if fitted {
                    self.frame_width = None;
                } else if zoomed.changed() {
                    self.frame_width = Some(width);
                }
            });
        });

        let height = 56.0;
        let scale = height / slowest.max(1000.0);

        let picked = egui::ScrollArea::horizontal()
            .id_salt("native-frame-strip")
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(width * frames.len() as f32, height),
                    egui::Sense::click(),
                );

                let wheel = ui.input(|input| input.modifiers.command.then_some(input.smooth_scroll_delta.y));
                let zoomed = wheel
                    .filter(|delta| *delta != 0.0 && response.hovered())
                    .map(|delta| (width * (1.0 + delta / 200.0)).clamp(MIN_BAR, MAX_BAR));

                let painter = ui.painter_at(rect);
                let visible = ui.clip_rect();
                for (index, frame) in frames.iter().enumerate() {
                    let left = rect.left() + index as f32 * width;
                    if left + width < visible.left() || left > visible.right() {
                        continue;
                    }

                    let top = rect.bottom() - frame.took_us as f32 * scale;
                    let color = if chosen == Some(index) {
                        ui.visuals().selection.bg_fill
                    } else if frame.took_us as f32 > BUDGET_US {
                        ui.visuals().warn_fg_color
                    } else {
                        ui.visuals().weak_text_color()
                    };
                    let right = (left + width - 1.0).max(left + 1.0);
                    painter.rect_filled(egui::Rect::from_min_max(egui::pos2(left, top), egui::pos2(right, rect.bottom())), 0.0, color);
                }

                let budget = rect.bottom() - BUDGET_US * scale;
                if budget >= rect.top() {
                    painter.hline(rect.x_range(), budget, ui.visuals().widgets.noninteractive.bg_stroke);
                }

                let clicked = response
                    .interact_pointer_pos()
                    .filter(|_| response.clicked())
                    .map(|pointer| ((pointer.x - rect.left()) / width) as usize)
                    .filter(|index| *index < frames.len());

                (zoomed, clicked)
            })
            .inner;

        let (zoomed, clicked) = picked;
        if let Some(width) = zoomed {
            self.frame_width = Some(width);
        }
        if let Some(index) = clicked {
            self.frame = Some(index);
        }

        let Some(frame) = chosen.and_then(|index| frames.get(index)) else {
            components::block(ui, |ui| ui.label(components::dim("click a bar for its layout passes")));
            return;
        };

        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            components::block(ui, |ui| {
                ui.label(format!(
                    "{:.2} ms at {:.2} s · measure {:.2} ms · arrange {:.2} ms",
                    frame.took_us as f32 / 1000.0,
                    frame.at_us as f32 / 1e6,
                    frame.measure_us as f32 / 1000.0,
                    frame.arrange_us as f32 / 1000.0
                ));

                for pass in &frame.passes {
                    let kind = tree.get(pass.element).map_or("(gone)", |element| short(&element.kind));
                    let line = format!("{:>6} µs  {:<8} {kind}", pass.took_us, pass.kind);
                    if ui.selectable_label(false, components::mono(line)).clicked() && tree.get(pass.element).is_some() {
                        dispatch.emit(Select(pass.element));
                    }
                }
            })
        });
    }
}
