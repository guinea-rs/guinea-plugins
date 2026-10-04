//! Turning a chart into strokes and fills. Nothing here knows where the
//! surface came from or how it reaches the screen.

use std::mem::ManuallyDrop;

use windows_canvas::{
    Brush, ColorF, DrawingSession, GpuDevice, Path, PathBuilder, Rect, Result as CanvasResult,
    RoundedRect, Vector2,
};

use windows_core::{Error, Interface};
use windows_numerics::Matrix3x2;

use super::d2d::{
    D2D_RECT_F, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE, D2D1_LAYER_OPTIONS1_NONE,
    D2D1_LAYER_PARAMETERS1, D2D1_ROUNDED_RECT, ID2D1DeviceContext, ID2D1Geometry,
};
use super::geometry::Frame;
use super::model::{ChartGrid, Interpolation, LineChartOptions, Series};
use crate::color::hex;
use crate::painted::Metrics;

const LINE_WIDTH: f32 = 1.0;
/// Vertical lines closer than this many device pixels are not drawn: they
/// would be a fill.
const GRID_LEAST_SPACING: f32 = 4.0;
pub(super) const BACKGROUND_TOP: ColorF = hex(0x1c1e26);

/// Draws the chart, its time axis ending at `end` when a live chart's clock
/// says where that is.
pub(super) fn render(
    draw: &DrawingSession<'_>,
    device: &GpuDevice,
    metrics: Metrics,
    series: &[Series],
    options: &LineChartOptions,
    end: Option<f64>,
) {
    let Metrics { width, height, .. } = metrics;
    if width <= 0.0 || height <= 0.0 {
        tracing::warn!(
            width,
            height,
            "line_chart: draw surface has zero size, skipping this frame"
        );
        return;
    }

    match options.corner_radius.filter(|radius| *radius > 0.0) {
        Some(radius) => clipped(draw, width, height, radius, || {
            content(draw, device, metrics, series, options, end)
        }),
        None => content(draw, device, metrics, series, options, end),
    }
}

/// Draws `inside` clipped to a rectangle of `width` by `height` rounded to
/// `radius` - or unclipped, and says so, when Direct2D will not give a layer.
pub(super) fn clipped(
    draw: &DrawingSession<'_>,
    width: f32,
    height: f32,
    radius: f32,
    inside: impl FnOnce(),
) {
    let mask = || -> windows_core::Result<(ID2D1DeviceContext, ID2D1Geometry)> {
        let raw = windows_core::Interface::as_raw(draw.raw());
        let context = unsafe { ID2D1DeviceContext::from_raw_borrowed(&raw) }
            .cloned()
            .ok_or_else(Error::empty)?;
        let factory = unsafe { context.GetFactory()? };
        let geometry = unsafe {
            factory.CreateRoundedRectangleGeometry(&D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: width,
                    bottom: height,
                },
                radiusX: radius,
                radiusY: radius,
            })?
        };
        Ok((context, geometry.cast()?))
    };

    let (context, geometry) = match mask() {
        Ok(mask) => mask,
        Err(error) => {
            tracing::warn!(%error, "line_chart: no layer to round the corners with");
            return inside();
        }
    };

    let layer = D2D1_LAYER_PARAMETERS1 {
        contentBounds: D2D_RECT_F {
            left: 0.0,
            top: 0.0,
            right: width,
            bottom: height,
        },
        geometricMask: ManuallyDrop::new(Some(geometry)),
        maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
        maskTransform: Matrix3x2::identity(),
        opacity: 1.0,
        opacityBrush: ManuallyDrop::new(None),
        layerOptions: D2D1_LAYER_OPTIONS1_NONE,
    };
    unsafe { context.PushLayer(&layer, None) };
    inside();
    unsafe { context.PopLayer() };
    drop(ManuallyDrop::into_inner(layer.geometricMask));
}

