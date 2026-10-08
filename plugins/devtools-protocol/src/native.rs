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
    /// The mark it carries - its `AutomationId` on WinUI - when it has one.
    /// guinea marks the element each page and layout begins at with the
    /// segment's name.
    #[serde(default)]
    pub mark: String,
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
    /// Every layout pass of the frame, costliest first.
    pub passes: Vec<Pass>,
    /// `QueryPerformanceCounter` when it started, as ETW stamped it; with
    /// [`ClockAnchor`](crate::ClockAnchor) it lands on the trace's timeline.
    /// Zero from a tap that does not read it.
    #[serde(default)]
    pub qpc: u64,
    /// The operating system's id of the thread that drew it.
    #[serde(default)]
    pub thread: u32,
}

/// The threads' stacks, sampled while sampling is on: each name once, each
/// distinct stack once, and every sample naming its stack and its thread.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stacks {
    /// Every function a stack names: its symbol, or the module and offset
    /// when there is none.
    pub functions: Vec<String>,
    /// Indices into `functions`, the innermost call first.
    pub stacks: Vec<Vec<u32>>,
    /// Oldest first.
    pub samples: Vec<Sample>,
    /// The path of every module a function is in.
    #[serde(default)]
    pub modules: Vec<String>,
    /// For each of `functions`, an index into `modules`; `None` when the
    /// address was in no module.
    #[serde(default)]
    pub origins: Vec<Option<u32>>,
    /// The name of every thread a sample was taken on, where it has one.
    #[serde(default)]
    pub threads: Vec<SampledThread>,
    /// How many of [`Sample::cycles`] go by in a second; zero from a tap that
    /// counts none.
    #[serde(default)]
    pub cycles_per_second: u64,
}

/// A thread the sampler looked at, by its operating system id.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampledThread {
    pub id: u32,
    pub name: String,
}

/// One look at a thread's stack.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sample {
    /// `QueryPerformanceCounter` when it was taken.
    pub qpc: u64,
    /// An index into [`Stacks::stacks`].
    pub stack: u32,
    /// The operating system's id of the thread; zero from a tap that samples
    /// the UI thread alone.
    #[serde(default)]
    pub thread: u32,
    /// The processor cycles the thread ran since it was last looked at.
    #[serde(default)]
    pub cycles: u64,
}

/// One element's measure or arrange, as long as it took including its
/// children.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Pass {
    pub element: u64,
    /// `measure` or `arrange`.
    pub kind: String,
    pub took_us: u64,
    /// When it started, in microseconds since its frame did.
    #[serde(default)]
    pub at_us: u64,
}

/// An element named the way a test names it: by the mark it carries - its
/// `AutomationId` on WinUI - and which of the elements that carry it, in
/// tree order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub mark: String,
    #[serde(default)]
    pub nth: usize,
    /// Only elements under the one that carries this mark - the row a button
    /// is in.
    #[serde(default)]
    pub within: Option<String>,
}

/// How a click or a keystroke reaches the element.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Input {
    /// The real pointer and keyboard, at the element's place on the screen:
    /// what a user does, hit testing and all. Brings the window to the front
    /// and moves the cursor.
    #[default]
    Pointer,
    /// The element's automation pattern - invoke, toggle, set the value -
    /// with no pointer and no focus change. Only for controls that have one:
    /// a button does, a border with a click handler does not.
    Automation,
}

/// A rectangle on the screen, in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
