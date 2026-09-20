//! The application as one tree: its actors, its windows, and each window's
//! route segments with their state, actors and view; and what is known about
//! any one of them.

use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use guinea_devtools_protocol::{
    Actor, Channel, Declared, Installed, Node, ReducerState, Root, Snapshot,
};
use serde::{Deserialize, Serialize};

use crate::names::{names, type_name, window_name};
use crate::panels::is_view;
use crate::sessions::Session;
use crate::words::{Kind, Tone, Word};

/// Something in the tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Element {
    App,
    Actor(u64),
    Window(u64),
    Segment { root: u64, depth: usize },
    /// A feature, installed in a window's segment or, with no `at`, in the
    /// application.
    Feature { at: Option<(u64, usize)>, name: String },
    State { root: u64, depth: usize, type_name: String },
    /// A node of a window's view tree, by its path from the tree's top.
    View { root: u64, path: Vec<usize> },
}

impl fmt::Display for Element {
    /// `app`, `actor/7`, `window/1`, `segment/1/0`, `feature/1/0/TabsFeature`,
    /// `feature/app/StartupFeature`, `state/1/0/a::Tabs`, `view/1/0.2.1`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Element::App => f.write_str("app"),
            Element::Actor(id) => write!(f, "actor/{id}"),
            Element::Window(root) => write!(f, "window/{root}"),
            Element::Segment { root, depth } => write!(f, "segment/{root}/{depth}"),
            Element::Feature { at: None, name } => write!(f, "feature/app/{name}"),
            Element::Feature {
                at: Some((root, depth)),
                name,
            } => write!(f, "feature/{root}/{depth}/{name}"),
            Element::State {
                root,
                depth,
                type_name,
            } => write!(f, "state/{root}/{depth}/{type_name}"),
            Element::View { root, path } => {
                let path: Vec<String> = path.iter().map(usize::to_string).collect();
                write!(f, "view/{root}/{}", path.join("."))
            }
        }
    }
}

impl FromStr for Element {
    type Err = String;

    fn from_str(id: &str) -> Result<Self, String> {
        let bad = || format!("not an element: {id}");
        let mut parts = id.splitn(4, '/');
        let number = |part: Option<&str>| part.and_then(|part| part.parse::<u64>().ok()).ok_or_else(bad);
        let index = |part: Option<&str>| part.and_then(|part| part.parse::<usize>().ok()).ok_or_else(bad);

        Ok(match parts.next() {
            Some("app") => Element::App,
            Some("actor") => Element::Actor(number(parts.next())?),
            Some("window") => Element::Window(number(parts.next())?),
            Some("segment") => Element::Segment {
                root: number(parts.next())?,
                depth: index(parts.next())?,
            },
            Some("feature") => match parts.next() {
                Some("app") => Element::Feature {
                    at: None,
                    name: parts.next().filter(|name| !name.is_empty()).ok_or_else(bad)?.to_string(),
                },
                root => Element::Feature {
                    at: Some((number(root)?, index(parts.next())?)),
                    name: parts.next().ok_or_else(bad)?.to_string(),
                },
            },
            Some("state") => Element::State {
                root: number(parts.next())?,
                depth: index(parts.next())?,
                type_name: parts.next().ok_or_else(bad)?.to_string(),
            },
            Some("view") => Element::View {
                root: number(parts.next())?,
                path: parts
                    .next()
                    .ok_or_else(bad)?
                    .split('.')
                    .map(|step| step.parse().map_err(|_| bad()))
                    .collect::<Result<_, _>>()?,
            },
            _ => return Err(bad()),
        })
    }
}

impl From<Element> for String {
    fn from(element: Element) -> String {
        element.to_string()
    }
}

impl TryFrom<String> for Element {
    type Error = String;

    fn try_from(id: String) -> Result<Self, String> {
        id.parse()
    }
}

/// One row of the tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub depth: usize,
    pub element: Element,
    /// Whether it has anything under it.
    pub branch: bool,
    /// Whether what is under it follows.
    pub open: bool,
    pub words: Vec<Word>,
}

