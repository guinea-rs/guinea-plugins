//! A session's trace, as it is read: filtered, as rows, and one record with
//! where it came from and what it set off.

use std::collections::VecDeque;

use guinea_devtools_protocol::{Declared, Span, TracePoint};
use serde::{Deserialize, Serialize};

use crate::clock::Clock;
use crate::names::took;
use crate::timers::Timers;
use crate::trace_log::TraceLog;
use crate::words::{self, Kind, Level, Tone, Word};

/// How many records between two steps of a chain are listed.
pub const BETWEEN_LIMIT: usize = 8;

/// How many causes of one record its chain lists, nearest first kept.
pub const CHAIN_LIMIT: usize = 24;

/// How many records below one record its tree of consequences walks.
pub const TREE_LIMIT: usize = 400;

/// How deep the tree of consequences goes: a loop sets itself off for as long
/// as it runs, and each step would be one level deeper.
pub const DEPTH_LIMIT: usize = 12;

/// What a session's trace is read against.
#[derive(Clone, Copy, Debug)]
pub struct Reading<'a> {
    pub log: &'a TraceLog,
    pub clock: Clock,
    /// What names a timer's ticks.
    pub timers: &'a Timers,
}

/// How long the framework's work has to take to be worth a look, in
/// microseconds.
pub const SLOW_US: u64 = 1_000;

/// Which records a list is about.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "class", content = "level", rename_all = "snake_case")]
pub enum Class {
    /// Every record, the framework's as well.
    #[default]
    Every,
    /// What the application's own code wrote: its log lines and spans.
    Own,
    /// The application's own, at one level.
    Level(Level),
    /// The framework's work that took [`SLOW_US`] or more - only the
    /// outermost, what ran inside it is its breakdown.
    Slow,
}

/// Whether `span` is framework work of [`SLOW_US`] or more that nothing
/// slow it ran inside of already accounts for.
pub fn slow(log: &TraceLog, span: &Span) -> bool {
    if worked(span).is_none_or(|took| took < SLOW_US) {
        return false;
    }

    let mut inner = span;
    for _ in 0..DEPTH_LIMIT * 4 {
        let Some(outer) = inner.parent.and_then(|parent| log.get(parent)) else {
            return true;
        };
        if !holds(outer, span) {
            return true;
        }
        if worked(outer).is_some_and(|took| took >= SLOW_US) {
            return false;
        }
        inner = outer;
    }

    true
}

/// How long the framework spent on `span` itself. `None` for what the
/// application wrote, and for what was waited on rather than worked at:
/// background work, a source's life.
fn worked(span: &Span) -> Option<u64> {
    match &span.point {
        TracePoint::Log { .. }
        | TracePoint::Span { .. }
        | TracePoint::Settled { .. }
        | TracePoint::Cancelled { .. }
        | TracePoint::Closed { .. }
        | TracePoint::Pull { .. }
        | TracePoint::Unknown => None,
        TracePoint::Render { took_us, .. } => Some(*took_us),
        _ => span.took,
    }
}

/// Whether `inner` started while `outer` was running.
fn holds(outer: &Span, inner: &Span) -> bool {
    let took = outer.took.or_else(|| measured(&outer.point));
    took.is_some_and(|took| inner.at >= outer.at && inner.at <= outer.at + took)
}

/// Which records to show.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Query {
    #[serde(default)]
    pub class: Class,
    /// Kinds left out.
    #[serde(default)]
    pub hidden: Vec<String>,
    /// Only records whose sentence has this, ignoring case.
    #[serde(default)]
    pub text: String,
    /// Only the first this many records of the log.
    #[serde(default)]
    pub upto: Option<usize>,
}

impl Query {
    pub fn shows(&self, kind: &str) -> bool {
        !self.hidden.iter().any(|hidden| hidden == kind)
    }

    /// By class and kind, then by the text: as shown, or with whole type
    /// paths. A source making its next item never: it is only what others
    /// ran in.
    pub fn matches(&self, reading: Reading, span: &Span) -> bool {
        if matches!(span.point, TracePoint::Pull { .. }) {
            return false;
        }
        let timers = reading.timers;
        let in_class = match self.class {
            Class::Every => true,
            Class::Own => Level::of(&span.point).is_some(),
            Class::Level(level) => Level::of(&span.point) == Some(level),
            Class::Slow => slow(reading.log, span),
        };
        if !in_class || !self.shows(span.point.kind()) {
            return false;
        }
        if self.text.is_empty() {
            return true;
        }

        let wanted = self.text.to_lowercase();
        let has = |text: String| text.to_lowercase().contains(&wanted);

        has(words::text(&words::sentence(&span.point, timers)))
            || has(words::searchable(&span.point, timers))
    }
}

/// The records the query lets through, oldest first.
pub fn shown<'a>(reading: Reading<'a>, query: &Query) -> Vec<&'a Span> {
    let mut shown = Shown::default();
    shown.refresh(reading, query);

    shown
        .ids
        .iter()
        .filter_map(|id| reading.log.get(*id))
        .collect()
}

