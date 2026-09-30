//! Widgets for guinea applications: tables, charts, drag-resize handles.
//!
//! Not a `Plugin` - nothing here installs into an application, it is a set of
//! components. It lives beside the plugins because it varies on the same axis
//! they do: the backend is a feature, and the parts that are pure layout or
//! geometry stay available without one.

pub mod chart;
pub mod table;

#[cfg(all(feature = "winui", windows))]
pub mod color;
#[cfg(all(feature = "winui", windows))]
pub mod resize;