/// A window's view tree, as its backend contributed it.
pub fn view_of(root: &Root) -> Option<&Node> {
    root.panels
        .iter()
        .find(|panel| is_view(panel))
        .and_then(|panel| panel.nodes.first())
}

/// Whether a view node has a route segment somewhere under it.
fn holds_segment(node: &Node) -> bool {
    node.children
        .iter()
        .any(|child| child.kind == "segment" || holds_segment(child))
}

fn word(text: impl Into<String>, tone: Tone) -> Word {
    Word::new(text, tone)
}

fn kind(kind: Kind) -> Tone {
    Tone::Kind(kind)
}

struct Lines<'a> {
    snapshot: &'a Snapshot,
    closed: &'a HashSet<Element>,
    reveal: Option<&'a Element>,
    lines: Vec<Line>,
}

impl Lines<'_> {
    /// Adds a row; whether what is under it follows.
    fn push(&mut self, depth: usize, element: Element, words: Vec<Word>, branch: bool, forced: bool) -> bool {
        let open = branch && (forced || !self.closed.contains(&element));
        self.lines.push(Line {
            depth,
            element,
            branch,
            open,
            words,
        });

        open
    }

    fn revealing_under(&self, root: u64, depth: usize) -> bool {
        match self.reveal {
            Some(
                Element::State { root: at, depth: d, .. }
                | Element::Feature {
                    at: Some((at, d)),
                    ..
                },
            ) => *at == root && *d >= depth,
            Some(Element::Actor(id)) => self.snapshot.actors.iter().any(|actor| {
                actor.id == *id && actor.root == Some(root) && actor.segment.is_some_and(|d| d >= depth)
            }),
            Some(Element::View { root: at, .. } | Element::Segment { root: at, .. }) => *at == root,
            _ => false,
        }
    }

    fn actor(&mut self, actor: &Actor, depth: usize) {
        self.push(
            depth,
            Element::Actor(actor.id),
            vec![
                word("actor ", Tone::Muted),
                word(type_name(&actor.type_name), kind(Kind::Handle)),
            ],
            false,
            false,
        );
    }

    fn state(&mut self, root: u64, at: usize, state: &ReducerState, depth: usize) {
        self.push(
            depth,
            Element::State {
                root,
                depth: at,
                type_name: state.type_name.clone(),
            },
            vec![
                word("state ", Tone::Muted),
                word(type_name(&state.type_name), kind(Kind::Push)),
            ],
            false,
            false,
        );
    }

    /// The features installed in one place, each with the states and actors
    /// it brought, then whatever no feature claims.
    fn members(
        &mut self,
        at: Option<(u64, usize)>,
        features: &[Installed],
        states: &[ReducerState],
        actors: &[&Actor],
        depth: usize,
    ) {
        let mut named: Vec<&str> = features.iter().map(|feature| feature.name.as_str()).collect();
        let claimed = states
            .iter()
            .filter_map(|state| state.feature.as_deref())
            .chain(actors.iter().filter_map(|actor| actor.feature.as_deref()));
        for feature in claimed {
            if !named.contains(&feature) {
                named.push(feature);
            }
        }

        for feature in named {
            let own_states: Vec<&ReducerState> = states
                .iter()
                .filter(|state| state.feature.as_deref() == Some(feature))
                .collect();
            let own_actors: Vec<&Actor> = actors
                .iter()
                .copied()
                .filter(|actor| actor.feature.as_deref() == Some(feature))
                .collect();

            let forced = match self.reveal {
                Some(Element::Actor(id)) => own_actors.iter().any(|actor| actor.id == *id),
                Some(Element::State { type_name, .. }) => {
                    own_states.iter().any(|state| state.type_name == *type_name)
                }
                _ => false,
            };

            let element = Element::Feature {
                at,
                name: feature.to_string(),
            };
            let words = vec![word("feature ", Tone::Muted), word(feature, kind(Kind::Spawn))];
            let branch = !own_states.is_empty() || !own_actors.is_empty();

            if self.push(depth, element, words, branch, forced) {
                if let Some((root, segment)) = at {
                    for state in own_states {
                        self.state(root, segment, state, depth + 1);
                    }
                }

                for actor in own_actors {
                    self.actor(actor, depth + 1);
                }
            }
        }

        if let Some((root, segment)) = at {
            for state in states.iter().filter(|state| state.feature.is_none()) {
                self.state(root, segment, state, depth);
            }
        }

        for actor in actors.iter().filter(|actor| actor.feature.is_none()) {
            self.actor(actor, depth);
        }
    }

    fn app(&mut self) {
        let snapshot = self.snapshot;
        let actors: Vec<&Actor> = snapshot.actors.iter().filter(|a| a.root.is_none()).collect();

        let forced = match self.reveal {
            Some(Element::Actor(id)) => actors.iter().any(|a| a.id == *id),
            Some(Element::Feature { at: None, .. }) => true,
            _ => false,
        };
        let branch = !actors.is_empty();

        if self.push(0, Element::App, vec![word("application", Tone::Plain)], branch, forced) {
            self.members(None, &[], &[], &actors, 1);
        }
    }

    fn window(&mut self, root: &Root, index: usize) {
        let snapshot = self.snapshot;
        let forced = self.revealing_under(root.id, 0);
        let words = vec![
            word("window ", Tone::Muted),
            word(window_name(root.label.as_deref(), index), Tone::Plain),
        ];
        if !self.push(0, Element::Window(root.id), words, true, forced) {
            return;
        }

        for actor in snapshot
            .actors
            .iter()
            .filter(|a| a.root == Some(root.id) && a.segment.is_none())
        {
            self.actor(actor, 1);
        }

        self.segment(root, 0, view_of(root).map(|view| (view, vec![0])), 1);
    }

    /// Segment `at` of `root`'s chain, with its node in the view tree and that
    /// node's path from the tree's top, when the backend drew one.
    fn segment(&mut self, root: &Root, at: usize, view: Option<(&Node, Vec<usize>)>, depth: usize) {
        let snapshot = self.snapshot;
        let Some(segment) = root.chain.get(at) else {
            return;
        };

        let words = vec![word("segment ", Tone::Muted), word(&segment.name, Tone::Accent)];
        let forced = self.revealing_under(root.id, at);
        let element = Element::Segment { root: root.id, depth: at };
        if !self.push(depth, element, words, true, forced) {
            return;
        }

        let actors: Vec<&Actor> = snapshot
            .actors
            .iter()
            .filter(|a| a.root == Some(root.id) && a.segment == Some(at))
            .collect();
        self.members(
            Some((root.id, at)),
            &segment.features,
            &segment.reducers,
            &actors,
            depth + 1,
        );

        let mut nested = false;
        if let Some((view, mut path)) = view {
            for (index, child) in view.children.iter().enumerate() {
                path.push(index);
                self.view(root, at, child, &mut path, depth + 1, &mut nested);
                path.pop();
            }
        }

        if !nested {
            self.segment(root, at + 1, None, depth + 1);
        }
    }

    fn view(
        &mut self,
        root: &Root,
        at: usize,
        node: &Node,
        path: &mut Vec<usize>,
        depth: usize,
        nested: &mut bool,
    ) {
        if node.kind == "segment" {
            *nested = true;
            self.segment(root, at + 1, Some((node, path.clone())), depth);
            return;
        }

        let words = match node.kind.as_str() {
            "native" => vec![
                word("<", Tone::Muted),
                word(&node.label, kind(Kind::Send)),
                word(">", Tone::Muted),
            ],
            "component" => vec![
                word("<", Tone::Muted),
                word(&node.label, kind(Kind::Deliver)),
                word(" />", Tone::Muted),
            ],
            text if text.starts_with('"') => vec![
                word("<", Tone::Muted),
                word(&node.label, kind(Kind::Send)),
                word(">", Tone::Muted),
                word(text, Tone::Quote),
            ],
            other => vec![word(&node.label, Tone::Plain), word(format!(" {other}"), Tone::Muted)],
        };

        let branch = !node.children.is_empty();
        let forced = holds_segment(node) && self.revealing_under(root.id, at + 1);
        let element = Element::View {
            root: root.id,
            path: path.clone(),
        };

        if self.push(depth, element, words, branch, forced) {
            for (index, child) in node.children.iter().enumerate() {
                path.push(index);
                self.view(root, at, child, path, depth + 1, nested);
                path.pop();
            }
        } else if holds_segment(node) {
            *nested = true;
        }
    }
}

