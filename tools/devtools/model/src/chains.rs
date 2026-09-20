//! The trace as chains of records that set each other off.
//!
//! A record whose cause devtools do not know starts a chain, and everything
//! it sets off belongs to that chain. A step that repeats one above it starts
//! a chain of its own, so a loop is a run of short chains rather than one that
//! never ends. Chains of the same shape are one group, and every chain belongs
//! to one stream: what started it - an action, a timer, a loop.
//!
//! Kept up to date as records arrive, one record at a time.

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use guinea_devtools_protocol::{Declared, Span, TracePoint};
use serde::{Deserialize, Serialize};

use crate::names::{period, took, type_name};
use crate::timers::Timers;
use crate::trace::{Reading, TREE_LIMIT};
use crate::trace_log::TraceLog;
use crate::words::{self, Kind, Level, Tone, Word};

/// How many steps of a chain its line shows.
pub const STEPS_SHOWN: usize = 6;

/// How many of a group's newest chains are listed under it.
pub const RUNS_SHOWN: usize = 20;

/// How many of a timer's newest ticks its chart draws.
pub const TICKS_DRAWN: usize = 40;

/// How far up a step looks for the same step, to tell a loop.
const LOOP_REACH: usize = 32;

/// What a list of chains is about.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Stream {
    /// Every chain, with each timer and loop as one line.
    All,
    /// Not chains: every record, one per line.
    Records,
    /// Chains an action started, by the action's message.
    Action(String),
    /// Chains a timer started, by where the timer was set up.
    Timer(String),
    /// Chains a repeating step started, by the step.
    Loop(String),
    Navigation,
    Store,
    Log,
    /// Chains whose start devtools cannot explain.
    Loose,
}

impl fmt::Display for Stream {
    /// `all`, `records`, `action/c::Kill`, `timer/src/a.rs:4:9`, `loop/…`,
    /// `navigation`, `store`, `log`, `loose`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stream::All => f.write_str("all"),
            Stream::Records => f.write_str("records"),
            Stream::Action(message) => write!(f, "action/{message}"),
            Stream::Timer(site) => write!(f, "timer/{site}"),
            Stream::Loop(step) => write!(f, "loop/{step}"),
            Stream::Navigation => f.write_str("navigation"),
            Stream::Store => f.write_str("store"),
            Stream::Log => f.write_str("log"),
            Stream::Loose => f.write_str("loose"),
        }
    }
}

impl FromStr for Stream {
    type Err = String;

    fn from_str(id: &str) -> Result<Self, String> {
        Ok(match id.split_once('/') {
            Some(("action", message)) => Stream::Action(message.to_string()),
            Some(("timer", site)) => Stream::Timer(site.to_string()),
            Some(("loop", step)) => Stream::Loop(step.to_string()),
            Some(_) => return Err(format!("not a stream: {id}")),
            None => match id {
                "all" => Stream::All,
                "records" => Stream::Records,
                "navigation" => Stream::Navigation,
                "store" => Stream::Store,
                "log" => Stream::Log,
                "loose" => Stream::Loose,
                _ => return Err(format!("not a stream: {id}")),
            },
        })
    }
}

impl From<Stream> for String {
    fn from(stream: Stream) -> String {
        stream.to_string()
    }
}

impl TryFrom<String> for Stream {
    type Error = String;

    fn try_from(id: String) -> Result<Self, String> {
        id.parse()
    }
}

/// Where a stream is listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    Actions,
    Timers,
    Loops,
    Other,
}

impl Stream {
    pub fn section(&self) -> Option<Section> {
        match self {
            Stream::All | Stream::Records => None,
            Stream::Action(_) => Some(Section::Actions),
            Stream::Timer(_) => Some(Section::Timers),
            Stream::Loop(_) => Some(Section::Loops),
            Stream::Navigation | Stream::Store | Stream::Log | Stream::Loose => Some(Section::Other),
        }
    }

    /// Whether its chains are one line in [`Stream::All`].
    fn repeats(&self) -> bool {
        matches!(self, Stream::Timer(_) | Stream::Loop(_))
    }
}

#[derive(Clone, Copy, Debug)]
struct Node {
    root: u64,
    parent: Option<u64>,
    key: u64,
    /// The steps from the root down to here, as one number.
    path: u64,
}