/// The records a query lets through, kept by id and brought up to date by
/// looking only at what arrived since, so a long log is not read again for
/// every frame.
#[derive(Clone, Debug, Default)]
pub struct Shown {
    /// The query it holds records for, without its limit.
    query: Option<Query>,
    ids: VecDeque<u64>,
    /// Where in the log's arrivals it stands.
    cursor: u64,
    /// Where in the log's ends it stands.
    ended: u64,
    /// The newest record the query's limit let in last time.
    limit: u64,
}

impl Shown {
    pub fn refresh(&mut self, reading: Reading, query: &Query) {
        let log = reading.log;
        let criteria = Query {
            upto: None,
            ..query.clone()
        };
        let limit = match query.upto {
            None => u64::MAX,
            Some(0) => 0,
            Some(count) => log.id_at(count - 1).unwrap_or(u64::MAX),
        };
        if self.query.as_ref() != Some(&criteria) {
            *self = Shown {
                query: Some(criteria.clone()),
                limit,
                ended: log.endings(),
                ..Shown::default()
            };
        }

        if limit < self.limit {
            while self.ids.back().is_some_and(|id| *id > limit) {
                self.ids.pop_back();
            }
        } else if limit > self.limit {
            for span in log.between(self.limit, limit.saturating_add(1)) {
                if criteria.matches(reading, span) {
                    self.insert(span.id);
                }
            }
        }
        self.limit = limit;

        if let Some(first) = log.first_id() {
            while self.ids.front().is_some_and(|id| *id < first) {
                self.ids.pop_front();
            }
        }

        for span in log.since(self.cursor) {
            if span.id <= limit && criteria.matches(reading, span) {
                self.insert(span.id);
            }
        }
        self.cursor = log.arrivals();

        for span in log.ended_since(self.ended) {
            if span.id > limit {
                continue;
            }
            if criteria.matches(reading, span) {
                self.insert(span.id);
            }
            if criteria.class == Class::Slow {
                self.recheck_inside(reading, &criteria, span);
            }
        }
        self.ended = log.endings();
    }

    /// Takes out what ran inside `outer` and no longer matches: work that
    /// was listed as slow until `outer`'s end said it ran inside something
    /// slower.
    fn recheck_inside(&mut self, reading: Reading, criteria: &Query, outer: &Span) {
        let from = self.ids.partition_point(|id| *id <= outer.id);
        let stale: Vec<u64> = self
            .ids
            .range(from..)
            .filter_map(|id| reading.log.get(*id))
            .take_while(|inner| holds(outer, inner))
            .filter(|inner| !criteria.matches(reading, inner))
            .map(|inner| inner.id)
            .collect();

        self.ids.retain(|id| !stale.contains(id));
    }

    fn insert(&mut self, id: u64) {
        if let Err(at) = self.ids.binary_search(&id) {
            self.ids.insert(at, id);
        }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Where the record `id` is among those shown.
    pub fn position(&self, id: u64) -> Option<usize> {
        self.ids.binary_search(&id).ok()
    }

    /// The records at `range` of those shown.
    pub fn spans<'a>(
        &'a self,
        log: &'a TraceLog,
        range: std::ops::Range<usize>,
    ) -> impl Iterator<Item = &'a Span> {
        self.ids.range(range).filter_map(|id| log.get(*id))
    }
}

/// Each class's records, brought up to date as the trace arrives rather
/// than read again when something asks.
#[derive(Clone, Debug, Default)]
pub struct Classes {
    lists: Vec<(Class, Shown)>,
}

impl Classes {
    /// Every class a list of records is offered for, apart from every record.
    pub fn offered() -> impl Iterator<Item = Class> {
        std::iter::once(Class::Own)
            .chain(Level::ALL.into_iter().map(Class::Level))
            .chain(std::iter::once(Class::Slow))
    }

    /// Takes in what arrived and what ended since the last time.
    pub fn absorb(&mut self, reading: Reading) {
        if self.lists.is_empty() {
            self.lists = Classes::offered()
                .map(|class| (class, Shown::default()))
                .collect();
        }

        for (class, shown) in &mut self.lists {
            shown.refresh(
                reading,
                &Query {
                    class: *class,
                    ..Query::default()
                },
            );
        }
    }

    pub fn count(&self, class: Class) -> usize {
        self.list(class).map_or(0, Shown::len)
    }

    pub fn list(&self, class: Class) -> Option<&Shown> {
        self.lists
            .iter()
            .find(|(listed, _)| *listed == class)
            .map(|(_, shown)| shown)
    }
}

/// What caused a record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "cause", rename_all = "snake_case")]
pub enum Cause {
    Known {
        id: u64,
        gist: String,
        tone: Tone,
    },
    /// Older than what devtools kept.
    Forgotten,
}

