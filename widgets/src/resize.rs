//! A drag handle for resizing a column.
//!
//! Rewritten for the component model. The hover, press and drag-anchor state
//! used to live in `RenderCx` hook slots, read back by a builder that the
//! caller then finished; it is a `Component` now, with the same three pieces
//! as fields and the pointer events arriving as messages. What the caller gets
//! back is a plain `View`, and the width it drags out arrives through a
//! `Callback` rather than a `SetState`.

use windows_reactor::{
    Border, Brush, ChildrenControl, Color, Component, ComponentContext, ContentControl, Grid,
    GridChildExt, GridLength, HorizontalAlignment, LayoutControl, PointerEventInfo, ThemeBrush,
    VerticalAlignment, View, ViewContext,
};

/// Width of the full drag surface (the hit-test area, not the visible pill).
pub const RESIZE_HANDLE_WIDTH: f64 = 6.0;

/// Width of the visible pill, matching the NavigationView selection indicator.
const INDICATOR_WIDTH: f64 = 3.0;

const TRANSPARENT: Color = Color {
    a: 0,
    r: 0,
    g: 0,
    b: 0,
};

/// Height of the visible pill indicator. The indicator is always centered
/// within the drag surface regardless of which variant is used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HandleSize {
    /// Fraction of the drag surface's height, clamped to `0.0..=1.0`.
    Percent(f64),
    /// Fixed height in DIPs.
    Absolute(f64),
}

/// What the handle is, as its component's input.
///
/// `Clone + PartialEq` because that is what an `Input` has to be - and it is
/// also what tells the reconciler that a handle whose column did not move does
/// not need rebuilding.
#[derive(Clone, PartialEq)]
pub struct ResizeHandle {
    current: f64,
    min: f64,
    max: f64,
    indicator_size: HandleSize,
    rail: Option<Color>,
    on_resize: windows_reactor::Callback<f64>,
}

/// A handle that drags `current` and reports every new value.
///
/// The value is reported rather than owned: a column's width belongs to
/// whatever laid the column out, and a handle that kept its own copy would be
/// a second answer to the same question.
pub fn resize_handle(
    current: f64,
    on_resize: impl windows_reactor::IntoPayloadCallback<f64>,
) -> ResizeHandle {
    ResizeHandle {
        current,
        min: 0.0,
        max: f64::MAX,
        indicator_size: HandleSize::Percent(0.24),
        rail: None,
        on_resize: on_resize.into_payload_callback(),
    }
}

impl ResizeHandle {
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Height of the visible pill indicator. Defaults to `Percent(0.24)`.
    pub fn indicator_size(mut self, v: HandleSize) -> Self {
        self.indicator_size = v;
        self
    }

    /// Draws a hairline down the handle that is always visible, under the
    /// pill.
    ///
    /// Without it a handle is invisible until pointed at, which is fine for a
    /// resize affordance and wrong for a table header, where the line is also
    /// what tells one column from the next. A separate layer rather than a
    /// resting colour for the pill itself: the pill's brush stays constant
    /// (see [`Handle::view`]), and this one is a plain colour that never
    /// changes either.
    pub fn rail(mut self, color: Color) -> Self {
        self.rail = Some(color);
        self
    }

    pub fn build(self) -> View {
        View::component::<Handle>(self)
    }
}

impl From<ResizeHandle> for View {
    fn from(handle: ResizeHandle) -> Self {
        handle.build()
    }
}

pub enum Dragging {
    Entered,
    Exited,
    /// Where the pointer was and how wide the column was, both captured at the
    /// moment of the press. The width comes in the message because `update`
    /// is not handed the input, and by the time a move arrives the press is
    /// the only thing that knew it.
    Pressed {
        window_x: f64,
        current: f64,
    },
    Released,
}

/// The handle's own state.
struct Handle {
    hovered: bool,
    pressed: bool,
    /// `(window_x, current)` captured on press - the anchor for computing a
    /// real drag delta. See `Dragging::Moved` below for why this cannot just
    /// be `current + info.x`.
    drag_start: (f64, f64),
}

impl Component for Handle {
    type Input = ResizeHandle;
    type Message = Dragging;

    fn create(_input: &ResizeHandle, _cx: &ComponentContext<Self>) -> Self {
        Self {
            hovered: false,
            pressed: false,
            drag_start: (0.0, 0.0),
        }
    }

    fn update(&mut self, message: Dragging, _cx: &ComponentContext<Self>) {
        match message {
            Dragging::Entered => self.hovered = true,
            Dragging::Exited => self.hovered = false,
            Dragging::Pressed { window_x, current } => {
                self.pressed = true;
                self.drag_start = (window_x, current);
            }
            Dragging::Released => self.pressed = false,
        }
    }