#[derive(Clone, Debug)]
struct Chain {
    stream: Stream,
    /// Every step's path, summed: the same for chains of the same shape,
    /// whatever order their steps arrived in.
    shape: u64,
    size: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Chains {
    /// The newest record taken in.
    seen: u64,
    nodes: BTreeMap<u64, Node>,
    /// By the record that started them.
    chains: BTreeMap<u64, Chain>,
    /// The chains of each stream and shape, by the record that started them.
    groups: HashMap<(Stream, u64), BTreeSet<u64>>,
}

impl Chains {
    /// Takes in what `log` got since last time, and lets go of what it
    /// forgot.
    pub fn absorb(&mut self, log: &TraceLog, timers: &Timers) {
        if let Some(first) = log.first_id() {
            self.forget(first);
        }

        let fresh: Vec<&Span> = log.after(self.seen).collect();
        for span in fresh {
            self.seen = span.id;
            self.take(span, timers);
        }
    }

    fn take(&mut self, span: &Span, timers: &Timers) {
        let key = key(&span.point, timers);
        let parent = span
            .parent
            .and_then(|parent| self.nodes.get(&parent).map(|node| (parent, *node)));

        let repeats = parent.is_some_and(|(parent, _)| self.repeats(parent, key));
        let (root, path, stream) = match parent {
            Some((_, node)) if !repeats => (node.root, mix(node.path, key), None),
            _ => {
                let stream = if repeats {
                    Stream::Loop(words::text(&words::sentence(&span.point, timers)))
                } else {
                    stream_of(span, timers)
                };
                (span.id, mix(0, key), Some(stream))
            }
        };

        self.nodes.insert(
            span.id,
            Node {
                root,
                parent: parent.filter(|_| !repeats).map(|(parent, _)| parent),
                key,
                path,
            },
        );

        if let Some(stream) = stream {
            self.chains.insert(
                root,
                Chain {
                    stream,
                    shape: 0,
                    size: 0,
                },
            );
        }

        let Some(chain) = self.chains.get_mut(&root) else {
            return;
        };

        if chain.size > 0 {
            let old = (chain.stream.clone(), chain.shape);
            if let Some(roots) = self.groups.get_mut(&old) {
                roots.remove(&root);
                if roots.is_empty() {
                    self.groups.remove(&old);
                }
            }
        }

        chain.shape = chain.shape.wrapping_add(path);
        chain.size += 1;
        self.groups
            .entry((chain.stream.clone(), chain.shape))
            .or_default()
            .insert(root);
    }

    /// Whether a step with `key` under `parent` repeats one above it in the
    /// same chain.
    fn repeats(&self, parent: u64, key: u64) -> bool {
        let mut at = Some(parent);

        for _ in 0..LOOP_REACH {
            let Some(node) = at.and_then(|id| self.nodes.get(&id)) else {
                return false;
            };
            if node.key == key {
                return true;
            }

            at = node.parent;
        }

        false
    }

    fn forget(&mut self, first: u64) {
        self.nodes = self.nodes.split_off(&first);

        let kept = self.chains.split_off(&first);
        let gone = std::mem::replace(&mut self.chains, kept);
        for (root, chain) in gone {
            let group = (chain.stream, chain.shape);
            if let Some(roots) = self.groups.get_mut(&group) {
                roots.remove(&root);
                if roots.is_empty() {
                    self.groups.remove(&group);
                }
            }
        }
    }

