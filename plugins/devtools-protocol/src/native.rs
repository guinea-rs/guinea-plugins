//! A backend's own UI elements, as its native inspector sees them.
//!
//! For WinUI that is the live XAML tree, reached through XAML diagnostics
//! from inside the process. Nothing here is specific to it: a handle is
//! whatever the inspector uses to name an element, and properties are text.

use serde::{Deserialize, Serialize};

/// One element of the native tree.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub handle: u64,
    /// Zero for a root.
    pub parent: u64,
    /// Its place among the parent's children.
    pub index: u32,
    /// The native type: `Microsoft.UI.Xaml.Controls.Border`.
    pub kind: String,
    /// The native name, when it has one.
    pub name: String,
}

/// How the tree changed, in the order it did.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "change", rename_all = "snake_case")]
pub enum Change {
    Added(Element),
    Removed { handle: u64, parent: u64 },
}

/// One property of an element, with where its value came from.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub value_type: String,
    /// The type that declares it: `Microsoft.UI.Xaml.FrameworkElement`.
    pub declaring_type: String,
    /// `local`, `style`, `built_in_style`, `default`, `inherited`…
    pub source: String,
    /// Another source wins over this value.
    pub overridden: bool,
    /// What [`crate::Command::NativeSetProperty`] names the property by.
    pub index: u32,
    /// Computed, like `ActualWidth`: shown, never set.
    #[serde(default)]
    pub read_only: bool,
    /// `value` is the handle of another element or object, `0` for none.
    #[serde(default)]
    pub object: bool,
    /// The value is a binding - `{TemplateBinding}`, `{Binding}` - and what
    /// it evaluates to is elsewhere in the chain.
    #[serde(default)]
    pub binding: bool,
    /// The colour a brush value paints, as the backend writes one: a name
    /// like `White`, or `#AARRGGBB`.
    #[serde(default)]
    pub color: Option<String>,
}

/// The names of an enumeration's values, so that `3` reads as `Stretch`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Enumeration {
    /// As [`Property::value_type`] names it.
    pub name: String,
    pub values: Vec<(i32, String)>,
}

/// One frame of the UI thread, from the backend's own performance events.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    /// When it started, in microseconds since the first frame of the capture.
    pub at_us: u64,
    pub took_us: u64,
    /// Time spent measuring and arranging, nested passes counted once.
    pub measure_us: u64,
    pub arrange_us: u64,
    /// The costliest layout passes of the frame, costliest first.
    pub passes: Vec<Pass>,
}

/// One element's measure or arrange, as long as it took including its
/// children.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Pass {
    pub element: u64,
    /// `measure` or `arrange`.
    pub kind: String,
    pub took_us: u64,
}

/// A rectangle on the screen, in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
