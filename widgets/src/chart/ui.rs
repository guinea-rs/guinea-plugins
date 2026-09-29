//! The chart widget: an image the page owns, and a pointer over it.
//!
//! The image shows a surface on the device every chart on the thread shares
//! (see `surface`), drawn when the data moves or the image changes size. The
//! grid around it reports that size.
//!
//! What is left is what was ours to begin with: when to redraw, and where the
//! pointer is. Both live on [`Chart`], which the page keeps as a field - a
//! page is an Elm node, so its state is its struct, and a chart's state is
//! part of the page's.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows_reactor::{
    Border, Callback, ChildrenControl, Color, Component, ComponentContext, CompositionHostEvent,
    ContentControl, ElementObservation, ElementRef, Grid, Image, IntoPayloadCallback,
    PointerEventInfo, Stretch, View, ViewContext,
};

use super::hover::hover_at;
use super::model::{ChartRevision, HoverInfo, LineChartOptions, Series, chart_revision};
use super::paint;
use super::surface::{Metrics, Surface};

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
    host: ElementRef<Grid>,
    image: ElementRef<Image>,
    surface: Rc<RefCell<Surface>>,
    _sized: ElementObservation,
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart {
    pub fn new() -> Self {
        let series = Rc::new(RefCell::new(Vec::new()));
        let options = Rc::new(RefCell::new(LineChartOptions::default()));
        let width = Rc::new(Cell::new(0.0));
        let host = ElementRef::new();
        let image = ElementRef::new();
        let surface = Rc::new(RefCell::new(Surface::new(image.clone())));

        let sized = host.observe_composition_host({
            let series = series.clone();
            let options = options.clone();
            let measured = width.clone();
            let surface = surface.clone();

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
                measured.set(metrics.width);

                let mut surface = surface.borrow_mut();
                if rebound {
                    surface.detached();
                }
                if surface.resize(metrics) || rebound {
                    surface.draw(|session, device, metrics| {
                        paint::render(
                            session,
                            device,
                            metrics,
                            &series.borrow(),
                            &options.borrow(),
                        );
                    });
                }
            }
        });

        Self {
            series,
            options,
            width,
            pointer: Rc::new(Cell::new(None)),
            drawn: Cell::new(None),
            host,
            image,
            surface,
            _sized: sized,
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
        let restyled = *self.options.borrow() != options;
        *self.series.borrow_mut() = series;
        *self.options.borrow_mut() = options;

        if restyled || self.drawn.get() != Some(revision) {
            self.drawn.set(Some(revision));

            let (series, options) = (self.series.borrow(), self.options.borrow());
            self.surface.borrow_mut().draw(|session, device, metrics| {
                paint::render(session, device, metrics, &series, &options);
            });
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
        hover_at(
            &self.series.borrow(),
            &self.options.borrow(),
            at,
            self.width.get(),
        )
    }

    /// The chart, drawn.
    ///
    /// `on_hover` takes what every reactor widget takes: a plain closure, or a
    /// `Callback` a segment already made with `cx.on(..)`.
    pub fn view(&self, on_hover: impl IntoPayloadCallback<Option<HoverInfo>>) -> View {
        let surface = View::component::<Mount>(Mounted {
            host: self.host.clone(),
            image: self.image.clone(),
            surface: self.surface.clone(),
        });

        let on_hover = on_hover.into_payload_callback();
        let moved_over = self.series.clone();
        let options_on_move = self.options.clone();
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
                    &options_on_move.borrow(),
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

/// The chart's image while it is on screen. Taken off, it lets the surface
/// go, and the shared device with it once no other chart holds it - the page
/// keeps the [`Chart`], so nothing else would.
#[derive(Clone)]
struct Mounted {
    host: ElementRef<Grid>,
    image: ElementRef<Image>,
    surface: Rc<RefCell<Surface>>,
}

impl PartialEq for Mounted {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.surface, &other.surface)
    }
}

struct Mount;

impl Component for Mount {
    type Input = Mounted;
    type Message = ();

    fn create(_input: &Mounted, _cx: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, mounted: &Mounted, cx: &mut ViewContext<Self>) -> View {
        let surface = mounted.surface.clone();
        cx.use_effect("surface", (), move || {
            Some(Box::new(move || surface.borrow_mut().release()))
        });

        Grid::new()
            .element_ref(&mounted.host)
            .children((Image::new()
                .element_ref(&mounted.image)
                .stretch(Stretch::Fill),))
    }
}
