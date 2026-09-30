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

#[derive(Clone, PartialEq)]
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
    /// Rounds the chart's corners to this radius in DIPs, and clips all it
    /// draws to them: a WinUI border does not clip its child to its own
    /// corners, so a chart in a rounded tile has to round itself.
    pub corner_radius: Option<f32>,
    /// Keeps the chart moving between publishes, with its right edge on the
    /// clock rather than on the newest sample. Needs `x_window`.
    pub live: Option<Live>,
}

/// A chart of data that arrives as it happens.
///
/// Between publishes the chart moves on by itself, one device pixel at a
/// time: it redraws when the picture has moved a whole pixel and not
/// otherwise, so a minute across a few hundred pixels costs a few redraws a
/// second, and a chart that is not on screen none.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Live {
    /// How many units of `t` pass in a second: 1000 when `t` counts
    /// milliseconds.
    pub per_second: f64,
    /// How far the right edge trails the clock, in units of `t`. About the
    /// time between two samples, so that each new one comes in from past the
    /// edge instead of appearing at it.
    pub lag: u64,
}

impl Default for LineChartOptions {
    fn default() -> Self {
        Self {
            background: Some(super::paint::BACKGROUND_TOP),
            border: Some(theme_border_color(ColorScheme::Dark)),
            grid: None,
            y_range: None,
            x_window: None,
            corner_radius: None,
            live: None,
        }
    }
}

/// Lines at places in the data, so they say something: a second, a limit.
/// Each is one device pixel wide and sits on whole pixels, so it stays sharp
/// at any scale.
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

/// The time of the newest sample of any series.
pub(super) fn newest(series: &[Series]) -> Option<u64> {
    series
        .iter()
        .filter_map(|s| s.points.last())
        .map(|&(t, _)| t)
        .max()
}
