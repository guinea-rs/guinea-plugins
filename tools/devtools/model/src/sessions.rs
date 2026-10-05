//! Every application that connected, and what it reported.

use std::collections::BTreeMap;

use guinea_devtools_protocol::{Answer, AppInfo, Capability, Report, Snapshot};
use serde::{Deserialize, Serialize};

use crate::chains::Chains;
use crate::clock::Clock;
use crate::native::{Inspection, Picked};
use crate::profile::{Profile, Timeline};
use crate::tasks::Tasks;
use crate::timers::Timers;
use crate::trace::{Classes, Reading};
use crate::trace_log::TraceLog;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", content = "detail", rename_all = "snake_case")]
pub enum Listening {
    #[default]
    Starting,
    On(String),
    Failed(String),
}

impl Listening {
    pub fn describe(&self) -> String {
        match self {
            Listening::Starting => "starting…".to_string(),
            Listening::On(addr) => format!("listening on {addr}"),
            Listening::Failed(why) => format!("not listening: {why}"),
        }
    }
}

/// How many answers a session keeps. A caller asks for its answer within
/// seconds of sending the request; ids only grow, so the oldest go first.
pub const ANSWERS_KEPT: usize = 256;

/// How many sessions that have gone away are kept, newest first, for their
/// trace and for [`Sessions::successor`].
pub const GONE_KEPT: usize = 16;

#[derive(Clone, Debug, Default)]
pub struct Session {
    pub id: u64,
    pub info: AppInfo,
    pub snapshot: Snapshot,
    pub trace: TraceLog,
    /// The trace's records by class, kept up to date as it arrives.
    pub classes: Classes,
    /// Every timer the snapshots mentioned.
    pub timers: Timers,
    /// The trace, as chains of records that set each other off.
    pub chains: Chains,
    /// The background work it is still waiting on.
    pub tasks: Tasks,
    /// What a native inspector reported, when this session is one.
    pub inspection: Inspection,
    /// Where its puffin profiler listens, while it is switched on.
    pub profiler: Option<String>,
    /// What the last [`ANSWERS_KEPT`] commands that carry a request came to,
    /// by request id.
    pub answers: BTreeMap<u64, Answer>,
    pub received: u64,
    /// Moves whenever something it shows changed: what is drawn from it is
    /// drawn again only then.
    pub revision: u64,
    pub connected: bool,
}

impl Session {
    pub fn clock(&self) -> Clock {
        Clock {
            epoch_ms: self.info.trace_epoch_ms,
        }
    }

    /// What the trace is read against.
    pub fn reading(&self) -> Reading<'_> {
        Reading {
            log: &self.trace,
            clock: self.clock(),
            timers: &self.timers,
        }
    }

    /// A native inspector loaded into an application, rather than an
    /// application: it shows the backend's tree and has no snapshot.
    pub fn inspects_only(&self) -> bool {
        self.info.can(Capability::NativeTree) && !self.info.can(Capability::Snapshot)
    }

    pub fn name(&self) -> String {
        if self.info.name.is_empty() {
            "unnamed application".to_string()
        } else {
            self.info.name.clone()
        }
    }

    pub fn summary(&self) -> Summary {
        Summary {
            id: self.id,
            name: self.name(),
            identifier: self.info.identifier.clone(),
            version: self.info.version.clone(),
            backend: self.info.backend.clone(),
            pid: self.info.pid,
            connected: self.connected,
            records: self.trace.len(),
        }
    }
}

/// An application, in a list of them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub id: u64,
    pub name: String,
    pub identifier: String,
    pub version: String,
    pub backend: String,
    pub pid: u32,
    pub connected: bool,
    pub records: usize,
}

/// What the endpoint hears.
#[derive(Clone, Debug)]
pub enum Incoming {
    Listening(String),
    Failed(String),
    Opened(u64),
    Report(u64, Box<Report>),
    Closed(u64),
}

#[derive(Debug, Default)]
pub struct Sessions {
    pub listening: Listening,
    pub by_id: BTreeMap<u64, Session>,
    moved: u64,
}

impl Sessions {
    /// Moves whenever anything shown changed: a session came or went, or one
    /// of them moved its own [`Session::revision`].
    pub fn revision(&self) -> u64 {
        self.moved
    }

