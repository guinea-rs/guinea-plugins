//! Turning a scatter chart into strokes, fills and labels.

use windows_canvas::{
    Brush, ColorF, DrawingSession, Ellipse, ParagraphAlignment, Rect, TextAlignment, TextFormat,
    Vector2, WordWrapping,
};

use super::super::paint::{Pixel, clipped, draw_backdrop, draw_border};
use super::model::{Area, Hit, Key, Level, Marker, ScatterOptions, ScatterSeries};
use super::plot::{LABELS_WIDE, Plot};
use crate::painted::Metrics;

const LABEL_SIZE: f32 = 11.0;
/// How tall a label's line is: it is kept inside the band or the scale it
/// names, so that labels at an edge of each do not run into one another.
const LABEL_TALL: f32 = 14.0;
const LABEL_GAP: f32 = 6.0;
const TICK_LONG: f32 = 4.0;
/// How wide a time's label may be; one at either end of the chart is kept
/// inside it rather than centred on its mark.
const TICK_LABEL_WIDE: f32 = 64.0;
const RING_WIDTH: f32 = 1.25;
const HOVER_GAP: f32 = 3.0;

/// The text formats labels are set in, made once.
pub(super) struct Labels {
    left: TextFormat,
    under: TextFormat,
}

impl Labels {
    pub fn new() -> Option<Self> {
        let made = || -> windows_canvas::Result<Self> {
            Ok(Self {
                left: TextFormat::new("Segoe UI", LABEL_SIZE)?
                    .with_alignment(TextAlignment::Trailing)
                    .with_paragraph_alignment(ParagraphAlignment::Center)
                    .with_word_wrapping(WordWrapping::NoWrap),
                under: TextFormat::new("Segoe UI", LABEL_SIZE)?
                    .with_alignment(TextAlignment::Center)
                    .with_paragraph_alignment(ParagraphAlignment::Top)
                    .with_word_wrapping(WordWrapping::NoWrap),
            })
        };
        made()
            .inspect_err(|error| tracing::warn!(%error, "scatter: no text format for labels"))
            .ok()
    }
}

/// What is drawn besides the data: the rectangle being dragged and the point
/// under the pointer.
pub(super) struct Pointing<'a, K> {
    pub brush: Option<((f32, f32), (f32, f32))>,
    pub hovered: Option<&'a Hit<K>>,
}

pub(super) fn render<K: Key>(
    draw: &DrawingSession<'_>,
    metrics: Metrics,
    plot: &Plot,
    series: &[ScatterSeries<K>],
    options: &ScatterOptions,
    pointing: &Pointing<'_, K>,
    labels: Option<&Labels>,
) {
    let Metrics { width, height, .. } = metrics;
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let content = || {
        let pixel = Pixel::of(metrics);
        if let Some(background) = options.background {
            draw_backdrop(draw, width, height, background);
        }
        if let Some(grid) = brush(draw, options.grid) {
            draw_grid(draw, &grid, plot, options, pixel);
        }
        if let Some(area) = options.selection {
            draw_area(draw, plot, area, options.accent, pixel);
        }
        draw_points(draw, plot, series, pointing.hovered, options.ink);
        if let Some((a, b)) = pointing.brush {
            draw_area(draw, plot, plot.area(a, b), options.accent, pixel);
        }
        if let (Some(labels), Some(ink)) = (labels, brush(draw, options.ink)) {
            draw_labels(draw, &ink, labels, plot, options, height);
        }
        if let Some(border) = options.border.and_then(|border| brush(draw, border)) {
            draw_border(draw, &border, pixel, options.corner_radius);
        }
    };

    match options.corner_radius.filter(|radius| *radius > 0.0) {
        Some(radius) => clipped(draw, width, height, radius, content),
        None => content(),
    }
}

fn brush(draw: &DrawingSession<'_>, color: ColorF) -> Option<Brush> {
    draw.create_solid_brush(color)
        .inspect_err(|error| tracing::warn!(%error, "scatter: no brush"))
        .ok()
}

fn draw_grid(
    draw: &DrawingSession<'_>,
    grid: &Brush,
    plot: &Plot,
    options: &ScatterOptions,
    pixel: Pixel,
) {
    let across = |y: f32| {
        let y = pixel.snap(y, pixel.height);
        draw.draw_line(
            Vector2 { x: plot.left, y },
            Vector2 { x: plot.right, y },
            grid,
            pixel.hair(),
        );
    };

    for &(value, _) in &options.y_lines {
        across(plot.y(Level::Value(value)));
    }
    for band in 0..plot.above {
        across(plot.scale_top - band as f32 * plot.band_height);
    }
    for band in 0..plot.below {
        across(plot.scale_bottom + band as f32 * plot.band_height);
    }
    for &(at, _) in &options.x_ticks {
        let x = plot.x(at);
        if x < plot.left || x > plot.right {
            continue;
        }
        let x = pixel.snap(x, pixel.width);
        draw.draw_line(
            Vector2 { x, y: plot.bottom },
            Vector2 {
                x,
                y: plot.bottom + TICK_LONG,
            },
            grid,
            pixel.hair(),
        );
    }
}