/// A record as a line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub id: u64,
    pub kind: Kind,
    /// Hour and minute.
    pub time: String,
    /// To the millisecond, and since the application started.
    pub when: String,
    pub took: Option<String>,
    pub words: Vec<Word>,
    pub cause: Option<Cause>,
}

/// How long a point that has no extent of its own says it took.
///
/// A span's `took` comes from the record that ended it; a mark has none, and
/// the ones that measured something carry it in the point instead.
fn measured(point: &TracePoint) -> Option<u64> {
    match point {
        TracePoint::Settled { took_us, .. }
        | TracePoint::Cancelled { took_us, .. }
        | TracePoint::Closed { took_us, .. }
        | TracePoint::Render { took_us, .. } => Some(*took_us),
        _ => None,
    }
}

pub fn row(reading: Reading, span: &Span) -> Row {
    Row {
        id: span.id,
        kind: Kind::of(&span.point),
        time: reading.clock.short(span.at),
        when: reading.clock.long(span.at),
        took: span.took.or_else(|| measured(&span.point)).map(took),
        words: words::sentence(&span.point, reading.timers),
        cause: span.parent.map(|parent| match reading.log.get(parent) {
            Some(cause) => Cause::Known {
                id: parent,
                gist: words::gist(&cause.point, reading.timers),
                tone: words::tone(&cause.point),
            },
            None => Cause::Forgotten,
        }),
    }
}

/// One step of a chain, and what else happened before the next one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub row: Row,
    pub between: Vec<Row>,
    /// How many more happened in between than are listed.
    pub more: usize,
}

/// A record's chain of causes, from the first one devtools kept.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Origins {
    pub steps: Vec<Step>,
    /// Said when the chain is only the record itself.
    pub note: Option<String>,
}

pub fn origins(reading: Reading, span: &Span, query: &Query) -> Origins {
    let log = reading.log;
    let (chain, cut) = log.provenance(span.id, CHAIN_LIMIT);
    let note = if cut {
        Some(format!(
            "the {CHAIN_LIMIT} nearest causes; the chain goes back further"
        ))
    } else if chain.len() <= 1 {
        Some(if span.parent.is_some() {
            "its cause is older than what devtools kept".to_string()
        } else {
            "nothing observed started it".to_string()
        })
    } else {
        None
    };

    // A send whose handling is the very next step is the same line twice, so
    // only the handling is kept - and the send is kept out of what happened
    // in between as well, or it would come back one line lower.
    let folded: Vec<u64> = chain
        .iter()
        .enumerate()
        .filter(|(at, step)| {
            chain
                .get(at + 1)
                .is_some_and(|next| says_it_again(step, next))
        })
        .map(|(_, step)| step.id)
        .collect();

    let chain: Vec<&Span> = chain
        .into_iter()
        .filter(|step| !folded.contains(&step.id))
        .collect();

    let steps = chain
        .iter()
        .enumerate()
        .map(|(at, step)| {
            let between: Vec<&Span> = chain
                .get(at + 1)
                .map(|next| {
                    log.between(step.id, next.id)
                        .filter(|span| query.shows(span.point.kind()))
                        .filter(|span| !folded.contains(&span.id))
                        .collect()
                })
                .unwrap_or_default();

            Step {
                row: row(reading, step),
                more: between.len().saturating_sub(BETWEEN_LIMIT),
                between: between
                    .iter()
                    .take(BETWEEN_LIMIT)
                    .map(|span| row(reading, span))
                    .collect(),
            }
        })
        .collect();

    Origins { steps, note }
}

/// A record and what it set off in turn.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Consequence {
    pub row: Row,
    pub children: Vec<Consequence>,
}

/// How much of a tree of consequences is left to walk, and whether some was
/// left out.
struct Walk {
    budget: usize,
    cut: bool,
}

/// Whether a send and what it caused are one thing said twice.
///
/// `send Kill to ProcessActor` followed by `ProcessActor handles Kill` is a
/// row that adds nothing and a level of depth in every tree it appears in.
/// The handling is the half worth keeping: it carries how long it took, and
/// whatever it set off hangs under it.
fn says_it_again(sent: &Span, handled: &Span) -> bool {
    match (&sent.point, &handled.point) {
        (
            TracePoint::Send { actor, message },
            TracePoint::Handle {
                actor: by,
                message: what,
            },
        ) => actor == by && message == what,
        _ => false,
    }
}

/// What to show for `sent`: its handling, when that is all the send came to.
///
/// A send whose message was never handled - the actor was gone, or the
/// handling is older than what devtools kept - stays a send, and says so by
/// having nothing under it.
fn instead_of<'a>(reading: Reading<'a>, sent: &'a Span) -> &'a Span {
    let mut caused = reading.log.children(sent.id);

    match (caused.next(), caused.next()) {
        (Some(only), None) if says_it_again(sent, only) => only,
        _ => sent,
    }
}