/// The tree as rows, top to bottom. Rows in `closed` hide what is under
/// them, except on the way to `reveal`.
pub fn lines(snapshot: &Snapshot, closed: &HashSet<Element>, reveal: Option<&Element>) -> Vec<Line> {
    let mut lines = Lines {
        snapshot,
        closed,
        reveal,
        lines: Vec::new(),
    };

    lines.app();
    for (index, root) in snapshot.roots.iter().enumerate() {
        lines.window(root, index);
    }

    lines.lines
}

/// The actor a link names.
pub fn actor_named(snapshot: &Snapshot, name: &str) -> Option<Element> {
    snapshot
        .actors
        .iter()
        .find(|actor| names(name, &actor.type_name))
        .map(|actor| Element::Actor(actor.id))
}

/// The state a link names.
pub fn state_named(snapshot: &Snapshot, name: &str) -> Option<Element> {
    snapshot.roots.iter().find_map(|root| {
        root.chain.iter().enumerate().find_map(|(depth, segment)| {
            segment
                .reducers
                .iter()
                .find(|state| names(name, &state.type_name))
                .map(|state| Element::State {
                    root: root.id,
                    depth,
                    type_name: state.type_name.clone(),
                })
        })
    })
}

/// What is known about one element.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Details {
    pub title: String,
    pub kind: String,
    pub rows: Vec<(String, String)>,
    /// A long text shown whole: a state's value.
    pub body: Option<String>,
    /// Where it was written: an actor's `actor!`, a page's `impl Page`.
    pub declared: Option<Declared>,
    /// Where a page or layout was listed in `routes!`.
    #[serde(default)]
    pub routed: Option<Declared>,
    /// What an actor answers, each where its handler was written.
    #[serde(default)]
    pub handlers: Vec<HandlerLine>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandlerLine {
    pub message: String,
    /// What it sends or starts, as declared: `sends Tick, works on Tick`.
    pub flows: String,
    pub declared: Option<Declared>,
}