fn draw_area(draw: &DrawingSession<'_>, plot: &Plot, area: Area, accent: ColorF, pixel: Pixel) {
    let left = plot.x(area.x.0).clamp(plot.left, plot.right);
    let right = plot.x(area.x.1).clamp(plot.left, plot.right);
    let top = plot.edge(area.y.1, true);
    let bottom = plot.edge(area.y.0, false);
    if right <= left || bottom <= top {
        return;
    }

    let rect = Rect::new(left, top, right, bottom);
    if let Some(fill) = brush(draw, ColorF { a: 0.16, ..accent }) {
        draw.fill_rect(&rect, &fill);
    }
    if let Some(edge) = brush(draw, ColorF { a: 0.7, ..accent }) {
        draw.draw_rect(&rect, &edge, pixel.hair());
    }
}

fn draw_points<K: Key>(
    draw: &DrawingSession<'_>,
    plot: &Plot,
    series: &[ScatterSeries<K>],
    hovered: Option<&Hit<K>>,
    ink: ColorF,
) {
    let mut ring = None;
    for (index, line) in series.iter().enumerate() {
        let Some(paint) = brush(draw, line.color) else {
            continue;
        };
        let half = line.size / 2.0;
        for point in &line.points {
            let x = plot.x(point.at);
            if x < plot.left - half || x > plot.right + half {
                continue;
            }
            let y = plot.y(point.value);
            let center = Vector2 { x, y };
            match line.marker {
                Marker::Dot => draw.fill_ellipse(&Ellipse::new(center, half, half), &paint),
                Marker::Ring => draw.draw_ellipse(
                    &Ellipse::new(center, half - RING_WIDTH / 2.0, half - RING_WIDTH / 2.0),
                    &paint,
                    RING_WIDTH,
                ),
                Marker::Tick => draw.draw_line(
                    Vector2 { x, y: y - half },
                    Vector2 { x, y: y + half },
                    &paint,
                    RING_WIDTH,
                ),
            }
            if hovered.is_some_and(|hit| hit.series == index && hit.key == point.key) {
                ring = Some((center, half + HOVER_GAP));
            }
        }
    }

    if let (Some((center, radius)), Some(ink)) = (ring, brush(draw, ink)) {
        draw.draw_ellipse(&Ellipse::new(center, radius, radius), &ink, RING_WIDTH);
    }
}

fn draw_labels(
    draw: &DrawingSession<'_>,
    ink: &Brush,
    labels: &Labels,
    plot: &Plot,
    options: &ScatterOptions,
    height: f32,
) {
    let left = |text: &str, y: f32, within: (f32, f32)| {
        let half = LABEL_TALL / 2.0;
        let middle = y.clamp(within.0 + half, (within.1 - half).max(within.0 + half));
        let rect = Rect::new(0.0, middle - half, LABELS_WIDE - LABEL_GAP, middle + half);
        draw.draw_text(text, &labels.left, &rect, ink);
    };

    let scale = (plot.scale_top, plot.scale_bottom);
    for (value, text) in &options.y_lines {
        left(text, plot.y(Level::Value(*value)), scale);
    }
    for (band, name) in options.above.iter().enumerate() {
        let level = Level::Above(band);
        left(name, plot.y(level), (plot.edge(level, true), plot.edge(level, false)));
    }
    for (band, name) in options.below.iter().enumerate() {
        let level = Level::Below(band);
        left(name, plot.y(level), (plot.edge(level, true), plot.edge(level, false)));
    }

    for (at, text) in &options.x_ticks {
        let x = plot.x(*at);
        if x < plot.left || x > plot.right {
            continue;
        }
        let middle = x.clamp(TICK_LABEL_WIDE / 2.0, (plot.right - TICK_LABEL_WIDE / 2.0).max(0.0));
        let rect = Rect::new(
            middle - TICK_LABEL_WIDE / 2.0,
            plot.bottom + TICK_LONG,
            middle + TICK_LABEL_WIDE / 2.0,
            height,
        );
        draw.draw_text(text, &labels.under, &rect, ink);
    }
}