    fn roots_of<'a>(&'a self, stream: &'a Stream) -> impl Iterator<Item = u64> + 'a {
        self.groups
            .iter()
            .filter(move |((of, _), _)| of == stream)
            .flat_map(|(_, roots)| roots.iter().copied())
    }
}

fn mix(path: u64, key: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    (path, key).hash(&mut hasher);
    hasher.finish()
}

/// What makes two steps the same step: what they did and to whom, not when
/// or with what numbers.
fn key(point: &TracePoint, timers: &Timers) -> u64 {
    let mut hasher = DefaultHasher::new();
    let h = &mut hasher;

    match point {
        TracePoint::Action { message } => ("action", message).hash(h),
        TracePoint::Send { actor, message } => ("send", actor, message).hash(h),
        TracePoint::Handle { actor, message } => ("handle", actor, message).hash(h),
        TracePoint::Spawn { actor, output, .. } => ("spawn", actor, output).hash(h),
        TracePoint::Settled { actor, output, .. } => ("settled", actor, output).hash(h),
        TracePoint::Cancelled { actor, output, .. } => ("cancelled", actor, output).hash(h),
        TracePoint::Publish { event, bus, .. } => ("publish", event, bus).hash(h),
        TracePoint::Deliver { event, bus } => ("deliver", event, bus).hash(h),
        TracePoint::Push { reducer } => ("push", reducer).hash(h),
        TracePoint::Navigate { root, to } => ("navigate", root, to).hash(h),
        TracePoint::Tick { timer } => ("tick", timer.map(|id| timers.site(id))).hash(h),
        TracePoint::Store { op, path, .. } => ("store", op, path).hash(h),
        // The segment, not how long it took: two slow frames of one page are
        // the same step of a chain.
        TracePoint::Render { segment, .. } => ("render", segment).hash(h),
        TracePoint::Log { level, target, .. } => ("log", level, target).hash(h),
        TracePoint::Note { text } => ("note", text).hash(h),
    }

    hasher.finish()
}

fn stream_of(span: &Span, timers: &Timers) -> Stream {
    match &span.point {
        TracePoint::Tick { timer: Some(id) } => Stream::Timer(timers.site(*id)),
        TracePoint::Tick { timer: None } => Stream::Timer("#?".to_string()),
        TracePoint::Action { message } => Stream::Action(message.clone()),
        TracePoint::Navigate { .. } => Stream::Navigation,
        TracePoint::Store { .. } => Stream::Store,
        TracePoint::Log { .. } | TracePoint::Note { .. } if span.parent.is_none() => Stream::Log,
        _ => Stream::Loose,
    }
}

/// A stream, in the list of them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StreamLine {
    pub stream: Stream,
    pub section: Section,
    pub title: String,
    /// How many chains it holds.
    pub chains: usize,
}

/// Every stream that holds a chain, by section, then by title.
pub fn streams(chains: &Chains, reading: Reading) -> Vec<StreamLine> {
    let mut counts: BTreeMap<&Stream, usize> = BTreeMap::new();
    for ((stream, _), roots) in &chains.groups {
        *counts.entry(stream).or_default() += roots.len();
    }

    let mut lines: Vec<StreamLine> = counts
        .into_iter()
        .filter_map(|(stream, count)| {
            Some(StreamLine {
                section: stream.section()?,
                title: title(chains, reading, stream),
                stream: stream.clone(),
                chains: count,
            })
        })
        .collect();

    lines.sort_by(|one, other| (one.section, &one.title).cmp(&(other.section, &other.title)));
    lines
}

/// What a stream is called on screen.
pub fn title(chains: &Chains, reading: Reading, stream: &Stream) -> String {
    match stream {
        Stream::All => "Everything".to_string(),
        Stream::Records => "Records".to_string(),
        Stream::Action(message) => {
            let actor = chains
                .roots_of(stream)
                .max()
                .and_then(|root| {
                    reading.log.children(root).find_map(|child| match &child.point {
                        TracePoint::Send { actor, .. } => Some(type_name(actor).to_string()),
                        _ => None,
                    })
                });

            match actor {
                Some(actor) => format!("{} → {actor}", type_name(message)),
                None => type_name(message).to_string(),
            }
        }
        Stream::Timer(site) => reading
            .timers
            .at(site)
            .next()
            .map_or_else(|| site.clone(), crate::timers::label),
        Stream::Loop(step) => step.clone(),
        Stream::Navigation => "Navigation".to_string(),
        Stream::Store => "Store".to_string(),
        Stream::Log => "Logs".to_string(),
        Stream::Loose => "Unexplained".to_string(),
    }
}

/// One chain of a group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Run {
    /// The record that started it.
    pub root: u64,
    pub time: String,
    /// Its longest step.
    pub took: Option<String>,
}

