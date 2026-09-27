//! Where a chart is drawn: an image surface on a device every chart on the
//! thread shares.
//!
//! A canvas of its own made each chart its own Direct3D and Direct2D device
//! and swap chain - some sixteen megabytes and a dozen driver threads apiece.
//! An image surface costs a texture, and the one device is made when the first
//! chart draws and let go when the last one is dropped.
//!
//! A lost device is dropped for everyone. The chart that found it lost makes
//! the next one straight away; the others notice their surface belongs to the
//! old device the next time they draw, and build a new one.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use windows_canvas::{CanvasImageSource, ColorF, DrawingSession, GpuDevice, Result};
use windows_reactor::{ElementRef, Image};

thread_local! {
    static DEVICE: RefCell<Weak<GpuDevice>> = const { RefCell::new(Weak::new()) };
}

fn shared() -> Result<Rc<GpuDevice>> {
    DEVICE.with_borrow_mut(|slot| {
        if let Some(device) = slot.upgrade() {
            return Ok(device);
        }

        let device = Rc::new(GpuDevice::new_or_warp()?);
        *slot = Rc::downgrade(&device);
        Ok(device)
    })
}

fn lost(device: &Rc<GpuDevice>) {
    DEVICE.with_borrow_mut(|slot| {
        if slot
            .upgrade()
            .is_some_and(|current| Rc::ptr_eq(&current, device))
        {
            *slot = Weak::new();
        }
    });
}

/// How big a surface is: its size in DIPs and its pixels per DIP.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Metrics {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

impl Metrics {
    pub fn new(width: f64, height: f64, scale: f64) -> Self {
        Self {
            width: width as f32,
            height: height as f32,
            scale: scale as f32,
        }
    }
}

struct Drawn {
    device: Rc<GpuDevice>,
    source: CanvasImageSource,
    metrics: Metrics,
    attached: bool,
}

/// One chart's surface, attached to its image.
pub(super) struct Surface {
    image: ElementRef<Image>,
    metrics: Option<Metrics>,
    drawn: Option<Drawn>,
}

impl Surface {
    pub fn new(image: ElementRef<Image>) -> Self {
        Self {
            image,
            metrics: None,
            drawn: None,
        }
    }

    /// Lets the surface and its hold on the device go, for a chart that is off
    /// screen, and forgets the size, so that data arriving meanwhile draws
    /// nothing until the image is laid out again.
    pub fn release(&mut self) {
        self.drawn = None;
        self.metrics = None;
    }

    /// Says the image is a new one, which has no surface on it yet.
    pub fn detached(&mut self) {
        if let Some(drawn) = self.drawn.as_mut() {
            drawn.attached = false;
        }
    }

    /// Takes the size the image is laid out at. `true` when it changed.
    pub fn resize(&mut self, metrics: Metrics) -> bool {
        if self.metrics == Some(metrics) {
            return false;
        }
        self.metrics = Some(metrics);
        true
    }

    /// Draws with `paint`, handed the session, the device and the size in
    /// DIPs. Does nothing until the image has a size.
    pub fn draw(&mut self, paint: impl Fn(&DrawingSession<'_>, &GpuDevice, (f32, f32))) {
        let Some(metrics) = self.metrics else {
            return;
        };
        if metrics.width <= 0.0 || metrics.height <= 0.0 {
            return;
        }

        for _ in 0..2 {
            let device = match shared() {
                Ok(device) => device,
                Err(error) => return tracing::warn!(%error, "chart: no graphics device"),
            };

            let current = self.drawn.as_ref().is_some_and(|drawn| {
                Rc::ptr_eq(&drawn.device, &device) && drawn.metrics == metrics
            });
            if !current {
                match CanvasImageSource::new(&device, metrics.width, metrics.height, metrics.scale)
                {
                    Ok(source) => {
                        self.drawn = Some(Drawn {
                            device: device.clone(),
                            source,
                            metrics,
                            attached: false,
                        });
                    }
                    Err(error) => return tracing::warn!(%error, "chart: no surface"),
                }
            }

            let Some(drawn) = self.drawn.as_mut() else {
                return;
            };

            let size = (metrics.width, metrics.height);
            match drawn.source.draw(ColorF::TRANSPARENT, |session| {
                paint(session, &drawn.device, size);
                Ok(())
            }) {
                Ok(true) => {
                    if !drawn.attached {
                        drawn.attached = drawn.source.attach_result(&self.image, |result| {
                            if let Err(error) = result {
                                tracing::warn!(?error, "chart: the surface did not attach");
                            }
                        });
                    }
                    return;
                }
                Ok(false) => {
                    lost(&drawn.device);
                    self.drawn = None;
                }
                Err(error) => return tracing::warn!(%error, "chart: drawing failed"),
            }
        }
    }
}
