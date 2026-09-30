//! What an application and devtools say to each other, and where.
//!
//! An application connects to devtools, not the other way round: devtools
//! are started once and applications come and go. The connection is an
//! ogurpchik session - a named pipe or a unix socket, opened only by a peer
//! that knows the key devtools wrote when they started - and every message is
//! one [`Report`] as JSON inside a `Peer.send` call. Devtools answer the same
//! way with a [`Command`], and only with one the other side listed among its
//! [`Capability`]s.

pub mod key;
pub mod native;
pub mod wire;

/// `schema/devtools.capnp`, compiled by `capnpc` and kept in the tree, so
/// that building the protocol - here or in an application - needs no `capnp`
/// binary. `tests/generated.rs` compiles the schema again and fails when the
/// two differ.
pub mod devtools_capnp {
    #![allow(clippy::all, missing_docs, unused)]
    include!("generated/devtools_capnp.rs");
}

/// The protocol both ends name in the handshake: the id of
/// `schema/devtools.capnp` and a version kept by hand. Something added - a
/// field with a default, a variant nobody older is sent - bumps the minor;
/// anything an older peer would misread bumps the major, and peers of
/// different majors refuse each other.
pub const PROTOCOL: Protocol = Protocol::new(0x96fa_2dd1_07e3_d402, 3, 0, 0);

use ogurpchik::auth::handshake::Protocol;
use ogurpchik::endpoint::Endpoint;
use serde::{Deserialize, Serialize};

/// Where devtools listen: `\\.\pipe\guinea.devtools`, or `guinea/devtools.sock`
/// under the runtime directory - which can fail to be made, when the
/// directory is not this user's.
pub fn endpoint() -> Result<Endpoint, String> {
    Endpoint::for_service("guinea", "devtools").map_err(|report| format!("{report:?}"))
}

/// Devtools' application identifier, which their single-instance lock is
/// named after.
pub const IDENTIFIER: &str = "dev.uniproc.guinea.devtools";

/// One message from an application.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Report {
    /// First on every connection.
    Hello(AppInfo),
    /// The whole observable state, replacing the previous one.
    Snapshot(Snapshot),
    /// What happened since the last batch, oldest first.
    Trace(TraceBatch),
    /// How the native tree changed. The first batch is the whole tree.
    NativeTree { changes: Vec<native::Change> },
    /// Every enumeration the native properties use, once per connection.
    NativeEnums { enums: Vec<native::Enumeration> },
    /// What [`Command::NativeProperties`] asked for.
    NativeProperties {
        element: u64,
        properties: Vec<native::Property>,
    },
    /// What lies under a point, innermost first, and where the innermost is.
    NativePicked {
        chain: Vec<u64>,
        bounds: Option<native::Bounds>,
    },
    /// The frames of the last few seconds, oldest first, as
    /// [`Command::NativePerfCapture`] asked for.
    NativePerf { frames: Vec<native::Frame> },
    /// Where the puffin profiler is listening, and `None` once it stops.
    /// Frames travel over that connection, not this one.
    Profiler { at: Option<String> },
    /// A command could not be carried out.
    Refused { command: String, reason: String },
    /// What a command that carries a `request` came to.
    Answered { request: u64, answer: Answer },
}

/// What a command that carries a `request` came to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Answer {
    /// The action or event went out; `cause` is its point in the trace, and
    /// what it set off is traced under it.
    Acted { cause: u64 },
    /// The element the target names, and where it is on the screen.
    Found {
        element: u64,
        bounds: Option<native::Bounds>,
    },
    /// Done, with nothing to say.
    Done,
    /// Not done, and why.
    Refused { reason: String },
}

