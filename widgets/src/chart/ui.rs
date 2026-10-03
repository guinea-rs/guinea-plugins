//! The chart widget: an image the page owns, and a pointer over it.
//!
//! The image shows a surface on the device every chart on the thread shares
//! (see `surface`), drawn when the data moves or the image changes size - and
//! for a live chart, whenever it has moved on by a device pixel. The grid
//! around it reports that size.
//!
//! What is left is what was ours to begin with: when to redraw, and where the
//! pointer is. Both live on [`Chart`], which the page keeps as a field - a
//! page is an Elm node, so its state is its struct, and a chart's state is
//! part of the page's.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use windows_reactor::{
    Border, Callback, Color, Component, ComponentContext, ComponentTimer, CompositionHostEvent,
    ElementObservation, ElementRef, Grid, Image, IntoPayloadCallback, PointerEventInfo, Stretch,
    View, ViewContext,
};

use super::hover::hover_at;
use super::live::{self, Clock};
use super::model::{HoverInfo, LineChartOptions, Series, newest};
use super::paint;
use super::surface::{Metrics, Surface};

/// What a chart draws with, shared by the page's [`Chart`], the host that
/// reports its size, and the timer that keeps a live chart moving.
struct Drawing {
    series: RefCell<Vec<Series>>,
    options: RefCell<LineChartOptions>,
    surface: RefCell<Surface>,
    clock: Cell<Clock>,
    /// The last width the canvas drew at, for turning a pointer position into
    /// a point on the series.
    width: Cell<f32>,
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

    fn draw(&self) {
        let end = self.end(Instant::now());
        let (series, options) = (self.series.borrow(), self.options.borrow());
        self.surface.borrow_mut().draw(|session, device, metrics| {
            paint::render(session, device, metrics, &series, &options, end);
        });
    }

    /// Moves a live chart on: draws it, and says how long until it has moved
    /// another device pixel. `None` for a chart that is not live, or not on
    /// screen.
    fn tick(&self) -> Option<Duration> {
        let (live, window) = {
            let options = self.options.borrow();
            (options.live?, options.x_window?)
        };
        let metrics = self.surface.borrow().metrics()?;
        self.draw();
        Some(live::step(
            window,
            live.per_second,
            metrics.width * metrics.scale,
        ))
    }

    fn hover(&self, at: f32) -> Option<HoverInfo> {
        hover_at(
            &self.series.borrow(),
            &self.options.borrow(),
            self.end(Instant::now()),
            at,
            self.width.get(),
        )
    }
}

/// A line chart, and what it needs between draws.
///
/// Held by the page rather than conjured per render: a canvas that is redrawn
/// only when its data grew has to remember what it last drew, and that
/// remembering is state like any other.
pub struct Chart {
    drawing: Rc<Drawing>,
    /// Where the pointer was last seen, or `None` when it is away.
    pointer: Rc<Cell<Option<f32>>>,
    /// Whether anything was published yet, so the first publish draws.
    published: Cell<bool>,
    host: ElementRef<Grid>,
    image: ElementRef<Image>,
    _sized: ElementObservation,
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart {
    pub fn new() -> Self {
        let host = ElementRef::new();
        let image = ElementRef::new();
        let drawing = Rc::new(Drawing {
            series: RefCell::new(Vec::new()),
            options: RefCell::new(LineChartOptions::default()),
            surface: RefCell::new(Surface::new(image.clone())),
            clock: Cell::new(Clock::default()),
            width: Cell::new(0.0),
        });

        let sized = host.observe_composition_host({
            let drawing = drawing.clone();

            move |event| {
                let (metrics, rebound) = match event {
                    CompositionHostEvent::Ready {
                        width,
                        height,
                        scale,
                        ..
                    } => (Metrics::new(width, height, scale), true),
                    CompositionHostEvent::Metrics {
                        width,
                        height,
                        scale,
                    } => (Metrics::new(width, height, scale), false),
                };
                drawing.width.set(metrics.width);

                let changed = {
                    let mut surface = drawing.surface.borrow_mut();
                    if rebound {
                        surface.detached();
                    }
                    surface.resize(metrics) || rebound
                };
                if changed {
                    drawing.draw();
                }
            }
        });

        Self {
            drawing,
            pointer: Rc::new(Cell::new(None)),
            published: Cell::new(false),
            host,
            image,
            _sized: sized,
        }
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
        let drawing = &self.drawing;
        let first = !self.published.replace(true);
        let restyled = *drawing.options.borrow() != options;
        let moved = *drawing.series.borrow() != series;

        *drawing.series.borrow_mut() = series;
        *drawing.options.borrow_mut() = options;
        drawing.saw_newest();

        if first || restyled || moved {
            drawing.draw();
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
        let drawing = &self.drawing;
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
        drawing.draw();
    }

    /// New options for the series the chart already has.
    pub fn restyle(&self, options: LineChartOptions) {
        let drawing = &self.drawing;
        let restyled = *drawing.options.borrow() != options;
        *drawing.options.borrow_mut() = options;
        drawing.saw_newest();

        if restyled {
            drawing.draw();
        }
    }

    /// What the readout should say right now, without the pointer having
    /// moved.
    ///
    /// The series tick on their own, and a pointer resting on the chart should
    /// follow them rather than report whatever was under it when it last
    /// moved. A page asks for this in the same breath as it publishes.
    pub fn hovered(&self) -> Option<HoverInfo> {
        self.drawing.hover(self.pointer.get()?)
    }

    /// The chart, drawn.
    ///
    /// `on_hover` takes what every reactor widget takes: a plain closure, or a
    /// `Callback` a segment already made with `cx.on(..)`.
    pub fn view(&self, on_hover: impl IntoPayloadCallback<Option<HoverInfo>>) -> View {
        let options = self.drawing.options.borrow();
        let surface = View::component::<Mount>(Mounted {
            host: self.host.clone(),
            image: self.image.clone(),
            drawing: self.drawing.clone(),
            live: options.live.is_some() && options.x_window.is_some(),
        });

        let on_hover = on_hover.into_payload_callback();
        let moved_over = self.drawing.clone();
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
                hover_on_move.call(moved_over.hover(at));
            }))
            .on_pointer_exited(Callback::new(move |_: PointerEventInfo| {
                pointer_on_exit.set(None);
                on_hover.call(None);
            }))
            .content(surface)
            .into()
    }
}

