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
    Border, Callback, ChildrenControl, Color, Component, ComponentContext, ComponentTimer,
    CompositionHostEvent, ContentControl, ElementObservation, ElementRef, Grid, Image,
    IntoPayloadCallback, PointerEventInfo, Stretch, View, ViewContext,
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

        let mut clock = drawing.clock.get();
        match (options.live, options.x_window, newest(&series)) {
            (Some(live), Some(window), Some(newest)) => {
                clock.saw(newest as f64, window as f64, live.per_second, Instant::now())
            }
            (Some(_), Some(_), None) => {}
            _ => clock.forget(),
        }
        drawing.clock.set(clock);

        *drawing.series.borrow_mut() = series;
        *drawing.options.borrow_mut() = options;

        if first || restyled || moved {
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
                let _ = hover_on_move.call(moved_over.hover(at));
            }))
            .on_pointer_exited(Callback::new(move |_: PointerEventInfo| {
                pointer_on_exit.set(None);
                let _ = on_hover.call(None);
            }))
            .content(surface)
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
        match cx.set_timeout(after, Moved) {
            Ok(timer) => self.timer = Some(timer),
            Err(error) => tracing::warn!(%error, "chart: no timer to move a live chart with"),
        }
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
    }
}
