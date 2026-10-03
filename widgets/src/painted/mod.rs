//! A view drawn with Direct2D, which the page keeps as a field.
//!
//! [`Painted`] is what a chart is drawn on, and what anything else drawn by
//! hand is drawn on too: an image showing a surface on the device every
//! painted view on the thread shares (see `surface`), drawn when it is told
//! to or when the image changes size, and for a picture that moves on by
//! itself, whenever it has moved by a device pixel. The pointer over it comes
//! back in DIPs, so what was drawn can be found again under it.

mod surface;

pub use surface::Metrics;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use windows_canvas::{DrawingSession, GpuDevice};
use windows_reactor::{
    Border, Callback, Color, Component, ComponentContext, ComponentTimer, CompositionHostEvent,
    ElementObservation, ElementRef, Grid, Image, IntoPayloadCallback, PointerEventInfo, Stretch,
    View, ViewContext,
};

use surface::Surface;

/// What a [`Painted`] view draws, and the state it draws from.
///
/// Its state is its own: the page reaches it through [`Painted::painter`],
/// so what changes between draws sits behind a `Cell` or a `RefCell`.
pub trait Paint: 'static {
    /// Draws the picture at `metrics`: its size in DIPs and its pixels per
    /// DIP.
    fn paint(&self, session: &DrawingSession<'_>, device: &GpuDevice, metrics: Metrics);

    /// Whether the picture moves on by itself, between the draws it is told
    /// to make.
    fn moving(&self) -> bool {
        false
    }

    /// For a picture that moves on by itself, how long after a draw at
    /// `metrics` it has moved by a device pixel.
    fn next_frame(&self, metrics: Metrics) -> Duration {
        let _ = metrics;
        Duration::from_millis(16)
    }
}

/// The pointer over a [`Painted`] view, at `x` and `y` in DIPs from its top
/// left corner.
///
/// The view captures the pointer when a button goes down on it, so a drag
/// that leaves it still reports its moves and its release.
#[derive(Clone, Debug, PartialEq)]
pub enum Pointer {
    Moved(PointerEventInfo),
    Pressed(PointerEventInfo),
    Released(PointerEventInfo),
    Exited,
    /// The capture ended. It comes just before the release at the end of
    /// every drag; with no release after it, the drag was taken away.
    Lost,
}

struct Shared<P> {
    painter: P,
    surface: RefCell<Surface>,
}

impl<P: Paint> Shared<P> {
    fn draw(&self) {
        self.surface
            .borrow_mut()
            .draw(|session, device, metrics| self.painter.paint(session, device, metrics));
    }

    fn tick(&self) -> Option<Duration> {
        if !self.painter.moving() {
            return None;
        }
        let metrics = self.surface.borrow().metrics()?;
        self.draw();
        Some(self.painter.next_frame(metrics))
    }
}

/// A view drawn by `P`, and what it needs between draws.
///
/// Held by the page rather than made per render: a surface that is drawn
/// only when something changed has to be kept, and so has the size it was
/// last laid out at.
pub struct Painted<P: Paint> {
    shared: Rc<Shared<P>>,
    host: ElementRef<Grid>,
    image: ElementRef<Image>,
    listeners: Listeners,
    _sized: ElementObservation,
}

impl<P: Paint + Default> Default for Painted<P> {
    fn default() -> Self {
        Self::new(P::default())
    }
}

impl<P: Paint> Painted<P> {
    pub fn new(painter: P) -> Self {
        let host = ElementRef::new();
        let image = ElementRef::new();
        let shared = Rc::new(Shared {
            painter,
            surface: RefCell::new(Surface::new(image.clone())),
        });

        let sized = host.observe_composition_host({
            let shared = shared.clone();

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

                let changed = {
                    let mut surface = shared.surface.borrow_mut();
                    if rebound {
                        surface.detached();
                    }
                    surface.resize(metrics) || rebound
                };
                if changed {
                    shared.draw();
                }
            }
        });

        Self {
            shared,
            host,
            image,
            listeners: Listeners::new(),
            _sized: sized,
        }
    }

    /// What draws the view, and the state it draws from.
    pub fn painter(&self) -> &P {
        &self.shared.painter
    }

    /// Draws the view again now. Does nothing until it is laid out, and
    /// nothing while it is off screen.
    pub fn redraw(&self) {
        self.shared.draw();
    }

    /// The size the view is laid out at, while it is on screen.
    pub fn metrics(&self) -> Option<Metrics> {
        self.shared.surface.borrow().metrics()
    }

    /// The view, drawn.
    ///
    /// `on_pointer` takes what every reactor widget takes: a plain closure,
    /// or a `Callback` a segment already made with `cx.on(..)`.
    pub fn view(&self, on_pointer: impl IntoPayloadCallback<Pointer>) -> View {
        let surface = View::component::<Mount<P>>(Mounted {
            host: self.host.clone(),
            image: self.image.clone(),
            shared: self.shared.clone(),
            moving: self.shared.painter.moving(),
        });

        *self.listeners.to.borrow_mut() = Some(on_pointer.into_payload_callback());
        let listeners = &self.listeners;

        Border::new()
            .background(Color::transparent())
            .capture_pointer_on_press(true)
            .on_pointer_pressed(listeners.pressed.clone())
            .on_pointer_moved(listeners.moved.clone())
            .on_pointer_released(listeners.released.clone())
            .on_pointer_exited(listeners.exited.clone())
            .on_pointer_capture_lost(listeners.lost.clone())
            .content(surface)
            .into()
    }
}

