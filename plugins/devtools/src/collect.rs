use guinea::actor::event_bus::HeardBy as Heard;
use guinea::actor::shape;
use guinea::actor::snapshot::ActorSnapshot;
use guinea::observability::trace::{Cause, Point, Trace};
use guinea::observability::{Event, Observed, ScopeId};
use guinea_devtools_protocol::{
    Actor, BusKind, BusListener, BusSubscription, Channel, Declared, End, Flow, Handled, HeardBy,
    Installed, Listener, Node, Panel, ReducerState, Root, Segment, Snapshot, Span, Timer,
    TraceBatch, TracePoint,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
const PENDING: usize = 16_384;

#[derive(Default)]
pub struct Traces {
    spans: Vec<Span>,
    pending: HashMap<u64, usize>,
    ends: Vec<End>,
    dropped: u64,
}

impl Traces {
    /// Counts `records` that were made and will never reach devtools.
    pub fn lose(&mut self, records: u64) {
        self.dropped += records;
    }

    /// Whether there is nothing to send.
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty() && self.ends.is_empty() && self.dropped == 0
    }

    pub fn push_on(&mut self, thread: u32, record: &Trace) {
        match record {
            Trace::Begin(record) | Trace::Mark(record) => {
                if self.spans.len() >= PENDING {
                    self.dropped += 1;
                    return;
                }
                self.pending.insert(record.id.get(), self.spans.len());
                self.spans.push(Span {
                    id: record.id.get(),
                    parent: record.parent.map(Cause::get),
                    at: record.at.as_micros() as u64,
                    took: None,
                    point: point(&record.point),
                    thread,
                });
            }
            Trace::End { id, took } => {
                let took = took.as_micros() as u64;
                match self.pending.get(&id.get()) {
                    Some(&at) => self.spans[at].took = Some(took),
                    None if self.ends.len() < PENDING => {
                        self.ends.push(End { id: id.get(), took });
                    }
                    None => {}
                }
            }
        }
    }

    pub fn take(&mut self) -> TraceBatch {
        self.pending.clear();
        TraceBatch {
            spans: std::mem::take(&mut self.spans),
            ends: std::mem::take(&mut self.ends),
            dropped: std::mem::take(&mut self.dropped),
        }
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
            to,
            output,
            source,
            file,
            line,
        } => TracePoint::Source {
            actor: to.to_string(),
            actor_id: 0,
            output: output.to_string(),
            source: Some(*source),
            declared: Some(written(file, *line)),
        },
        Point::Arrived {
            to, output, source, ..
        } => TracePoint::Arrived {
            actor: to.to_string(),
            actor_id: 0,
            output: output.to_string(),
            source: *source,
        },
        Point::Pull { to, output, source } => TracePoint::Pull {
            actor: to.to_string(),
            actor_id: 0,
            output: output.to_string(),
            source: *source,
        },
        Point::Closed {
            to,
            output,
            took_us,
            gone,
            source,
        } => TracePoint::Closed {
            actor: to.to_string(),
            actor_id: 0,
            output: output.to_string(),
            took_us: *took_us,
            gone: *gone,
            source: Some(*source),
        },
        Point::Publish { event, subscribers } => TracePoint::Publish {
            event: event.to_string(),
            bus: BusKind::Global,
            subscribers: *subscribers,
        },
        Point::Deliver { event } => TracePoint::Deliver {
            event: event.to_string(),
            bus: BusKind::Global,
        },
        Point::Push { reducer } => TracePoint::Push {
            reducer: reducer.to_string(),
        },
        Point::Navigate { root, to } => TracePoint::Navigate {
            root: root.clone(),
            to: to.clone(),
        },
        Point::Plugin { kind, fields } => TracePoint::Plugin {
            name: kind.to_string(),
            fields: fields
                .iter()
                .map(|(key, value)| (key.to_string(), value.clone()))
                .collect(),
        },
        Point::Superseded {
            actor,
            actor_id,
            output,
            took_us,
        } => TracePoint::Superseded {
            actor: actor.to_string(),
            actor_id: *actor_id,
            output: output.to_string(),
            took_us: *took_us,
        },
        Point::Skipped {
            actor,
            actor_id,
            output,
        } => TracePoint::Skipped {
            actor: actor.to_string(),
            actor_id: *actor_id,
            output: output.to_string(),
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
            level,
            ..
        } => TracePoint::Span {
            name: name.to_string(),
            target: target.to_string(),
            fields: fields.clone(),
            declared: file.zip(*line).map(|(file, line)| written(file, line)),
            level: level.to_string(),
        },
    }
}
pub fn actor(
    snapshot: &guinea::actor::snapshot::ActorSnapshot,
    root: Option<u64>,
    segments: &[ScopeId],
) -> Actor {
    note_crate(snapshot);
    let shape = snapshot.shape;
    let owner = snapshot.owner;
    Actor {
        id: snapshot.id as u64,
        type_name: snapshot.type_name.to_string(),
        state: snapshot.state.preview.clone(),
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
            shape::Channel::Task => Channel::Bg,
        },
        target: (edge.target)().to_string(),
        looping: edge.looping,
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
fn note_crate(snapshot: &ActorSnapshot) {
    if let Some(declared) = snapshot.shape.declared {
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

/// A feature's type without its path: `ProcessesFeature`.
fn feature_name(name: &str) -> String {
    let generic = name.find('<').unwrap_or(name.len());
    let start = name[..generic].rfind("::").map_or(0, |at| at + 2);
    name[start..].to_string()
}

fn panel(panel: guinea::observability::panels::Panel) -> Panel {
    Panel {
        id: panel.id.to_string(),
        title: panel.title.to_string(),
        nodes: panel.nodes.into_iter().map(node).collect(),
    }
}

fn node(node: guinea::observability::panels::PanelNode) -> Node {
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

#[derive(Default)]
pub struct Tree {
    scopes: BTreeMap<u64, ScopeEntry>,
    roots: BTreeMap<u64, ScopeId>,
    labels: HashMap<u64, String>,
    actors: BTreeMap<usize, Option<ScopeId>>,
    sources: BTreeMap<u64, guinea::sources::SourceInfo>,
    states_moved: BTreeMap<u64, Vec<&'static str>>,
    subscriptions: BTreeMap<u64, (&'static str, Heard, bool)>,
    pub plugins: Vec<String>,
    pub backend: String,
}

struct ScopeEntry {
    id: ScopeId,
    parent: Option<ScopeId>,
    segment: Segment,
    navigation: Option<guinea::observability::Navigation>,
    awake: bool,
}

impl Tree {
    pub fn apply(&mut self, event: Event) {
        match event {
            Event::ScopeOpened { scope, parent } => {
                self.scopes.insert(
                    scope.get(),
                    ScopeEntry {
                        id: scope,
                        parent,
                        segment: Segment::default(),
                        navigation: None,
                        awake: true,
                    },
                );
            }
            Event::ScopeClosed { scope } => {
                self.scopes.remove(&scope.get());
            }
            Event::Slept { scope } | Event::Woke { scope } => {
                if let Some(entry) = self.scopes.get_mut(&scope.get()) {
                    entry.awake = matches!(event, Event::Woke { .. });
                }
            }
            Event::Installed {
                scope,
                name,
                declared,
            } => {
                if let Some(entry) = self.scopes.get_mut(&scope.get()) {
                    entry.segment.features.push(Installed {
                        name: feature_name(name),
                        declared: declared.map(place),
                    });
                }
            }
            Event::Withdrawn { scope, name } => {
                if let Some(entry) = self.scopes.get_mut(&scope.get()) {
                    entry
                        .segment
                        .features
                        .retain(|feature| feature.name != feature_name(name));
                }
            }
            Event::Mounted {
                scope,
                name,
                written,
            } => {
                if let Some(entry) = self.scopes.get_mut(&scope.get()) {
                    entry.segment.name = name.to_string();
                    entry.segment.written = written.map(place);
                }
            }
            Event::Listened { scope, listener } => {
                if let Some(entry) = self.scopes.get_mut(&scope.get()) {
                    entry.segment.listeners.push(Listener {
                        event: listener.event.into(),
                        actor: listener.actor.map(str::to_string),
                        bus: BusKind::Global,
                        feature: listener.feature.map(feature_name),
                    });
                }
            }
            Event::StateMoved { scope, reducer } => {
                let moved = self.states_moved.entry(scope.get()).or_default();
                if !moved.contains(&reducer) {
                    moved.push(reducer);
                }
            }
            Event::ActorAdded { id, owner, .. } => {
                self.actors.insert(id, owner.scope);
            }
            Event::ActorRemoved { id } => {
                self.actors.remove(&id);
            }
            Event::SourceOpened(info) => {
                self.sources.insert(info.id, info);
            }
            Event::SourceNamed { id, name } => {
                if let Some(source) = self.sources.get_mut(&id) {
                    source.name = Some(name);
                }
            }
            Event::SourceClosed { id } => {
                self.sources.remove(&id);
            }
            Event::Subscribed {
                seq,
                event,
                by,
                answers,
            } => {
                self.subscriptions.insert(seq, (event, by, answers));
            }
            Event::Unsubscribed { seq } => {
                self.subscriptions.remove(&seq);
            }
            Event::Plugin { name } => {
                self.plugins.push(name.to_string());
            }
            Event::Backend { name } => {
                self.backend = name.to_string();
            }
            Event::RootOpened { root, scope } => {
                self.roots.insert(root, scope);
            }
            Event::RootClosed { root } => {
                self.roots.remove(&root);
                self.labels.remove(&root);
            }
            Event::Labelled { root, label } => {
                self.labels.insert(root, label);
            }
            Event::Navigated { scope, navigation } => {
                if let Some(entry) = self.scopes.get_mut(&scope.get()) {
                    entry.navigation = Some(navigation);
                }
            }
            _ => {}
        }
    }

    fn under(&self, scope: ScopeId, ancestor: ScopeId) -> bool {
        let mut cursor = Some(scope);
        while let Some(id) = cursor {
            if id == ancestor {
                return true;
            }
            cursor = self.scopes.get(&id.get()).and_then(|entry| entry.parent);
        }
        false
    }

    fn location(&self, scope: ScopeId) -> (Option<u64>, Vec<ScopeId>) {
        for (&root, &ancestor) in &self.roots {
            if self.under(scope, ancestor) {
                let scopes = self
                    .scopes
                    .values()
                    .filter(|entry| self.under(entry.id, ancestor))
                    .map(|entry| entry.id)
                    .collect();
                return (Some(root), scopes);
            }
        }
        (None, Vec::new())
    }

    fn panels(&self, observed: Observed) -> Vec<Panel> {
        fn branch(tree: &Tree, entry: &ScopeEntry) -> Node {
            let mut children: Vec<Node> = entry
                .segment
                .reducers
                .iter()
                .map(|state| Node {
                    label: state.type_name.clone(),
                    kind: "state".into(),
                    properties: vec![("value".into(), state.state.clone().unwrap_or_default())],
                    children: Vec::new(),
                })
                .collect();
            children.extend(entry.segment.features.iter().map(|feature| Node {
                label: feature.name.clone(),
                kind: "feature".into(),
                ..Node::default()
            }));
            children.extend(
                tree.scopes
                    .values()
                    .filter(|child| child.parent == Some(entry.id))
                    .map(|child| branch(tree, child)),
            );
            let mut properties = vec![
                ("scope".into(), entry.id.get().to_string()),
                ("awake".into(), entry.awake.to_string()),
            ];
            if let Some(nav) = &entry.navigation {
                properties.extend([
                    ("routes".into(), nav.routes.into()),
                    ("route".into(), nav.route.clone()),
                    ("failure".into(), nav.failure.clone().unwrap_or_default()),
                ]);
            }
            Node {
                label: if entry.segment.name.is_empty() {
                    format!("scope #{}", entry.id.get())
                } else {
                    entry.segment.name.clone()
                },
                kind: "scope".into(),
                properties,
                children,
            }
        }
        let mut panels: Vec<Panel> = observed.panels(None).into_iter().map(panel).collect();
        panels.push(Panel {
            id: "guinea.scopes".into(),
            title: "Scopes".into(),
            nodes: self
                .scopes
                .values()
                .filter(|entry| entry.parent.is_none())
                .map(|entry| branch(self, entry))
                .collect(),
        });
        panels
    }

    pub fn snapshot(&mut self, observed: Observed, started: Instant) -> Snapshot {
        for (scope, reducers) in std::mem::take(&mut self.states_moved) {
            let Some(entry) = self.scopes.get_mut(&scope) else {
                continue;
            };
            for reducer in reducers {
                let Some(state) = observed.state(entry.id, reducer) else {
                    continue;
                };
                let state = ReducerState {
                    type_name: short(state.type_name),
                    state: Some(state.state.preview),
                    feature: state.feature.map(feature_name),
                    declared: state.declared.map(place),
                };
                if let Some(known) = entry
                    .segment
                    .reducers
                    .iter_mut()
                    .find(|known| known.type_name == state.type_name)
                {
                    *known = state;
                } else {
                    entry.segment.reducers.push(state);
                }
            }
        }
        let roots = self
            .roots
            .iter()
            .map(|(&root, &scope)| {
                let entries: Vec<_> = self
                    .scopes
                    .values()
                    .filter(|entry| self.under(entry.id, scope))
                    .collect();
                let navigation = entries.iter().find_map(|entry| entry.navigation.as_ref());
                Root {
                    id: root,
                    label: self.labels.get(&root).cloned(),
                    route: navigation.map(|nav| nav.route.clone()),
                    chain: entries.iter().map(|entry| entry.segment.clone()).collect(),
                    back: navigation.map_or(0, |nav| nav.back),
                    forward: navigation.map_or(0, |nav| nav.forward),
                    pending: navigation.and_then(|nav| nav.pending.clone()),
                    panels: observed.panels(Some(root)).into_iter().map(panel).collect(),
                    bus: Vec::new(),
                }
            })
            .collect();
        let actors = self
            .actors
            .keys()
            .filter_map(|id| {
                let snapshot = observed.actor(*id)?;
                let (root, scopes) = snapshot
                    .owner
                    .scope
                    .map(|scope| self.location(scope))
                    .unwrap_or_default();
                Some(actor(&snapshot, root, &scopes))
            })
            .collect();
        let mut subscriptions: BTreeMap<&str, Vec<BusListener>> = BTreeMap::new();
        for (event, by, answers) in self.subscriptions.values() {
            let asleep = match by {
                Heard::Actor { id, .. } => self
                    .actors
                    .get(id)
                    .and_then(|scope| *scope)
                    .and_then(|scope| self.scopes.get(&scope.get()))
                    .is_some_and(|scope| !scope.awake),
                _ => false,
            };
            let by = match by {
                Heard::Actor { name, id } => HeardBy::Actor {
                    name: name.to_string(),
                    id: *id as u64,
                },
                Heard::Callback(at) => HeardBy::Callback {
                    declared: written(at.file(), at.line()),
                },
                _ => HeardBy::Unknown,
            };
            subscriptions.entry(event).or_default().push(BusListener {
                by,
                answers: *answers,
                asleep,
            });
        }
        let timers = self
            .sources
            .values()
            .map(|source| {
                let (root, scopes) = self.location(source.scope);
                Timer {
                    id: source.id,
                    name: source.name.map(str::to_string),
                    declared: Some(written(source.place.file(), source.place.line())),
                    feature: source.feature.map(feature_name),
                    period_ms: 0,
                    root,
                    segment: scopes.iter().position(|scope| *scope == source.scope),
                }
            })
            .collect();
        Snapshot {
            at: since(started),
            roots,
            actors,
            panels: self.panels(observed),
            global_bus: subscriptions
                .into_iter()
                .map(|(event, listeners)| BusSubscription {
                    event: event.into(),
                    subscribers: listeners.len(),
                    listeners,
                })
                .collect(),
            timers,
        }
    }
}

pub fn since(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea::prelude::*;

    #[derive(Clone, Default, Debug, PartialEq)]
    pub struct Counter(pub u32);

    #[guinea::reducer]
    fn counter(this: &mut Counter, value: u32) {
        this.0 = value;
    }

    guinea::plugin! {
        Probe {
            exports { Counter }
            provides { Observed }
        }
    }

    #[guinea::builds]
    fn probe(app: &mut PluginBuilder) -> anyhow::Result<Probe> {
        Ok(Probe(app.state::<Counter>().plain(), app.observed()))
    }

    fn absorb(tree: &mut Tree, watching: &guinea::observability::Watching) {
        for event in watching.take().events {
            tree.apply(event);
        }
    }

    #[test]
    fn reconnect_replays_the_tree_and_application_state_and_teardown_removes_it() {
        let mut app = guinea::app::TestApp::new();
        let Probe(counter, observed) = app.plugin::<Probe>(()).unwrap();
        let watching = observed.watch(|| {}).unwrap();
        let mut tree = Tree::default();
        absorb(&mut tree, &watching);
        counter.push(7);
        absorb(&mut tree, &watching);
        let started = Instant::now();
        let snapshot = tree.snapshot(observed, started);
        let scopes = snapshot
            .panels
            .iter()
            .find(|panel| panel.id == "guinea.scopes")
            .unwrap();
        assert!(
            scopes.nodes[0]
                .children
                .iter()
                .any(|node| node.kind == "state"
                    && node.properties.iter().any(|(_, value)| value.contains('7')))
        );
        drop(watching);
        let watching = observed.watch(|| {}).unwrap();
        let mut reconnected = Tree::default();
        absorb(&mut reconnected, &watching);
        assert_eq!(
            reconnected.snapshot(observed, started).panels,
            snapshot.panels
        );
        drop(app);
        absorb(&mut reconnected, &watching);
        assert!(reconnected.scopes.is_empty());
    }

    #[test]
    fn retained_trace_records_stay_bounded_and_keep_completed_durations() {
        let mut app = guinea::app::TestApp::new();
        let Probe(_, observed) = app.plugin::<Probe>(()).unwrap();
        let watching = observed.watch(|| {}).unwrap();
        watching.take();
        let mut traces = Traces::default();
        for index in 0..PENDING + 5 {
            {
                let _work =
                    guinea::observability::trace::enter(|| Point::Action { message: "Tick" });
            }
            if index.is_multiple_of(512) {
                for (thread, record) in watching.take().trace {
                    traces.push_on(thread, &record);
                }
            }
        }
        for (thread, record) in watching.take().trace {
            traces.push_on(thread, &record);
        }
        let batch = traces.take();
        assert_eq!(batch.spans.len(), PENDING);
        assert_eq!(batch.dropped, 5);
        assert!(batch.spans.iter().all(|span| span.took.is_some()));
        assert!(traces.take().spans.is_empty());
    }

    #[test]
    fn trace_conversion_keeps_source_identity_and_lane_outcomes() {
        assert!(matches!(
            point(&Point::Pull {
                source: 9,
                to: "Agent",
                output: "Record"
            }),
            TracePoint::Pull { source: 9, .. }
        ));
        assert!(matches!(
            point(&Point::Superseded {
                actor: "Agent",
                actor_id: 3,
                output: "Reply",
                took_us: 4
            }),
            TracePoint::Superseded {
                actor_id: 3,
                took_us: 4,
                ..
            }
        ));
        assert!(matches!(
            point(&Point::Skipped {
                actor: "Agent",
                actor_id: 3,
                output: "Reply"
            }),
            TracePoint::Skipped { actor_id: 3, .. }
        ));
    }
}