fn content(
    draw: &DrawingSession<'_>,
    device: &GpuDevice,
    metrics: Metrics,
    series: &[Series],
    options: &LineChartOptions,
    end: Option<f64>,
) {
    let Metrics { width, height, .. } = metrics;
    let pixel = Pixel::of(metrics);

    if let Some(background) = options.background {
        draw_backdrop(draw, width, height, background);
    }

    let frame = Frame::of(series, options.x_window, options.y_range, end);
    if let (Some(grid), Some(frame)) = (&options.grid, &frame) {
        match draw.create_solid_brush(grid.color) {
            Ok(brush) => draw_grid(draw, &brush, grid, frame, pixel),
            Err(e) => tracing::warn!(error = %e, "line_chart: failed to create grid brush"),
        }
    }

    if let Some(border) = options.border {
        match draw.create_solid_brush(border) {
            Ok(brush) => draw_border(draw, &brush, pixel, options.corner_radius),
            Err(e) => tracing::warn!(error = %e, "line_chart: failed to create border brush"),
        }
    }

    let Some(frame) = frame else {
        tracing::trace!("line_chart: no points in any series yet, nothing to draw");
        return;
    };

    let to_screen = move |t: u64, v: f32| -> Vector2 {
        Vector2 {
            x: frame.x(t, width),
            y: frame.y(v, height),
        }
    };

    for s in series {
        if s.points.len() < 2 {
            tracing::trace!(
                points = s.points.len(),
                "line_chart: series has fewer than 2 points, skipping"
            );
            continue;
        }

        if let Some(fill_color) = s.fill {
            match build_path(device, s, &to_screen, height, true) {
                Ok(path) => match draw.create_solid_brush(fill_color) {
                    Ok(brush) => draw.fill_path(&path, &brush),
                    Err(e) => tracing::warn!(error = %e, "line_chart: failed to create fill brush"),
                },
                Err(e) => tracing::warn!(error = %e, "line_chart: failed to build fill path"),
            }
        }

        match build_path(device, s, &to_screen, height, false) {
            Ok(path) => match draw.create_solid_brush(s.color) {
                Ok(brush) => draw.draw_path(&path, &brush, LINE_WIDTH),
                Err(e) => tracing::warn!(error = %e, "line_chart: failed to create line brush"),
            },
            Err(e) => tracing::warn!(error = %e, "line_chart: failed to build line path"),
        }
    }
}

pub(super) fn draw_backdrop(
    draw: &DrawingSession<'_>,
    width: f32,
    height: f32,
    background: ColorF,
) {
    let rect = Rect::from_xywh(0.0, 0.0, width, height);
    match draw.create_solid_brush(background) {
        Ok(brush) => draw.fill_rect(&rect, &brush),
        Err(e) => tracing::warn!(error = %e, "line_chart: failed to create background brush"),
    }
}

/// Where the surface's device pixels fall, in DIPs: a hairline one pixel wide
/// on a pixel's middle covers that pixel and no other, where one DIP wide at
/// any other place smears across two at a fractional scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Pixel {
    /// Device pixels per DIP.
    scale: f32,
    pub(super) width: f32,
    pub(super) height: f32,
}

impl Pixel {
    pub(super) fn of(metrics: Metrics) -> Self {
        Self {
            scale: if metrics.scale > 0.0 {
                metrics.scale
            } else {
                1.0
            },
            width: metrics.width,
            height: metrics.height,
        }
    }

    /// One device pixel, in DIPs.
    pub(super) fn hair(self) -> f32 {
        1.0 / self.scale
    }

    /// The middle of the pixel `at` falls in, kept within `extent`.
    pub(super) fn snap(self, at: f32, extent: f32) -> f32 {
        let last = ((extent * self.scale).ceil() - 1.0).max(0.0);
        ((at * self.scale).floor().clamp(0.0, last) + 0.5) / self.scale
    }
}

pub(super) fn draw_border(
    draw: &DrawingSession<'_>,
    brush: &Brush,
    pixel: Pixel,
    radius: Option<f32>,
) {
    let half = pixel.hair() / 2.0;
    let rect = Rect::new(half, half, pixel.width - half, pixel.height - half);
    match radius.filter(|radius| *radius > 0.0) {
        Some(radius) => draw.draw_rounded_rect(
            &RoundedRect::uniform(rect, (radius - half).max(0.0)),
            brush,
            pixel.hair(),
        ),
        None => draw.draw_rect(&rect, brush, pixel.hair()),
    }
}

