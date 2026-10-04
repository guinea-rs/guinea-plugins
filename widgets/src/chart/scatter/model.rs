//! What a scatter chart is made of: points in time against a value, the
//! scale they are read on, and what the chart hands back.

use windows_canvas::ColorF;
use windows_reactor::ColorScheme;

use super::super::model::theme_border_color;
use super::super::paint::BACKGROUND_TOP;
use crate::color::hex_alpha;

/// How a series' points are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    Dot,
    Ring,
    /// A short upright stroke.
    Tick,
}

/// The page's key for a point: whatever it knows the point by, handed back
/// when the point is hovered or clicked.
pub trait Key: Clone + PartialEq + std::fmt::Debug + 'static {}

impl<K: Clone + PartialEq + std::fmt::Debug + 'static> Key for K {}

#[derive(Clone, Debug, PartialEq)]
pub struct ScatterSeries<K = u64> {
    pub color: ColorF,
    pub marker: Marker,
    /// Across, in DIPs.
    pub size: f32,
    pub points: Vec<ScatterPoint<K>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScatterPoint<K = u64> {
    /// What the page knows the point by; handed back when it is hovered.
    pub key: K,
    pub at: u64,
    pub value: Level,
}

/// Where on the value axis something sits: a value on the scale, or one of
/// the named bands past either end of it, counted outwards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level {
    Value(f32),
    Above(usize),
    Below(usize),
}

impl Level {
    fn rank(self) -> (i8, f32) {
        match self {
            Level::Below(band) => (-1, -(band as f32)),
            Level::Value(value) => (0, value),
            Level::Above(band) => (1, band as f32),
        }
    }

    /// Whether `self` sits lower than `other`.
    pub fn below(self, other: Level) -> bool {
        self.rank() < other.rank()
    }
}

/// How values are laid along the axis between `from` at the bottom and `to`
/// at the top. A value past either end sits at that end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scale {
    Linear { from: f32, to: f32 },
    /// By powers of ten; both ends above zero.
    Log { from: f32, to: f32 },
}

/// A rectangle of the chart, in its own units: from the earlier time to the
/// later, from the lower level to the higher.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub x: (u64, u64),
    pub y: (Level, Level),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScatterOptions {
    /// Solid background color; `None` for a fully transparent background.
    pub background: Option<ColorF>,
    pub border: Option<ColorF>,
    pub corner_radius: Option<f32>,
    /// The span of time shown, earliest first.
    pub x: (u64, u64),
    /// How many units of `x` pass in a second, for a span that moves on
    /// with the clock from where it was published: 1000 when `x` counts
    /// milliseconds. `None` for a span that stays put.
    pub live: Option<f64>,
    pub y: Scale,
    /// The bands above the scale's top, nearest first, by name.
    pub above: Vec<String>,
    /// The bands below the scale's bottom, nearest first, by name.
    pub below: Vec<String>,
    /// How tall each band is, in DIPs.
    pub band_height: f32,
    /// A line across at each of these values, labelled at the left.
    pub y_lines: Vec<(f32, String)>,
    /// A mark at each of these times, labelled underneath.
    pub x_ticks: Vec<(u64, String)>,
    pub grid: ColorF,
    /// The labels' color.
    pub ink: ColorF,
    /// The brush's and the selection's color.
    pub accent: ColorF,
    /// What is drawn as selected, until the page says otherwise.
    pub selection: Option<Area>,
    /// How near the pointer a point has to be to be hovered, in DIPs.
    pub reach: f32,
}

impl Default for ScatterOptions {
    fn default() -> Self {
        Self {
            background: Some(BACKGROUND_TOP),
            border: Some(theme_border_color(ColorScheme::Dark)),
            corner_radius: None,
            x: (0, 1),
            live: None,
            y: Scale::Linear { from: 0.0, to: 1.0 },
            above: Vec::new(),
            below: Vec::new(),
            band_height: 16.0,
            y_lines: Vec::new(),
            x_ticks: Vec::new(),
            grid: hex_alpha(0xffffff, 20),
            ink: hex_alpha(0xffffff, 140),
            accent: hex_alpha(0x6ea8fe, 255),
            selection: None,
            reach: 8.0,
        }
    }
}

/// A point under the pointer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit<K = u64> {
    pub series: usize,
    pub key: K,
    /// Where it is drawn, in DIPs from the chart's top left corner: for
    /// placing what the page shows beside it.
    pub x: f32,
    pub y: f32,
}

/// What the pointer did on a scatter chart.
#[derive(Clone, Debug, PartialEq)]
pub enum ScatterEvent<K = u64> {
    /// The point under the pointer changed; `None` when it left every point.
    Hovered(Option<Hit<K>>),
    /// A rectangle was dragged out and let go.
    Brushed(Area),
    /// A press and release that hardly moved, and the point under it if any.
    Clicked(Option<Hit<K>>),
}