    pub fn get(&self, id: u64) -> Option<&Session> {
        self.by_id.get(&id)
    }

    /// Newest first.
    pub fn summaries(&self) -> Vec<Summary> {
        self.by_id.values().rev().map(Session::summary).collect()
    }

    /// The newest connected application. An inspector attached to one is
    /// not an application of its own.
    pub fn newest(&self) -> Option<&Session> {
        self.by_id
            .values()
            .rev()
            .find(|session| session.connected && !session.inspects_only())
    }

    /// What `latest` names: the newest connected application, or else the
    /// newest one at all.
    pub fn latest(&self) -> Option<&Session> {
        self.newest().or_else(|| {
            self.by_id
                .values()
                .rev()
                .find(|session| !session.inspects_only())
        })
    }

    /// The connected native inspector in the same process as session `id`:
    /// the application's own session, or another one that shares its pid.
    pub fn native_for(&self, id: u64) -> Option<&Session> {
        let app = self.get(id)?;

        let inspects =
            |session: &&Session| session.connected && session.info.can(Capability::NativeTree);
        if inspects(&app) {
            return Some(app);
        }

        self.by_id
            .values()
            .rev()
            .filter(inspects)
            .find(|session| session.info.pid == app.info.pid && app.info.pid != 0)
    }

    /// The frames of session `id`'s last capture on its trace's timeline:
    /// the clock comes from the application, the frames from its inspector.
    /// Why there is none, when there is none.
    pub fn profile(&self, id: u64) -> Result<Profile<'_>, String> {
        let app = self.clocked(id).ok_or(
            "the application sent no clock: its devtools plugin speaks a protocol older than 3.3",
        )?;
        let anchor = app.info.clock.unwrap_or_default();
        let timeline = Timeline::new(anchor)
            .ok_or("the application read no QPC: frames are placed on Windows only")?;
        let inspector = self
            .native_for(app.id)
            .ok_or("no native inspector: attach one first")?;