fn draw_grid(
    draw: &DrawingSession<'_>,
    brush: &Brush,
    grid: &ChartGrid,
    frame: &Frame,
    pixel: Pixel,
) {
    let Pixel { width, height, .. } = pixel;

    for &v in &grid.at_v {
        if v < frame.min_v || v > frame.max_v {
            continue;
        }
        let y = pixel.snap(frame.y(v, height), height);
        draw.draw_line(
            Vector2 { x: 0.0, y },
            Vector2 { x: width, y },
            brush,
            pixel.hair(),
        );
    }

    let Some(every) = grid.every_t else {
        return;
    };
    let spacing = every as f64 / (frame.to - frame.from) * (width * pixel.scale) as f64;
    if spacing < GRID_LEAST_SPACING as f64 {
        return;
    }
    for t in frame.multiples(every) {
        let x = pixel.snap(frame.x(t, width), width);
        draw.draw_line(
            Vector2 { x, y: 0.0 },
            Vector2 { x, y: height },
            brush,
            pixel.hair(),
        );
    }
}

fn build_path(
    device: &GpuDevice,
    series: &Series,
    to_screen: &impl Fn(u64, f32) -> Vector2,
    height: f32,
    filled: bool,
) -> CanvasResult<Path> {
    let screen_points: Vec<Vector2> = series
        .points
        .iter()
        .map(|&(t, v)| to_screen(t, v))
        .collect();
    let first = screen_points[0];

    let builder = PathBuilder::new(device)?;
    let mut figure = if filled {
        builder
            .begin(Vector2 {
                x: first.x,
                y: height,
            })
            .line_to(first)
    } else {
        builder.begin_hollow(first)
    };

    let last_idx = screen_points.len() - 1;
    for i in 0..last_idx {
        let prev = screen_points[i];
        let next = screen_points[i + 1];
        figure = match series.interpolation {
            Interpolation::Linear => figure.line_to(next),
            Interpolation::Step => figure
                .line_to(Vector2 {
                    x: next.x,
                    y: prev.y,
                })
                .line_to(next),
            Interpolation::Smooth => {
                // Catmull-Rom through (before, prev, next, after), converted to a cubic
                // bezier - control points are derived from the *neighboring* points so
                // the tangent is continuous across segment boundaries. A per-segment S-curve
                // (fixed control points at this segment's own midpoint) looks smooth in
                // isolation but has a slope discontinuity at every sample, which reads as
                // jagged/zigzaggy once points are dense relative to how fast the series moves.
                let before = if i == 0 { prev } else { screen_points[i - 1] };
                let after = if i + 1 == last_idx {
                    next
                } else {
                    screen_points[i + 2]
                };
                let c1 = Vector2 {
                    x: prev.x + (next.x - before.x) / 6.0,
                    y: prev.y + (next.y - before.y) / 6.0,
                };
                let c2 = Vector2 {
                    x: next.x - (after.x - prev.x) / 6.0,
                    y: next.y - (after.y - prev.y) / 6.0,
                };
                figure.bezier_to(c1, c2, next)
            }
        };
    }

    if filled {
        let last = *screen_points.last().expect("checked len >= 2 above");
        figure
            .line_to(Vector2 {
                x: last.x,
                y: height,
            })
            .close()
            .build()
    } else {
        figure.end_open().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(scale: f32) -> Pixel {
        Pixel::of(Metrics {
            width: 100.0,
            height: 40.0,
            scale,
        })
    }

    #[test]
    fn a_hairline_sits_in_the_middle_of_one_device_pixel() {
        let pixel = at(1.25);
        assert_eq!(pixel.hair(), 0.8);
        let x = pixel.snap(10.3, 100.0);
        assert_eq!(x * 1.25, 12.5, "pixel 12, across its middle");
        assert_eq!(at(1.0).snap(10.9, 100.0), 10.5);
        assert_eq!(at(2.0).snap(10.3, 100.0), 10.25);
    }

    #[test]
    fn a_hairline_at_an_edge_stays_on_the_surface() {
        let pixel = at(1.25);
        assert_eq!(pixel.snap(100.0, 100.0) * 1.25, 124.5);
        assert_eq!(pixel.snap(-3.0, 100.0) * 1.25, 0.5);
        assert_eq!(at(0.0).hair(), 1.0, "no scale yet reads as one");
    }
}