/// One message from devtools, to a peer that listed what it needs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Command {
    /// Needs [`Capability::NativeProperties`].
    NativeProperties { element: u64 },
    /// Needs [`Capability::NativeEdit`]. `value` is parsed as `type_name`, the
    /// way XAML would: `Stretch` for an alignment, `4,0,4,0` for a thickness.
    NativeSetProperty {
        element: u64,
        property: u32,
        type_name: String,
        value: String,
    },
    /// Needs [`Capability::NativeHitTest`]. A point on the screen, in
    /// physical pixels.
    NativeHitTest { x: i32, y: i32 },
    /// Needs [`Capability::NativeHighlight`]. `None` takes the highlight away.
    NativeHighlight { element: Option<u64> },
    /// Needs [`Capability::NativePerf`]. What the recording that runs while
    /// devtools are connected holds now.
    NativePerfCapture,
    /// Needs [`Capability::Profiler`]. Switches the puffin profiler on or
    /// off; the application answers with [`Report::Profiler`].
    Profiler { on: bool },
    /// Needs [`Capability::Act`]. The action the application registered as
    /// `action`, decoded from `payload`, to the scope that answers it - under
    /// window `root`, or the newest. Answered with [`Answer::Acted`].
    Act {
        request: u64,
        root: Option<u64>,
        action: String,
        payload: String,
    },
    /// Needs [`Capability::Act`]. The event the application registered as
    /// `event`, published on its global bus. Answered with [`Answer::Acted`].
    Publish {
        request: u64,
        event: String,
        payload: String,
    },
    /// Needs [`Capability::NativeInput`]. The element `target` names.
    /// Answered with [`Answer::Found`].
    NativeFind {
        request: u64,
        target: native::Target,
    },
    /// Needs [`Capability::NativeInput`]. Clicks the element `target` names.
    NativeClick {
        request: u64,
        target: native::Target,
        input: native::Input,
    },
    /// Needs [`Capability::NativeInput`]. Types `text` into the element
    /// `target` names.
    NativeType {
        request: u64,
        target: native::Target,
        text: String,
        input: native::Input,
    },
}

impl Command {
    /// What a peer has to have said it can do for this to reach it.
    pub fn needs(&self) -> Capability {
        match self {
            Command::NativeProperties { .. } => Capability::NativeProperties,
            Command::NativeSetProperty { .. } => Capability::NativeEdit,
            Command::NativeHitTest { .. } => Capability::NativeHitTest,
            Command::NativeHighlight { .. } => Capability::NativeHighlight,
            Command::NativePerfCapture => Capability::NativePerf,
            Command::Profiler { .. } => Capability::Profiler,
            Command::Act { .. } | Command::Publish { .. } => Capability::Act,
            Command::NativeFind { .. }
            | Command::NativeClick { .. }
            | Command::NativeType { .. } => Capability::NativeInput,
        }
    }

    /// The request this command is answered under, when it is answered.
    pub fn request(&self) -> Option<u64> {
        match self {
            Command::Act { request, .. }
            | Command::Publish { request, .. }
            | Command::NativeFind { request, .. }
            | Command::NativeClick { request, .. }
            | Command::NativeType { request, .. } => Some(*request),
            _ => None,
        }
    }
}

/// Something a peer can report or be asked to do.
///
/// Backends differ in what they can show: WinUI has a live native tree with
/// properties, egui has none, a terminal has cells. Devtools offer what the
/// peer listed and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// [`Report::Snapshot`]: roots, actors, reducers, panels.
    Snapshot,
    /// [`Report::Trace`].
    Trace,
    /// [`Report::NativeTree`].
    NativeTree,
    NativeProperties,
    NativeEdit,
    NativeHitTest,
    NativeHighlight,
    /// [`Report::NativePerf`], on request.
    NativePerf,
    /// [`Report::Profiler`]: the puffin profiler, switched on from here and
    /// read over its own connection.
    Profiler,
    /// [`Command::Act`] and [`Command::Publish`]: what the application lists
    /// in [`AppInfo::actions`] and [`AppInfo::events`].
    Act,
    /// [`Command::NativeFind`], [`Command::NativeClick`] and
    /// [`Command::NativeType`]: the backend's own elements, by the mark they
    /// carry.
    NativeInput,
    /// What a newer peer can do and this version of devtools cannot name.
    #[serde(other)]
    Unknown,
}