fn set_off(reading: Reading, id: u64, depth: usize, walk: &mut Walk) -> Vec<Consequence> {
    let mut out = Vec::new();

    for child in reading.log.children(id) {
        if walk.budget == 0 || depth == DEPTH_LIMIT {
            walk.cut = true;
            break;
        }

        let shown = instead_of(reading, child);

        walk.budget -= 1;
        out.push(Consequence {
            row: row(reading, shown),
            children: set_off(reading, shown.id, depth + 1, walk),
        });
    }

    out
}

/// One record, whole.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub row: Row,
    pub origins: Origins,
    pub set_off: Vec<Consequence>,
    /// Whether the tree of consequences stopped at [`TREE_LIMIT`] records or
    /// [`DEPTH_LIMIT`] levels.
    pub cut: bool,
    /// The line that wrote it, for a log or a span that said.
    #[serde(default)]
    pub written: Option<Declared>,
}

pub fn record(reading: Reading, id: u64, query: &Query) -> Option<Record> {
    let span = reading.log.get(id)?;
    let mut walk = Walk {
        budget: TREE_LIMIT,
        cut: false,
    };
    let set_off = set_off(reading, id, 0, &mut walk);

    Some(Record {
        row: row(reading, span),
        origins: origins(reading, span, query),
        set_off,
        cut: walk.cut,
        written: match &span.point {
            TracePoint::Log { written, .. } => written.clone(),
            TracePoint::Span { declared, .. } => declared.clone(),
            _ => None,
        },
    })
}

/// One record as the panel beside a list reads it: what it says, where it
/// was written, what the framework was doing at the time and what ran
/// inside it, with how long each part took. Nothing in it leads elsewhere.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct About {
    pub row: Row,
    pub level: Option<Level>,
    pub target: Option<String>,
    /// A span's fields as they were when it opened, `name=value`.
    pub fields: Option<String>,
    pub written: Option<Declared>,
    /// From what started it down to what it ran in, itself left out; a send
    /// followed by its handling is the handling.
    pub within: Vec<Row>,
    /// Whether older causes than [`CHAIN_LIMIT`] were left out.
    pub within_cut: bool,
    /// What ran inside it while it ran: the application's own records, and
    /// the framework's work that took time.
    pub inside: Vec<Consequence>,
    /// Whether what ran inside stopped at [`TREE_LIMIT`] records or
    /// [`DEPTH_LIMIT`] levels.
    pub inside_cut: bool,
}

pub fn about(reading: Reading, id: u64) -> Option<About> {
    let span = reading.log.get(id)?;

    let (mut chain, within_cut) = reading.log.provenance(id, CHAIN_LIMIT + 1);
    chain.pop();
    let within = chain
        .iter()
        .enumerate()
        .filter(|(at, step)| {
            let next = chain.get(at + 1).copied().unwrap_or(span);
            !says_it_again(step, next)
        })
        .map(|(_, step)| row(reading, step))
        .collect();

    let mut walk = Walk {
        budget: TREE_LIMIT,
        cut: false,
    };
    let inside = ran_inside(reading, span, span.id, 0, &mut walk);

    let (target, fields, written) = match &span.point {
        TracePoint::Log {
            target, written, ..
        } => (Some(target.clone()), None, written.clone()),
        TracePoint::Span {
            target,
            fields,
            declared,
            ..
        } => (
            Some(target.clone()),
            Some(fields.clone()).filter(|fields| !fields.is_empty()),
            declared.clone(),
        ),
        _ => (None, None, None),
    };

    Some(About {
        row: row(reading, span),
        level: Level::of(&span.point),
        target,
        fields,
        written,
        within,
        within_cut,
        inside,
        inside_cut: walk.cut,
    })
}

