//! What an application and devtools say to each other, and where.
//!
//! An application connects to devtools, not the other way round: devtools
//! are started once and applications come and go. The connection is an
//! ogurpchik session - a named pipe or a unix socket, opened only by a peer
//! that knows the key devtools wrote when they started - and every message is
//! one [`Report`] as JSON inside a `Peer.send` call.

pub mod key;
pub mod wire;

pub mod devtools_capnp {
    #![allow(clippy::all, missing_docs, unused)]
    include!(concat!(env!("OUT_DIR"), "/devtools_capnp.rs"));
}

use ogurpchik::endpoint::Endpoint;
use serde::{Deserialize, Serialize};

/// Where devtools listen: `\\.\pipe\guinea.devtools`, or `guinea/devtools.sock`
/// under the runtime directory.
pub fn endpoint() -> Endpoint {
    Endpoint::for_service("guinea", "devtools").expect("a valid, constant service name")
}

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
    Action { message: String },
    Send { actor: String, message: String },
    Handle { actor: String, message: String },
    Spawn { actor: String, output: String },
    Publish { event: String, bus: BusKind, subscribers: usize },
    Deliver { event: String, bus: BusKind },
    Push { reducer: String },
    Navigate { root: String, to: String },
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
    /// An ordinary `tracing` event; `level` as tracing spells it, `INFO`.
    Log {
        level: String,
        target: String,
        text: String,
    },
    Note { text: String },
}

impl TracePoint {
    pub fn kind(&self) -> &'static str {
        match self {
            TracePoint::Action { .. } => "action",
            TracePoint::Send { .. } => "send",
            TracePoint::Handle { .. } => "handle",
            TracePoint::Spawn { .. } => "spawn",
            TracePoint::Publish { .. } => "publish",
            TracePoint::Deliver { .. } => "deliver",
            TracePoint::Push { .. } => "push",
            TracePoint::Navigate { .. } => "navigate",
            TracePoint::Tick { .. } => "tick",
            TracePoint::Store { .. } => "store",
            TracePoint::Log { .. } => "log",
            TracePoint::Note { .. } => "note",
        }
    }

    /// One line, for a list.
    pub fn describe(&self) -> String {
        match self {
            TracePoint::Action { message } => format!("action {message}"),
            TracePoint::Send { actor, message } => format!("send {message} → {actor}"),
            TracePoint::Handle { actor, message } => format!("{actor} handles {message}"),
            TracePoint::Spawn { actor, output } => format!("{actor} starts work for {output}"),
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
            TracePoint::Log {
                level,
                target,
                text,
            } => format!("{level} {target}: {text}"),
            TracePoint::Note { text } => text.clone(),
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
    pub features: Vec<String>,
    pub listeners: Vec<Listener>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReducerState {
    pub type_name: String,
    /// `None` when the state cannot be printed.
    pub state: Option<String>,
    /// The feature that claimed it; `None` for the segment's own.
    #[serde(default)]
    pub feature: Option<String>,
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