        Ok(Profile::new(timeline, app.clock(), &inspector.inspection.kept)
            .with_samples(&inspector.inspection.sampled))
    }

    /// The session of the application `id` belongs to that sent a clock:
    /// `id` itself, or the newest session in its process that did. Its trace
    /// is the one a profile's frames are placed on.
    pub fn clocked(&self, id: u64) -> Option<&Session> {
        let asked = self.get(id)?;
        if asked.info.clock.is_some() {
            return Some(asked);
        }

        self.by_id
            .values()
            .rev()
            .filter(|session| session.info.clock.is_some())
            .find(|session| session.info.pid == asked.info.pid && asked.info.pid != 0)
    }

    /// The same application started again, when `id` went away.
    pub fn successor(&self, id: u64) -> Option<u64> {
        let gone = self.get(id).filter(|session| !session.connected)?;

        self.by_id
            .values()
            .rev()
            .find(|other| {
                other.connected
                    && other.id != gone.id
                    && other.info.identifier == gone.info.identifier
            })
            .map(|other| other.id)
    }

    /// Drops sessions that went away, past the newest [`GONE_KEPT`] of them.
    fn forget_the_long_gone(&mut self) {
        let gone: Vec<u64> = self
            .by_id
            .values()
            .rev()
            .filter(|session| !session.connected)
            .skip(GONE_KEPT)
            .map(|session| session.id)
            .collect();
        for id in gone {
            self.by_id.remove(&id);
        }
    }

    pub fn apply(&mut self, incoming: Incoming) {
        match incoming {
            Incoming::Listening(addr) => {
                self.listening = Listening::On(addr);
                self.moved += 1;
            }
            Incoming::Failed(why) => {
                self.listening = Listening::Failed(why);
                self.moved += 1;
            }
            Incoming::Opened(id) => {
                self.moved += 1;
                self.by_id.insert(
                    id,
                    Session {
                        id,
                        connected: true,
                        ..Session::default()
                    },
                );
            }
            Incoming::Report(id, report) => {
                let Some(session) = self.by_id.get_mut(&id) else {
                    return;
                };

                session.received += 1;
                let shown = !matches!(
                    *report,
                    Report::Answered { .. } | Report::Profiler { .. } | Report::Unknown
                );
                match *report {
                    Report::Hello(info) => session.info = info,
                    Report::Snapshot(mut snapshot) => {
                        let at = std::mem::replace(&mut snapshot.at, session.snapshot.at);
                        if snapshot == session.snapshot {
                            session.snapshot.at = at;
                            return;
                        }
                        snapshot.at = at;
                        session.timers.note(&snapshot.timers);
                        session.snapshot = snapshot;
                    }
                    Report::Trace(batch) => {
                        session.trace.absorb(batch);
                        session.classes.absorb(Reading {
                            log: &session.trace,
                            clock: session.clock(),
                            timers: &session.timers,
                        });
                        session.chains.absorb(&session.trace, &session.timers);
                        session.tasks.absorb(&session.trace);
                    }
                    Report::NativeTree { changes } => session.inspection.tree.apply(changes),
                    Report::NativeProperties {
                        element,
                        properties,
                    } => {
                        session.inspection.properties = Some((element, properties));
                    }
                    Report::NativePicked { chain, bounds } => {
                        session.inspection.picked = Some(Picked { chain, bounds });
                    }
                    Report::NativePerf { frames, stacks } => {
                        session.inspection.captured(frames);
                        session.inspection.sampled.absorb(stacks);
                    }
                    Report::NativeEnums { enums } => {
                        session.inspection.enums = enums
                            .into_iter()
                            .map(|kind| (kind.name, kind.values))
                            .collect();
                    }
                    Report::Changed(changes) => {
                        if let Some(timers) = &changes.timers {
                            session.timers.note(timers);
                        }
                        session.snapshot.apply(changes);
                    }
                    Report::Profiler { at } => session.profiler = at,
                    Report::Unknown => {}
                    Report::Refused { command, reason } => {
                        session.inspection.refused = Some((command, reason));
                    }
                    Report::Answered { request, answer } => {
                        session.answers.insert(request, answer);
                        while session.answers.len() > ANSWERS_KEPT {
                            session.answers.pop_first();
                        }
                    }
                }
                if shown {
                    session.revision += 1;
                    self.moved += 1;
                }
            }
            Incoming::Closed(id) => {
                self.moved += 1;
                if let Some(session) = self.by_id.get_mut(&id) {
                    session.connected = false;
                }
                self.forget_the_long_gone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(identifier: &str) -> Incoming {
        Incoming::Report(
            0,
            Box::new(Report::Hello(AppInfo {
                identifier: identifier.into(),
                ..AppInfo::default()
            })),
        )
    }

    #[test]
    fn a_profile_takes_the_clock_from_the_application_and_the_frames_from_its_inspector() {
        use guinea_devtools_protocol::ClockAnchor;
        use guinea_devtools_protocol::native::{Frame, Stacks};

        let anchor = ClockAnchor {
            qpc: 1_000_000,
            qpc_frequency: 1_000_000,
            trace_us: 0,
            puffin_ns: None,
            ui_thread: 9,
        };
        let report = |id: u64, report: Report| Incoming::Report(id, Box::new(report));
        let mut sessions = Sessions::default();

        sessions.apply(Incoming::Opened(1));
        sessions.apply(report(
            1,
            Report::Hello(AppInfo {
                pid: 40,
                clock: Some(anchor),
                ..AppInfo::default()
            }),
        ));
        assert!(
            sessions.profile(1).is_err(),
            "no inspector, no frames to place"
        );

        sessions.apply(Incoming::Opened(2));
        sessions.apply(report(
            2,
            Report::Hello(AppInfo {
                pid: 40,
                capabilities: vec![Capability::NativeTree],
                ..AppInfo::default()
            }),
        ));
        sessions.apply(report(
            2,
            Report::NativePerf {
                frames: vec![Frame {
                    took_us: 20_000,
                    qpc: 1_250_000,
                    thread: 9,
                    ..Frame::default()
                }],
                stacks: Stacks::default(),
            },
        ));

        let profile = sessions.profile(1).expect("a profile");
        assert_eq!(profile.frames.len(), 1);
        assert_eq!(profile.frames[0].at_us, 250_000);

        let frame = |qpc: u64| Frame {
            took_us: 4_000,
            qpc,
            thread: 9,
            ..Frame::default()
        };
        sessions.apply(report(
            2,
            Report::NativePerf {
                frames: vec![frame(1_250_000), frame(1_400_000)],
                stacks: Stacks::default(),
            },
        ));
        sessions.apply(report(
            2,
            Report::NativePerf {
                frames: vec![frame(1_600_000)],
                stacks: Stacks::default(),
            },
        ));
        let at: Vec<i64> = sessions
            .profile(1)
            .expect("a profile")
            .frames
            .iter()
            .map(|frame| frame.at_us)
            .collect();
        assert_eq!(
            at,
            [250_000, 400_000, 600_000],
            "captures add up, a frame in two of them counted once"
        );
        assert_eq!(
            sessions.profile(2).map(|profile| profile.frames.len()),
            Ok(3),
            "asked through the inspector's own session, the same profile"
        );
    }

    #[test]
    fn a_restarted_application_is_its_own_successor() {
        let mut sessions = Sessions::default();

        sessions.apply(Incoming::Opened(1));
        sessions.apply(match hello("app") {
            Incoming::Report(_, report) => Incoming::Report(1, report),
            other => other,
        });
        sessions.apply(Incoming::Closed(1));
        assert_eq!(sessions.successor(1), None);

        sessions.apply(Incoming::Opened(2));
        sessions.apply(match hello("app") {
            Incoming::Report(_, report) => Incoming::Report(2, report),
            other => other,
        });

        assert_eq!(sessions.successor(1), Some(2));
        assert_eq!(sessions.newest().map(|s| s.id), Some(2));
    }

    #[test]
    fn only_the_newest_gone_sessions_and_answers_are_kept() {
        let mut sessions = Sessions::default();
        let total = GONE_KEPT as u64 + 5;
        for id in 1..=total {
            sessions.apply(Incoming::Opened(id));
            sessions.apply(Incoming::Closed(id));
        }
        sessions.apply(Incoming::Opened(100));

        assert_eq!(sessions.by_id.len(), GONE_KEPT + 1);
        assert!(sessions.get(1).is_none(), "the oldest went");
        assert!(sessions.get(total).is_some(), "the newest stayed");
        assert!(sessions.get(100).is_some(), "a connected one always stays");

        for request in 0..ANSWERS_KEPT as u64 + 10 {
            sessions.apply(Incoming::Report(
                100,
                Box::new(Report::Answered {
                    request,
                    answer: Answer::Done,
                }),
            ));
        }
        let answers = &sessions.get(100).expect("there").answers;
        assert_eq!(answers.len(), ANSWERS_KEPT);
        assert!(!answers.contains_key(&0) && answers.contains_key(&(ANSWERS_KEPT as u64 + 9)));
    }

    #[test]
    fn the_classes_of_records_follow_the_trace_as_it_arrives() {
        use crate::trace::Class;
        use guinea_devtools_protocol::{Span, TraceBatch, TracePoint};

        let mut sessions = Sessions::default();
        sessions.apply(Incoming::Opened(1));
        sessions.apply(Incoming::Report(
            1,
            Box::new(Report::Trace(TraceBatch {
                spans: vec![Span {
                    id: 1,
                    parent: None,
                    at: 0,
                    took: None,
                    point: TracePoint::Log {
                        level: "ERROR".into(),
                        target: "app".into(),
                        text: "failed".into(),
                        written: None,
                    },
                }],
                ends: Vec::new(),
                dropped: 0,
            })),
        ));

        let classes = &sessions.get(1).expect("there").classes;
        assert_eq!(classes.count(Class::Own), 1);
        assert_eq!(classes.count(Class::Level(crate::words::Level::Error)), 1);
    }

    #[test]
    fn changes_bring_the_snapshot_and_the_timers_up_to_date() {
        use guinea_devtools_protocol::{Actor, Changes, Snapshot, Timer};

        let mut sessions = Sessions::default();
        sessions.apply(Incoming::Opened(1));
        sessions.apply(Incoming::Report(
            1,
            Box::new(Report::Snapshot(Snapshot {
                actors: vec![Actor {
                    id: 1,
                    state: "before".into(),
                    ..Actor::default()
                }],
                ..Snapshot::default()
            })),
        ));
        sessions.apply(Incoming::Report(
            1,
            Box::new(Report::Changed(Changes {
                actors: vec![Actor {
                    id: 1,
                    state: "after".into(),
                    ..Actor::default()
                }],
                timers: Some(vec![Timer {
                    id: 7,
                    name: Some("ping".into()),
                    ..Timer::default()
                }]),
                ..Changes::default()
            })),
        ));

        let session = sessions.get(1).expect("there");
        assert_eq!(session.snapshot.actors[0].state, "after");
        assert_eq!(session.snapshot.timers.len(), 1);
        assert!(
            session.timers.get(7).is_some(),
            "a timer that came with changes is known by its id"
        );
    }

    #[test]
    fn the_revision_moves_only_when_something_shown_changed() {
        use guinea_devtools_protocol::{Actor, Changes, Snapshot, TraceBatch};

        let mut sessions = Sessions::default();
        sessions.apply(Incoming::Opened(1));
        let revision = |sessions: &Sessions| sessions.get(1).expect("there").revision;
        let report = |report| Incoming::Report(1, Box::new(report));
        let snapshot = Snapshot {
            actors: vec![Actor {
                id: 1,
                ..Actor::default()
            }],
            ..Snapshot::default()
        };

        let before = revision(&sessions);
        sessions.apply(report(Report::Snapshot(snapshot.clone())));
        let first = revision(&sessions);
        assert!(first > before);

        sessions.apply(report(Report::Snapshot(snapshot)));
        assert_eq!(revision(&sessions), first, "the same snapshot again");

        sessions.apply(report(Report::Answered {
            request: 1,
            answer: Answer::Done,
        }));
        assert_eq!(revision(&sessions), first, "an answer is not shown");

        sessions.apply(report(Report::Changed(Changes {
            actors_gone: vec![1],
            ..Changes::default()
        })));
        let changed = revision(&sessions);
        assert!(changed > first);

        sessions.apply(report(Report::Trace(TraceBatch::default())));
        assert!(revision(&sessions) > changed);
    }

    #[test]
    fn the_sessions_move_when_one_comes_goes_or_moves() {
        let mut sessions = Sessions::default();
        let start = sessions.revision();

        sessions.apply(Incoming::Opened(1));
        let opened = sessions.revision();
        assert!(opened > start);

        sessions.apply(Incoming::Report(
            1,
            Box::new(Report::Answered {
                request: 1,
                answer: Answer::Done,
            }),
        ));
        assert_eq!(sessions.revision(), opened);

        sessions.apply(Incoming::Report(1, Box::new(Report::Profiler { at: None })));
        assert_eq!(sessions.revision(), opened);

        sessions.apply(Incoming::Report(
            1,
            Box::new(Report::Trace(
                guinea_devtools_protocol::TraceBatch::default(),
            )),
        ));
        let traced = sessions.revision();
        assert!(traced > opened);

        sessions.apply(Incoming::Closed(1));
        assert!(sessions.revision() > traced);
    }

    #[test]
    fn latest_passes_over_an_inspector_attached_after_the_application() {
        let mut sessions = Sessions::default();
        let said = |id, capabilities: Vec<Capability>| {
            Incoming::Report(
                id,
                Box::new(Report::Hello(AppInfo {
                    pid: 7,
                    capabilities,
                    ..AppInfo::default()
                })),
            )
        };

        sessions.apply(Incoming::Opened(1));
        sessions.apply(said(1, vec![Capability::Snapshot, Capability::Act]));
        sessions.apply(Incoming::Opened(2));
        sessions.apply(said(2, vec![Capability::NativeTree]));

        assert_eq!(sessions.latest().map(|s| s.id), Some(1));
        assert_eq!(sessions.native_for(1).map(|s| s.id), Some(2));

        sessions.apply(Incoming::Closed(1));
        sessions.apply(Incoming::Closed(2));
        assert_eq!(sessions.latest().map(|s| s.id), Some(1));
    }
}
