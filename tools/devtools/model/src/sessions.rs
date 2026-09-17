//! Every application that connected, and what it reported.

use std::collections::BTreeMap;

use guinea_devtools_protocol::{AppInfo, Report, Snapshot};
use serde::{Deserialize, Serialize};

use crate::chains::Chains;
use crate::clock::Clock;
use crate::timers::Timers;
use crate::trace::Reading;
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

#[derive(Clone, Debug, Default)]
pub struct Session {
    pub id: u64,
    pub info: AppInfo,
    pub snapshot: Snapshot,
    pub trace: TraceLog,
    /// Every timer the snapshots mentioned.
    pub timers: Timers,
    /// The trace, as chains of records that set each other off.
    pub chains: Chains,
    pub received: u64,
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
}

impl Sessions {
    pub fn get(&self, id: u64) -> Option<&Session> {
        self.by_id.get(&id)
    }

    /// Newest first.
    pub fn summaries(&self) -> Vec<Summary> {
        self.by_id.values().rev().map(Session::summary).collect()
    }

    /// The newest connected application.
    pub fn newest(&self) -> Option<&Session> {
        self.by_id.values().rev().find(|session| session.connected)
    }

    /// The same application started again, when `id` went away.
    pub fn successor(&self, id: u64) -> Option<u64> {
        let gone = self.get(id).filter(|session| !session.connected)?;

        self.by_id
            .values()
            .rev()
            .find(|other| {
                other.connected && other.id != gone.id && other.info.identifier == gone.info.identifier
            })
            .map(|other| other.id)
    }

    pub fn apply(&mut self, incoming: Incoming) {
        match incoming {
            Incoming::Listening(addr) => self.listening = Listening::On(addr),
            Incoming::Failed(why) => self.listening = Listening::Failed(why),
            Incoming::Opened(id) => {
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
                match *report {
                    Report::Hello(info) => session.info = info,
                    Report::Snapshot(snapshot) => {
                        session.timers.note(&snapshot.timers);
                        session.snapshot = snapshot;
                    }
                    Report::Trace(batch) => {
                        session.trace.absorb(batch);
                        session.chains.absorb(&session.trace, &session.timers);
                    }
                }
            }
            Incoming::Closed(id) => {
                if let Some(session) = self.by_id.get_mut(&id) {
                    session.connected = false;
                }
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
}
