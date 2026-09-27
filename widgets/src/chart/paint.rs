//! Turning a chart into strokes and fills. Nothing here knows where the
//! surface came from or how it reaches the screen.

use windows_canvas::{
    Brush, ColorF, DrawingSession, GpuDevice, Path, PathBuilder, Rect, Result as CanvasResult,
    Vector2,
};

use super::bounds;
use super::model::{Interpolation, LineChartOptions, Series};
use crate::color::{hex, hex_alpha};

const LINE_WIDTH: f32 = 1.0;
const GRID_LINE_WIDTH: f32 = 1.0;
const GRID_COLOR: ColorF = hex_alpha(0xffffff, 20);
const GRID_TARGET_SPACING_X: f32 = 80.0;
const GRID_TARGET_SPACING_Y: f32 = 40.0;
pub(super) const BACKGROUND_TOP: ColorF = hex(0x1c1e26);
const BORDER_WIDTH: f32 = 1.0;

pub(super) fn render(
    draw: &DrawingSession<'_>,
    device: &GpuDevice,
    (width, height): (f32, f32),
    series: &[Series],
    options: &LineChartOptions,
) {
    if width <= 0.0 || height <= 0.0 {
        tracing::warn!(
            width,
            height,
            "line_chart: draw surface has zero size, skipping this frame"
        );
        return;
    }

    if let Some(background) = options.background {
        draw_backdrop(draw, width, height, background);
    }
    if options.show_grid
        && let Ok(grid_brush) = draw.create_solid_brush(GRID_COLOR)
    {
        draw_grid(draw, &grid_brush, width, height);
    }
    if let Some(border) = options.border {
        let rect = Rect::from_xywh(0.0, 0.0, width, height);
        match draw.create_solid_brush(border) {
            Ok(brush) => draw.draw_rect(&rect, &brush, BORDER_WIDTH),
            Err(e) => tracing::warn!(error = %e, "line_chart: failed to create border brush"),
        }
    }

    let Some((min_t, max_t, min_v, max_v)) = bounds(series) else {
        tracing::trace!("line_chart: no points in any series yet, nothing to draw");
        return;
    };

    let (min_v, max_v) = options.y_range.unwrap_or((min_v, max_v));

    let t_span = (max_t - min_t).max(1) as f32;
    let v_span = (max_v - min_v).max(f32::EPSILON);

    let to_screen = move |t: u64, v: f32| -> Vector2 {
        let x = ((t - min_t) as f32 / t_span) * width;
        let y = height - ((v - min_v) / v_span) * height;
        Vector2 { x, y }
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

fn draw_backdrop(draw: &DrawingSession<'_>, width: f32, height: f32, background: ColorF) {
    let rect = Rect::from_xywh(0.0, 0.0, width, height);
    match draw.create_solid_brush(background) {
        Ok(brush) => draw.fill_rect(&rect, &brush),
        Err(e) => tracing::warn!(error = %e, "line_chart: failed to create background brush"),
    }
}

fn draw_grid(draw: &DrawingSession<'_>, brush: &Brush, width: f32, height: f32) {
    let cols = (width / GRID_TARGET_SPACING_X).ceil().max(1.0) as u32;
    let rows = (height / GRID_TARGET_SPACING_Y).ceil().max(1.0) as u32;
    for row in 1..rows {
        let y = height * (row as f32 / rows as f32);
        draw.draw_line(
            Vector2 { x: 0.0, y },
            Vector2 { x: width, y },
            brush,
            GRID_LINE_WIDTH,
        );
    }
    for col in 1..cols {
        let x = width * (col as f32 / cols as f32);
        draw.draw_line(
            Vector2 { x, y: 0.0 },
            Vector2 { x, y: height },
            brush,
            GRID_LINE_WIDTH,
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
