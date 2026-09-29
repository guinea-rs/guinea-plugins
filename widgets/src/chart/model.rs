//! What a chart is made of, before anything is drawn: the data, the knobs, and
//! the readout handed back on hover.

use windows_canvas::ColorF;
use windows_reactor::ColorScheme;

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

#[derive(Clone, Debug, PartialEq)]
pub struct LineChartOptions {
    /// Solid background color; `None` for a fully transparent background.
    pub background: Option<ColorF>,
    /// Border color; `None` for no border. Use `theme_border_color()` to match
    /// the current dark/light scheme.
    pub border: Option<ColorF>,
    /// Lines behind the data; `None` for none.
    pub grid: Option<ChartGrid>,
    /// Fixed Y range; `None` means auto-fit to the data in view.
    pub y_range: Option<(f32, f32)>,
    /// How much time the chart spans, ending at the newest sample; `None`
    /// spans all the data. With it, a history still filling up sits at the
    /// right edge instead of stretching across, and lines at every second stay
    /// a second apart.
    pub x_window: Option<u64>,
}

impl Default for LineChartOptions {
    fn default() -> Self {
        Self {
            background: Some(super::paint::BACKGROUND_TOP),
            border: Some(theme_border_color(ColorScheme::Dark)),
            grid: None,
            y_range: None,
            x_window: None,
        }
    }
}

/// Lines at places in the data, so they say something: a second, a limit.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartGrid {
    /// A vertical line at every multiple of this on the time axis. They move
    /// with the data as it scrolls.
    pub every_t: Option<u64>,
    /// A horizontal line at each of these values, when it is in range.
    pub at_v: Vec<f32>,
    pub color: ColorF,
}

/// The border that reads as a hairline against `scheme`.
///
/// Handed the scheme rather than asking for it: the reactor used to answer
/// `current_color_scheme()` process-wide, and now tells each component through
/// `ViewContext::on_color_scheme`. A page that follows the system theme keeps
/// the last one it was told and passes it here; one that does not gets the
/// default above, which assumes the chart's own dark backdrop.
pub fn theme_border_color(scheme: ColorScheme) -> ColorF {
    match scheme {
        ColorScheme::Dark => hex_alpha(0xffffff, 36),
        ColorScheme::Light => hex_alpha(0x000000, 36),
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
