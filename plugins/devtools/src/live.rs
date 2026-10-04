//! What devtools are told while they are there: everything once, then only
//! what guinea said moved.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use guinea::observability::snapshot::installed_plugins;
use guinea_core::observability::changes::{self, Change};
use guinea_core::trace::{self, Bus, Point, Trace};
use guinea_devtools_protocol::{AppInfo, Changes, Panel, Report, TraceBatch};

use crate::collect::{self, Traces};
use crate::link::Outbox;

thread_local! {
    static LINK: RefCell<Option<Link>> = const { RefCell::new(None) };
}

/// The UI thread's end of the link.
struct Link {
    outbox: Outbox,
    info: Arc<Mutex<AppInfo>>,
    started: Instant,
    wait: Duration,
    reread: Duration,
    connection: Option<Connection>,
}

struct Connection {
    generation: u64,
    live: Live,
    inbox: Rc<RefCell<Inbox>>,
}

/// What guinea said since the last flush. Kept apart from [`Live`], which a
/// flush holds while it reads, so that what is said meanwhile has somewhere
/// to go.
struct Inbox {
    changes: Vec<Change>,
    traces: Traces,
    roots_moved: bool,
    flushing: bool,
    wait: Duration,
}

/// Hands the link to the UI thread. Nothing is read until devtools connect.
pub fn start(
    outbox: Outbox,
    info: Arc<Mutex<AppInfo>>,
    started: Instant,
    wait: Duration,
    reread: Duration,
) {
    LINK.set(Some(Link {
        outbox,
        info,
        started,
        wait,
        reread,
        connection: None,
    }));
}

pub fn stop() {
    stop_listening();
    LINK.take();
}

/// Devtools connected: they are sent everything, then what moves.
pub fn connected(generation: u64, takes_changes: bool) {
    let listening = LINK.with_borrow_mut(|link| {
        let link = link.as_mut()?;
        let inbox = Rc::new(RefCell::new(Inbox {
            changes: Vec::new(),
            traces: Traces::default(),
            roots_moved: false,
            flushing: false,
            wait: link.wait,
        }));
        link.connection = Some(Connection {
            generation,
            live: Live::new(link.started, takes_changes).reread_every(link.reread),
            inbox: inbox.clone(),
        });
        Some(inbox)
    });
    let Some(inbox) = listening else {
        return;
    };

    let traced = inbox.clone();
    trace::observe(move |record| {
        let moved = match record {
            Trace::Begin(record) | Trace::Mark(record) => {
                matches!(record.point, Point::Push { .. } | Point::Navigate { .. })
            }
            Trace::End { .. } => false,
        };
        let mut inbox = traced.borrow_mut();
        inbox.traces.push(record);
        inbox.roots_moved |= moved;
        soon(&mut inbox);
    });
    changes::watch(move |change| {
        let mut inbox = inbox.borrow_mut();
        inbox.changes.push(change.clone());
        soon(&mut inbox);
    });

    flush();
}

/// Devtools went: nothing is read for nobody.
pub fn disconnected(generation: u64) {
    let current = LINK.with_borrow_mut(|link| {
        let link = link.as_mut()?;
        let connection = link
            .connection
            .take_if(|connection| connection.generation == generation);
        Some(connection.is_some())
    });
    if current == Some(true) {
        stop_listening();
    }
}

fn stop_listening() {
    trace::stop_observing();
    changes::stop_watching();
}

fn soon(inbox: &mut Inbox) {
    let wait = inbox.wait;
    flush_in(inbox, wait);
}

fn flush_in(inbox: &mut Inbox, wait: Duration) {
    if !std::mem::replace(&mut inbox.flushing, true) {
        guinea::timers::after(wait, flush);
    }
}

fn flush() {
    LINK.with_borrow_mut(|link| {
        let Some(link) = link.as_mut() else {
            return;
        };
        let Some(connection) = link.connection.as_mut() else {
            return;
        };

        let (changes, traced, roots_moved) = {
            let mut inbox = connection.inbox.borrow_mut();
            inbox.flushing = false;
            (
                std::mem::take(&mut inbox.changes),
                inbox.traces.take(),
                std::mem::take(&mut inbox.roots_moved),
            )
        };
        for change in &changes {
            connection.live.note(change);
        }
        if roots_moved {
            connection.live.roots_moved();
        }
        let flushed = connection.live.reports(traced, Instant::now());

        {
            let mut info = link
                .info
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(backend) = flushed.backend {
                info.backend = backend.to_string();
            }
            if !flushed.plugins.is_empty() {
                info.plugins = flushed.plugins.iter().map(|id| id.to_string()).collect();
            }
        }
        let mut dropped = false;
        for report in flushed.reports {
            dropped |= !link.outbox.send(report);
        }
        if dropped {
            connection.live.lost();
            soon(&mut connection.inbox.borrow_mut());
        } else if let Some(due) = flushed.due {
            let wait = due.saturating_duration_since(Instant::now());
            flush_in(&mut connection.inbox.borrow_mut(), wait);
        }
    });
}