/// What `from` set off while `outer` ran, as the tree of what is worth a
/// line: the application's own records and work that took time. A mark in
/// between - a send, a push - is passed through to what it set off.
fn ran_inside(
    reading: Reading,
    outer: &Span,
    from: u64,
    depth: usize,
    walk: &mut Walk,
) -> Vec<Consequence> {
    let mut out = Vec::new();

    for child in reading.log.children(from) {
        if !holds(outer, child) {
            continue;
        }
        if walk.budget == 0 || depth == DEPTH_LIMIT {
            walk.cut = true;
            break;
        }

        let worth_a_line =
            Level::of(&child.point).is_some() || worked(child).is_some_and(|took| took > 0);
        if worth_a_line {
            walk.budget -= 1;
            out.push(Consequence {
                row: row(reading, child),
                children: ran_inside(reading, child, child.id, depth + 1, walk),
            });
        } else {
            out.extend(ran_inside(reading, outer, child.id, depth, walk));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea_devtools_protocol::{TraceBatch, TracePoint};

    fn read<'a>(log: &'a TraceLog, timers: &'a Timers) -> Reading<'a> {
        Reading {
            log,
            clock: Clock::default(),
            timers,
        }
    }

    fn log() -> TraceLog {
        let span = |id, parent, point| Span {
            id,
            parent,
            at: id * 10,
            took: None,
            point,
            thread: 0,
        };

        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                span(
                    1,
                    None,
                    TracePoint::Action {
                        message: "a::Kill".into(),
                    },
                ),
                span(2, Some(1), TracePoint::Tick { timer: None }),
                span(
                    3,
                    None,
                    TracePoint::Note {
                        text: "else".into(),
                    },
                ),
                span(
                    4,
                    Some(2),
                    TracePoint::Push {
                        reducer: "a::Tabs".into(),
                    },
                ),
            ],
            ends: Vec::new(),
            dropped: 0,
        });

        log
    }

    #[test]
    fn what_happened_between_the_steps_is_listed_under_the_step() {
        let log = log();
        let timers = Timers::default();
        let query = Query {
            hidden: vec!["tick".into()],
            ..Query::default()
        };
        let record = record(read(&log, &timers), 4, &query).expect("kept");

        let steps: Vec<u64> = record
            .origins
            .steps
            .iter()
            .map(|step| step.row.id)
            .collect();
        assert_eq!(steps, [1, 2, 4]);

        let between: Vec<u64> = record.origins.steps[1]
            .between
            .iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(between, [3]);
        assert!(
            record.origins.steps[0].between.is_empty(),
            "nothing between 1 and 2"
        );
    }

    /// `send X to A` followed by `A handles X` is one thing said twice, and
    /// it doubles the depth of every tree it appears in.
    #[test]
    fn a_send_and_the_handling_it_caused_are_one_row() {
        let span = |id, parent, point| Span {
            id,
            parent,
            at: id * 10,
            took: None,
            point,
            thread: 0,
        };
        let send = |to: &str, what: &str| TracePoint::Send {
            actor: to.into(),
            message: what.into(),
        };
        let handle = |by: &str, what: &str| TracePoint::Handle {
            actor: by.into(),
            message: what.into(),
        };

        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                span(
                    1,
                    None,
                    TracePoint::Action {
                        message: "a::Kill".into(),
                    },
                ),
                span(2, Some(1), send("a::Worker", "a::Kill")),
                span(3, Some(2), handle("a::Worker", "a::Kill")),
                span(
                    4,
                    Some(3),
                    TracePoint::Push {
                        reducer: "a::Tabs".into(),
                    },
                ),
                // A send nobody handled: the actor was gone by then.
                span(5, Some(3), send("a::Gone", "a::Later")),
            ],
            ends: Vec::new(),
            dropped: 0,
        });

        let timers = Timers::default();
        let record = record(read(&log, &timers), 1, &Query::default()).expect("kept");

        let set_off: Vec<u64> = record.set_off.iter().map(|next| next.row.id).collect();
        assert_eq!(set_off, [3], "the send folded into the handling under it");
        assert_eq!(record.set_off[0].row.kind, Kind::Handle);

        let under: Vec<u64> = record.set_off[0]
            .children
            .iter()
            .map(|next| next.row.id)
            .collect();
        assert_eq!(
            under,
            [4, 5],
            "what the handling set off hangs where the send was"
        );
        assert_eq!(
            record.set_off[0].children[1].row.kind,
            Kind::Send,
            "a send nobody handled stays a send"
        );

        // And the same pair, read the other way, from the push.
        let pushed = super::record(read(&log, &timers), 4, &Query::default()).expect("kept");
        let steps: Vec<u64> = pushed
            .origins
            .steps
            .iter()
            .map(|step| step.row.id)
            .collect();
        assert_eq!(steps, [1, 3, 4], "the send is not a step of its own");
        assert!(
            pushed
                .origins
                .steps
                .iter()
                .all(|step| step.between.is_empty()),
            "and it does not come back as something that happened in between"
        );
    }

    #[test]
    fn a_loop_that_ran_long_lists_only_its_nearest_causes() {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: (1..=50_000)
                .map(|id| Span {
                    id,
                    parent: id.checked_sub(1).filter(|parent| *parent > 0),
                    at: id,
                    took: None,
                    point: TracePoint::Handle {
                        actor: "a::Worker".into(),
                        message: "a::Sweep".into(),
                    },
                    thread: 0,
                })
                .collect(),
            ends: Vec::new(),
            dropped: 0,
        });

        let timers = Timers::default();
        let record = record(read(&log, &timers), 50_000, &Query::default()).expect("kept");
        assert_eq!(record.origins.steps.len(), CHAIN_LIMIT);
        assert_eq!(
            record.origins.steps.last().map(|step| step.row.id),
            Some(50_000)
        );
        assert!(record.origins.note.is_some(), "says the chain goes further");

        fn depth(set_off: &[Consequence]) -> usize {
            set_off
                .iter()
                .map(|next| 1 + depth(&next.children))
                .max()
                .unwrap_or(0)
        }

        let first = super::record(read(&log, &timers), 1, &Query::default()).expect("kept");
        assert_eq!(
            depth(&first.set_off),
            DEPTH_LIMIT,
            "a loop does not nest past the limit"
        );
        assert!(first.cut, "and says it went further");
    }

    #[test]
    fn what_is_shown_follows_the_log_and_the_limit() {
        let mut log = log();
        let timers = Timers::default();
        let query = Query {
            hidden: vec!["tick".into()],
            ..Query::default()
        };
        let ids = |shown: &Shown, log: &TraceLog| -> Vec<u64> {
            shown
                .spans(log, 0..shown.len())
                .map(|span| span.id)
                .collect()
        };

        let mut shown = Shown::default();
        shown.refresh(read(&log, &timers), &query);
        assert_eq!(ids(&shown, &log), [1, 3, 4]);

        log.absorb(TraceBatch {
            spans: vec![Span {
                id: 5,
                parent: None,
                at: 50,
                took: None,
                point: TracePoint::Note {
                    text: "later".into(),
                },
                thread: 0,
            }],
            ends: Vec::new(),
            dropped: 0,
        });

        shown.refresh(read(&log, &timers), &query);
        assert_eq!(ids(&shown, &log), [1, 3, 4, 5]);
        assert_eq!(shown.position(4), Some(2));

        let frozen = Query {
            upto: Some(3),
            ..query.clone()
        };
        shown.refresh(read(&log, &timers), &frozen);
        assert_eq!(ids(&shown, &log), [1, 3]);

        shown.refresh(read(&log, &timers), &query);
        assert_eq!(ids(&shown, &log), [1, 3, 4, 5]);

        let other = Query {
            text: "later".into(),
            ..query
        };
        shown.refresh(read(&log, &timers), &other);
        assert_eq!(ids(&shown, &log), [5]);
    }

    #[test]
    fn a_record_that_arrives_late_is_shown_in_its_place() {
        let mut log = log();
        let timers = Timers::default();
        let note = |id: u64| Span {
            id,
            parent: None,
            at: id * 10,
            took: None,
            point: TracePoint::Note {
                text: "later".into(),
            },
            thread: 0,
        };
        let ids = |shown: &Shown, log: &TraceLog| -> Vec<u64> {
            shown
                .spans(log, 0..shown.len())
                .map(|span| span.id)
                .collect()
        };

        let mut shown = Shown::default();
        let frozen = Query {
            upto: Some(4),
            ..Query::default()
        };
        shown.refresh(read(&log, &timers), &frozen);
        assert_eq!(ids(&shown, &log), [1, 2, 3, 4]);

        log.absorb(TraceBatch {
            spans: vec![note(7)],
            ends: Vec::new(),
            dropped: 0,
        });
        shown.refresh(read(&log, &timers), &frozen);
        assert_eq!(ids(&shown, &log), [1, 2, 3, 4], "past the limit");

        log.absorb(TraceBatch {
            spans: vec![note(6), note(5)],
            ends: Vec::new(),
            dropped: 0,
        });
        shown.refresh(read(&log, &timers), &Query::default());
        assert_eq!(ids(&shown, &log), [1, 2, 3, 4, 5, 6, 7]);
    }

    fn at(id: u64, parent: Option<u64>, at: u64, took: Option<u64>, point: TracePoint) -> Span {
        Span {
            id,
            parent,
            at,
            took,
            point,
            thread: 0,
        }
    }

    fn handle(message: &str) -> TracePoint {
        TracePoint::Handle {
            actor: "a::Worker".into(),
            message: message.into(),
        }
    }

    fn logged(level: &str) -> TracePoint {
        TracePoint::Log {
            level: level.into(),
            target: "app".into(),
            text: "said".into(),
            written: None,
        }
    }

    fn spanned(level: &str) -> TracePoint {
        TracePoint::Span {
            name: "scan".into(),
            target: "app".into(),
            fields: String::new(),
            declared: None,
            level: level.into(),
        }
    }

    fn of(class: Class) -> Query {
        Query {
            class,
            ..Query::default()
        }
    }

    fn ids(reading: Reading, query: &Query) -> Vec<u64> {
        shown(reading, query).iter().map(|span| span.id).collect()
    }

    #[test]
    fn what_the_application_wrote_is_listed_apart_and_by_level() {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                at(
                    1,
                    None,
                    0,
                    None,
                    TracePoint::Action {
                        message: "a::Kill".into(),
                    },
                ),
                at(2, Some(1), 10, Some(5), handle("a::Kill")),
                at(3, Some(2), 11, None, logged("INFO")),
                at(4, Some(2), 12, Some(2), spanned("DEBUG")),
                at(5, None, 20, None, TracePoint::Tick { timer: None }),
                at(6, Some(5), 21, Some(1), spanned("")),
                at(7, Some(5), 22, None, logged("WARN")),
            ],
            ends: Vec::new(),
            dropped: 0,
        });
        let timers = Timers::default();
        let reading = read(&log, &timers);

        assert_eq!(ids(reading, &of(Class::Own)), [3, 4, 6, 7]);
        assert_eq!(
            ids(reading, &of(Class::Level(Level::Info))),
            [3, 6],
            "a span that did not say is info"
        );
        assert_eq!(ids(reading, &of(Class::Level(Level::Debug))), [4]);
        assert_eq!(ids(reading, &of(Class::Level(Level::Warn))), [7]);
        assert_eq!(ids(reading, &of(Class::Every)).len(), 7);
    }

    #[test]
    fn slow_is_the_outermost_framework_work_of_a_millisecond_or_more() {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                at(1, None, 0, Some(3_000), handle("a::Outer")),
                at(2, Some(1), 100, Some(1_500), handle("a::Inner")),
                at(
                    3,
                    Some(2),
                    200,
                    None,
                    TracePoint::Send {
                        actor: "a::Worker".into(),
                        message: "a::Later".into(),
                    },
                ),
                at(4, Some(3), 5_000, Some(1_200), handle("a::Later")),
                at(
                    5,
                    None,
                    6_000,
                    None,
                    TracePoint::Render {
                        segment: "Processes".into(),
                        took_us: 2_000,
                    },
                ),
                at(
                    6,
                    None,
                    6_500,
                    None,
                    TracePoint::Settled {
                        actor: "a::Worker".into(),
                        actor_id: 1,
                        output: "a::Scan".into(),
                        took_us: 90_000,
                    },
                ),
                at(7, None, 7_000, Some(5_000), spanned("INFO")),
                at(8, None, 9_000, Some(900), handle("a::Quick")),
            ],
            ends: Vec::new(),
            dropped: 0,
        });
        let timers = Timers::default();

        assert_eq!(
            ids(read(&log, &timers), &of(Class::Slow)),
            [1, 4, 5],
            "not what ran inside a slow one, not background work waited on, not the application's own, not under a millisecond"
        );
    }

    #[test]
    fn work_turns_slow_when_its_end_arrives_and_what_ran_inside_it_stops_being_listed() {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                at(
                    1,
                    None,
                    0,
                    None,
                    TracePoint::Publish {
                        event: "a::Changed".into(),
                        bus: guinea_devtools_protocol::BusKind::Global,
                        subscribers: 1,
                    },
                ),
                at(2, Some(1), 100, None, handle("a::Changed")),
            ],
            ends: vec![guinea_devtools_protocol::End { id: 2, took: 1_500 }],
            dropped: 0,
        });
        let timers = Timers::default();
        let query = of(Class::Slow);
        let listed = |shown: &Shown, log: &TraceLog| -> Vec<u64> {
            shown
                .spans(log, 0..shown.len())
                .map(|span| span.id)
                .collect()
        };

        let mut shown = Shown::default();
        shown.refresh(read(&log, &timers), &query);
        assert_eq!(listed(&shown, &log), [2]);

        log.absorb(TraceBatch {
            spans: Vec::new(),
            ends: vec![guinea_devtools_protocol::End { id: 1, took: 3_000 }],
            dropped: 0,
        });
        shown.refresh(read(&log, &timers), &query);
        assert_eq!(
            listed(&shown, &log),
            [1],
            "the handling is the publish's breakdown now"
        );
    }

    fn pulled() -> TracePoint {
        TracePoint::Pull {
            actor: "a::Agent".into(),
            actor_id: 1,
            output: "a::Streamed".into(),
            source: 9,
        }
    }

    #[test]
    fn a_source_making_its_next_item_is_not_listed_and_is_what_the_item_ran_in() {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                at(1, None, 0, Some(400_000), pulled()),
                at(
                    2,
                    Some(1),
                    399_000,
                    None,
                    TracePoint::Publish {
                        event: "a::Changed".into(),
                        bus: guinea_devtools_protocol::BusKind::Global,
                        subscribers: 1,
                    },
                ),
            ],
            ends: Vec::new(),
            dropped: 0,
        });
        let timers = Timers::default();
        let reading = read(&log, &timers);

        assert_eq!(
            ids(reading, &of(Class::Every)),
            [2],
            "every record but the pull"
        );
        assert!(
            ids(reading, &of(Class::Slow)).is_empty(),
            "a pull is mostly waiting"
        );

        let publish = about(reading, 2).expect("kept");
        let within: Vec<(u64, Kind)> = publish
            .within
            .iter()
            .map(|row| (row.id, row.kind))
            .collect();
        assert_eq!(within, [(1, Kind::Pull)]);
        assert_eq!(
            words::text(&publish.within[0].words),
            "Agent's source makes the next Streamed"
        );
    }

    #[test]
    fn about_a_record_says_what_it_ran_in_and_what_ran_inside_it() {
        let mut scan = spanned("DEBUG");
        if let TracePoint::Span {
            fields, declared, ..
        } = &mut scan
        {
            *fields = "rows=3".into();
            *declared = Some(Declared {
                file: "src/scan.rs".into(),
                line: 7,
                ..Declared::default()
            });
        }

        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                at(1, None, 0, None, TracePoint::Tick { timer: None }),
                at(2, Some(1), 10, Some(3_000), handle("a::Scan")),
                at(
                    3,
                    Some(2),
                    20,
                    None,
                    TracePoint::Send {
                        actor: "a::Worker".into(),
                        message: "a::Later".into(),
                    },
                ),
                at(4, Some(2), 30, Some(2_000), scan),
                at(5, Some(4), 40, None, logged("INFO")),
                at(
                    6,
                    Some(4),
                    50,
                    None,
                    TracePoint::Push {
                        reducer: "a::Rows".into(),
                    },
                ),
                at(7, Some(3), 5_000, Some(100), handle("a::Later")),
                at(
                    8,
                    Some(4),
                    60,
                    None,
                    TracePoint::Render {
                        segment: "Processes".into(),
                        took_us: 500,
                    },
                ),
            ],
            ends: Vec::new(),
            dropped: 0,
        });
        let timers = Timers::default();
        let reading = read(&log, &timers);
        let rows = |rows: &[Row]| -> Vec<u64> { rows.iter().map(|row| row.id).collect() };
        let tree = |inside: &[Consequence]| -> Vec<(u64, Vec<u64>)> {
            inside
                .iter()
                .map(|next| {
                    (
                        next.row.id,
                        next.children.iter().map(|child| child.row.id).collect(),
                    )
                })
                .collect()
        };

        let span = about(reading, 4).expect("kept");
        assert_eq!(span.level, Some(Level::Debug));
        assert_eq!(span.target.as_deref(), Some("app"));
        assert_eq!(span.fields.as_deref(), Some("rows=3"));
        assert_eq!(span.written.as_ref().map(|at| at.line), Some(7));
        assert_eq!(rows(&span.within), [1, 2]);
        assert_eq!(
            tree(&span.inside),
            [(5, vec![]), (8, vec![])],
            "its log line and the render that ran in it; a push took no time"
        );

        let handling = about(reading, 2).expect("kept");
        assert_eq!(handling.level, None);
        assert_eq!(rows(&handling.within), [1]);
        assert_eq!(
            tree(&handling.inside),
            [(4, vec![5, 8])],
            "the handling its send caused ran later, not inside"
        );

        let later = about(reading, 7).expect("kept");
        assert_eq!(rows(&later.within), [1, 2], "the send is its handling");

        let line = about(reading, 5).expect("kept");
        assert_eq!(rows(&line.within), [1, 2, 4]);
        assert!(line.inside.is_empty());
    }

    #[test]
    fn each_class_is_counted_as_the_trace_arrives() {
        let mut log = TraceLog::default();
        let timers = Timers::default();
        let mut classes = Classes::default();

        log.absorb(TraceBatch {
            spans: vec![
                at(1, None, 0, None, handle("a::Scan")),
                at(2, Some(1), 10, None, logged("WARN")),
                at(3, Some(1), 20, Some(5), spanned("")),
            ],
            ends: Vec::new(),
            dropped: 0,
        });
        classes.absorb(read(&log, &timers));
        assert_eq!(classes.count(Class::Own), 2);
        assert_eq!(classes.count(Class::Level(Level::Warn)), 1);
        assert_eq!(classes.count(Class::Level(Level::Info)), 1);
        assert_eq!(classes.count(Class::Slow), 0, "not ended yet");

        log.absorb(TraceBatch {
            spans: Vec::new(),
            ends: vec![guinea_devtools_protocol::End { id: 1, took: 4_000 }],
            dropped: 0,
        });
        classes.absorb(read(&log, &timers));
        assert_eq!(classes.count(Class::Slow), 1);
        assert_eq!(Classes::offered().count(), 7);
    }

    #[test]
    fn a_query_leaves_out_kinds_and_matches_whole_paths() {
        let log = log();
        let timers = Timers::default();
        let query = Query {
            class: Class::Every,
            hidden: vec!["tick".into()],
            text: "A::TABS".into(),
            upto: None,
        };

        let ids: Vec<u64> = shown(read(&log, &timers), &query)
            .iter()
            .map(|span| span.id)
            .collect();
        assert_eq!(ids, [4]);

        let as_shown = Query {
            text: "push into tabs".into(),
            ..query.clone()
        };
        assert_eq!(
            shown(read(&log, &timers), &as_shown).len(),
            1,
            "the text on screen finds it too"
        );

        let row = row(read(&log, &timers), log.get(4).expect("kept"));
        assert!(matches!(row.cause, Some(Cause::Known { id: 2, .. })));
    }
}
