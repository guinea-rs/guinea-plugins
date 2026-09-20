//! The backend's own element tree, the way the browser's devtools show the
//! DOM: pick an element in the window, see where it sits, what its
//! properties are and where each value came from, and change them live.
//!
//! What the tree holds depends on the backend's inspector. For WinUI it is
//! the live XAML tree, from a tap loaded into the process on demand.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_core::feature::Dispatch;
use guinea_devtools_model::native::NativeTree;
use guinea_devtools_model::sessions::{Session, Sessions};
use guinea_devtools_model::words::{Kind, Tone, Word};
use guinea_devtools_protocol::native::Frame;
use guinea_devtools_protocol::{Capability, Command};

mod properties;

use crate::components;
use crate::components::tree;
use crate::features::focus::contracts::Focus;
use crate::features::native::NativeFeature;
use crate::features::native::contracts::{NativeState, Picking, Select};
use crate::features::sessions::contracts::Live;

const HIT_EVERY: Duration = Duration::from_millis(40);

/// The page: the pick in progress, what was last asked of the inspector, and
/// how the tree and the frame strip are being looked at.
///
/// The `Picker` lives here too. It is a live mouse hook rather than a value,
/// which is exactly why it belongs to the page and to nothing else.
#[derive(Default)]
pub struct Native {
    #[cfg(windows)]
    picker: Option<guinea_xaml_tap::pick::Picker>,
    hit_at: Option<(i32, i32)>,
    hit_when: Option<Instant>,
    highlighted: Option<u64>,
    asked_for: Option<u64>,
    /// Asked for once: bring this element into view.
    reveal: Option<u64>,
    attach_error: Option<String>,
    /// Rows opened or closed by hand; the rest are open for the first two
    /// levels.
    opened: HashMap<u64, bool>,
    /// The frame whose layout passes are listed.
    frame: Option<usize>,
    /// How wide a frame's bar is drawn; `None` fits them all.
    frame_width: Option<f32>,
}

impl Page for Native {
    type Params = crate::routes::NativeParams;
    type Installs = NativeFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, _) = cx.state::<Focus, _>();
        let (view, dispatch) = cx.state::<NativeState, _>();
        let ui = cx.ui();

        let sessions = live.read();
        let Some(app) = sessions.get(focus.app) else {
            return;
        };
        let Some(inspector) = sessions.native_for(focus.app) else {
            self.picker_off();
            self.attach(ui, &sessions, app);
            return;
        };

        let send = |command: Command| {
            let _ = live.hub.send_in(&sessions, inspector.id, command);
        };

        if view.picking {
            self.pick(ui, app, inspector, &dispatch, &send);
        } else {
            self.picker_off();
        }

        if let Some(selected) = view.selected {
            let asked = self.asked_for.replace(selected);
            if asked != Some(selected) {
                send(Command::NativeProperties { element: selected });
            }
        }

        egui::Panel::top("native-tools")
            .resizable(false)
            .exact_size(30.0)
            .frame(components::side())
            .show(ui, |ui| {
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
            });

        let tree = &inspector.inspection.tree;

        if !inspector.inspection.frames.is_empty() {
            egui::Panel::bottom("native-frames")
                .resizable(true)
                .size_range(120.0..=420.0)
                .frame(components::side())
                .show(ui, |ui| self.frames(ui, tree, &inspector.inspection.frames, &dispatch));
        }

        if let Some(selected) = view.selected.filter(|&handle| tree.get(handle).is_some()) {
            egui::Panel::right("native-properties")
                .resizable(true)
                .default_size(420.0)
                .size_range(300.0..=720.0)
                .frame(components::side())
                .show(ui, |ui| {
                    let properties = inspector
                        .inspection
                        .properties
                        .as_ref()
                        .filter(|(element, _)| *element == selected)
                        .map(|(_, properties)| properties.as_slice());
                    let context = properties::Context {
                        tree,
                        enums: &inspector.inspection.enums,
                        edits: inspector.info.can(Capability::NativeEdit),
                        send: &send,
                        dispatch: &dispatch,
                    };
                    properties::show(ui, selected, properties, &context);
                });
        }

        let pane = components::bare(ui);
        let hovered = egui::CentralPanel::default()
            .frame(pane)
            .show(ui, |ui| self.rows(ui, tree, view.selected, &dispatch))
            .inner;

        if !view.picking {
            let wanted = hovered;
            let shown = std::mem::replace(&mut self.highlighted, wanted);
            if shown != wanted {
                send(Command::NativeHighlight { element: wanted });
            }
        }
    }
}

impl Native {
    fn picker_off(&mut self) {
        #[cfg(windows)]
        {
            self.picker = None;
        }
        self.hit_at = None;
    }

