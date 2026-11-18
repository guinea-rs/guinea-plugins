//! Report-only source filtering, independent of topology and Perfetto.
use guinea::observability::Event;
use guinea::observability::trace::{Cause, Point, Trace};
use std::collections::{BTreeMap, BTreeSet};

const LIMIT: usize = 16_384;
#[derive(Clone, Copy)]
struct Seen {
    hidden: bool,
    parent: Option<Cause>,
}
pub(crate) struct Filter {
    names: BTreeSet<String>,
    sources: BTreeMap<u64, bool>,
    seen: BTreeMap<u64, Seen>,
}
impl Filter {
    pub fn new(names: Vec<String>) -> Self {
        Self {
            names: names.into_iter().collect(),
            sources: BTreeMap::new(),
            seen: BTreeMap::new(),
        }
    }
    pub fn filter(
        &mut self,
        events: &[Event],
        mut records: Vec<(u32, Trace)>,
    ) -> Vec<(u32, Trace)> {
        if self.names.is_empty() {
            return records;
        }
        // All metadata precedes classification, including a source closed in this batch.
        for event in events {
            match event {
                Event::SourceOpened(info) => {
                    self.sources.insert(
                        info.id,
                        info.name.is_some_and(|name| self.names.contains(name)),
                    );
                }
                Event::SourceNamed { id, name } => {
                    self.sources.insert(*id, self.names.contains(*name));
                }
                _ => {}
            }
        }
        // Causes are allocated before their children. Threads may deliver them out of order.
        records.sort_by_key(|(_, trace)| match trace {
            Trace::Begin(r) | Trace::Mark(r) => (0, r.id.get()),
            Trace::End { id, .. } => (1, id.get()),
        });
        let mut kept = Vec::new();
        for (thread, trace) in records {
            let begin = matches!(trace, Trace::Begin(_));
            match trace {
                Trace::Begin(mut r) | Trace::Mark(mut r) => {
                    let inherited = r.parent.and_then(|p| self.seen.get(&p.get()).copied());
                    let ignored = match &r.point {
                        Point::Arrived { source, name, .. } => {
                            name.is_some_and(|n| self.names.contains(n))
                                || self.sources.get(source) == Some(&true)
                        }
                        Point::Pull { source, .. }
                        | Point::Source { source, .. }
                        | Point::Closed { source, .. } => self.sources.get(source) == Some(&true),
                        _ => false,
                    };
                    // Explicit application diagnostics and actions form visible branches.
                    let explicit = matches!(
                        r.point,
                        Point::Action { .. } | Point::Log { .. } | Point::Span { .. }
                    );
                    let hidden = !explicit && (ignored || inherited.is_some_and(|p| p.hidden));
                    if inherited.is_some_and(|p| p.hidden) {
                        r.parent = inherited.and_then(|p| p.parent);
                    }
                    self.seen.insert(
                        r.id.get(),
                        Seen {
                            hidden,
                            parent: if hidden { r.parent } else { Some(r.id) },
                        },
                    );
                    if !hidden {
                        kept.push((
                            thread,
                            if begin {
                                Trace::Begin(r)
                            } else {
                                Trace::Mark(r)
                            },
                        ));
                    }
                }
                Trace::End { id, .. } => {
                    if self.seen.get(&id.get()).is_none_or(|r| !r.hidden) {
                        kept.push((thread, trace));
                    }
                }
            }
        }
        for event in events {
            if let Event::SourceClosed { id } = event {
                self.sources.remove(id);
            }
        }
        while self.sources.len() > LIMIT {
            self.sources.pop_first();
        }
        while self.seen.len() > LIMIT {
            self.seen.pop_first();
        }
        kept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea::observability::trace::{self, CodeLocation, Record};
    use std::time::Duration;
    fn record(parent: Option<Cause>, point: Point) -> Record {
        Record {
            id: trace::mark(|| Point::Action { message: "test" }),
            parent,
            at: Duration::ZERO,
            point,
            location: CodeLocation::caller(),
        }
    }
    #[test]
    fn named_tick_hides_descendants_keeps_actions_and_ends_across_batches() {
        let mut filter = Filter::new(vec!["refresh".into()]);
        let root = record(
            None,
            Point::Pull {
                source: 7,
                to: "A",
                output: "Tick",
            },
        );
        let child = record(Some(root.id), Point::Push { reducer: "State" });
        let action = record(Some(child.id), Point::Action { message: "Click" });
        let independent = record(None, Point::Action { message: "Other" });
        let hidden_id = root.id;
        let visible_id = action.id;
        let output = filter.filter(
            &[
                Event::SourceNamed {
                    id: 7,
                    name: "refresh",
                },
                Event::SourceClosed { id: 7 },
            ],
            vec![
                (1, Trace::Begin(child)),
                (1, Trace::Begin(root)),
                (1, Trace::Begin(action)),
                (2, Trace::Mark(independent)),
            ],
        );
        assert_eq!(output.len(), 2);
        assert!(
            matches!(&output[0].1, Trace::Begin(r) if r.id == visible_id && r.parent.is_none())
        );
        let ends = filter.filter(
            &[],
            vec![
                (
                    1,
                    Trace::End {
                        id: hidden_id,
                        took: Duration::ZERO,
                    },
                ),
                (
                    1,
                    Trace::End {
                        id: visible_id,
                        took: Duration::ZERO,
                    },
                ),
            ],
        );
        assert_eq!(ends.len(), 1);
        assert!(matches!(ends[0].1, Trace::End { id, .. } if id == visible_id));
        assert!(filter.sources.is_empty());
    }
    #[test]
    fn application_errors_survive_hidden_ancestry_and_reconnect_resets_state() {
        let mut filter = Filter::new(vec!["refresh".into()]);
        let root = record(
            None,
            Point::Arrived {
                source: 9,
                name: Some("refresh"),
                to: "A",
                output: "Tick",
            },
        );
        let error = record(
            Some(root.id),
            Point::Log {
                level: tracing::Level::ERROR,
                target: "app",
                file: None,
                line: None,
                module: None,
                text: "failed".into(),
            },
        );
        let records = vec![(1, Trace::Mark(root)), (1, Trace::Mark(error))];
        assert_eq!(
            Filter::new(Vec::new()).filter(&[], records.clone()),
            records
        );
        let kept = filter.filter(&[], records);
        assert_eq!(kept.len(), 1);
        assert!(
            matches!(&kept[0].1, Trace::Mark(r) if r.parent.is_none() && matches!(r.point, Point::Log { .. }))
        );
        let fresh = Filter::new(vec!["refresh".into()]);
        assert!(fresh.sources.is_empty() && fresh.seen.is_empty());
    }

    #[test]
    fn initial_source_metadata_filters_a_pull_without_an_arrival() {
        let tree = guinea::__private::core::scope::ScopeTree::new();
        let info = guinea::sources::SourceInfo {
            id: 19,
            name: Some("refresh"),
            place: std::panic::Location::caller(),
            feature: None,
            scope: tree.scope().id(),
            to: "A",
            output: "Tick",
        };
        let mut filter = Filter::new(vec!["refresh".into()]);
        filter.filter(&[Event::SourceOpened(info)], Vec::new());
        let pull = record(
            None,
            Point::Pull {
                source: 19,
                to: "A",
                output: "Tick",
            },
        );
        assert!(filter.filter(&[], vec![(1, Trace::Begin(pull))]).is_empty());
    }

    #[test]
    fn evicted_visible_start_still_receives_its_end() {
        let mut filter = Filter::new(vec!["refresh".into()]);
        let start = record(None, Point::Action { message: "Long" });
        let id = start.id;
        assert_eq!(filter.filter(&[], vec![(1, Trace::Begin(start))]).len(), 1);
        let noise = (0..LIMIT + 1)
            .map(|_| {
                (
                    1,
                    Trace::Mark(record(
                        None,
                        Point::Arrived {
                            source: 0,
                            name: Some("refresh"),
                            to: "A",
                            output: "Tick",
                        },
                    )),
                )
            })
            .collect();
        filter.filter(&[], noise);
        assert!(!filter.seen.contains_key(&id.get()));
        assert_eq!(
            filter
                .filter(
                    &[],
                    vec![(
                        1,
                        Trace::End {
                            id,
                            took: Duration::from_secs(5)
                        }
                    )]
                )
                .len(),
            1
        );
    }

    #[test]
    fn correlation_storage_is_bounded() {
        let mut filter = Filter::new(vec!["refresh".into()]);
        let events = (0..LIMIT as u64 + 10)
            .map(|id| Event::SourceNamed {
                id,
                name: "refresh",
            })
            .collect::<Vec<_>>();
        let records = (0..LIMIT + 10)
            .map(|_| {
                (
                    1,
                    Trace::Mark(record(
                        None,
                        Point::Arrived {
                            source: 0,
                            name: Some("refresh"),
                            to: "A",
                            output: "Tick",
                        },
                    )),
                )
            })
            .collect();
        assert!(filter.filter(&events, records).is_empty());
        assert_eq!(filter.seen.len(), LIMIT);
        assert_eq!(filter.sources.len(), LIMIT);
    }

    #[test]
    fn arrival_name_works_without_metadata_and_unknown_causes_fail_open() {
        let mut filter = Filter::new(vec!["refresh".into()]);
        let root = record(
            None,
            Point::Arrived {
                source: 9,
                name: Some("refresh"),
                to: "A",
                output: "Tick",
            },
        );
        let id = root.id;
        assert!(filter.filter(&[], vec![(1, Trace::Mark(root))]).is_empty());
        filter.seen.clear();
        let unknown = record(Some(id), Point::Push { reducer: "Visible" });
        assert_eq!(filter.filter(&[], vec![(1, Trace::Mark(unknown))]).len(), 1);
    }
}