/// Trace records, and the ends of points that were still open last time.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TraceBatch {
    pub spans: Vec<Span>,
    pub ends: Vec<End>,
    /// Records the application could not send in time and dropped.
    pub dropped: u64,
}

/// One observed point. `parent` is what caused it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub id: u64,
    pub parent: Option<u64>,
    /// Microseconds since the application started tracing.
    pub at: u64,
    /// Microseconds, for points with an extent that already ended.
    pub took: Option<u64>,
    pub point: TracePoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct End {
    pub id: u64,
    pub took: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusKind {
    Global,
    Window,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreOp {
    Set,
    Delete,
    DeletePrefix,
}

impl StoreOp {
    /// What it does, as a verb: `sets`.
    pub fn verb(self) -> &'static str {
        match self {
            StoreOp::Set => "sets",
            StoreOp::Delete => "deletes",
            StoreOp::DeletePrefix => "clears",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TracePoint {
    Action {
        message: String,
    },
    Send {
        actor: String,
        message: String,
    },
    Handle {
        actor: String,
        message: String,
    },
    /// An actor started background work. `actor_id` is its id in
    /// [`Snapshot::actors`], so a task sits where its actor does.
    Spawn {
        actor: String,
        #[serde(default)]
        actor_id: u64,
        output: String,
    },
    /// The work finished, and its result is on its way to the actor.
    Settled {
        actor: String,
        #[serde(default)]
        actor_id: u64,
        output: String,
        took_us: u64,
    },
    /// The work was dropped unfinished: the actor that started it is gone.
    Cancelled {
        actor: String,
        #[serde(default)]
        actor_id: u64,
        output: String,
        took_us: u64,
    },
    /// An actor opened a source whose items come to it as `output`.
    Source {
        actor: String,
        actor_id: u64,
        output: String,
    },
    /// An item came from a source. A root, as a tick is; `source` is the id
    /// of the [`TracePoint::Source`] record it came from.
    Arrived {
        actor: String,
        actor_id: u64,
        output: String,
        source: u64,
    },
    /// A source ended: it ran dry, or `gone` - its actor went away.
    Closed {
        actor: String,
        actor_id: u64,
        output: String,
        took_us: u64,
        gone: bool,
    },
    Publish {
        event: String,
        bus: BusKind,
        subscribers: usize,
    },
    Deliver {
        event: String,
        bus: BusKind,
    },
    Push {
        reducer: String,
    },
    Navigate {
        root: String,
        to: String,
    },
    /// A timer fired; `timer` is its id in [`Snapshot::timers`].
    Tick {
        #[serde(default)]
        timer: Option<u64>,
    },
    Store {
        op: StoreOp,
        path: String,
        field: Option<String>,
        outside: bool,
    },
    /// A page or layout drew itself, and how long that took.
    Render {
        segment: String,
        took_us: u64,
    },
    /// An ordinary `tracing` event; `level` as tracing spells it, `INFO`.
    Log {
        level: String,
        target: String,
        text: String,
        /// The line that logged it, when the event said.
        #[serde(default)]
        written: Option<Declared>,
    },
    Note {
        text: String,
    },
    /// A `tracing` span of the application's own: `took` on the record is
    /// the time spent inside it, and what happened inside is under it.
    Span {
        name: String,
        target: String,
        /// Its fields as they were when it opened, `name=value` apart by
        /// spaces.
        #[serde(default)]
        fields: String,
        /// Where it was declared, when the span said.
        #[serde(default)]
        declared: Option<Declared>,
    },
}

impl TracePoint {
    pub fn kind(&self) -> &'static str {
        match self {
            TracePoint::Action { .. } => "action",
            TracePoint::Send { .. } => "send",
            TracePoint::Handle { .. } => "handle",
            TracePoint::Spawn { .. } => "spawn",
            TracePoint::Settled { .. } => "settled",
            TracePoint::Cancelled { .. } => "cancelled",
            TracePoint::Source { .. } => "source",
            TracePoint::Arrived { .. } => "arrived",
            TracePoint::Closed { .. } => "closed",
            TracePoint::Publish { .. } => "publish",
            TracePoint::Deliver { .. } => "deliver",
            TracePoint::Push { .. } => "push",
            TracePoint::Navigate { .. } => "navigate",
            TracePoint::Tick { .. } => "tick",
            TracePoint::Store { .. } => "store",
            TracePoint::Render { .. } => "render",
            TracePoint::Log { .. } => "log",
            TracePoint::Note { .. } => "note",
            TracePoint::Span { .. } => "span",
        }
    }

    /// One line, for a list.
    pub fn describe(&self) -> String {
        match self {
            TracePoint::Action { message } => format!("action {message}"),
            TracePoint::Send { actor, message } => format!("send {message} → {actor}"),
            TracePoint::Handle { actor, message } => format!("{actor} handles {message}"),
            TracePoint::Spawn { actor, output, .. } => format!("{actor} starts work for {output}"),
            TracePoint::Settled {
                actor,
                output,
                took_us,
                ..
            } => format!(
                "{actor} has its {output} after {:.1} ms",
                *took_us as f64 / 1000.0
            ),
            TracePoint::Cancelled {
                actor,
                output,
                took_us,
                ..
            } => format!(
                "{actor} is gone: {output} cancelled after {:.1} ms",
                *took_us as f64 / 1000.0
            ),
            TracePoint::Source { actor, output, .. } => {
                format!("{actor} opens a source of {output}")
            }
            TracePoint::Arrived { actor, output, .. } => format!("{output} arrives at {actor}"),
            TracePoint::Closed {
                actor,
                output,
                took_us,
                gone: false,
                ..
            } => format!(
                "{actor}'s source of {output} ran dry after {:.1} ms",
                *took_us as f64 / 1000.0
            ),
            TracePoint::Closed {
                actor,
                output,
                took_us,
                gone: true,
                ..
            } => format!(
                "{actor} is gone: its source of {output} closed after {:.1} ms",
                *took_us as f64 / 1000.0
            ),
            TracePoint::Publish {
                event,
                bus,
                subscribers,
            } => format!("publish {event} on {bus:?} bus → {subscribers}"),
            TracePoint::Deliver { event, bus } => format!("deliver {event} from {bus:?} bus"),
            TracePoint::Push { reducer } => format!("push into {reducer}"),
            TracePoint::Navigate { root, to } => format!("{root} → {to}"),
            TracePoint::Tick { .. } => "timer".to_string(),
            TracePoint::Store {
                op, path, outside, ..
            } => format!(
                "{} {} {path}",
                if *outside { "disk" } else { "store" },
                op.verb()
            ),
            TracePoint::Render { segment, took_us } => {
                format!("{segment} drew in {:.1} ms", *took_us as f64 / 1000.0)
            }
            TracePoint::Log {
                level,
                target,
                text,
                ..
            } => format!("{level} {target}: {text}"),
            TracePoint::Note { text } => text.clone(),
            TracePoint::Span { name, fields, .. } if fields.is_empty() => name.clone(),
            TracePoint::Span { name, fields, .. } => format!("{name} {fields}"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub identifier: String,
    pub version: String,
    pub backend: String,
    pub pid: u32,
    pub plugins: Vec<String>,
    /// The wall clock, in milliseconds since the Unix epoch, that
    /// [`Span::at`] counts from. Zero when unknown.
    #[serde(default)]
    pub trace_epoch_ms: u64,
    /// What this peer reports and which [`Command`]s it takes.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    /// The actions a tool may send, by name.
    #[serde(default)]
    pub actions: Vec<String>,
    /// The events a tool may publish, by name.
    #[serde(default)]
    pub events: Vec<String>,
}

impl AppInfo {
    pub fn can(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Milliseconds since the application connected.
    pub at: u64,
    pub roots: Vec<Root>,
    pub actors: Vec<Actor>,
    /// Application-wide panels contributed by plugins.
    pub panels: Vec<Panel>,
    /// What is subscribed to the global bus.
    pub global_bus: Vec<BusSubscription>,
    /// The timers running now.
    #[serde(default)]
    pub timers: Vec<Timer>,
}

/// A running timer.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Timer {
    pub id: u64,
    /// What it was called, when it was.
    pub name: Option<String>,
    /// Where it was set up.
    pub declared: Option<Declared>,
    /// The feature that set it up.
    pub feature: Option<String>,
    pub period_ms: u64,
    /// The window whose segment owns it; `None` for the application.
    pub root: Option<u64>,
    pub segment: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BusSubscription {
    pub event: String,
    pub subscribers: usize,
}

/// A subscription a feature made.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Listener {
    pub event: String,
    /// The actor that listens, or `None` for the feature's own callback.
    pub actor: Option<String>,
    pub bus: BusKind,
    pub feature: Option<String>,
}

/// Something a backend or plugin wants shown that devtools know nothing about:
/// devtools draw the tree, the contributor decides what it means.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    /// Stable, for remembering which panel was open: `winui.components`.
    pub id: String,
    pub title: String,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub label: String,
    /// Shown dimmed after the label: a type name, a key.
    pub kind: String,
    /// Shown when the node is selected.
    pub properties: Vec<(String, String)>,
    pub children: Vec<Node>,
}

/// A window, or whatever the backend calls the thing a router draws into.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub id: u64,
    pub label: Option<String>,
    /// The route as `Debug` prints it.
    pub route: Option<String>,
    pub chain: Vec<Segment>,
    pub back: usize,
    pub forward: usize,
    /// The guard question waiting for an answer, if any.
    pub pending: Option<String>,
    /// Backend panels for this window: `winui.components`, say.
    pub panels: Vec<Panel>,
    /// What is subscribed to this window's bus.
    pub bus: Vec<BusSubscription>,
}

/// One mounted layout or page.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub name: String,
    pub params: String,
    pub reducers: Vec<ReducerState>,
    pub features: Vec<Installed>,
    pub listeners: Vec<Listener>,
    /// Where `routes!` listed this page or layout.
    #[serde(default)]
    pub declared: Option<Declared>,
    /// Where the page or layout itself was written.
    #[serde(default)]
    pub written: Option<Declared>,
}

/// A feature a segment installed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Installed {
    pub name: String,
    /// Where the feature's `#[installs]` function was written.
    #[serde(default)]
    pub declared: Option<Declared>,
}

impl From<&str> for Installed {
    /// A feature known by name alone.
    fn from(name: &str) -> Self {
        Installed {
            name: name.to_string(),
            declared: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReducerState {
    pub type_name: String,
    /// `None` when the state cannot be printed.
    pub state: Option<String>,
    /// The feature that claimed it; `None` for the segment's own.
    #[serde(default)]
    pub feature: Option<String>,
    /// Where it was claimed: the `cx.state::<R>()` that did it.
    #[serde(default)]
    pub declared: Option<Declared>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Actor {
    pub id: u64,
    pub type_name: String,
    pub state: String,
    /// What it belongs to: `None` for the application, a root's id otherwise.
    pub root: Option<u64>,
    /// Which segment of that root's chain owns it.
    pub segment: Option<usize>,
    /// The feature that created it.
    pub feature: Option<String>,
    /// The reducer it drives.
    pub drives: Option<String>,
    /// What `actor!` declared: the messages it answers and where each goes.
    pub handles: Vec<Handled>,
    pub publishes: Vec<String>,
    pub subscribes: Vec<String>,
    /// Where `actor!` was written.
    #[serde(default)]
    pub declared: Option<Declared>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Declared {
    /// Absolute when the application found its sources, as the compiler
    /// wrote it otherwise.
    pub file: String,
    pub line: u32,
    pub column: u32,
    /// Whether `file` is a file on the application's machine.
    pub found: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Handled {
    pub message: String,
    /// `None` when the handler declared nothing about what it sends.
    pub edges: Option<Vec<Flow>>,
    /// Where the handler was written.
    #[serde(default)]
    pub declared: Option<Declared>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flow {
    pub channel: Channel,
    pub target: String,
    pub looping: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Send,
    Bg,
    Emit,
    Ask,
}
