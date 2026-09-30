//! Reading the application, on the UI thread.

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use guinea::app::actors::app_actors;
use guinea::app::installed_plugins;
use guinea::devtools::{self, RouterView};
use guinea::timers::{TimerInfo, running};
use guinea_core::actor::event_bus::GlobalEventBus;
use guinea_core::actor::registry::ActorSnapshot;
use guinea_core::actor::shape;
use guinea_core::trace::{self, Bus, Cause, Point, Trace};
use guinea_devtools_protocol::{
    Actor, BusKind, BusSubscription, Channel, Declared, End, Flow, Handled, Installed, Listener,
    Node, Panel, ReducerState, Root, Segment, Snapshot, Span, StoreOp, Timer, TraceBatch,
    TracePoint,
};

/// How many records wait for the next batch before new ones are dropped.
const PENDING: usize = 16_384;

#[derive(Default)]
pub struct Traces {
    spans: Vec<Span>,
    ends: Vec<End>,
    dropped: u64,
}

impl Traces {
    pub fn push(&mut self, record: &Trace) {
        match record {
            Trace::Begin(record) | Trace::Mark(record) => {
                if self.spans.len() >= PENDING {
                    self.dropped += 1;
                    return;
                }
                self.spans.push(Span {
                    id: record.id.get(),
                    parent: record.parent.map(Cause::get),
                    at: record.at.as_micros() as u64,
                    took: None,
                    point: point(&record.point),
                });
            }
            Trace::End { id, took } => {
                let took = took.as_micros() as u64;
                match self.spans.iter_mut().rev().find(|span| span.id == id.get()) {
                    Some(span) => span.took = Some(took),
                    None => self.ends.push(End { id: id.get(), took }),
                }
            }
        }
    }

    pub fn take(&mut self) -> TraceBatch {
        TraceBatch {
            spans: std::mem::take(&mut self.spans),
            ends: std::mem::take(&mut self.ends),
            dropped: std::mem::take(&mut self.dropped),
        }
    }
}

fn bus(bus: Bus) -> BusKind {
    match bus {
        Bus::Global => BusKind::Global,
        Bus::Window => BusKind::Window,
    }
}

fn point(point: &Point) -> TracePoint {
    match point {
        Point::Action { message } => TracePoint::Action {
            message: message.to_string(),
        },
        Point::Send { actor, message } => TracePoint::Send {
            actor: actor.to_string(),
            message: message.to_string(),
        },
        Point::Handle { actor, message } => TracePoint::Handle {
            actor: actor.to_string(),
            message: message.to_string(),
        },
        Point::Spawn {
            actor,
            actor_id,
            output,
        } => TracePoint::Spawn {
            actor: actor.to_string(),
            actor_id: *actor_id,
            output: output.to_string(),
        },
        Point::Settled {
            actor,
            actor_id,
            output,
            took_us,
        } => TracePoint::Settled {
            actor: actor.to_string(),
            actor_id: *actor_id,
            output: output.to_string(),
            took_us: *took_us,
        },
        Point::Cancelled {
            actor,
            actor_id,
            output,
            took_us,
        } => TracePoint::Cancelled {
            actor: actor.to_string(),
            actor_id: *actor_id,
            output: output.to_string(),
            took_us: *took_us,
        },
        Point::Source {
            actor,
            actor_id,
            output,
        } => TracePoint::Source {
            actor: actor.to_string(),
            actor_id: *actor_id,
            output: output.to_string(),
        },
        Point::Arrived {
            actor,
            actor_id,
            output,
            source,
        } => TracePoint::Arrived {
            actor: actor.to_string(),
            actor_id: *actor_id,
            output: output.to_string(),
            source: *source,
        },
        Point::Closed {
            actor,
            actor_id,
            output,
            took_us,
            gone,
        } => TracePoint::Closed {
            actor: actor.to_string(),
            actor_id: *actor_id,
            output: output.to_string(),
            took_us: *took_us,
            gone: *gone,
        },
        Point::Publish {
            event,
            bus: kind,
            subscribers,
        } => TracePoint::Publish {
            event: event.to_string(),
            bus: bus(*kind),
            subscribers: *subscribers,
        },
        Point::Deliver { event, bus: kind } => TracePoint::Deliver {
            event: event.to_string(),
            bus: bus(*kind),
        },
        Point::Push { reducer } => TracePoint::Push {
            reducer: reducer.to_string(),
        },
        Point::Navigate { root, to } => TracePoint::Navigate {
            root: root.clone(),
            to: to.clone(),
        },
        Point::Tick { timer } => TracePoint::Tick {
            timer: Some(*timer),
        },
        Point::Store {
            op,
            path,
            field,
            outside,
        } => TracePoint::Store {
            op: match op {
                trace::StoreOp::Set => StoreOp::Set,
                trace::StoreOp::Delete => StoreOp::Delete,
                trace::StoreOp::DeletePrefix => StoreOp::DeletePrefix,
            },
            path: path.clone(),
            field: field.clone(),
            outside: *outside,
        },
        Point::Render { segment, took_us } => TracePoint::Render {
            segment: segment.to_string(),
            took_us: *took_us,
        },
        Point::Log {
            level,
            target,
            file,
            line,
            text,
            ..
        } => TracePoint::Log {
            level: level.to_string(),
            target: target.to_string(),
            text: text.clone(),
            written: file.zip(*line).map(|(file, line)| written(file, line)),
        },
        Point::Span {
            name,
            target,
            file,
            line,
            fields,
            ..
        } => TracePoint::Span {
            name: name.to_string(),
            target: target.to_string(),
            fields: fields.clone(),
            declared: file.zip(*line).map(|(file, line)| written(file, line)),
        },
        Point::Note(text) => TracePoint::Note { text: text.clone() },
    }
}

