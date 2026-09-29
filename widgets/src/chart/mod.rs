mod ring;

pub use ring::RingSeries;

#[cfg(feature = "winui")]
mod geometry;
#[cfg(feature = "winui")]
pub use geometry::{bounds, nearest_point};

#[cfg(feature = "winui")]
mod hover;
#[cfg(feature = "winui")]
mod live;
#[cfg(feature = "winui")]
mod model;
#[cfg(feature = "winui")]
mod paint;
#[cfg(feature = "winui")]
mod surface;
#[cfg(feature = "winui")]
mod ui;

#[cfg(feature = "winui")]
pub use model::{
    ChartGrid, HoverInfo, Interpolation, LineChartOptions, Live, Series, theme_border_color,
};
#[cfg(feature = "winui")]
pub use ui::Chart;