/// The chart's image while it is on screen. Taken off, it lets the surface
/// go, and the shared device with it once no other chart holds it - the page
/// keeps the [`Chart`], so nothing else would.
#[derive(Clone)]
struct Mounted {
    host: ElementRef<Grid>,
    image: ElementRef<Image>,
    drawing: Rc<Drawing>,
    /// Whether the chart moves on by itself.
    live: bool,
}

impl PartialEq for Mounted {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.drawing, &other.drawing) && self.live == other.live
    }
}

/// A live chart's next pixel is due.
struct Moved;

/// How soon a chart that has just gone live first moves.
const FIRST_MOVE: Duration = Duration::from_millis(16);

/// The mounted image, and for a live chart the timer that moves it on. The
/// timer is a component's because only a component can have one; it stops
/// when the chart stops being live or leaves the screen, and goes with it.
struct Mount {
    drawing: Rc<Drawing>,
    timer: Option<ComponentTimer>,
}

impl Mount {
    fn start(&mut self, after: Duration, cx: &ComponentContext<Self>) {
        self.timer = Some(cx.set_timeout(after, Moved));
    }
}

impl Component for Mount {
    type Input = Mounted;
    type Message = Moved;

    fn create(input: &Mounted, cx: &ComponentContext<Self>) -> Self {
        let mut mount = Self {
            drawing: input.drawing.clone(),
            timer: None,
        };
        if input.live {
            mount.start(FIRST_MOVE, cx);
        }
        mount
    }

    fn input_changed(&mut self, input: &Mounted, cx: &ComponentContext<Self>) {
        self.drawing = input.drawing.clone();
        if input.live && self.timer.is_none() {
            self.start(FIRST_MOVE, cx);
        }
    }

    fn update(&mut self, _moved: Moved, cx: &ComponentContext<Self>) {
        self.timer = None;
        if let Some(next) = self.drawing.tick() {
            self.start(next, cx);
        }
    }

    fn view(&self, mounted: &Mounted, cx: &mut ViewContext<Self>) -> View {
        let drawing = mounted.drawing.clone();
        cx.use_effect("surface", (), move || {
            Some(Box::new(move || drawing.surface.borrow_mut().release()))
        });

        Grid::new()
            .element_ref(&mounted.host)
            .children((Image::new()
                .element_ref(&mounted.image)
                .stretch(Stretch::Fill),))
            .into()
    }
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
        chart.drawing.series.borrow()[0].points.clone()
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
        assert_eq!(chart.drawing.series.borrow().len(), 1);
        assert_eq!(points(&chart), [(0, 0.0)]);
    }

    #[test]
    fn restyled_the_chart_keeps_its_series() {
        let chart = Chart::new();
        chart.publish(vec![series(&[(0, 0.0)])], LineChartOptions::default());

        chart.restyle(windowed(10));
        assert_eq!(chart.drawing.options.borrow().x_window, Some(10));
        assert_eq!(points(&chart), [(0, 0.0)]);
    }
}