fn since(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

pub struct Collected {
    pub report: Snapshot,
    pub backend: Option<&'static str>,
    pub plugins: Vec<&'static str>,
}

fn actor(snapshot: &ActorSnapshot, root: Option<u64>, segments: &[usize]) -> Actor {
    let shape = snapshot.shape;
    let owner = snapshot.owner;
    Actor {
        id: snapshot.id as u64,
        type_name: snapshot.type_name.to_string(),
        state: snapshot.state.clone(),
        root,
        segment: owner
            .scope
            .and_then(|scope| segments.iter().position(|key| *key == scope)),
        feature: owner.feature.map(feature_name),
        drives: owner.drives.map(str::to_string),
        handles: shape
            .handles
            .iter()
            .map(|handles| Handled {
                message: (handles.message)().to_string(),
                edges: handles.edges.map(|edges| edges.iter().map(flow).collect()),
                declared: handles.declared.map(place),
            })
            .collect(),
        publishes: shape
            .publishes
            .iter()
            .map(|name| name().to_string())
            .collect(),
        subscribes: shape
            .subscribes
            .iter()
            .map(|name| name().to_string())
            .collect(),
        declared: shape.declared.map(place),
    }
}

thread_local! {
    /// Where each declaring file was found, or that it was not: finding one
    /// asks the file system along every ancestor of its crate, and a
    /// snapshot four times a second asked again for every actor and handler.
    static FOUND: RefCell<HashMap<(&'static str, &'static str), Option<std::path::PathBuf>>> =
        RefCell::new(HashMap::new());
}

/// Where something was written, with the file found on this machine when the
/// sources are here.
fn place(declared: shape::Declared) -> Declared {
    let path = FOUND.with_borrow_mut(|found| {
        found
            .entry((declared.crate_dir, declared.file))
            .or_insert_with(|| declared.path())
            .clone()
    });
    Declared {
        found: path.is_some(),
        file: path.map_or_else(
            || declared.file.to_string(),
            |path| path.display().to_string(),
        ),
        line: declared.line,
        column: declared.column,
    }
}

fn flow(edge: &shape::Edge) -> Flow {
    Flow {
        channel: match edge.channel {
            shape::Channel::Send => Channel::Send,
            shape::Channel::Bg => Channel::Bg,
            shape::Channel::Emit => Channel::Emit,
            shape::Channel::Ask => Channel::Ask,
        },
        target: (edge.target)().to_string(),
        looping: edge.looping,
    }
}

pub fn snapshot(started: Instant) -> Collected {
    let routers = devtools::routers();
    let backend = routers.first().map(|router| router.backend);

    let app = app_actors();
    let mut crate_dirs: Vec<&'static str> = Vec::new();
    let mut scopes: Vec<(usize, u64, usize)> = Vec::new();

    let mut actors: Vec<Actor> = app
        .iter()
        .inspect(|snapshot| note_crate(&mut crate_dirs, snapshot))
        .map(|snapshot| actor(snapshot, None, &[]))
        .collect();

    let roots = routers
        .into_iter()
        .map(|router| {
            let id = router.root.get();
            let segments: Vec<usize> = router
                .segments
                .iter()
                .map(|segment| segment.scope)
                .collect();
            scopes.extend(
                segments
                    .iter()
                    .enumerate()
                    .map(|(depth, key)| (*key, id, depth)),
            );

            for snapshot in &router.actors {
                note_crate(&mut crate_dirs, snapshot);
                actors.push(actor(snapshot, Some(id), &segments));
            }

            root(router)
        })
        .collect();

    let timers = running()
        .into_iter()
        .map(|info| timer(info, &scopes, &crate_dirs))
        .collect();

    Collected {
        report: Snapshot {
            at: since(started),
            roots,
            actors,
            panels: guinea_core::devtools::app_panels()
                .into_iter()
                .map(panel)
                .collect(),
            global_bus: subscriptions(&GlobalEventBus::bus().subscriptions()),
            timers,
        },
        backend,
        plugins: installed_plugins(),
    }
}

thread_local! {
    /// Every crate an actor was declared in, over every snapshot so far.
    static CRATE_DIRS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    /// A log's file as found on this machine, and how many crates it was
    /// looked for under.
    static WRITTEN: RefCell<HashMap<&'static str, (usize, Option<String>)>> =
        RefCell::new(HashMap::new());
}

/// Remembers the crate an actor was declared in: where a timer's or a log's
/// file, which carries no crate of its own, is looked for.
fn note_crate(crate_dirs: &mut Vec<&'static str>, snapshot: &ActorSnapshot) {
    if let Some(declared) = snapshot.shape.declared
        && !crate_dirs.contains(&declared.crate_dir)
    {
        crate_dirs.push(declared.crate_dir);
        CRATE_DIRS.with_borrow_mut(|known| {
            if !known.contains(&declared.crate_dir) {
                known.push(declared.crate_dir);
            }
        });
    }
}

/// Where a log line was written. The file is the workspace's, relative, so it
/// is looked for above every crate seen, once per file until more crates are.
fn written(file: &'static str, line: u32) -> Declared {
    let found = CRATE_DIRS.with_borrow(|crate_dirs| {
        WRITTEN.with_borrow_mut(|written| {
            let (tried, found) = written.entry(file).or_insert((usize::MAX, None));
            if found.is_none() && *tried != crate_dirs.len() {
                *found = crate_dirs.iter().find_map(|crate_dir| {
                    shape::Declared {
                        file,
                        line,
                        column: 1,
                        crate_dir,
                    }
                    .path()
                    .map(|path| path.display().to_string())
                });
                *tried = crate_dirs.len();
            }
            found.clone()
        })
    });

    Declared {
        found: found.is_some(),
        file: found.unwrap_or_else(|| file.to_string()),
        line,
        column: 1,
    }
}

fn timer(info: TimerInfo, scopes: &[(usize, u64, usize)], crate_dirs: &[&'static str]) -> Timer {
    let owner = info
        .scope
        .and_then(|scope| scopes.iter().find(|(key, _, _)| *key == scope));

    let at = |crate_dir| shape::Declared {
        file: info.place.file(),
        line: info.place.line(),
        column: info.place.column(),
        crate_dir,
    };
    let declared = crate_dirs
        .iter()
        .map(|dir| at(dir))
        .find(|declared| declared.path().is_some())
        .unwrap_or_else(|| at(""));

    Timer {
        id: info.id,
        name: info.name.map(str::to_string),
        declared: Some(place(declared)),
        feature: info.feature.map(feature_name),
        period_ms: info.period.as_millis() as u64,
        root: owner.map(|(_, root, _)| *root),
        segment: owner.map(|(_, _, depth)| *depth),
    }
}

fn root(router: RouterView) -> Root {
    Root {
        id: router.root.get(),
        label: router.label,
        route: router.route,
        chain: router
            .segments
            .into_iter()
            .map(|segment| Segment {
                name: segment.name.to_string(),
                params: String::new(),
                reducers: segment
                    .states
                    .into_iter()
                    .map(|state| ReducerState {
                        type_name: short(state.type_name),
                        state: Some(state.state),
                        feature: state.feature.map(feature_name),
                        declared: state.declared.map(place),
                    })
                    .collect(),
                features: segment
                    .features
                    .into_iter()
                    .map(|feature| Installed {
                        name: feature_name(feature.name),
                        declared: feature.declared.map(place),
                    })
                    .collect(),
                listeners: segment
                    .listeners
                    .into_iter()
                    .map(|listener| Listener {
                        event: listener.event.to_string(),
                        actor: listener.actor.map(str::to_string),
                        bus: bus(listener.bus),
                        feature: listener.feature.map(feature_name),
                    })
                    .collect(),
                declared: segment.declared.map(place),
                written: segment.written.map(place),
            })
            .collect(),
        back: router.back,
        forward: router.forward,
        pending: router.pending,
        panels: router.panels.into_iter().map(panel).collect(),
        bus: subscriptions(&router.bus),
    }
}

fn subscriptions(listed: &[(&'static str, usize)]) -> Vec<BusSubscription> {
    listed
        .iter()
        .map(|(event, subscribers)| BusSubscription {
            event: event.to_string(),
            subscribers: *subscribers,
        })
        .collect()
}

/// A feature's type without its path: `ProcessesFeature`.
fn feature_name(name: &str) -> String {
    let generic = name.find('<').unwrap_or(name.len());
    let start = name[..generic].rfind("::").map_or(0, |at| at + 2);
    name[start..].to_string()
}

fn panel(panel: guinea_core::devtools::Panel) -> Panel {
    Panel {
        id: panel.id.to_string(),
        title: panel.title.to_string(),
        nodes: panel.nodes.into_iter().map(node).collect(),
    }
}

fn node(node: guinea_core::devtools::PanelNode) -> Node {
    Node {
        label: node.label,
        kind: node.kind,
        properties: node.properties,
        children: node.children.into_iter().map(self::node).collect(),
    }
}

/// The last two path segments: `metrics::Metrics` rather than the crate path.
fn short(name: &str) -> String {
    let generic = name.find('<').unwrap_or(name.len());
    let (path, rest) = name.split_at(generic);
    let parts: Vec<&str> = path.rsplitn(3, "::").collect();
    let kept = match parts.as_slice() {
        [last, parent, ..] => format!("{parent}::{last}"),
        [last] => last.to_string(),
        [] => String::new(),
    };
    format!("{kept}{rest}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_type_keeps_its_module_and_loses_the_crate() {
        assert_eq!(
            short("processes_core::metrics::contracts::Metrics"),
            "contracts::Metrics"
        );
        assert_eq!(short("tabs::Tabs"), "tabs::Tabs");
        assert_eq!(short("Plain"), "Plain");
        assert_eq!(short("a::b::List<c::Recent>"), "b::List<c::Recent>");
    }

    fn collected() -> std::rc::Rc<std::cell::RefCell<Traces>> {
        let traces = std::rc::Rc::new(std::cell::RefCell::new(Traces::default()));
        let sink = traces.clone();
        trace::observe(move |record| sink.borrow_mut().push(record));
        traces
    }

    #[test]
    fn an_ended_point_carries_its_duration() {
        let traces = collected();
        {
            let _action = trace::enter(|| Point::Action { message: "Kill" });
            trace::mark(|| Point::Push { reducer: "Theirs" });
        }
        trace::stop_observing();

        let batch = traces.borrow_mut().take();
        let kinds: Vec<&str> = batch.spans.iter().map(|span| span.point.kind()).collect();
        assert_eq!(kinds, ["action", "push"]);
        assert!(
            batch.spans[0].took.is_some(),
            "the action ended inside the batch"
        );
        assert_eq!(batch.spans[1].parent, Some(batch.spans[0].id));
    }

    #[test]
    fn records_past_the_limit_are_counted_rather_than_kept() {
        let traces = collected();
        for _ in 0..PENDING + 5 {
            trace::mark(|| Point::Tick { timer: 1 });
        }
        trace::stop_observing();

        let batch = traces.borrow_mut().take();
        assert_eq!(batch.spans.len(), PENDING);
        assert_eq!(batch.dropped, 5);
        assert_eq!(traces.borrow_mut().take().dropped, 0);
    }
}
