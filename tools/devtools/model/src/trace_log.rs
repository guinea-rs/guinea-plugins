//! Every trace record a session sent, indexed both ways.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use guinea_devtools_protocol::{Span, TraceBatch, TracePoint};

/// How many records a session keeps before forgetting the oldest: enough
/// that the profiler's last minutes still have what ran in them.
pub const KEPT: usize = 500_000;

/// Ids grow in the order an application creates its points, but a point
/// made off the UI thread is recorded when it gets there, after points made
/// since - so records mostly arrive in id order, not always. They are kept
/// sorted by id, and a reader that takes in what is new asks by arrival.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TraceLog {
    spans: HashMap<u64, Span>,
    children: HashMap<u64, Vec<u64>>,
    /// By id.
    order: VecDeque<u64>,
    /// By when they happened, then id.
    by_time: BTreeSet<(u64, u64)>,
    /// As they arrived; the last [`KEPT`] of them.
    arrived: VecDeque<u64>,
    /// How many ever arrived.
    arrivals: u64,
    /// Records whose end arrived, in that order; the last [`KEPT`] of them.
    ended: VecDeque<u64>,
    /// How many ends of kept records ever arrived.
    endings: u64,
    pub dropped: u64,
}

impl TraceLog {
    pub fn absorb(&mut self, batch: TraceBatch) {
        self.dropped += batch.dropped;

        for span in batch.spans {
            if let Some(parent) = span.parent {
                insert_sorted(self.children.entry(parent).or_default(), span.id);
            }

            if self.order.back().is_some_and(|last| *last > span.id) {
                let at = self.order.partition_point(|id| *id < span.id);
                self.order.insert(at, span.id);
            } else {
                self.order.push_back(span.id);
            }
            self.arrived.push_back(span.id);
            self.arrivals += 1;
            self.by_time.insert((span.at, span.id));
            if let Some(old) = self.spans.insert(span.id, span)
                && self.spans.get(&old.id).is_none_or(|new| new.at != old.at)
            {
                self.by_time.remove(&(old.at, old.id));
            }
        }

        for end in batch.ends {
            if let Some(span) = self.spans.get_mut(&end.id) {
                span.took = Some(end.took);
                self.ended.push_back(end.id);
                self.endings += 1;
            }
        }

        while self.order.len() > KEPT {
            if let Some(old) = self.order.pop_front() {
                if let Some(span) = self.spans.remove(&old) {
                    self.by_time.remove(&(span.at, span.id));
                }
                self.children.remove(&old);
            }
        }
        while self.arrived.len() > KEPT {
            self.arrived.pop_front();
        }
        while self.ended.len() > KEPT {
            self.ended.pop_front();
        }
    }

    /// How many records ever arrived: where a reader that has taken in
    /// everything so far stands.
    pub fn arrivals(&self) -> u64 {
        self.arrivals
    }

    /// How many ends ever arrived: where a reader of [`ended_since`]
    /// (Self::ended_since) that has taken in every end stands.
    pub fn endings(&self) -> u64 {
        self.endings
    }

    /// The records whose end arrived after the reader stood at `cursor`, in
    /// the order the ends arrived: what they took is known only now.
    pub fn ended_since(&self, cursor: u64) -> impl Iterator<Item = &Span> {
        let first = self.endings - self.ended.len() as u64;
        let skip = (cursor.saturating_sub(first) as usize).min(self.ended.len());
        self.ended.range(skip..).filter_map(|id| self.spans.get(id))
    }

    /// What arrived after the reader stood at `cursor`, in the order it
    /// arrived.
    pub fn since(&self, cursor: u64) -> impl Iterator<Item = &Span> {
        let first = self.arrivals - self.arrived.len() as u64;
        let skip = (cursor.saturating_sub(first) as usize).min(self.arrived.len());
        self.arrived
            .range(skip..)
            .filter_map(|id| self.spans.get(id))
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn get(&self, id: u64) -> Option<&Span> {
        self.spans.get(&id)
    }

    /// Oldest first.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Span> {
        self.order.iter().filter_map(|id| self.spans.get(id))
    }

    /// What `id` set off directly, in the order it happened.
    pub fn children(&self, id: u64) -> impl Iterator<Item = &Span> {
        self.children
            .get(&id)
            .into_iter()
            .flatten()
            .filter_map(|child| self.spans.get(child))
    }