/// The view's pointer handlers, made once, so that a render does not hand
/// the border new ones: an event already on its way to the old handler
/// would be dropped. They pass the pointer on to whatever the latest
/// render was handed.
struct Listeners {
    to: Rc<RefCell<Option<Callback<Pointer>>>>,
    pressed: Callback<PointerEventInfo>,
    moved: Callback<PointerEventInfo>,
    released: Callback<PointerEventInfo>,
    exited: Callback<PointerEventInfo>,
    lost: Callback<()>,
}

impl Listeners {
    fn new() -> Self {
        let to: Rc<RefCell<Option<Callback<Pointer>>>> = Rc::default();
        let tell = |pointer: fn(PointerEventInfo) -> Pointer| {
            let to = to.clone();
            Callback::new(move |info: PointerEventInfo| {
                let listener = to.borrow().clone();
                if let Some(listener) = listener {
                    listener.call(pointer(info));
                }
            })
        };

        Self {
            pressed: tell(Pointer::Pressed),
            moved: tell(Pointer::Moved),
            released: tell(Pointer::Released),
            exited: tell(|_| Pointer::Exited),
            lost: {
                let to = to.clone();
                Callback::new(move |()| {
                    let listener = to.borrow().clone();
                    if let Some(listener) = listener {
                        listener.call(Pointer::Lost);
                    }
                })
            },
            to,
        }
    }
}

/// The view's image while it is on screen. Taken off, it lets the surface
/// go, and the shared device with it once no other view holds it.
struct Mounted<P> {
    host: ElementRef<Grid>,
    image: ElementRef<Image>,
    shared: Rc<Shared<P>>,
    moving: bool,
}

impl<P> Clone for Mounted<P> {
    fn clone(&self) -> Self {
        Self {
            host: self.host.clone(),
            image: self.image.clone(),
            shared: self.shared.clone(),
            moving: self.moving,
        }
    }
}

impl<P> PartialEq for Mounted<P> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.shared, &other.shared) && self.moving == other.moving
    }
}

/// A moving picture's next pixel is due.
struct Moved;

/// How soon a picture that has just started moving first moves.
const FIRST_MOVE: Duration = Duration::from_millis(16);

/// The mounted image, and for a moving picture the timer that moves it on.
/// The timer is a component's because only a component can have one; it
/// stops when the picture stops moving or leaves the screen, and goes with
/// it.
struct Mount<P> {
    shared: Rc<Shared<P>>,
    timer: Option<ComponentTimer>,
}

impl<P: Paint> Mount<P> {
    fn start(&mut self, after: Duration, cx: &ComponentContext<Self>) {
        self.timer = Some(cx.set_timeout(after, Moved));
    }
}

impl<P: Paint> Component for Mount<P> {
    type Input = Mounted<P>;
    type Message = Moved;

    fn create(input: &Mounted<P>, cx: &ComponentContext<Self>) -> Self {
        let mut mount = Self {
            shared: input.shared.clone(),
            timer: None,
        };
        if input.moving {
            mount.start(FIRST_MOVE, cx);
        }
        mount
    }

    fn input_changed(&mut self, input: &Mounted<P>, cx: &ComponentContext<Self>) {
        self.shared = input.shared.clone();
        if input.moving && self.timer.is_none() {
            self.start(FIRST_MOVE, cx);
        }
    }

    fn update(&mut self, _moved: Moved, cx: &ComponentContext<Self>) {
        self.timer = None;
        if let Some(next) = self.shared.tick() {
            self.start(next, cx);
        }
    }

    fn view(&self, mounted: &Mounted<P>, cx: &mut ViewContext<Self>) -> View {
        let shared = mounted.shared.clone();
        cx.use_effect("surface", (), move || {
            Some(Box::new(move || shared.surface.borrow_mut().release()))
        });

        Grid::new()
            .element_ref(&mounted.host)
            .children((Image::new()
                .element_ref(&mounted.image)
                .stretch(Stretch::Fill),))
            .into()
    }
}
