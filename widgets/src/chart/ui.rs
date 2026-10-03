//! The chart widget: a [`Painted`] view the page owns, and a pointer over it.
//!
//! The view draws when the data moves or the image changes size - and for a
//! live chart, whenever it has moved on by a device pixel.
//!
//! What is left is what was ours to begin with: when to redraw, and where the
//! pointer is. Both live on [`Chart`], which the page keeps as a field - a
//! page is an Elm node, so its state is its struct, and a chart's state is
//! part of the page's.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use windows_canvas::{DrawingSession, GpuDevice};
use windows_reactor::{IntoPayloadCallback, View};

use super::hover::hover_at;
use super::live::{self, Clock};
use super::model::{HoverInfo, LineChartOptions, Series, newest};
use super::paint;
use crate::painted::{Metrics, Paint, Painted, Pointer};

/// What a chart draws with.
#[derive(Default)]
struct Drawing {
    series: RefCell<Vec<Series>>,
    options: RefCell<LineChartOptions>,
    clock: Cell<Clock>,
}

impl Paint for Drawing {
    fn paint(&self, session: &DrawingSession<'_>, device: &GpuDevice, metrics: Metrics) {
        let end = self.end(Instant::now());
        let (series, options) = (self.series.borrow(), self.options.borrow());
        paint::render(session, device, metrics, &series, &options, end);
    }

    fn moving(&self) -> bool {
        let options = self.options.borrow();
        options.live.is_some() && options.x_window.is_some()
    }

    fn next_frame(&self, metrics: Metrics) -> Duration {
        let options = self.options.borrow();
        match (options.live, options.x_window) {
            (Some(live), Some(window)) => {
                live::step(window, live.per_second, metrics.width * metrics.scale)
            }
            _ => Duration::from_secs(1),
        }
    }
}

impl Drawing {
    /// Where a live chart's time axis ends at `at`; `None` for a chart that is
    /// not live, which ends at its newest sample.
    fn end(&self, at: Instant) -> Option<f64> {
        let options = self.options.borrow();
        let live = options.live?;
        options.x_window?;
        let now = self.clock.get().now(live.per_second, at)?;
        Some(now - live.lag as f64)
    }

    /// Puts a live chart's clock on its newest sample, or forgets it for a
    /// chart that is not live.
    fn saw_newest(&self) {
        let options = self.options.borrow();
        let mut clock = self.clock.get();
        match (options.live, options.x_window, newest(&self.series.borrow())) {
            (Some(live), Some(window), Some(newest)) => clock.saw(
                newest as f64,
                window as f64,
                live.per_second,
                Instant::now(),
            ),
            (Some(_), Some(_), None) => {}
            _ => clock.forget(),
        }
        self.clock.set(clock);
    }

    fn hover(&self, at: f32, width: f32) -> Option<HoverInfo> {
        hover_at(
            &self.series.borrow(),
            &self.options.borrow(),
            self.end(Instant::now()),
            at,
            width,
        )
    }
}