/// Chains of one shape, or in [`Stream::All`] a whole timer or loop.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupLine {
    /// Tells this group from the others of its stream, while its shape holds.
    pub key: String,
    pub stream: Stream,
    pub count: usize,
    /// The newest chain's steps, joined by arrows.
    pub words: Vec<Word>,
    /// When the newest chain started.
    pub time: String,
    pub newest: u64,
    /// The longest step of the chains listed.
    pub longest: Option<String>,
    /// The newest chains, newest first.
    pub runs: Vec<Run>,
    /// The stream to open instead of listing runs: a timer's or a loop's.
    pub opens: Option<Stream>,
}

/// A timer, above its chains.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimerView {
    pub title: String,
    pub declared: Option<Declared>,
    pub feature: Option<String>,
    pub period: Option<String>,
    /// How many of it are running: one per window that set it up.
    pub running: usize,
    pub fires: usize,
    pub average: Option<String>,
    pub longest: Option<String>,
    /// The newest ticks' longest steps in microseconds, oldest first.
    pub recent: Vec<u64>,
}

/// A stream, whole.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StreamView {
    pub stream: Stream,
    pub title: String,
    pub timer: Option<TimerView>,
    /// Newest first.
    pub groups: Vec<GroupLine>,
}

pub fn view(chains: &Chains, reading: Reading, stream: &Stream, text: &str) -> StreamView {
    let wanted = text.to_lowercase();
    let wants = |line: &GroupLine| {
        wanted.is_empty() || words::text(&line.words).to_lowercase().contains(&wanted)
    };

    let mut groups: Vec<GroupLine> = match stream {
        Stream::Records => Vec::new(),
        Stream::All => {
            let mut lines: Vec<GroupLine> = chains
                .groups
                .iter()
                .filter(|((of, _), _)| !of.repeats())
                .map(|((of, shape), roots)| group(chains, reading, of, *shape, roots))
                .collect();

            let repeating: BTreeSet<&Stream> = chains
                .groups
                .keys()
                .map(|(of, _)| of)
                .filter(|of| of.repeats())
                .collect();
            lines.extend(repeating.into_iter().map(|of| whole(chains, reading, of)));

            lines
        }
        _ => chains
            .groups
            .iter()
            .filter(|((of, _), _)| of == stream)
            .map(|((of, shape), roots)| group(chains, reading, of, *shape, roots))
            .collect(),
    };

    groups.retain(|line| wants(line));
    groups.sort_by_key(|group| std::cmp::Reverse(group.newest));

    StreamView {
        title: title(chains, reading, stream),
        timer: match stream {
            Stream::Timer(site) => Some(timer(chains, reading, stream, site)),
            _ => None,
        },
        stream: stream.clone(),
        groups,
    }
}

fn group(
    chains: &Chains,
    reading: Reading,
    stream: &Stream,
    shape: u64,
    roots: &BTreeSet<u64>,
) -> GroupLine {
    let runs: Vec<Run> = roots
        .iter()
        .rev()
        .take(RUNS_SHOWN)
        .filter_map(|root| run(chains, reading, *root))
        .collect();
    let newest = roots.last().copied().unwrap_or_default();

    GroupLine {
        key: format!("{shape:016x}"),
        stream: stream.clone(),
        count: roots.len(),
        words: steps(chains, reading, newest),
        time: time(reading, newest),
        newest,
        longest: longest(chains, reading, runs.iter().map(|run| run.root)).map(took),
        runs,
        opens: None,
    }
}

/// A timer or a loop as one line of [`Stream::All`].
fn whole(chains: &Chains, reading: Reading, stream: &Stream) -> GroupLine {
    let count = chains.roots_of(stream).count();
    let newest = chains.roots_of(stream).max().unwrap_or_default();

    let mut words = match stream {
        Stream::Timer(_) => vec![Word::new("timer ", Tone::Kind(Kind::Tick))],
        _ => vec![Word::new("loop ", Tone::Kind(Kind::Spawn))],
    };
    words.push(Word::new(title(chains, reading, stream), Tone::Plain));

    GroupLine {
        key: stream.to_string(),
        stream: stream.clone(),
        count,
        words,
        time: time(reading, newest),
        newest,
        longest: None,
        runs: Vec::new(),
        opens: Some(stream.clone()),
    }
}

