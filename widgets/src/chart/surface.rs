//! Where the chart's pixels live: a Direct2D surface owned by the system
//! compositor, and the bookkeeping that keeps it the right size.
//!
//! The widget above talks to [`ChartHost`] and never touches a surface
//! directly - size, rasterisation scale and the surface itself arrive from
//! three independent callbacks that can fire in any order, and reconciling
//! them is this module's whole job.

use std::cell::Cell;

use windows_canvas::{CanvasCompositionExt, GpuDevice, Vector2};
use windows_composition::{
    CompositionDrawingSurface, CompositionGraphicsDevice, CompositionSurfaceBrush, SpriteVisual,
    Stretch,
};
use windows_reactor::{CompositionHostHandle, DrawContext, HookRef, RenderCx, Result};

use super::model::{ChartRevision, LineChartOptions, Series, chart_revision};
use super::paint::render;

/// The chart's Direct2D content, drawn into a composition surface that a WinUI
/// host element paints with.
///
/// A drawing surface is sized in physical pixels while the visual painting it
/// is sized in DIPs, so the surface is allocated at `size * scale` and the
/// brush is pinned to [`Stretch::Fill`]; the drawing itself stays in DIPs,
/// which is what keeps stroke widths and grid spacing the same physical size
/// at any scale.
struct ChartSurface {
    device: GpuDevice,
    surface: CompositionDrawingSurface,
    width: f32,
    height: f32,
    scale: f32,
    // Held for their lifetime alone. The host element references the visual,
    // the visual its brush, and the brush the surface, so the chain is already
    // anchored - but that chain is owned by the element, and this keeps it from
    // depending on when the element lets go.
    _graphics: CompositionGraphicsDevice,
    _brush: CompositionSurfaceBrush,
    _visual: SpriteVisual,
}

fn surface_pixels(dip: f32, scale: f32) -> f32 {
    (dip * scale).max(1.0)
}

impl ChartSurface {
    fn new(host: &CompositionHostHandle, width: f32, height: f32, scale: f32) -> Result<Self> {
        let compositor = host.compositor()?;
        let device = GpuDevice::new_or_warp()?;
        let graphics = device.create_graphics_device(&compositor)?;
        let surface = graphics
            .create_drawing_surface(surface_pixels(width, scale), surface_pixels(height, scale))?;

        let brush = compositor.create_surface_brush(&surface);
        brush.set_stretch(Stretch::Fill);

        let visual = compositor.create_sprite_visual();
        visual.set_brush(&brush);
        // Tracks the host's size on its own, so a layout pass that does not
        // reach `on_resize` still leaves the visual covering the element.
        visual.set_relative_size_adjustment(Vector2 { x: 1.0, y: 1.0 });
        host.set_child_visual(&visual)?;

        Ok(Self {
            device,
            surface,
            width,
            height,
            scale,
            _graphics: graphics,
            _brush: brush,
            _visual: visual,
        })
    }

    fn resize(&mut self, width: f32, height: f32, scale: f32) -> Result<()> {
        if (self.width, self.height, self.scale) == (width, height, scale) {
            return Ok(());
        }
        self.surface.resize(
            surface_pixels(width, scale) as i32,
            surface_pixels(height, scale) as i32,
        )?;
        self.width = width;
        self.height = height;
        self.scale = scale;
        Ok(())
    }

    /// Redraws the chart. `Ok(false)` means the GPU device was lost and the
    /// whole surface has to be rebuilt.
    fn draw(&self, series: &[Series], options: &LineChartOptions) -> Result<bool> {
        if self.width <= 0.0 || self.height <= 0.0 {
            return Ok(true);
        }
        self.surface.draw_with_dpi(96.0 * self.scale, |session| {
            render(
                &DrawContext::from_session(session, &self.device, self.width, self.height),
                series,
                options,
            );
            Ok(())
        })
    }
}

/// The chart's surface, its geometry, and the data waiting to be drawn on it.
///
/// Every field is a `use_ref` cell rather than a plain `Rc`: the mount and
/// resize callbacks are registered once and outlive the render that made them,
/// so anything they write to has to outlive it too. (A `Rc::new` here used to
/// leave the callbacks writing into the cells of the first render while later
/// renders read fresh, empty ones - the chart still drew, because its own
/// callbacks agreed with each other, and only hit-testing noticed that the
/// width it read was always zero.)
#[derive(Clone)]
pub(super) struct ChartHost {
    surface: HookRef<Option<ChartSurface>>,
    handle: HookRef<Option<CompositionHostHandle>>,
    size: HookRef<Cell<(f32, f32)>>,
    scale: HookRef<Cell<f32>>,
    series: HookRef<Vec<Series>>,
    options: HookRef<LineChartOptions>,
}

