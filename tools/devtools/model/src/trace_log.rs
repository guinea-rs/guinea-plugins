//! Every trace record a session sent, indexed both ways.

use std::collections::{HashMap, HashSet, VecDeque};

use guinea_devtools_protocol::{Span, TraceBatch};

/// How many records a session keeps before forgetting the oldest.
pub const KEPT: usize = 50_000;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TraceLog {
    spans: HashMap<u64, Span>,
    children: HashMap<u64, Vec<u64>>,
    order: VecDeque<u64>,
    pub dropped: u64,
}

impl TraceLog {
    pub fn absorb(&mut self, batch: TraceBatch) {
        self.dropped += batch.dropped;

        for span in batch.spans {
            if let Some(parent) = span.parent {
                self.children.entry(parent).or_default().push(span.id);
            }

            self.order.push_back(span.id);
            self.spans.insert(span.id, span);
        }

        for end in batch.ends {
            if let Some(span) = self.spans.get_mut(&end.id) {
                span.took = Some(end.took);
            }
        }

        while self.order.len() > KEPT {
            if let Some(old) = self.order.pop_front() {
                self.spans.remove(&old);
                self.children.remove(&old);
            }
        }
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

    /// The oldest record kept.
    pub fn first_id(&self) -> Option<u64> {
        self.order.front().copied()
    }

    /// The id of the record at `index`, oldest first.
    pub fn id_at(&self, index: usize) -> Option<u64> {
        self.order.get(index).copied()
    }

    /// What was recorded after `after`, oldest first.
    pub fn after(&self, after: u64) -> impl Iterator<Item = &Span> {
        let from = self.order.partition_point(|id| *id <= after);
        self.order.range(from..).filter_map(|id| self.spans.get(id))
    }

    /// What was recorded after `after` and before `before`, oldest first.
    ///
    /// Ids grow in the order an application records points, and records arrive
    /// in that order, so the kept ones are sorted by id.
    pub fn between(&self, after: u64, before: u64) -> impl Iterator<Item = &Span> {
        let from = self.order.partition_point(|id| *id <= after);
        let to = self.order.partition_point(|id| *id < before);

        self.order
            .range(from..to.max(from))
            .filter_map(|id| self.spans.get(id))
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
            spans: vec![span(1, None), span(2, Some(1)), span(3, Some(2)), span(4, Some(1))],
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
    }
}