fn timer(chains: &Chains, reading: Reading, stream: &Stream, site: &str) -> TimerView {
    let known: Vec<_> = reading.timers.at(site).collect();
    let newest = known.iter().max_by_key(|timer| timer.id);

    let mut roots: Vec<u64> = chains.roots_of(stream).collect();
    roots.sort_unstable();

    let recent: Vec<u64> = roots
        .iter()
        .rev()
        .take(TICKS_DRAWN)
        .filter_map(|root| longest(chains, reading, [*root]))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    TimerView {
        title: title(chains, reading, stream),
        declared: newest.and_then(|timer| timer.declared.clone()),
        feature: newest.and_then(|timer| timer.feature.clone()),
        period: newest.map(|timer| period(timer.period_ms)),
        running: known
            .iter()
            .filter(|timer| reading.timers.runs(timer.id))
            .count(),
        fires: roots.len(),
        average: (!recent.is_empty())
            .then(|| recent.iter().sum::<u64>() / recent.len() as u64)
            .map(took),
        longest: recent.iter().max().copied().map(took),
        recent,
    }
}

fn run(chains: &Chains, reading: Reading, root: u64) -> Option<Run> {
    let span = reading.log.get(root)?;

    Some(Run {
        root,
        time: reading.clock.short(span.at),
        took: longest(chains, reading, [root]).map(took),
    })
}

fn time(reading: Reading, root: u64) -> String {
    reading
        .log
        .get(root)
        .map(|span| reading.clock.short(span.at))
        .unwrap_or_default()
}

/// The records of the chain `root` started, in the order they happened.
fn members(chains: &Chains, reading: Reading, root: u64) -> Vec<u64> {
    let mut found = Vec::new();
    let mut waiting = vec![root];

    while let Some(id) = waiting.pop() {
        if found.len() >= TREE_LIMIT {
            break;
        }

        found.push(id);
        let children: Vec<u64> = reading
            .log
            .children(id)
            .map(|child| child.id)
            .filter(|child| chains.nodes.get(child).is_some_and(|node| node.root == root))
            .collect();
        waiting.extend(children.into_iter().rev());
    }

    found
}

/// The longest step of the chains `roots` started, in microseconds.
fn longest(chains: &Chains, reading: Reading, roots: impl IntoIterator<Item = u64>) -> Option<u64> {
    roots
        .into_iter()
        .flat_map(|root| members(chains, reading, root))
        .filter_map(|id| reading.log.get(id).and_then(|span| span.took))
        .max()
}