/// What one flush hands the link, and what the application's info learnt.
#[derive(Default)]
pub struct Flushed {
    pub reports: Vec<Report>,
    pub backend: Option<&'static str>,
    pub plugins: Vec<&'static str>,
    /// When what moved and was not read yet is to be read.
    pub due: Option<Instant>,
}

/// What was said to have moved since the last flush.
#[derive(Default)]
struct Stale {
    actors: BTreeSet<u64>,
    actors_gone: BTreeSet<u64>,
    roots: BTreeSet<u64>,
    roots_gone: BTreeSet<u64>,
    every_root: bool,
    timers: bool,
    global_bus: bool,
}

impl Stale {
    fn is_empty(&self) -> bool {
        self.actors.is_empty()
            && self.actors_gone.is_empty()
            && self.roots.is_empty()
            && self.roots_gone.is_empty()
            && !self.every_root
            && !self.timers
            && !self.global_bus
    }
}

/// One connection's view of what devtools hold.
pub struct Live {
    started: Instant,
    takes_changes: bool,
    whole: bool,
    stale: Stale,
    reread: Duration,
    read_at: Option<Instant>,
    /// Each actor's window, `None` for the application's own.
    homes: HashMap<u64, Option<u64>>,
    /// Each window's segments as their scopes' keys.
    chains: BTreeMap<u64, Vec<usize>>,
    panels: Vec<Panel>,
}

impl Live {
    pub fn new(started: Instant, takes_changes: bool) -> Self {
        Self {
            started,
            takes_changes,
            whole: true,
            stale: Stale::default(),
            reread: Duration::ZERO,
            read_at: None,
            homes: HashMap::new(),
            chains: BTreeMap::new(),
            panels: Vec::new(),
        }
    }

    /// Reads state that keeps moving again at most once per `period`.
    pub fn reread_every(mut self, period: Duration) -> Self {
        self.reread = period;
        self
    }

    /// Remembers what `change` made stale, to read at the next flush.
    pub fn note(&mut self, change: &Change) {
        let stale = &mut self.stale;
        match *change {
            Change::ActorAdded { root, id, .. } => {
                let id = id as u64;
                self.homes.insert(id, root);
                stale.actors_gone.remove(&id);
                stale.actors.insert(id);
            }
            Change::ActorRemoved { id, .. } => {
                let id = id as u64;
                self.homes.remove(&id);
                stale.actors.remove(&id);
                stale.actors_gone.insert(id);
            }
            Change::ActorHandled { id } => {
                let id = id as u64;
                if self.homes.contains_key(&id) {
                    stale.actors.insert(id);
                }
            }
            Change::TimerStarted { .. } | Change::TimerStopped { .. } => stale.timers = true,
            Change::Subscriptions {
                bus: Bus::Global, ..
            } => stale.global_bus = true,
            Change::Subscriptions {
                root: Some(root), ..
            }
            | Change::RouterOpened { root } => {
                stale.roots_gone.remove(&root);
                stale.roots.insert(root);
            }
            Change::RouterClosed { root } => {
                stale.roots.remove(&root);
                stale.roots_gone.insert(root);
            }
            _ => stale.every_root = true,
        }
    }

    /// A reducer moved or a window navigated: every window reads again.
    pub fn roots_moved(&mut self) {
        self.stale.every_root = true;
    }

    /// A report never reached devtools: what they hold is unknown, so they
    /// are sent everything again.
    pub fn lost(&mut self) {
        self.whole = true;
    }

    /// What devtools are to be sent now, the trace after the state it explains.
    pub fn reports(&mut self, traced: TraceBatch, now: Instant) -> Flushed {
        let waiting = self
            .read_at
            .map(|read_at| read_at + self.reread)
            .filter(|due| *due > now);
        let mut flushed = match waiting {
            Some(due) => Flushed {
                due: (self.whole || !self.takes_changes || !self.stale.is_empty()).then_some(due),
                ..Flushed::default()
            },
            None => {
                self.read_at = Some(now);
                if self.whole || !self.takes_changes {
                    self.whole()
                } else {
                    self.changes()
                }
            }
        };

        if !traced.spans.is_empty() || !traced.ends.is_empty() || traced.dropped > 0 {
            flushed.reports.push(Report::Trace(traced));
        }
        flushed
    }

