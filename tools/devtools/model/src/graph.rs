//! Actors, reducers and buses, grouped the way the application is built.
//!
//! The shape comes from what `actor!` declared and from what features
//! subscribed to; what actually flowed recently comes from the trace, and both
//! add edges: a feature callback pushing into a reducer, or the UI asking an
//! actor for something, is only visible there.

use std::collections::{BTreeMap, HashMap};

use guinea_devtools_protocol::{BusKind, Channel, Snapshot, Span, TracePoint};
use serde::{Deserialize, Serialize};

use crate::trace_log::TraceLog;

/// How far back, in microseconds, a record still counts as activity.
pub const RECENT: u64 = 3_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterKind {
    App,
    Bus(BusKind),
    Window,
    Segment,
    Feature,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cluster {
    pub label: String,
    pub kind: ClusterKind,
    pub parent: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Actor,
    Reducer,
    Listener,
    Event,
    Message,
    Ui,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub key: String,
    pub label: String,
    pub kind: NodeKind,
    pub cluster: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Flow(Channel),
    Publish,
    Deliver,
    Drives,
    Action,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub label: String,
    pub kind: EdgeKind,
    pub looping: bool,
    /// How many times it was taken within [`RECENT`].
    pub recent: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    pub clusters: Vec<Cluster>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

fn short(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

#[derive(Default)]
struct Builder {
    graph: Graph,
    clusters: HashMap<String, usize>,
    nodes: HashMap<String, usize>,
    edges: HashMap<(usize, usize, String), usize>,
    /// Which window a node belongs to, where it belongs to one.
    roots: HashMap<usize, u64>,
}

impl Builder {
    fn cluster(
        &mut self,
        key: String,
        label: &str,
        kind: ClusterKind,
        parent: Option<usize>,
    ) -> usize {
        if let Some(&at) = self.clusters.get(&key) {
            return at;
        }

        self.graph.clusters.push(Cluster {
            label: label.to_string(),
            kind,
            parent,
        });

        let at = self.graph.clusters.len() - 1;
        self.clusters.insert(key, at);
        at
    }

    fn node(&mut self, key: String, label: &str, kind: NodeKind, cluster: usize) -> usize {
        if let Some(&at) = self.nodes.get(&key) {
            return at;
        }

        self.graph.nodes.push(Node {
            key: key.clone(),
            label: label.to_string(),
            kind,
            cluster,
        });

        let at = self.graph.nodes.len() - 1;
        self.nodes.insert(key, at);
        at
    }

    fn edge(
        &mut self,
        from: usize,
        to: usize,
        label: String,
        kind: EdgeKind,
        looping: bool,
    ) -> usize {
        let key = (from, to, label.clone());
        if let Some(&at) = self.edges.get(&key) {
            return at;
        }

        self.graph.edges.push(Edge {
            from,
            to,
            label,
            kind,
            looping,
            recent: 0,
        });

        let at = self.graph.edges.len() - 1;
        self.edges.insert(key, at);
        at
    }
}

/// Where an actor, reducer or listener sits.
fn home(
    builder: &mut Builder,
    windows: &HashMap<u64, usize>,
    snapshot: &Snapshot,
    app: usize,
    root: Option<u64>,
    segment: Option<usize>,
    feature: Option<&str>,
) -> usize {
    let Some(root) = root else { return app };
    let Some(&window) = windows.get(&root) else {
        return app;
    };
    let Some(index) = segment else { return window };

    let name = snapshot
        .roots
        .iter()
        .find(|r| r.id == root)
        .and_then(|r| r.chain.get(index))
        .map_or("segment", |s| s.name.as_str());
    let segment_cluster = builder.cluster(
        format!("segment:{root}:{index}"),
        name,
        ClusterKind::Segment,
        Some(window),
    );

    let label = feature.unwrap_or("(own)");
    builder.cluster(
        format!("feature:{root}:{index}:{label}"),
        label,
        ClusterKind::Feature,
        Some(segment_cluster),
    )
}

pub fn build(snapshot: &Snapshot, trace: &TraceLog) -> Graph {
    let mut b = Builder::default();
    let app = b.cluster("app".into(), "Application", ClusterKind::App, None);
    let global = b.cluster(
        "bus:global".into(),
        "Global bus",
        ClusterKind::Bus(BusKind::Global),
        None,
    );

    let mut windows = HashMap::new();
    let mut window_buses = HashMap::new();

    for (index, root) in snapshot.roots.iter().enumerate() {
        let label = crate::names::window_name(root.label.as_deref(), index);
        let window = b.cluster(
            format!("window:{}", root.id),
            &label,
            ClusterKind::Window,
            None,
        );
        windows.insert(root.id, window);

        for (index, segment) in root.chain.iter().enumerate() {
            b.cluster(
                format!("segment:{}:{index}", root.id),
                &segment.name,
                ClusterKind::Segment,
                Some(window),
            );
        }

        let bus = b.cluster(
            format!("bus:window:{}", root.id),
            "Window bus",
            ClusterKind::Bus(BusKind::Window),
            Some(window),
        );
        window_buses.insert(root.id, bus);
    }

    let mut actors_by_type: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut answering: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    let mut reducers: HashMap<(u64, &str), usize> = HashMap::new();

    for actor in &snapshot.actors {
        let cluster = home(
            &mut b,
            &windows,
            snapshot,
            app,
            actor.root,
            actor.segment,
            actor.feature.as_deref(),
        );
        let node = b.node(
            format!("actor:{}", actor.id),
            short(&actor.type_name),
            NodeKind::Actor,
            cluster,
        );
        if let Some(root) = actor.root {
            b.roots.insert(node, root);
        }

        actors_by_type
            .entry(&actor.type_name)
            .or_default()
            .push(node);
        for handled in &actor.handles {
            answering.entry(&handled.message).or_default().push(node);
        }

        if let (Some(drives), Some(root)) = (&actor.drives, actor.root) {
            let reducer = b.node(
                format!("reducer:{root}:{drives}"),
                short(drives),
                NodeKind::Reducer,
                cluster,
            );

            reducers.insert((root, drives.as_str()), reducer);
            b.roots.insert(reducer, root);
            b.edge(node, reducer, "drives".into(), EdgeKind::Drives, false);
        }
    }

    for root in &snapshot.roots {
        for (index, segment) in root.chain.iter().enumerate() {
            for reducer in &segment.reducers {
                if reducers.contains_key(&(root.id, reducer.type_name.as_str())) {
                    continue;
                }

                let cluster = home(
                    &mut b,
                    &windows,
                    snapshot,
                    app,
                    Some(root.id),
                    Some(index),
                    None,
                );
                let node = b.node(
                    format!("reducer:{}:{}", root.id, reducer.type_name),
                    short(&reducer.type_name),
                    NodeKind::Reducer,
                    cluster,
                );

                b.roots.insert(node, root.id);
                reducers.insert((root.id, reducer.type_name.as_str()), node);
            }
        }
    }

    let event_node = |b: &mut Builder, root: Option<u64>, event: &str, bus: BusKind| -> usize {
        let (cluster, key) = match (bus, root.and_then(|r| window_buses.get(&r))) {
            (BusKind::Window, Some(&cluster)) => {
                (cluster, format!("event:{}:{event}", root.unwrap_or(0)))
            }
            _ => (global, format!("event:global:{event}")),
        };

        b.node(key, short(event), NodeKind::Event, cluster)
    };

    let mut listening: HashMap<(String, BusKind), Vec<usize>> = HashMap::new();

    for root in &snapshot.roots {
        for (index, segment) in root.chain.iter().enumerate() {
            for listener in &segment.listeners {
                let event = event_node(&mut b, Some(root.id), &listener.event, listener.bus);
                let target = match &listener.actor {
                    Some(actor) => actors_by_type
                        .get(actor.as_str())
                        .and_then(|nodes| nodes.first().copied()),
                    None => None,
                };

                let target = match target {
                    Some(actor) => actor,
                    None => {
                        let cluster = home(
                            &mut b,
                            &windows,
                            snapshot,
                            app,
                            Some(root.id),
                            Some(index),
                            listener.feature.as_deref(),
                        );
                        b.node(
                            format!(
                                "listener:{}:{index}:{:?}:{}:{:?}",
                                root.id, listener.feature, listener.event, listener.bus
                            ),
                            &format!("on {}", short(&listener.event)),
                            NodeKind::Listener,
                            cluster,
                        )
                    }
                };

                b.roots.entry(target).or_insert(root.id);
                listening
                    .entry((listener.event.clone(), listener.bus))
                    .or_default()
                    .push(target);
                b.edge(event, target, "deliver".into(), EdgeKind::Deliver, false);
            }
        }
    }

    for actor in &snapshot.actors {
        let Some(&from) = b.nodes.get(&format!("actor:{}", actor.id)) else {
            continue;
        };

        for handled in &actor.handles {
            for flow in handled.edges.iter().flatten() {
                let label = format!("{} {}", verb(flow.channel), short(&flow.target));
                let targets = match flow.channel {
                    Channel::Bg => vec![from],
                    _ => answering
                        .get(flow.target.as_str())
                        .cloned()
                        .unwrap_or_default(),
                };

                let targets = if targets.is_empty() {
                    vec![b.node(
                        format!("message:{}", flow.target),
                        short(&flow.target),
                        NodeKind::Message,
                        app,
                    )]
                } else {
                    targets
                };

                for to in targets {
                    b.edge(
                        from,
                        to,
                        label.clone(),
                        EdgeKind::Flow(flow.channel),
                        flow.looping,
                    );
                }
            }
        }

        for event in &actor.publishes {
            let bus = if listening.contains_key(&(event.clone(), BusKind::Window))
                && !listening.contains_key(&(event.clone(), BusKind::Global))
            {
                BusKind::Window
            } else {
                BusKind::Global
            };

            let to = event_node(&mut b, actor.root, event, bus);
            b.edge(
                from,
                to,
                format!("publish {}", short(event)),
                EdgeKind::Publish,
                false,
            );
        }

        for event in &actor.subscribes {
            let at = event_node(&mut b, actor.root, event, BusKind::Global);
            b.edge(at, from, "deliver".into(), EdgeKind::Deliver, false);
        }
    }

    activity(&mut b, trace, &actors_by_type, &reducers, app, &event_node);

    b.graph
}

/// Finds or makes the node for an event on a bus, given the root it was seen in.
type EventNode<'a> = dyn Fn(&mut Builder, Option<u64>, &str, BusKind) -> usize + 'a;

fn activity(
    b: &mut Builder,
    trace: &TraceLog,
    actors_by_type: &HashMap<&str, Vec<usize>>,
    reducers: &HashMap<(u64, &str), usize>,
    app: usize,
    event_node: &EventNode<'_>,
) {
    let Some(latest) = trace.iter().next_back().map(|span| span.at) else {
        return;
    };
    let since = latest.saturating_sub(RECENT);

    let parent = |span: &Span| span.parent.and_then(|id| trace.get(id));
    let actor = |name: &str| {
        actors_by_type
            .get(name)
            .and_then(|nodes| nodes.first().copied())
    };
    let reducer = |name: &str, root: Option<u64>| match root {
        Some(root) => reducers.get(&(root, name)).copied(),
        None => reducers
            .iter()
            .find(|((_, reducer), _)| *reducer == name)
            .map(|(_, node)| *node),
    };
    let root_of = |node: usize, b: &Builder| b.roots.get(&node).copied();

    let recent: Vec<&Span> = trace
        .iter()
        .rev()
        .take_while(|span| span.at >= since)
        .collect();

    for span in recent {
        let cause = parent(span);
        match (&span.point, cause.map(|c| &c.point)) {
            (
                TracePoint::Send { actor: to, message },
                Some(TracePoint::Handle { actor: from, .. }),
            ) => {
                if let (Some(from), Some(to)) = (actor(from), actor(to)) {
                    let at = find_flow(b, from, to, message).unwrap_or_else(|| {
                        b.edge(
                            from,
                            to,
                            format!("send {}", short(message)),
                            EdgeKind::Flow(Channel::Send),
                            false,
                        )
                    });
                    b.graph.edges[at].recent += 1;
                }
            }
            (TracePoint::Send { actor: to, message }, Some(TracePoint::Action { .. })) => {
                if let Some(to) = actor(to) {
                    let ui = b.node("ui".into(), "UI", NodeKind::Ui, app);
                    let at = b.edge(
                        ui,
                        to,
                        format!("action {}", short(message)),
                        EdgeKind::Action,
                        false,
                    );
                    b.graph.edges[at].recent += 1;
                }
            }
            (
                TracePoint::Publish { event, bus, .. },
                Some(TracePoint::Handle { actor: from, .. }),
            ) => {
                if let Some(from) = actor(from) {
                    let root = root_of(from, b);
                    let to = event_node(b, root, event, *bus);
                    let at = b.edge(
                        from,
                        to,
                        format!("publish {}", short(event)),
                        EdgeKind::Publish,
                        false,
                    );
                    b.graph.edges[at].recent += 1;
                }
            }
            (TracePoint::Push { reducer: name }, Some(TracePoint::Handle { actor: from, .. })) => {
                if let Some(from) = actor(from)
                    && let Some(to) = reducer(name, root_of(from, b))
                {
                    let at = b.edge(from, to, "drives".into(), EdgeKind::Drives, false);
                    b.graph.edges[at].recent += 1;
                }
            }
            (TracePoint::Push { reducer: name }, Some(TracePoint::Deliver { event, bus })) => {
                let listeners: Vec<usize> = b
                    .graph
                    .edges
                    .iter()
                    .filter(|e| e.kind == EdgeKind::Deliver)
                    .filter(|e| {
                        let source = &b.graph.nodes[e.from];
                        source.kind == NodeKind::Event
                            && source.label == short(event)
                            && b.graph.clusters[source.cluster].kind == ClusterKind::Bus(*bus)
                    })
                    .map(|e| e.to)
                    .filter(|&n| b.graph.nodes[n].kind == NodeKind::Listener)
                    .collect();

                for listener in listeners {
                    let Some(to) = reducer(name, root_of(listener, b)) else {
                        continue;
                    };

                    let at = b.edge(listener, to, "pushes".into(), EdgeKind::Drives, false);
                    b.graph.edges[at].recent += 1;
                }
            }
            _ => {}
        }
    }
}

fn find_flow(b: &Builder, from: usize, to: usize, message: &str) -> Option<usize> {
    b.graph.edges.iter().position(|e| {
        e.from == from
            && e.to == to
            && matches!(e.kind, EdgeKind::Flow(_))
            && e.label.ends_with(short(message))
    })
}

fn verb(channel: Channel) -> &'static str {
    match channel {
        Channel::Send => "send",
        Channel::Bg => "bg",
        Channel::Emit => "emit",
        Channel::Ask => "ask",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea_devtools_protocol::{
        Actor, Flow, Handled, Listener, ReducerState, Root, Segment, TraceBatch,
    };

    fn snapshot() -> Snapshot {
        Snapshot {
            roots: vec![Root {
                id: 1,
                label: Some("main".into()),
                chain: vec![
                    Segment {
                        name: "Tabs".into(),
                        features: vec!["TabsFeature".into()],
                        reducers: vec![ReducerState {
                            type_name: "tabs::Tabs".into(),
                            ..ReducerState::default()
                        }],
                        listeners: vec![Listener {
                            event: "events::Killed".into(),
                            actor: None,
                            bus: BusKind::Global,
                            feature: Some("TabsFeature".into()),
                        }],
                        ..Segment::default()
                    },
                    Segment {
                        name: "Processes".into(),
                        features: vec!["ProcessesFeature".into()],
                        ..Segment::default()
                    },
                ],
                ..Root::default()
            }],
            actors: vec![
                Actor {
                    id: 7,
                    type_name: "actor::ProcessActor".into(),
                    root: Some(1),
                    segment: Some(1),
                    feature: Some("ProcessesFeature".into()),
                    drives: Some("contracts::Processes".into()),
                    handles: vec![Handled {
                        message: "contracts::Kill".into(),
                        edges: Some(vec![Flow {
                            channel: Channel::Send,
                            target: "startup::Sweep".into(),
                            looping: false,
                        }]),
                        declared: None,
                    }],
                    publishes: vec!["events::Killed".into()],
                    ..Actor::default()
                },
                Actor {
                    id: 1,
                    type_name: "startup::Housekeeping".into(),
                    handles: vec![Handled {
                        message: "startup::Sweep".into(),
                        edges: None,
                        declared: None,
                    }],
                    ..Actor::default()
                },
            ],
            ..Snapshot::default()
        }
    }

    fn node<'a>(graph: &'a Graph, label: &str) -> (usize, &'a Node) {
        graph
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.label == label)
            .unwrap_or_else(|| panic!("no node {label}: {:?}", graph.nodes))
    }

    fn path(graph: &Graph, mut cluster: usize) -> Vec<String> {
        let mut out = vec![graph.clusters[cluster].label.clone()];
        while let Some(parent) = graph.clusters[cluster].parent {
            out.push(graph.clusters[parent].label.clone());
            cluster = parent;
        }

        out.reverse();
        out
    }

    #[test]
    fn things_sit_in_the_window_segment_and_feature_that_made_them() {
        let graph = build(&snapshot(), &TraceLog::default());

        let (_, actor) = node(&graph, "ProcessActor");
        assert_eq!(
            path(&graph, actor.cluster),
            ["main", "Processes", "ProcessesFeature"]
        );

        let (_, reducer) = node(&graph, "Processes");
        assert_eq!(reducer.kind, NodeKind::Reducer);
        assert_eq!(
            reducer.cluster, actor.cluster,
            "a driven reducer sits with its actor"
        );

        let (_, listener) = node(&graph, "on Killed");
        assert_eq!(
            path(&graph, listener.cluster),
            ["main", "Tabs", "TabsFeature"]
        );

        let (_, event) = node(&graph, "Killed");
        assert_eq!(path(&graph, event.cluster), ["Global bus"]);

        let (_, app_actor) = node(&graph, "Housekeeping");
        assert_eq!(path(&graph, app_actor.cluster), ["Application"]);
    }

    #[test]
    fn a_publication_reaches_the_listener_and_sends_find_their_actor() {
        let graph = build(&snapshot(), &TraceLog::default());

        let (actor, _) = node(&graph, "ProcessActor");
        let (event, _) = node(&graph, "Killed");
        let (listener, _) = node(&graph, "on Killed");
        let (housekeeping, _) = node(&graph, "Housekeeping");

        let has = |from, to, kind| {
            graph
                .edges
                .iter()
                .any(|e| e.from == from && e.to == to && e.kind == kind)
        };
        assert!(has(actor, event, EdgeKind::Publish));
        assert!(has(event, listener, EdgeKind::Deliver));
        assert!(has(actor, housekeeping, EdgeKind::Flow(Channel::Send)));
    }

    #[test]
    fn recent_traffic_lights_up_edges_and_adds_what_only_the_trace_knows() {
        let mut trace = TraceLog::default();
        let span = |id: u64, parent: Option<u64>, point: TracePoint| Span {
            id,
            parent,
            at: 1_000 + id,
            took: None,
            point,
            thread: 0,
        };
        trace.absorb(TraceBatch {
            spans: vec![
                span(
                    1,
                    None,
                    TracePoint::Action {
                        message: "contracts::Kill".into(),
                    },
                ),
                span(
                    2,
                    Some(1),
                    TracePoint::Send {
                        actor: "actor::ProcessActor".into(),
                        message: "contracts::Kill".into(),
                    },
                ),
                span(
                    3,
                    Some(2),
                    TracePoint::Handle {
                        actor: "actor::ProcessActor".into(),
                        message: "contracts::Kill".into(),
                    },
                ),
                span(
                    4,
                    Some(3),
                    TracePoint::Publish {
                        event: "events::Killed".into(),
                        bus: BusKind::Global,
                        subscribers: 1,
                    },
                ),
                span(
                    5,
                    Some(4),
                    TracePoint::Deliver {
                        event: "events::Killed".into(),
                        bus: BusKind::Global,
                    },
                ),
                span(
                    6,
                    Some(5),
                    TracePoint::Push {
                        reducer: "tabs::Tabs".into(),
                    },
                ),
            ],
            ends: Vec::new(),
            dropped: 0,
        });

        let graph = build(&snapshot(), &trace);

        let (ui, _) = node(&graph, "UI");
        let (actor, _) = node(&graph, "ProcessActor");
        let (listener, _) = node(&graph, "on Killed");
        let (tabs, _) = node(&graph, "Tabs");
        let edge = |from, to| graph.edges.iter().find(|e| e.from == from && e.to == to);

        assert_eq!(edge(ui, actor).map(|e| e.recent), Some(1));
        assert_eq!(
            edge(listener, tabs).map(|e| (e.kind, e.recent)),
            Some((EdgeKind::Drives, 1))
        );

        let (event, _) = node(&graph, "Killed");
        assert_eq!(edge(actor, event).map(|e| e.recent), Some(1));
    }
}
