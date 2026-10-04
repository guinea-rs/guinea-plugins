mod ring;

pub use ring::RingSeries;

#[cfg(all(feature = "winui", windows))]
#[allow(non_snake_case, non_camel_case_types, dead_code, clippy::all)]
mod d2d;
#[cfg(all(feature = "winui", windows))]
mod geometry;
#[cfg(all(feature = "winui", windows))]
pub use geometry::{bounds, nearest_point};

#[cfg(all(feature = "winui", windows))]
mod hover;
#[cfg(all(feature = "winui", windows))]
mod live;
#[cfg(all(feature = "winui", windows))]
mod model;
#[cfg(all(feature = "winui", windows))]
mod paint;
#[cfg(all(feature = "winui", windows))]
pub mod scatter;
#[cfg(all(feature = "winui", windows))]
mod ui;

#[cfg(all(feature = "winui", windows))]
pub use model::{
    ChartGrid, HoverInfo, Interpolation, LineChartOptions, Live, Series, theme_border_color,
};
#[cfg(all(feature = "winui", windows))]
pub use ui::Chart;