    fn input_changed(&mut self, _input: &ResizeHandle, _cx: &ComponentContext<Self>) {}

    fn view(&self, input: &ResizeHandle, cx: &mut ViewContext<Self>) -> View {
        // Fluent reserves the fully saturated accent for prominent controls
        // (buttons, toggles); it reads as loud for a hairline handle, and the
        // muted token the old API exposed as `ThemeRef::AccentSecondary` has no
        // equivalent in `ThemeBrush`. Opacity carries the difference instead,
        // which it was already doing for the three states.
        //
        // The brush binding is deliberately *constant* across all three states
        // (never a direct colour). A local value - even a null one -
        // permanently outranks a Style Setter's `{ThemeResource ...}`, so
        // switching this prop between a direct colour and a theme brush across
        // renders would silently and irrecoverably stop the theme brush from
        // ever painting again. Visibility is driven by `opacity`, which has no
        // such precedence trap.
        let indicator_opacity = if self.pressed {
            0.9
        } else if self.hovered {
            0.6
        } else {
            0.0
        };

        let pill = Border::new()
            .width(INDICATOR_WIDTH)
            .corner_radius(INDICATOR_WIDTH / 2.0)
            .background(Brush::from(ThemeBrush::Accent))
            .opacity(indicator_opacity)
            .horizontal_alignment(HorizontalAlignment::Center)
            .grid_row(1)
            .content(View::empty());

        // Two equal `Star` rows on either side of the pill centre it whatever
        // the drag surface's actual height, without needing measured layout.
        let (top, mid, bottom) = match input.indicator_size {
            HandleSize::Percent(p) => {
                let p = p.clamp(0.0, 1.0);
                let side = ((1.0 - p) / 2.0).max(0.0);
                (
                    GridLength::Star(side),
                    GridLength::Star(p.max(0.0001)),
                    GridLength::Star(side),
                )
            }
            HandleSize::Absolute(px) => (
                GridLength::Star(1.0),
                GridLength::Pixel(px.max(0.0)),
                GridLength::Star(1.0),
            ),
        };

        let indicator = Grid::new()
            .rows([top, mid, bottom])
            .columns([GridLength::Star(1.0)])
            .children((
                Border::new().grid_row(0).content(View::empty()),
                pill,
                Border::new().grid_row(2).content(View::empty()),
            ));

        // Both children land in the single implicit cell, so the pill draws
        // over the rail rather than beside it.
        let layered = match input.rail {
            Some(color) => Grid::new().children((
                Border::new()
                    .width(1.0)
                    .background(color)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Stretch)
                    .content(View::empty()),
                indicator,
            )),
            None => indicator,
        };

        let (min, max) = (input.min, input.max);
        let on_resize = input.on_resize.clone();
        // Read at view time, which is after the press was applied: pressing
        // changes state, the component publishes again, and the move callback
        // built here carries the anchor that press recorded.
        let (start_window_x, start_current) = self.drag_start;
        let current = input.current;

        Border::new()
            .width(RESIZE_HANDLE_WIDTH)
            // Background must stay set (even fully transparent) so the whole
            // drag surface hit-tests - a null background receives no pointer
            // events in WinUI, and only the visible pill would.
            .background(TRANSPARENT)
            .horizontal_alignment(HorizontalAlignment::Left)
            .vertical_alignment(VerticalAlignment::Stretch)
            // A flag now rather than a call: without it the drag stops the
            // moment the pointer leaves the six-pixel handle, which is
            // immediately.
            .capture_pointer_on_press(true)
            .on_pointer_entered(cx.callback(|_: PointerEventInfo| Dragging::Entered))
            .on_pointer_exited(cx.callback(|_: PointerEventInfo| Dragging::Exited))
            // Anchored to where the drag started, in `window_x` - this
            // handle's own margin follows `current` every render, so its own
            // local coordinate origin moves out from under the drag on every
            // frame. `window_x` does not move with it.
            .on_pointer_pressed(
                cx.callback(move |info: PointerEventInfo| Dragging::Pressed {
                    window_x: info.window_x,
                    current,
                }),
            )
            .on_pointer_released(cx.callback(|_: PointerEventInfo| Dragging::Released))
            .on_pointer_moved(windows_reactor::Callback::new(
                move |info: PointerEventInfo| {
                    if info.is_left_button_pressed {
                        let delta = info.window_x - start_window_x;
                        // Dropped when whoever owns the width is not publishing;
                        // the handle then simply does not move.
                        let _ = on_resize.call((start_current + delta).clamp(min, max));
                    }
                },
            ))
            .content(layered)
    }
}