impl ChartHost {
    pub(super) fn new(cx: &mut RenderCx) -> Self {
        Self {
            surface: cx.use_ref(None),
            handle: cx.use_ref(None),
            size: cx.use_ref(Cell::new((0.0, 0.0))),
            scale: cx.use_ref(Cell::new(1.0)),
            series: cx.use_ref(Vec::new()),
            options: cx.use_ref(LineChartOptions::default()),
        }
    }

    /// Hands over this render's data. Drawing does not happen here - see
    /// [`ChartHost::redraw`].
    pub(super) fn publish(&self, series: Vec<Series>, options: LineChartOptions) {
        self.series.set(series);
        self.options.set(options);
    }

    pub(super) fn revision(&self) -> ChartRevision {
        chart_revision(&self.series.borrow())
    }

    /// The width the chart was last laid out at, in DIPs - what a pointer
    /// position has to be read against.
    pub(super) fn width(&self) -> f32 {
        self.size.borrow().get().0
    }

    pub(super) fn series(&self) -> std::cell::Ref<'_, Vec<Series>> {
        self.series.borrow()
    }

    /// Takes the host element and follows its rasterisation scale from here on.
    pub(super) fn attach(&self, handle: CompositionHostHandle) {
        self.handle.set(Some(handle.clone()));

        let this = self.clone();
        match handle.on_rasterization_scale_changed(move |s| {
            this.scale.borrow().set(s as f32);
            this.sync();
        }) {
            // Downgraded to a token: a dropped revoker would unsubscribe right
            // here, and holding one instead keeps the host element alive for as
            // long as this closure does. The subscription dies with the element.
            Ok(revoker) => {
                let _ = revoker.into_token();
            }
            Err(e) => tracing::warn!(error = %e, "line_chart: no rasterization scale updates"),
        }

        self.sync();
    }

    pub(super) fn resize(&self, width: f32, height: f32) {
        self.size.borrow().set((width, height));
        self.sync();
    }

    pub(super) fn detach(&self) {
        self.surface.set(None);
        self.handle.set(None);
    }

    /// Builds the surface, or resizes the one there is, and draws. The single
    /// path mount, resize and scale changes all funnel through.
    pub(super) fn sync(&self) {
        let Some(host) = self.handle.borrow().clone() else {
            return;
        };
        let (width, height) = self.size.borrow().get();
        let scale = self.scale.borrow().get();

        if self.surface.borrow().is_none() {
            match ChartSurface::new(&host, width, height, scale) {
                Ok(surface) => self.surface.set(Some(surface)),
                Err(e) => {
                    tracing::warn!(error = %e, "line_chart: failed to create composition surface");
                    return;
                }
            }
        } else if let Some(surface) = self.surface.borrow_mut().as_mut()
            && let Err(e) = surface.resize(width, height, scale)
        {
            tracing::warn!(error = %e, "line_chart: failed to resize composition surface");
            return;
        }

        self.redraw();
    }

    /// Draws the current data, rebuilding the surface once if the GPU device
    /// was lost underneath it.
    pub(super) fn redraw(&self) {
        let series = self.series.borrow();
        let options = *self.options.borrow();

        let lost = match self.surface.borrow().as_ref() {
            Some(surface) => match surface.draw(&series, &options) {
                Ok(true) => return,
                Ok(false) => Some((surface.width, surface.height, surface.scale)),
                Err(e) => {
                    tracing::warn!(error = %e, "line_chart: draw failed");
                    return;
                }
            },
            None => return,
        };

        let (Some((width, height, scale)), Some(host)) = (lost, self.handle.borrow().clone())
        else {
            return;
        };
        match ChartSurface::new(&host, width, height, scale) {
            Ok(surface) => {
                let _ = surface.draw(&series, &options);
                self.surface.set(Some(surface));
            }
            Err(e) => {
                tracing::warn!(error = %e, "line_chart: failed to rebuild surface after device loss")
            }
        }
    }
}
