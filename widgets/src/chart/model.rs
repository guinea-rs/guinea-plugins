//! What a chart is made of, before anything is drawn: the data, the knobs, and
//! the readout handed back on hover.

use windows_canvas::ColorF;

use crate::color::hex_alpha;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Step,
    Smooth,
}

#[derive(Clone)]
pub struct Series {
    pub color: ColorF,
    pub interpolation: Interpolation,
    pub fill: Option<ColorF>,
    pub points: Vec<(u64, f32)>,
}

/// What the chart reports under the pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverInfo {
    pub x: u64,
    pub values: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct LineChartOptions {
    /// Solid background color; `None` for a fully transparent background.
    pub background: Option<ColorF>,
    /// Border color; `None` for no border. Use `theme_border_color()` to match
    /// the current dark/light scheme.
    pub border: Option<ColorF>,
    /// Whether to draw the background grid lines.
    pub show_grid: bool,
    /// Fixed Y range; `None` means auto-fit to the data.
    pub y_range: Option<(f32, f32)>,
}

impl Default for LineChartOptions {
    fn default() -> Self {
        Self {
            background: Some(super::paint::BACKGROUND_TOP),
            border: Some(theme_border_color()),
            show_grid: true,
            y_range: None,
        }
    }
}

pub fn theme_border_color() -> ColorF {
    match windows_reactor::current_color_scheme() {
        windows_reactor::ColorScheme::Dark => hex_alpha(0xffffff, 36),
        windows_reactor::ColorScheme::Light => hex_alpha(0x000000, 36),
    }
}

/// Cheap, comparable summary of a chart's data, used as a `use_effect`
/// dependency so the surface only redraws when the data actually changed (grew
/// or shifted), not on every unrelated re-render.
pub(super) type ChartRevision = (usize, u64);

pub(super) fn chart_revision(series: &[Series]) -> ChartRevision {
    let total_points: usize = series.iter().map(|s| s.points.len()).sum();
    let last_t = series
        .iter()
        .filter_map(|s| s.points.last())
        .map(|&(t, _)| t)
        .max()
        .unwrap_or(0);
    (total_points, last_t)
}
