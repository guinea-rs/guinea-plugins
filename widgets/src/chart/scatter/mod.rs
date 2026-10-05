//! A scatter chart: points in time against a value read on a linear or a
//! logarithmic scale, with named bands past its ends; a point under the
//! pointer, a rectangle dragged out, a click.

mod gesture;
mod model;
mod paint;
mod plot;
mod ui;

pub use model::{
    Area, Hit, IntoScatterData, Key, Level, Marker, Scale, ScatterData, ScatterEvent,
    ScatterOptions, ScatterPoint, ScatterSeries, SeriesStyle,
};
pub use ui::Scatter;