    /// Everything `cause` set off, however far down, depth first and each
    /// with how far below `cause` it is. Each record once: a peer that names
    /// a record its own ancestor does not send this round in circles.
    pub fn under(&self, cause: u64) -> Vec<(usize, &Span)> {
        let mut unseen: Vec<(usize, u64)> = self
            .children
            .get(&cause)
            .into_iter()
            .flatten()
            .rev()
            .map(|child| (1, *child))
            .collect();

        let mut seen = HashSet::from([cause]);
        let mut found = Vec::new();
        while let Some((depth, id)) = unseen.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(span) = self.spans.get(&id) else {
                continue;
            };
            found.push((depth, span));
            if let Some(children) = self.children.get(&id) {
                unseen.extend(children.iter().rev().map(|child| (depth + 1, *child)));
            }
        }
        found
    }

    /// Background work under `cause` that has neither settled nor been
    /// cancelled yet.
    pub fn unfinished(&self, cause: u64) -> usize {
        self.under(cause)
            .into_iter()
            .filter(|(_, span)| matches!(span.point, TracePoint::Spawn { .. }))
            .filter(|(_, span)| {
                !self.children(span.id).any(|child| {
                    matches!(
                        child.point,
                        TracePoint::Settled { .. } | TracePoint::Cancelled { .. }
                    )
                })
            })
            .count()
    }

    /// The oldest record kept.
    pub fn first_id(&self) -> Option<u64> {
        self.order.front().copied()
    }

    /// The id of the record at `index`, oldest first.
    pub fn id_at(&self, index: usize) -> Option<u64> {
        self.order.get(index).copied()
    }

    /// What was recorded after `after` and before `before`, oldest first.
    pub fn between(&self, after: u64, before: u64) -> impl Iterator<Item = &Span> {
        let from = self.order.partition_point(|id| *id <= after);
        let to = self.order.partition_point(|id| *id < before);

        self.order
            .range(from..to.max(from))
            .filter_map(|id| self.spans.get(id))
    }

    /// What was recorded from the trace's `from` microsecond to its `to`,
    /// both included, in the order it happened.
    pub fn during(&self, from: u64, to: u64) -> impl Iterator<Item = &Span> {
        let range = (from <= to).then_some((from, 0)..=(to, u64::MAX));
        range
            .into_iter()
            .flat_map(|range| self.by_time.range(range))
            .filter_map(|(_, id)| self.spans.get(id))
    }

    /// From the first known cause down to `id`, inclusive, but no more than
    /// `limit` records; and whether older causes were left out.
    ///
    /// A loop that sets itself off again is one chain as long as the loop has
    /// run, so the nearest causes are the ones kept.
    pub fn provenance(&self, id: u64, limit: usize) -> (Vec<&Span>, bool) {
        let mut chain = Vec::new();
        let mut seen = HashSet::new();
        let mut next = self.spans.get(&id);

        while let Some(span) = next {
            if !seen.insert(span.id) {
                break;
            }
            if chain.len() == limit {
                chain.reverse();
                return (chain, true);
            }

            chain.push(span);
            next = span.parent.and_then(|parent| self.spans.get(&parent));
        }

        chain.reverse();
        (chain, false)
    }
}

