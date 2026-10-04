//! A scatter chart: points in time against a value read on a linear or a
//! logarithmic scale, with named bands past its ends; a point under the
//! pointer, a rectangle dragged out, a click.

mod gesture;
mod model;
mod paint;
mod plot;
mod ui;

pub use model::{
    Area, Hit, Key, Level, Marker, Scale, ScatterEvent, ScatterOptions, ScatterPoint, ScatterSeries,
};
pub use ui::Scatter;
