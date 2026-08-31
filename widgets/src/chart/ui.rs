//! The chart widget: a canvas the page owns, and a pointer over it.
//!
//! This used to be 250 lines of composition plumbing - a graphics device, a
//! drawing surface, a sprite visual, an attach/resize/detach dance against a
//! host element - because there was nothing to do it for us. `windows-canvas`
//! has a `reactor` feature now, and all of that is one call.
//!
//! What is left is what was ours to begin with: when to redraw, and where the
//! pointer is. Both live on [`Chart`], which the page keeps as a field - a
//! page is an Elm node, so its state is its struct, and a chart's state is
//! part of the page's.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows_canvas::{Invalidator, canvas_invalidated};
use windows_reactor::{
    Border, Callback, Color, ContentControl, IntoPayloadCallback, PointerEventInfo, View,
};

use super::hover::hover_at;
use super::model::{ChartRevision, HoverInfo, LineChartOptions, Series, chart_revision};
use super::paint;

/// A line chart, and what it needs between draws.
///
/// Held by the page rather than conjured per render: a canvas that is redrawn
/// only when its data grew has to remember what it last drew, and that
/// remembering is state like any other.
pub struct Chart {
    series: Rc<RefCell<Vec<Series>>>,
    options: Rc<RefCell<LineChartOptions>>,
    /// The last width the canvas drew at, for turning a pointer position into
    /// a point on the series.
    width: Rc<Cell<f32>>,
    /// Where the pointer was last seen, or `None` when it is away.
    pointer: Rc<Cell<Option<f32>>>,
    drawn: Cell<Option<ChartRevision>>,
    invalidator: Invalidator,
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart {
    pub fn new() -> Self {
        Self {
            series: Rc::new(RefCell::new(Vec::new())),
            options: Rc::new(RefCell::new(LineChartOptions::default())),
            width: Rc::new(Cell::new(0.0)),
            pointer: Rc::new(Cell::new(None)),
            drawn: Cell::new(None),
            invalidator: Invalidator::new(),
        }
    }

    /// Hands the chart new data.
    ///
    /// A redraw is asked for only when the data actually moved. A drawing
    /// surface presents a frame when it is drawn into rather than every vsync,
    /// so this gate is what keeps a chart that ticks twice a second from
    /// burning several percent of a core - which an unconditional animated
    /// canvas did.
    pub fn publish(&self, series: Vec<Series>, options: LineChartOptions) {
        let revision = chart_revision(&series);
        *self.series.borrow_mut() = series;
        *self.options.borrow_mut() = options;

        if self.drawn.get() != Some(revision) {
            self.drawn.set(Some(revision));
            self.invalidator.invalidate();
        }
    }

    /// What the readout should say right now, without the pointer having
    /// moved.
    ///
    /// The series tick on their own, and a pointer resting on the chart should
    /// follow them rather than report whatever was under it when it last
    /// moved. A page asks for this in the same breath as it publishes.
    pub fn hovered(&self) -> Option<HoverInfo> {
        let at = self.pointer.get()?;
        hover_at(&self.series.borrow(), at, self.width.get())
    }

    /// The chart, drawn.
    ///
    /// `on_hover` takes what every reactor widget takes: a plain closure, or a
    /// `Callback` a segment already made with `cx.on(..)`.
    pub fn view(&self, on_hover: impl IntoPayloadCallback<Option<HoverInfo>>) -> View {
        let painting = self.series.clone();
        let options = self.options.clone();
        let measured = self.width.clone();

        let surface = canvas_invalidated(&self.invalidator, move |draw| {
            measured.set(draw.width);
            paint::render(draw, &painting.borrow(), &options.borrow());
            Ok(())
        });

        let on_hover = on_hover.into_payload_callback();
        let moved_over = self.series.clone();
        let width_on_move = self.width.clone();
        let pointer_on_move = self.pointer.clone();
        let hover_on_move = on_hover.clone();
        let pointer_on_exit = self.pointer.clone();

        Border::new()
            // A composition child visual is invisible to XAML hit-testing, so
            // without a brush on the host itself the chart would never see a
            // pointer. Transparent is enough, and stays out of the way of
            // whatever the chart draws.
            .background(Color::transparent())
            .on_pointer_moved(Callback::new(move |info: PointerEventInfo| {
                let at = info.x as f32;
                pointer_on_move.set(Some(at));
                // Dropped when the segment that drew this chart is no longer
                // publishing; the readout then simply stays where it was.
                let _ = hover_on_move.call(hover_at(
                    &moved_over.borrow(),
                    at,
                    width_on_move.get(),
                ));
            }))
            .on_pointer_exited(Callback::new(move |_: PointerEventInfo| {
                pointer_on_exit.set(None);
                let _ = on_hover.call(None);
            }))
            .content(surface)
    }
}