/// A chain's steps as one line: `timer housekeeping → Housekeeping handles
/// Sweep → debug app::startup`. Sends are left out; what handled them says
/// it.
fn steps(chains: &Chains, reading: Reading, root: u64) -> Vec<Word> {
    let shown: Vec<&Span> = members(chains, reading, root)
        .into_iter()
        .filter_map(|id| reading.log.get(id))
        .filter(|span| !matches!(span.point, TracePoint::Send { .. }) || span.id == root)
        .collect();

    let mut out = Vec::new();
    for (at, span) in shown.iter().take(STEPS_SHOWN).enumerate() {
        if at > 0 {
            out.push(Word::new(" → ", Tone::Muted));
        }

        match &span.point {
            TracePoint::Log { level, target, .. } => {
                out.push(Word::new(format!("{} {target}", level.to_lowercase()), Tone::Level(Level::named(level))));
            }
            point => out.extend(words::gist_words(point, reading.timers)),
        }
    }

    if shown.len() > STEPS_SHOWN {
        out.push(Word::new(format!(" → … {} more", shown.len() - STEPS_SHOWN), Tone::Muted));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Clock;
    use guinea_devtools_protocol::{Timer, TraceBatch};

    fn span(id: u64, parent: Option<u64>, point: TracePoint) -> Span {
        Span {
            id,
            parent,
            at: id,
            took: Some(id * 10),
            point,
        }
    }

    fn handle(message: &str) -> TracePoint {
        TracePoint::Handle {
            actor: "a::Worker".into(),
            message: message.into(),
        }
    }

    fn log_of(spans: Vec<Span>) -> TraceLog {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans,
            ends: Vec::new(),
            dropped: 0,
        });

        log
    }

    fn timers() -> Timers {
        let mut timers = Timers::default();
        timers.note(&[Timer {
            id: 7,
            name: Some("sweep".into()),
            period_ms: 5_000,
            ..Timer::default()
        }]);

        timers
    }

    fn tick(id: u64) -> Vec<Span> {
        vec![
            span(id, None, TracePoint::Tick { timer: Some(7) }),
            span(id + 1, Some(id), handle("a::Sweep")),
        ]
    }

    #[test]
    fn ticks_of_one_shape_are_one_group_of_one_timer() {
        let timers = timers();
        let mut spans = tick(1);
        spans.extend(tick(3));
        spans.extend(tick(5));
        spans.push(span(7, Some(5), TracePoint::Push { reducer: "a::List".into() }));
        let log = log_of(spans);

        let mut chains = Chains::default();
        chains.absorb(&log, &timers);

        let reading = Reading {
            log: &log,
            clock: Clock::default(),
            timers: &timers,
        };
        let timer = Stream::Timer("#7".into());

        let listed = streams(&chains, reading);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title, "sweep");
        assert_eq!(listed[0].chains, 3);

        let view = view(&chains, reading, &timer, "");
        let counts: Vec<usize> = view.groups.iter().map(|group| group.count).collect();
        assert_eq!(counts, [1, 2], "the one that also pushed is its own group, and newest");
        assert_eq!(
            words::text(&view.groups[1].words),
            "timer sweep → Worker handles Sweep"
        );

        let timer = view.timer.expect("a timer's stream says what the timer is");
        assert_eq!(timer.fires, 3);
        assert_eq!(timer.period.as_deref(), Some("5 s"));
        assert_eq!(timer.running, 1);
        assert_eq!(timer.recent, [20, 40, 70]);
    }

    #[test]
    fn a_loop_is_a_run_of_short_chains() {
        let timers = Timers::default();
        let log = log_of(vec![
            span(1, None, TracePoint::Action { message: "a::Start".into() }),
            span(2, Some(1), handle("a::Tick")),
            span(3, Some(2), handle("a::Work")),
            span(4, Some(3), handle("a::Tick")),
            span(5, Some(4), handle("a::Work")),
            span(6, Some(5), handle("a::Tick")),
            span(7, Some(6), handle("a::Work")),
        ]);

        let mut chains = Chains::default();
        chains.absorb(&log, &timers);

        let reading = Reading {
            log: &log,
            clock: Clock::default(),
            timers: &timers,
        };
        let listed: Vec<(Stream, usize)> = streams(&chains, reading)
            .into_iter()
            .map(|line| (line.stream, line.chains))
            .collect();

        assert_eq!(
            listed,
            [
                (Stream::Action("a::Start".into()), 1),
                (Stream::Loop("Worker handles Tick".into()), 2),
            ]
        );

        let all = view(&chains, reading, &Stream::All, "");
        let opens: Vec<Option<Stream>> = all.groups.iter().map(|group| group.opens.clone()).collect();
        assert!(opens.contains(&Some(Stream::Loop("Worker handles Tick".into()))));
    }

    #[test]
    fn forgotten_chains_leave_their_groups() {
        let timers = timers();
        let mut log = log_of(tick(1));

        let mut chains = Chains::default();
        chains.absorb(&log, &timers);

        let mut more = Vec::new();
        for at in 0..crate::trace_log::KEPT as u64 {
            more.extend(tick(10 + at * 2));
        }
        log.absorb(TraceBatch {
            spans: more,
            ends: Vec::new(),
            dropped: 0,
        });
        chains.absorb(&log, &timers);

        assert!(chains.chains.keys().all(|root| *root >= log.first_id().unwrap_or(0)));
        assert!(chains.groups.values().all(|roots| !roots.contains(&1)));
    }

    #[test]
    fn a_stream_reads_back_from_its_id() {
        for stream in [
            Stream::All,
            Stream::Records,
            Stream::Action("c::Kill".into()),
            Stream::Timer("C:/a/src/x.rs:4:9".into()),
            Stream::Loop("Worker handles Tick".into()),
            Stream::Navigation,
            Stream::Store,
            Stream::Log,
            Stream::Loose,
        ] {
            assert_eq!(stream.to_string().parse::<Stream>(), Ok(stream));
        }
        assert!("nothing".parse::<Stream>().is_err());
    }
}
