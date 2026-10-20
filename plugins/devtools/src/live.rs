use crate::collect::{Traces, Tree};
use crate::link::Outbox;
use guinea::observability::{Observed, Watching};
use guinea_devtools_protocol::{AppInfo, Changes, Report, Snapshot};
use std::cell::RefCell;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

thread_local! {
    static LINK: RefCell<Option<Link>> = const { RefCell::new(None) };
}

struct Link {
    outbox: Outbox,
    info: Arc<Mutex<AppInfo>>,
    observed: Observed,
    started: Instant,
    wait: Duration,
    reread: Duration,
    scheduler: mpsc::Sender<(u64, Instant)>,
    runtime: Option<tokio::runtime::Handle>,
    connection: Option<Connection>,
}

struct Connection {
    generation: u64,
    watching: Watching,
    tree: Tree,
    previous: Option<Snapshot>,
    read_at: Option<Instant>,
    dirty: bool,
    takes_changes: bool,
    lost: u64,
}

pub fn start(
    outbox: Outbox,
    info: Arc<Mutex<AppInfo>>,
    observed: Observed,
    wait: Duration,
    reread: Duration,
) {
    let (scheduler, pending) = mpsc::channel::<(u64, Instant)>();
    std::thread::spawn(move || {
        let mut latest = 0;
        while let Ok((mut generation, mut due)) = pending.recv() {
            if generation < latest {
                continue;
            }
            latest = generation;
            loop {
                match pending.recv_timeout(due.saturating_duration_since(Instant::now())) {
                    Ok((next, at)) => {
                        if next > generation {
                            generation = next;
                            latest = next;
                            due = at;
                        } else if next == generation {
                            due = due.min(at);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        guinea::ui_thread::invoke_on_ui(move || flush(generation));
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
        }
    });
    LINK.set(Some(Link {
        outbox,
        info,
        observed,
        started: Instant::now(),
        wait,
        reread,
        scheduler,
        runtime: tokio::runtime::Handle::try_current().ok(),
        connection: None,
    }));
}

pub fn stop() {
    LINK.take();
    crate::profiler::stop();
}

pub fn is_started() -> bool {
    LINK.with_borrow(Option::is_some)
}

pub fn observed() -> Option<Observed> {
    LINK.with_borrow(|link| link.as_ref().map(|link| link.observed))
}

pub fn connected(generation: u64, takes_changes: bool) {
    LINK.with_borrow_mut(|link| {
        let Some(link) = link.as_mut() else {
            return;
        };
        link.connection = None;
        let scheduler = link.scheduler.clone();
        let wait = link.wait;
        let watching = match link.observed.watch(move || {
            let _ = scheduler.send((generation, Instant::now() + wait));
        }) {
            Ok(watching) => watching,
            Err(error) => {
                tracing::warn!(%error, "cannot observe the application");
                return;
            }
        };
        link.connection = Some(Connection {
            generation,
            watching,
            tree: Tree::default(),
            previous: None,
            read_at: None,
            dirty: true,
            takes_changes,
            lost: 0,
        });
    });
    flush(generation);
}

pub fn disconnected(generation: u64) {
    LINK.with_borrow_mut(|link| {
        if let Some(link) = link.as_mut() {
            link.connection
                .take_if(|connection| connection.generation == generation);
        }
    });
}

fn delta(previous: &Snapshot, current: &Snapshot) -> Changes {
    Changes {
        at: current.at,
        actors: current
            .actors
            .iter()
            .filter(|actor| !previous.actors.contains(actor))
            .cloned()
            .collect(),
        actors_gone: previous
            .actors
            .iter()
            .filter(|actor| !current.actors.iter().any(|known| known.id == actor.id))
            .map(|actor| actor.id)
            .collect(),
        roots: current
            .roots
            .iter()
            .filter(|root| !previous.roots.contains(root))
            .cloned()
            .collect(),
        roots_gone: previous
            .roots
            .iter()
            .filter(|root| !current.roots.iter().any(|known| known.id == root.id))
            .map(|root| root.id)
            .collect(),
        panels: (previous.panels != current.panels).then(|| current.panels.clone()),
        global_bus: (previous.global_bus != current.global_bus).then(|| current.global_bus.clone()),
        timers: (previous.timers != current.timers).then(|| current.timers.clone()),
    }
}

fn flush(generation: u64) {
    LINK.with_borrow_mut(|link| {
        let Some(link) = link.as_mut() else {
            return;
        };
        let Some(connection) = link
            .connection
            .as_mut()
            .filter(|connection| connection.generation == generation)
        else {
            return;
        };
        let batch = connection.watching.take();
        let mut traces = Traces::default();
        traces.lose(batch.dropped + std::mem::take(&mut connection.lost));
        for (thread, record) in batch.trace {
            traces.push_on(thread, &record);
        }
        connection.dirty |= !batch.events.is_empty() || !traces.is_empty();
        for event in batch.events {
            connection.tree.apply(event);
        }
        {
            let mut info = link.info.lock().unwrap_or_else(|error| error.into_inner());
            info.backend = connection.tree.backend.clone();
            info.plugins = connection.tree.plugins.clone();
        }
        let now = Instant::now();
        let ready = connection
            .read_at
            .is_none_or(|read| now.saturating_duration_since(read) >= link.reread);
        if connection.dirty && ready {
            let snapshot = connection.tree.snapshot(link.observed, link.started);
            let report = match &connection.previous {
                Some(previous) if connection.takes_changes => {
                    let changes = delta(previous, &snapshot);
                    (!changes.is_empty()).then_some(Report::Changed(changes))
                }
                _ => Some(Report::Snapshot(snapshot.clone())),
            };
            let sent = report.is_none_or(|report| link.outbox.send(report));
            if sent {
                connection.previous = Some(snapshot);
                connection.dirty = false;
                connection.read_at = Some(now);
            } else {
                connection.previous = None;
            }
        }
        if !traces.is_empty() {
            let batch = traces.take();
            let lost = batch.spans.len() as u64 + batch.ends.len() as u64 + batch.dropped;
            if !link.outbox.send(Report::Trace(batch)) {
                connection.previous = None;
                connection.dirty = true;
                connection.watching_loss(lost);
            }
        }
        if let Some(runtime) = &link.runtime {
            let at = guinea::observability::trace::now().as_micros() as u64;
            link.outbox.send(Report::Runtime {
                readings: vec![crate::runtime::read(runtime, at)],
            });
        }
        let _ = link
            .scheduler
            .send((generation, now + Duration::from_millis(100)));
    });
}

impl Connection {
    fn watching_loss(&mut self, lost: u64) {
        self.lost += lost;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delta_replaces_changed_actors_and_removes_closed_roots() {
        let previous = Snapshot {
            roots: vec![guinea_devtools_protocol::Root {
                id: 7,
                ..Default::default()
            }],
            ..Default::default()
        };
        let current = Snapshot {
            actors: vec![guinea_devtools_protocol::Actor {
                id: 9,
                state: "ready".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let changes = delta(&previous, &current);
        let mut applied = previous;
        applied.apply(changes);
        assert_eq!(applied, current);
    }
}