fn find<'a>(node: &'a Node, path: &[usize]) -> Option<&'a Node> {
    match path.split_first() {
        None => Some(node),
        Some((first, rest)) => find(node.children.get(*first)?, rest),
    }
}

fn root_of(snapshot: &Snapshot, id: u64) -> Option<(usize, &Root)> {
    snapshot.roots.iter().enumerate().find(|(_, root)| root.id == id)
}

fn row(name: &str, value: impl Into<String>) -> (String, String) {
    (name.to_string(), value.into())
}

fn details(title: impl Into<String>, kind: &str, rows: Vec<(String, String)>) -> Details {
    Details {
        title: title.into(),
        kind: kind.to_string(),
        rows,
        body: None,
        declared: None,
        routed: None,
        handlers: Vec::new(),
    }
}

impl Details {
    /// The same details, knowing where what they are about was written.
    fn written(mut self, declared: Option<Declared>) -> Self {
        self.declared = declared;
        self
    }
}

pub fn describe(session: &Session, element: &Element) -> Option<Details> {
    let snapshot = &session.snapshot;

    Some(match element {
        Element::App => {
            let info = &session.info;
            details(
                session.name(),
                "application",
                vec![
                    row("identifier", &info.identifier),
                    row("version", &info.version),
                    row("backend", &info.backend),
                    row("pid", info.pid.to_string()),
                    row("plugins", info.plugins.join(", ")),
                ],
            )
        }
        Element::Window(id) => {
            let (index, root) = root_of(snapshot, *id)?;
            let mut rows = vec![
                row("route", root.route.clone().unwrap_or_default()),
                row("history", format!("{} back · {} forward", root.back, root.forward)),
            ];
            if let Some(question) = &root.pending {
                rows.push(row("waiting on", question));
            }

            details(window_name(root.label.as_deref(), index), "window", rows)
        }
        Element::Segment { root, depth } => {
            let (_, root) = root_of(snapshot, *root)?;
            let segment = root.chain.get(*depth)?;

            let installs: Vec<&str> = segment.features.iter().map(|feature| feature.name.as_str()).collect();
            let mut rows = vec![row("installs", installs.join(", "))];
            for listener in &segment.listeners {
                rows.push(row(
                    "listens",
                    format!(
                        "{} on the {:?} bus{}",
                        type_name(&listener.event),
                        listener.bus,
                        listener
                            .actor
                            .as_deref()
                            .map(|actor| format!(" for {}", type_name(actor)))
                            .unwrap_or_default()
                    ),
                ));
            }
            if !segment.params.is_empty() {
                rows.push(row("params", &segment.params));
            }

            let mine: Vec<u64> = snapshot
                .actors
                .iter()
                .filter(|actor| actor.root == Some(root.id) && actor.segment == Some(*depth))
                .map(|actor| actor.id)
                .collect();
            for task in session.tasks.of_actors(&mine) {
                rows.push(row(
                    "waiting on",
                    format!("{} for {}", type_name(&task.output), type_name(&task.actor)),
                ));
            }

            Details {
                routed: segment.declared.clone(),
                ..details(&segment.name, "segment", rows).written(segment.written.clone())
            }
        }
        Element::Feature { at, name } => {
            let mine = |owner: Option<&str>| owner == Some(name.as_str());

            let (place, states, listeners) = match at {
                None => ("application".to_string(), Vec::new(), Vec::new()),
                Some((root, depth)) => {
                    let (index, root) = root_of(snapshot, *root)?;
                    let segment = root.chain.get(*depth)?;
                    (
                        format!("{} in {}", segment.name, window_name(root.label.as_deref(), index)),
                        segment
                            .reducers
                            .iter()
                            .filter(|state| mine(state.feature.as_deref()))
                            .map(|state| type_name(&state.type_name))
                            .collect(),
                        segment
                            .listeners
                            .iter()
                            .filter(|listener| mine(listener.feature.as_deref()))
                            .map(|listener| {
                                format!("{} on the {:?} bus", type_name(&listener.event), listener.bus)
                            })
                            .collect(),
                    )
                }
            };
            let actors: Vec<&str> = snapshot
                .actors
                .iter()
                .filter(|actor| {
                    mine(actor.feature.as_deref())
                        && match at {
                            None => actor.root.is_none(),
                            Some((root, depth)) => {
                                actor.root == Some(*root) && actor.segment == Some(*depth)
                            }
                        }
                })
                .map(|actor| type_name(&actor.type_name))
                .collect();

            let mut rows = vec![row("installed in", place)];
            if !states.is_empty() {
                rows.push(row("states", states.join(", ")));
            }
            if !actors.is_empty() {
                rows.push(row("actors", actors.join(", ")));
            }
            for listener in listeners {
                rows.push(row("listens", listener));
            }

            let declared = match at {
                None => None,
                Some((root, depth)) => root_of(snapshot, *root)
                    .and_then(|(_, root)| root.chain.get(*depth))
                    .and_then(|segment| segment.features.iter().find(|feature| mine(Some(&feature.name))))
                    .and_then(|feature| feature.declared.clone()),
            };

            details(name, "feature", rows).written(declared)
        }
        Element::State {
            root,
            depth,
            type_name: full,
        } => {
            let (_, root) = root_of(snapshot, *root)?;
            let state = root
                .chain
                .get(*depth)?
                .reducers
                .iter()
                .find(|state| state.type_name == *full)?;

            Details {
                body: Some(
                    state
                        .state
                        .clone()
                        .unwrap_or_else(|| "not printable".to_string()),
                ),
                declared: state.declared.clone(),
                ..details(type_name(full), "state", vec![row("type", full)])
            }
        }
        Element::Actor(id) => {
            let actor = snapshot.actors.iter().find(|actor| actor.id == *id)?;

            let mut rows = actor_rows(snapshot, actor);
            let waiting: Vec<&str> = session
                .tasks
                .of_actor(*id)
                .map(|task| type_name(&task.output))
                .collect();
            if !waiting.is_empty() {
                rows.push(row("waiting on", waiting.join(", ")));
            }

            Details {
                body: Some(actor.state.clone()),
                declared: actor.declared.clone(),
                handlers: handlers(actor),
                ..details(type_name(&actor.type_name), "actor", rows)
            }
        }
        Element::View { root, path } => {
            let (_, root) = root_of(snapshot, *root)?;
            let node = find(view_of(root)?, path.get(1..)?)?;

            details(&node.label, "view", node.properties.clone())
        }
    })
}