    fn whole(&mut self) -> Flushed {
        let collected = collect::snapshot(self.started);

        self.whole = false;
        self.stale = Stale::default();
        self.homes = collected
            .report
            .actors
            .iter()
            .map(|actor| (actor.id, actor.root))
            .collect();
        self.chains = collected.segments.into_iter().collect();
        self.panels = collected.report.panels.clone();

        Flushed {
            reports: vec![Report::Snapshot(collected.report)],
            backend: collected.backend,
            plugins: collected.plugins,
            due: None,
        }
    }

    fn changes(&mut self) -> Flushed {
        let stale = std::mem::take(&mut self.stale);
        let mut changes = Changes {
            at: collect::since(self.started),
            ..Changes::default()
        };
        let mut backend = None;

        let mut reread = stale.roots;
        if stale.every_root {
            reread.extend(self.chains.keys());
        }
        let mut roots_gone = stale.roots_gone;
        for id in reread {
            match guinea::observability::snapshot::router(id) {
                Some(router) => {
                    backend = Some(router.backend);
                    self.chains.insert(id, collect::segments(&router));
                    changes.roots.push(collect::root(router));
                }
                None => {
                    roots_gone.insert(id);
                }
            }
        }
        for id in &roots_gone {
            self.chains.remove(id);
        }
        changes.roots_gone = roots_gone.into_iter().collect();

        let mut actors_gone = stale.actors_gone;
        for id in stale.actors {
            let home = self.homes.get(&id).copied().flatten();
            let read = guinea::observability::snapshot::actor(home, id as usize);
            match read {
                Some(snapshot) => {
                    let segments = home
                        .and_then(|root| self.chains.get(&root))
                        .map_or(&[][..], Vec::as_slice);
                    changes
                        .actors
                        .push(collect::actor(&snapshot, home, segments));
                }
                None => {
                    self.homes.remove(&id);
                    actors_gone.insert(id);
                }
            }
        }
        changes.actors_gone = actors_gone.into_iter().collect();

        if stale.timers {
            changes.timers = Some(collect::timers(&collect::scopes(&self.chains)));
        }
        if stale.global_bus {
            changes.global_bus = Some(collect::global_bus());
        }
        let panels = collect::app_panels();
        if panels != self.panels {
            self.panels = panels.clone();
            changes.panels = Some(panels);
        }

        Flushed {
            reports: if changes.is_empty() {
                Vec::new()
            } else {
                vec![Report::Changed(changes)]
            },
            backend,
            plugins: installed_plugins(),
            due: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use guinea_core::actor::registry::Owner;
    use guinea_core::trace::Bus;
    use guinea_devtools_protocol::{Span, TracePoint};

    use super::*;

    fn kinds(flushed: &Flushed) -> Vec<&'static str> {
        flushed
            .reports
            .iter()
            .map(|report| match report {
                Report::Snapshot(_) => "snapshot",
                Report::Changed(_) => "changed",
                Report::Trace(_) => "trace",
                _ => "other",
            })
            .collect()
    }

    fn traced() -> TraceBatch {
        TraceBatch {
            spans: vec![Span {
                id: 1,
                parent: None,
                at: 0,
                took: None,
                point: TracePoint::Note { text: "hi".into() },
            }],
            ..TraceBatch::default()
        }
    }

    #[test]
    fn devtools_hear_everything_once_and_then_only_what_moved() {
        let mut live = Live::new(Instant::now(), true);

        assert_eq!(
            kinds(&live.reports(TraceBatch::default(), Instant::now())),
            ["snapshot"]
        );
        assert_eq!(
            kinds(&live.reports(TraceBatch::default(), Instant::now())),
            Vec::<&str>::new(),
            "nothing moved"
        );

        live.note(&Change::Subscriptions {
            bus: Bus::Global,
            root: None,
        });
        let flushed = live.reports(TraceBatch::default(), Instant::now());
        let [Report::Changed(changes)] = flushed.reports.as_slice() else {
            panic!("one change, not {:?}", kinds(&flushed));
        };
        assert!(changes.global_bus.is_some(), "the global bus is read again");
        assert!(changes.timers.is_none() && changes.actors.is_empty() && changes.roots.is_empty());
    }

    #[test]
    fn devtools_that_take_no_changes_are_sent_the_whole_snapshot() {
        let mut live = Live::new(Instant::now(), false);
        assert_eq!(
            kinds(&live.reports(TraceBatch::default(), Instant::now())),
            ["snapshot"]
        );

        live.note(&Change::TimerStarted { id: 1 });
        assert_eq!(
            kinds(&live.reports(TraceBatch::default(), Instant::now())),
            ["snapshot"]
        );
    }

    #[test]
    fn an_actor_that_cannot_be_read_any_more_is_said_to_be_gone() {
        let mut live = Live::new(Instant::now(), true);
        live.reports(TraceBatch::default(), Instant::now());

        live.note(&Change::ActorAdded {
            root: None,
            id: 41,
            type_name: "Counter",
            owner: Owner::default(),
        });
        live.note(&Change::ActorHandled { id: 41 });
        live.note(&Change::ActorRemoved { root: None, id: 42 });
        live.note(&Change::ActorHandled { id: 99 });

        let flushed = live.reports(TraceBatch::default(), Instant::now());
        let [Report::Changed(changes)] = flushed.reports.as_slice() else {
            panic!("one change, not {:?}", kinds(&flushed));
        };
        assert!(changes.actors.is_empty());
        assert_eq!(
            changes.actors_gone,
            [41, 42],
            "an actor never said to be added is not devtools' to drop"
        );
    }

    #[test]
    fn after_a_report_was_lost_devtools_are_sent_everything_again() {
        let mut live = Live::new(Instant::now(), true);
        live.reports(TraceBatch::default(), Instant::now());

        live.lost();
        assert_eq!(
            kinds(&live.reports(TraceBatch::default(), Instant::now())),
            ["snapshot"]
        );
        assert_eq!(
            kinds(&live.reports(TraceBatch::default(), Instant::now())),
            Vec::<&str>::new()
        );
    }

    mod counting {
        use guinea::prelude::*;

        #[derive(Default, Debug)]
        pub struct Counter {
            counted: u64,
        }

        #[derive(Debug, Clone)]
        pub struct Count;

        actor! {
            Counter {
                handlers { Count }
            }
        }

        #[handler]
        fn count(this: &mut Counter, _: Count) {
            this.counted += 1;
        }

        thread_local! {
            pub static SPAWNED: std::cell::RefCell<Option<Addr<Counter>>> =
                const { std::cell::RefCell::new(None) };
        }

        pub struct Counting;

        impl AppFeature for Counting {
            type Exports = ();

            fn install(self, app: &mut FeatureBuilder) -> anyhow::Result<()> {
                let counter = app.spawn(Counter::default());
                SPAWNED.set(Some(counter));
                Ok(())
            }
        }
    }

    #[test]
    fn an_actor_that_handled_a_message_is_sent_as_it_reads_now() {
        use counting::{Count, Counting, SPAWNED};

        let said = Rc::new(RefCell::new(Vec::new()));
        let sink = said.clone();
        changes::watch(move |change| sink.borrow_mut().push(change.clone()));

        let mut h = guinea::app::Harness::new(1);
        h.feature(Counting).expect("install");
        let counter = SPAWNED.take().expect("spawned");

        let mut live = Live::new(Instant::now(), true);
        live.reports(TraceBatch::default(), Instant::now());
        said.borrow_mut().clear();

        h.record("Count", || counter.send(Count)).settle();
        changes::stop_watching();

        for change in said.borrow().iter() {
            live.note(change);
        }
        let flushed = live.reports(TraceBatch::default(), Instant::now());
        let [Report::Changed(changes)] = flushed.reports.as_slice() else {
            panic!("one change, not {:?}", kinds(&flushed));
        };
        let sent = changes
            .actors
            .iter()
            .find(|actor| actor.type_name.ends_with("Counter"))
            .unwrap_or_else(|| panic!("the counter, among {:?}", changes.actors));
        assert!(sent.state.contains("counted: 1"), "{}", sent.state);
        assert!(changes.actors_gone.is_empty());
    }

    #[test]
    fn state_that_keeps_moving_is_read_again_at_most_once_a_period() {
        let started = Instant::now();
        let mut live = Live::new(started, true).reread_every(Duration::from_secs(1));
        assert_eq!(
            kinds(&live.reports(TraceBatch::default(), started)),
            ["snapshot"]
        );

        let moved = Change::Subscriptions {
            bus: Bus::Global,
            root: None,
        };
        live.note(&moved);
        let soon = live.reports(traced(), started + Duration::from_millis(100));
        assert_eq!(
            kinds(&soon),
            ["trace"],
            "the trace goes now, the state waits"
        );
        assert_eq!(
            soon.due,
            Some(started + Duration::from_secs(1)),
            "the state is read when its period is up"
        );

        let due = live.reports(TraceBatch::default(), started + Duration::from_secs(1));
        assert_eq!(kinds(&due), ["changed"]);
        assert_eq!(due.due, None, "nothing is left to read");
    }

    #[test]
    fn the_trace_follows_the_state_it_explains() {
        let mut live = Live::new(Instant::now(), true);

        assert_eq!(
            kinds(&live.reports(traced(), Instant::now())),
            ["snapshot", "trace"]
        );
        assert_eq!(kinds(&live.reports(traced(), Instant::now())), ["trace"]);
    }
}
