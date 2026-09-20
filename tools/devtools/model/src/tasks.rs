//! The background work an application is still waiting on, read off the
//! trace: a spawn with no end under it.
//!
//! A task sits where its actor sits, so what a segment holds is what the
//! actors of that segment hold.

use std::collections::BTreeMap;

use guinea_devtools_protocol::TracePoint;

use crate::trace_log::TraceLog;

/// A task that has neither settled nor been cancelled.
#[derive(Clone, Debug, PartialEq)]
pub struct Running {
    /// The record that started it, to link back to.
    pub record: u64,
    pub actor: String,
    /// The actor's id in the snapshot: what the task belongs to.
    pub actor_id: u64,
    /// What its result comes back as.
    pub output: String,
    /// Microseconds since the application started tracing.
    pub at: u64,
}

/// Every task started and not yet ended, oldest first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tasks {
    running: BTreeMap<u64, Running>,
    seen: u64,
}

impl Tasks {
    /// Reads whatever the log gained since the last call.
    pub fn absorb(&mut self, log: &TraceLog) {
        if let Some(first) = log.first_id() {
            self.running.retain(|record, _| *record >= first);
        }

        let fresh: Vec<_> = log.after(self.seen).collect();
        for span in fresh {
            self.seen = span.id;

            match &span.point {
                TracePoint::Spawn {
                    actor,
                    actor_id,
                    output,
                } => {
                    self.running.insert(
                        span.id,
                        Running {
                            record: span.id,
                            actor: actor.clone(),
                            actor_id: *actor_id,
                            output: output.clone(),
                            at: span.at,
                        },
                    );
                }
                TracePoint::Settled { .. } | TracePoint::Cancelled { .. } => {
                    if let Some(spawn) = span.parent {
                        self.running.remove(&spawn);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn len(&self) -> usize {
        self.running.len()
    }

    pub fn is_empty(&self) -> bool {
        self.running.is_empty()
    }

    /// All of them, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &Running> {
        self.running.values()
    }

    /// What one actor is waiting on.
    pub fn of_actor(&self, actor_id: u64) -> impl Iterator<Item = &Running> {
        self.iter().filter(move |task| task.actor_id == actor_id)
    }

    /// What the actors in `actors` are waiting on, together.
    pub fn of_actors<'a>(&'a self, actors: &'a [u64]) -> impl Iterator<Item = &'a Running> {
        self.iter().filter(move |task| actors.contains(&task.actor_id))
    }
}

#[cfg(test)]
mod tests {
    use guinea_devtools_protocol::{Span, TraceBatch};

    use super::*;

    fn span(id: u64, parent: Option<u64>, point: TracePoint) -> Span {
        Span {
            id,
            parent,
            at: id * 10,
            took: None,
            point,
        }
    }

    fn spawn(actor_id: u64, output: &str) -> TracePoint {
        TracePoint::Spawn {
            actor: "Poller".to_string(),
            actor_id,
            output: output.to_string(),
        }
    }

    fn log(spans: Vec<Span>) -> TraceLog {
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans,
            ..TraceBatch::default()
        });
        log
    }

    #[test]
    fn a_spawn_with_no_end_under_it_is_still_running() {
        let log = log(vec![
            span(1, None, spawn(7, "Tick")),
            span(2, None, spawn(7, "Fetched")),
            span(
                3,
                Some(1),
                TracePoint::Settled {
                    actor: "Poller".to_string(),
                    actor_id: 7,
                    output: "Tick".to_string(),
                    took_us: 500,
                },
            ),
        ]);

        let mut tasks = Tasks::default();
        tasks.absorb(&log);

        let waiting: Vec<&str> = tasks.iter().map(|task| task.output.as_str()).collect();
        assert_eq!(waiting, ["Fetched"]);
    }

    #[test]
    fn a_cancelled_task_stops_being_held() {
        let log = log(vec![
            span(1, None, spawn(7, "Tick")),
            span(
                2,
                Some(1),
                TracePoint::Cancelled {
                    actor: "Poller".to_string(),
                    actor_id: 7,
                    output: "Tick".to_string(),
                    took_us: 3,
                },
            ),
        ]);

        let mut tasks = Tasks::default();
        tasks.absorb(&log);

        assert!(tasks.is_empty());
    }

    #[test]
    fn tasks_are_kept_by_the_actor_that_started_them() {
        let log = log(vec![
            span(1, None, spawn(7, "Tick")),
            span(2, None, spawn(9, "Fetched")),
        ]);

        let mut tasks = Tasks::default();
        tasks.absorb(&log);

        assert_eq!(tasks.of_actor(9).count(), 1);
        assert_eq!(tasks.of_actors(&[7, 9]).count(), 2);
    }
}