/// A line chart, and what it needs between draws.
///
/// Held by the page rather than conjured per render: a canvas that is redrawn
/// only when its data grew has to remember what it last drew, and that
/// remembering is state like any other.
pub struct Chart {
    painted: Rc<Painted<Drawing>>,
    /// Where the pointer was last seen, or `None` when it is away.
    pointer: Rc<Cell<Option<f32>>>,
    /// Whether anything was published yet, so the first publish draws.
    published: Cell<bool>,
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart {
    pub fn new() -> Self {
        Self {
            painted: Rc::new(Painted::default()),
            pointer: Rc::new(Cell::new(None)),
            published: Cell::new(false),
        }
    }

    fn drawing(&self) -> &Drawing {
        self.painted.painter()
    }

    /// Hands the chart new data.
    ///
    /// A redraw is asked for only when the data or the options actually
    /// changed. A drawing surface presents a frame when it is drawn into
    /// rather than every vsync, so this gate is what keeps a chart that ticks
    /// twice a second from burning several percent of a core - which an
    /// unconditional animated canvas did. A live chart moves on between
    /// publishes by itself, a pixel at a time.
    pub fn publish(&self, series: Vec<Series>, options: LineChartOptions) {
        let drawing = self.drawing();
        let first = !self.published.replace(true);
        let restyled = *drawing.options.borrow() != options;
        let moved = *drawing.series.borrow() != series;

        *drawing.series.borrow_mut() = series;
        *drawing.options.borrow_mut() = options;
        drawing.saw_newest();

        if first || restyled || moved {
            self.painted.redraw();
        }
    }

    /// Adds one point to the series at `series`, without handing the chart
    /// the rest again.
    ///
    /// What has scrolled out of `x_window` goes, but for the one point
    /// before its left edge that the line comes in from. A chart without a
    /// window keeps every point it is given. A series the chart was not
    /// published with is not made.
    pub fn push(&self, series: usize, point: (u64, f32)) {
        let drawing = self.drawing();
        {
            let mut all = drawing.series.borrow_mut();
            let Some(line) = all.get_mut(series) else {
                return;
            };
            line.points.push(point);

            let options = drawing.options.borrow();
            if let Some(window) = options.x_window {
                let lag = options.live.map_or(0, |live| live.lag);
                let start = point.0.saturating_sub(window + lag);
                let shown = line.points.partition_point(|&(at, _)| at < start);
                line.points.drain(..shown.saturating_sub(1));
            }
        }
        drawing.saw_newest();
        self.painted.redraw();
    }

    /// New options for the series the chart already has.
    pub fn restyle(&self, options: LineChartOptions) {
        let drawing = self.drawing();
        let restyled = *drawing.options.borrow() != options;
        *drawing.options.borrow_mut() = options;
        drawing.saw_newest();

        if restyled {
            self.painted.redraw();
        }
    }

    /// What the readout should say right now, without the pointer having
    /// moved.
    ///
    /// The series tick on their own, and a pointer resting on the chart should
    /// follow them rather than report whatever was under it when it last
    /// moved. A page asks for this in the same breath as it publishes.
    pub fn hovered(&self) -> Option<HoverInfo> {
        hover(&self.painted, self.pointer.get()?)
    }

    /// The chart, drawn.
    ///
    /// `on_hover` takes what every reactor widget takes: a plain closure, or a
    /// `Callback` a segment already made with `cx.on(..)`.
    pub fn view(&self, on_hover: impl IntoPayloadCallback<Option<HoverInfo>>) -> View {
        let on_hover = on_hover.into_payload_callback();
        let painted = self.painted.clone();
        let pointer = self.pointer.clone();

        self.painted.view(move |event: Pointer| match event {
            Pointer::Moved(info) => {
                let at = info.x as f32;
                pointer.set(Some(at));
                on_hover.call(hover(&painted, at));
            }
            Pointer::Exited => {
                pointer.set(None);
                on_hover.call(None);
            }
            _ => {}
        })
    }
}

fn hover(painted: &Painted<Drawing>, at: f32) -> Option<HoverInfo> {
    let width = painted.metrics().map_or(0.0, |metrics| metrics.width);
    painted.painter().hover(at, width)
}

#[cfg(test)]
mod tests {
    use super::super::model::Interpolation;
    use super::*;

    fn series(points: &[(u64, f32)]) -> Series {
        Series {
            color: windows_canvas::ColorF {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            interpolation: Interpolation::Linear,
            fill: None,
            points: points.to_vec(),
        }
    }

    fn windowed(window: u64) -> LineChartOptions {
        LineChartOptions {
            x_window: Some(window),
            ..LineChartOptions::default()
        }
    }

    fn points(chart: &Chart) -> Vec<(u64, f32)> {
        chart.drawing().series.borrow()[0].points.clone()
    }

    #[test]
    fn a_pushed_point_joins_its_series() {
        let chart = Chart::new();
        chart.publish(vec![series(&[(0, 0.0), (1, 1.0)])], LineChartOptions::default());

        chart.push(0, (2, 2.0));
        assert_eq!(points(&chart), [(0, 0.0), (1, 1.0), (2, 2.0)]);
    }

    #[test]
    fn what_scrolled_out_of_the_window_goes_but_the_point_the_line_comes_in_from() {
        let chart = Chart::new();
        let published: Vec<_> = (0..=4).map(|at| (at * 5, at as f32)).collect();
        chart.publish(vec![series(&published)], windowed(10));

        chart.push(0, (27, 5.0));
        assert_eq!(
            points(&chart),
            [(15, 3.0), (20, 4.0), (27, 5.0)],
            "the window starts at 17: 15 is where the line comes in from, 10 is gone"
        );
    }

    #[test]
    fn a_live_chart_keeps_what_its_lag_still_shows() {
        let chart = Chart::new();
        let published: Vec<_> = (0..=4).map(|at| (at * 5, at as f32)).collect();
        chart.publish(
            vec![series(&published)],
            LineChartOptions {
                live: Some(super::super::model::Live {
                    per_second: 1000.0,
                    lag: 5,
                }),
                ..windowed(10)
            },
        );

        chart.push(0, (27, 5.0));
        assert_eq!(
            points(&chart),
            [(10, 2.0), (15, 3.0), (20, 4.0), (27, 5.0)],
            "trailing the clock by 5, the window may start as early as 12"
        );
    }

    #[test]
    fn a_series_the_chart_was_not_given_is_not_made() {
        let chart = Chart::new();
        chart.publish(vec![series(&[(0, 0.0)])], LineChartOptions::default());

        chart.push(1, (1, 1.0));
        assert_eq!(chart.drawing().series.borrow().len(), 1);
        assert_eq!(points(&chart), [(0, 0.0)]);
    }

    #[test]
    fn restyled_the_chart_keeps_its_series() {
        let chart = Chart::new();
        chart.publish(vec![series(&[(0, 0.0)])], LineChartOptions::default());

        chart.restyle(windowed(10));
        assert_eq!(chart.drawing().options.borrow().x_window, Some(10));
        assert_eq!(points(&chart), [(0, 0.0)]);
    }
}