fn handlers(actor: &Actor) -> Vec<HandlerLine> {
    actor
        .handles
        .iter()
        .map(|handled| HandlerLine {
            message: type_name(&handled.message).to_string(),
            flows: handled
                .edges
                .iter()
                .flatten()
                .map(|flow| {
                    let how = match flow.channel {
                        Channel::Send => "sends",
                        Channel::Bg => "works on",
                        Channel::Emit => "emits",
                        Channel::Ask => "asks",
                    };
                    format!("{how} {}", type_name(&flow.target))
                })
                .collect::<Vec<_>>()
                .join(", "),
            declared: handled.declared.clone(),
        })
        .collect()
}

fn actor_rows(snapshot: &Snapshot, actor: &Actor) -> Vec<(String, String)> {
    let mut rows = vec![row("type", &actor.type_name)];

    if let Some(declared) = &actor.declared {
        rows.push(row(
            "declared",
            format!("{}:{}:{}", declared.file, declared.line, declared.column),
        ));
    }

    if let Some((index, root)) = actor.root.and_then(|id| root_of(snapshot, id)) {
        rows.push(row("window", window_name(root.label.as_deref(), index)));
        if let Some(segment) = actor.segment.and_then(|depth| root.chain.get(depth)) {
            rows.push(row("segment", &segment.name));
        }
    }

    if let Some(feature) = &actor.feature {
        rows.push(row("started by", feature));
    }
    if let Some(drives) = &actor.drives {
        rows.push(row("drives", type_name(drives)));
    }

    for event in &actor.publishes {
        rows.push(row("publishes", type_name(event)));
    }
    for event in &actor.subscribes {
        rows.push(row("hears", type_name(event)));
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea_devtools_protocol::{Panel, Segment};

    fn node(label: &str, kind: &str, children: Vec<Node>) -> Node {
        Node {
            label: label.into(),
            kind: kind.into(),
            children,
            ..Node::default()
        }
    }

    fn snapshot() -> Snapshot {
        let view = node(
            "Shell",
            "segment",
            vec![node(
                "StackPanel",
                "native",
                vec![node("TextBlock", "\"hi\"", vec![]), node("Page", "segment", vec![])],
            )],
        );

        Snapshot {
            roots: vec![Root {
                id: 1,
                label: Some("main".into()),
                chain: vec![
                    Segment {
                        name: "Shell".into(),
                        ..Segment::default()
                    },
                    Segment {
                        name: "Page".into(),
                        ..Segment::default()
                    },
                ],
                panels: vec![Panel {
                    id: "view".into(),
                    title: "View".into(),
                    nodes: vec![view],
                }],
                ..Root::default()
            }],
            actors: vec![Actor {
                id: 7,
                type_name: "a::Worker".into(),
                root: Some(1),
                segment: Some(1),
                ..Actor::default()
            }],
            ..Snapshot::default()
        }
    }

    fn rows(closed: &HashSet<Element>) -> Vec<(usize, Element)> {
        lines(&snapshot(), closed, None)
            .into_iter()
            .map(|line| (line.depth, line.element))
            .collect()
    }

    #[test]
    fn a_nested_segment_sits_where_its_parent_drew_the_outlet() {
        assert_eq!(
            rows(&HashSet::new()),
            [
                (0, Element::App),
                (0, Element::Window(1)),
                (1, Element::Segment { root: 1, depth: 0 }),
                (2, Element::View { root: 1, path: vec![0, 0] }),
                (3, Element::View { root: 1, path: vec![0, 0, 0] }),
                (3, Element::Segment { root: 1, depth: 1 }),
                (4, Element::Actor(7)),
            ]
        );
    }

    #[test]
    fn a_closed_row_hides_what_is_under_it() {
        let closed = HashSet::from([Element::View { root: 1, path: vec![0, 0] }]);
        let rows = rows(&closed);

        assert_eq!(rows.len(), 4);
        assert_eq!(rows[3], (2, Element::View { root: 1, path: vec![0, 0] }));
    }

    #[test]
    fn a_revealed_row_opens_the_way_to_it() {
        let closed = HashSet::from([Element::View { root: 1, path: vec![0, 0] }]);
        let shown = lines(&snapshot(), &closed, Some(&Element::Actor(7)));

        assert!(shown.iter().any(|line| line.element == Element::Actor(7)));
    }

    #[test]
    fn an_element_reads_back_from_its_id() {
        for element in [
            Element::App,
            Element::Actor(7),
            Element::Window(1),
            Element::Segment { root: 1, depth: 2 },
            Element::State {
                root: 1,
                depth: 0,
                type_name: "a::b::Tabs".into(),
            },
            Element::View {
                root: 3,
                path: vec![0, 2, 1],
            },
            Element::Feature {
                at: Some((1, 0)),
                name: "TabsFeature".into(),
            },
            Element::Feature {
                at: None,
                name: "StartupFeature".into(),
            },
        ] {
            assert_eq!(element.to_string().parse::<Element>(), Ok(element));
        }

        assert!("nothing/1".parse::<Element>().is_err());
        assert!("feature/app".parse::<Element>().is_err());
    }

    #[test]
    fn what_a_feature_brought_sits_under_it() {
        let mut snapshot = snapshot();
        let page = &mut snapshot.roots[0].chain[1];
        page.features = vec!["PageFeature".into(), "EmptyFeature".into()];
        page.reducers = vec![
            ReducerState {
                type_name: "a::Page".into(),
                feature: Some("PageFeature".into()),
                ..ReducerState::default()
            },
            ReducerState {
                type_name: "a::Loose".into(),
                ..ReducerState::default()
            },
        ];

        snapshot.actors[0].feature = Some("PageFeature".into());
        snapshot.actors.push(Actor {
            id: 8,
            type_name: "a::Sweeper".into(),
            feature: Some("StartupFeature".into()),
            ..Actor::default()
        });

        let feature = |at, name: &str| Element::Feature {
            at,
            name: name.into(),
        };
        let state = |name: &str| Element::State {
            root: 1,
            depth: 1,
            type_name: name.into(),
        };

        let rows: Vec<(usize, Element)> = lines(&snapshot, &HashSet::new(), None)
            .into_iter()
            .map(|line| (line.depth, line.element))
            .collect();

        assert_eq!(rows[..3], [
            (0, Element::App),
            (1, feature(None, "StartupFeature")),
            (2, Element::Actor(8)),
        ]);
        assert_eq!(rows[rows.len() - 6..], [
            (3, Element::Segment { root: 1, depth: 1 }),
            (4, feature(Some((1, 1)), "PageFeature")),
            (5, state("a::Page")),
            (5, Element::Actor(7)),
            (4, feature(Some((1, 1)), "EmptyFeature")),
            (4, state("a::Loose")),
        ]);
    }
}
