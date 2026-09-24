//! A session's trace, as it is read: filtered, as rows, and one record with
//! where it came from and what it set off.

use std::collections::VecDeque;

use guinea_devtools_protocol::{Span, TracePoint};
use serde::{Deserialize, Serialize};

use crate::clock::Clock;
use crate::names::took;
use crate::timers::Timers;
use crate::trace_log::TraceLog;
use crate::words::{self, Kind, Tone, Word};

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

/// Which records to show.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Query {
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

    /// By kind, then by the text: as shown, or with whole type paths.
    pub fn matches(&self, span: &Span, timers: &Timers) -> bool {
        if !self.shows(span.point.kind()) {
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

    shown.ids.iter().filter_map(|id| reading.log.get(*id)).collect()
}

/// The records a query lets through, kept by id and brought up to date by
/// looking only at what arrived since, so a long log is not read again for
/// every frame.
#[derive(Clone, Debug, Default)]
pub struct Shown {
    /// The query it holds records for, without its limit.
    query: Option<Query>,
    ids: VecDeque<u64>,
    /// The newest record looked at.
    seen: u64,
}

impl Shown {
    pub fn refresh(&mut self, reading: Reading, query: &Query) {
        let log = reading.log;
        let criteria = Query {
            upto: None,
            ..query.clone()
        };
        if self.query.as_ref() != Some(&criteria) {
            *self = Shown {
                query: Some(criteria.clone()),
                ..Shown::default()
            };
        }

        let limit = match query.upto {
            None => u64::MAX,
            Some(0) => 0,
            Some(count) => log.id_at(count - 1).unwrap_or(u64::MAX),
        };
        while self.ids.back().is_some_and(|id| *id > limit) {
            self.ids.pop_back();
        }
        self.seen = self.seen.min(limit);

        if let Some(first) = log.first_id() {
            while self.ids.front().is_some_and(|id| *id < first) {
                self.ids.pop_front();
            }
        }

        for span in log.after(self.seen) {
            if span.id > limit {
                break;
            }
            if criteria.matches(span, reading.timers) {
                self.ids.push_back(span.id);
            }
            self.seen = span.id;
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

/// What caused a record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "cause", rename_all = "snake_case")]
pub enum Cause {
    Known { id: u64, gist: String, tone: Tone },
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
        Some(format!("the {CHAIN_LIMIT} nearest causes; the chain goes back further"))
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
    })
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
        };

        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                span(1, None, TracePoint::Action { message: "a::Kill".into() }),
                span(2, Some(1), TracePoint::Tick { timer: None }),
                span(3, None, TracePoint::Note { text: "else".into() }),
                span(4, Some(2), TracePoint::Push { reducer: "a::Tabs".into() }),
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

        let steps: Vec<u64> = record.origins.steps.iter().map(|step| step.row.id).collect();
        assert_eq!(steps, [1, 2, 4]);

        let between: Vec<u64> = record.origins.steps[1].between.iter().map(|row| row.id).collect();
        assert_eq!(between, [3]);
        assert!(record.origins.steps[0].between.is_empty(), "nothing between 1 and 2");
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
                span(1, None, TracePoint::Action { message: "a::Kill".into() }),
                span(2, Some(1), send("a::Worker", "a::Kill")),
                span(3, Some(2), handle("a::Worker", "a::Kill")),
                span(4, Some(3), TracePoint::Push { reducer: "a::Tabs".into() }),
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
        assert_eq!(under, [4, 5], "what the handling set off hangs where the send was");
        assert_eq!(
            record.set_off[0].children[1].row.kind,
            Kind::Send,
            "a send nobody handled stays a send"
        );

        // And the same pair, read the other way, from the push.
        let pushed = super::record(read(&log, &timers), 4, &Query::default()).expect("kept");
        let steps: Vec<u64> = pushed.origins.steps.iter().map(|step| step.row.id).collect();
        assert_eq!(steps, [1, 3, 4], "the send is not a step of its own");
        assert!(
            pushed.origins.steps.iter().all(|step| step.between.is_empty()),
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
                })
                .collect(),
            ends: Vec::new(),
            dropped: 0,
        });

        let timers = Timers::default();
        let record = record(read(&log, &timers), 50_000, &Query::default()).expect("kept");
        assert_eq!(record.origins.steps.len(), CHAIN_LIMIT);
        assert_eq!(record.origins.steps.last().map(|step| step.row.id), Some(50_000));
        assert!(record.origins.note.is_some(), "says the chain goes further");

        fn depth(set_off: &[Consequence]) -> usize {
            set_off.iter().map(|next| 1 + depth(&next.children)).max().unwrap_or(0)
        }

        let first = super::record(read(&log, &timers), 1, &Query::default()).expect("kept");
        assert_eq!(depth(&first.set_off), DEPTH_LIMIT, "a loop does not nest past the limit");
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
            shown.spans(log, 0..shown.len()).map(|span| span.id).collect()
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
                point: TracePoint::Note { text: "later".into() },
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
    fn a_query_leaves_out_kinds_and_matches_whole_paths() {
        let log = log();
        let timers = Timers::default();
        let query = Query {
            hidden: vec!["tick".into()],
            text: "A::TABS".into(),
            upto: None,
        };

        let ids: Vec<u64> = shown(read(&log, &timers), &query).iter().map(|span| span.id).collect();
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