fn insert_sorted(ids: &mut Vec<u64>, id: u64) {
    let at = ids.partition_point(|known| *known < id);
    ids.insert(at, id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea_devtools_protocol::{End, TracePoint};

    fn span(id: u64, parent: Option<u64>) -> Span {
        Span {
            id,
            parent,
            at: id * 10,
            took: None,
            point: TracePoint::Tick { timer: None },
        }
    }

    fn log() -> TraceLog {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                span(1, None),
                span(2, Some(1)),
                span(3, Some(2)),
                span(4, Some(1)),
            ],
            ends: Vec::new(),
            dropped: 0,
        });

        log
    }

    #[test]
    fn a_record_knows_where_it_came_from_and_what_it_set_off() {
        let log = log();

        let (chain, cut) = log.provenance(3, 10);
        let chain: Vec<u64> = chain.iter().map(|s| s.id).collect();
        assert_eq!(chain, [1, 2, 3]);
        assert!(!cut);

        let (nearest, cut) = log.provenance(3, 2);
        let nearest: Vec<u64> = nearest.iter().map(|s| s.id).collect();
        assert_eq!(nearest, [2, 3]);
        assert!(cut);

        let children: Vec<u64> = log.children(1).map(|s| s.id).collect();
        assert_eq!(children, [2, 4]);
        assert_eq!(log.children(3).count(), 0);
    }

    #[test]
    fn records_that_name_each_other_as_parents_are_listed_once() {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![span(1, Some(2)), span(2, Some(1)), span(3, Some(3))],
            ends: Vec::new(),
            dropped: 0,
        });

        let under: Vec<u64> = log.under(1).iter().map(|(_, s)| s.id).collect();
        assert_eq!(under, [2]);
        assert!(log.under(3).is_empty(), "its own child is not under it");
    }

    #[test]
    fn what_happened_between_two_records_is_everything_recorded_in_between() {
        let log = log();

        let between: Vec<u64> = log.between(1, 4).map(|s| s.id).collect();
        assert_eq!(between, [2, 3]);
        assert_eq!(log.between(3, 2).count(), 0);
        assert_eq!(log.between(4, 9).count(), 0);
    }

    #[test]
    fn an_end_that_arrives_later_fills_in_the_duration() {
        let mut log = log();
        log.absorb(TraceBatch {
            spans: Vec::new(),
            ends: vec![End { id: 2, took: 77 }],
            dropped: 3,
        });

        assert_eq!(log.get(2).and_then(|s| s.took), Some(77));
        assert_eq!(log.dropped, 3);
    }

    #[test]
    fn a_reader_learns_which_records_ended_since_it_last_looked() {
        let mut log = log();
        let cursor = log.endings();
        log.absorb(TraceBatch {
            spans: Vec::new(),
            ends: vec![
                End { id: 3, took: 5 },
                End { id: 1, took: 9 },
                End { id: 42, took: 1 },
            ],
            dropped: 0,
        });

        let ended: Vec<u64> = log.ended_since(cursor).map(|s| s.id).collect();
        assert_eq!(ended, [3, 1], "an end for a record never kept is not one");
        assert_eq!(log.ended_since(log.endings()).count(), 0);
    }

    #[test]
    fn a_record_that_arrives_late_takes_its_place_by_id() {
        let mut log = log();
        let cursor = log.arrivals();
        log.absorb(TraceBatch {
            spans: vec![span(7, Some(1)), span(5, Some(1)), span(6, Some(5))],
            ends: Vec::new(),
            dropped: 0,
        });

        let kept: Vec<u64> = log.iter().map(|s| s.id).collect();
        assert_eq!(kept, [1, 2, 3, 4, 5, 6, 7]);
        let children: Vec<u64> = log.children(1).map(|s| s.id).collect();
        assert_eq!(children, [2, 4, 5, 7]);
        let between: Vec<u64> = log.between(4, 7).map(|s| s.id).collect();
        assert_eq!(between, [5, 6]);

        let fresh: Vec<u64> = log.since(cursor).map(|s| s.id).collect();
        assert_eq!(fresh, [7, 5, 6]);
        assert_eq!(log.since(log.arrivals()).count(), 0);
        assert_eq!(log.since(0).count(), 7);
    }

    #[test]
    fn the_oldest_are_forgotten_past_the_limit() {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: (1..=(KEPT as u64 + 2)).map(|id| span(id, None)).collect(),
            ends: Vec::new(),
            dropped: 0,
        });

        assert_eq!(log.len(), KEPT);
        assert!(log.get(1).is_none());
        assert!(log.get(KEPT as u64 + 2).is_some());
        assert_eq!(log.during(0, u64::MAX).count(), KEPT);
        assert_eq!(log.during(0, 29).count(), 0, "1 and 2 are forgotten");
    }

    #[test]
    fn records_are_found_by_when_they_happened_whatever_order_they_came_in() {
        let mut log = log();
        log.absorb(TraceBatch {
            spans: vec![
                Span {
                    at: 15,
                    ..span(9, Some(1))
                },
                span(5, None),
            ],
            ends: Vec::new(),
            dropped: 0,
        });

        let during: Vec<u64> = log.during(11, 40).map(|s| s.id).collect();
        assert_eq!(during, [9, 2, 3, 4], "from 11 to 40, by time");
        let at: Vec<u64> = log.during(50, 50).map(|s| s.id).collect();
        assert_eq!(at, [5]);
        assert_eq!(log.during(40, 10).count(), 0);
    }
}