    fn attach(&mut self, ui: &mut egui::Ui, sessions: &Sessions, app: &Session) {
        components::block(ui, |ui| {
            ui.label(components::dim(format!(
                "{} has no native inspector connected",
                app.name()
            )));

            #[cfg(windows)]
            if app.connected && ui.button("attach the XAML inspector").clicked() {
                self.attach_error = guinea_devtools_hub::attach_native_in(sessions, app.id).err();
            }

            if let Some(error) = &self.attach_error {
                ui.label(egui::RichText::new(error).color(ui.visuals().warn_fg_color));
            }
        });
    }

    #[cfg(windows)]
    fn pick(
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

        if let Some(innermost) = innermost
            && self.highlighted.replace(innermost) != Some(innermost)
        {
            send(Command::NativeHighlight { element: Some(innermost) });
        }

        if clicked {
            if let Some(innermost) = innermost {
                dispatch.emit(Select(innermost));
                self.reveal = Some(innermost);
            }
            dispatch.emit(Picking(false));
        }
    }

    #[cfg(not(windows))]
    fn pick(
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

struct Row {
    handle: u64,
    depth: usize,
    branch: bool,
    open: bool,
}

/// The rows in view: every element whose ancestors are all open.
fn flatten(tree: &NativeTree, opened: &HashMap<u64, bool>) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut stack: Vec<(u64, usize)> = tree.roots().iter().rev().map(|&root| (root, 0)).collect();

    while let Some((handle, depth)) = stack.pop() {
        let children = tree.children(handle);
        let open = opened.get(&handle).copied().unwrap_or(depth < 2);
        rows.push(Row {
            handle,
            depth,
            branch: !children.is_empty(),
            open,
        });

        if open {
            stack.extend(children.iter().rev().map(|&child| (child, depth + 1)));
        }
    }

    rows
}

impl Native {
    /// The tree, and the element under the pointer.
    fn rows(
        &mut self,
        ui: &mut egui::Ui,
        tree: &NativeTree,
        selected: Option<u64>,
        dispatch: &Dispatch,
    ) -> Option<u64> {
        let reveal = self.reveal.take();
        if let Some(handle) = reveal {
            for ancestor in tree.path(handle).into_iter().filter(|&ancestor| ancestor != handle) {
                self.opened.insert(ancestor, true);
            }
        }

        let rows = flatten(tree, &self.opened);
        let wanted = reveal.and_then(|handle| rows.iter().position(|row| row.handle == handle));

        let answer = tree::show(ui, rows.len(), wanted, |ui, index| {
            let row = &rows[index];
            let element = tree.get(row.handle);

            let kind = element.map_or("?", |element| short(&element.kind));
            let mut words = vec![
                Word::new("<", Tone::Muted),
                Word::new(kind, Tone::Kind(Kind::Send)),
            ];
            if let Some(name) = element.map(|element| &element.name).filter(|name| !name.is_empty()) {
                words.push(Word::new(format!(" #{name}"), Tone::Accent));
            }
            words.push(Word::new(if row.branch { ">" } else { " />" }, Tone::Muted));

            tree::Line {
                depth: row.depth,
                branch: row.branch,
                open: row.open,
                selected: selected == Some(row.handle),
                text: components::line(ui, &words),
            }
        });

        if let Some(index) = answer.toggled {
            self.opened.insert(rows[index].handle, !rows[index].open);
        }

        if let Some(index) = answer.clicked {
            dispatch.emit(Select(rows[index].handle));
        }

        answer.hovered.map(|index| rows[index].handle)
    }
}

/// A frame at 60 Hz, the line a bar crosses when the frame ran late.
const BUDGET_US: f32 = 16_667.0;
/// How narrow and how wide a frame's bar can be zoomed.
const MIN_BAR: f32 = 1.0;
const MAX_BAR: f32 = 48.0;

impl Native {
    /// Every captured frame as a bar, and the layout passes of the one
    /// clicked.
    fn frames(&mut self, ui: &mut egui::Ui, tree: &NativeTree, frames: &[Frame], dispatch: &Dispatch) {
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

        let reveal = egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                components::block(ui, |ui| {
                    ui.label(format!(
                        "{:.2} ms at {:.2} s · measure {:.2} ms · arrange {:.2} ms",
                        frame.took_us as f32 / 1000.0,
                        frame.at_us as f32 / 1e6,
                        frame.measure_us as f32 / 1000.0,
                        frame.arrange_us as f32 / 1000.0
                    ));

                    let mut reveal = None;
                    for pass in &frame.passes {
                        let kind = tree.get(pass.element).map_or("(gone)", |element| short(&element.kind));
                        let line = format!("{:>6} µs  {:<8} {kind}", pass.took_us, pass.kind);
                        if ui.selectable_label(false, components::mono(line)).clicked() && tree.get(pass.element).is_some() {
                            dispatch.emit(Select(pass.element));
                            reveal = Some(pass.element);
                        }
                    }

                    reveal
                })
            })
            .inner;

        if let Some(element) = reveal {
            self.reveal = Some(element);
        }
    }
}

