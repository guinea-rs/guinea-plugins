//! Text broken into words that say what they are, so a window can colour
//! them and link them, and a terminal can print them.

use guinea_devtools_protocol::TracePoint;
use serde::{Deserialize, Serialize};

use crate::chains::Stream;
use crate::names::{bus_name, type_name};
use crate::timers::Timers;

/// What a word is, for choosing its colour.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tone", content = "of", rename_all = "snake_case")]
pub enum Tone {
    Plain,
    Muted,
    /// The one thing a row is about: a window segment's name.
    Accent,
    /// What a trace record is, or what a word shares a colour with.
    Kind(Kind),
    /// A log line's `tracing` level.
    Level(Level),
    /// A quoted value.
    Quote,
}

/// What a trace record is, as `TracePoint::kind` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Action,
    Send,
    Handle,
    Spawn,
    Settled,
    Cancelled,
    Source,
    Arrived,
    /// A source making its next item. Never a row of its own: it is what a
    /// record ran in.
    Pull,
    Closed,
    Publish,
    Deliver,
    Push,
    Navigate,
    Store,
    Render,
    Span,
    Log,
    Tick,
    Note,
}

impl Kind {
    /// Every kind, in the order a filter lists them.
    pub const ALL: [Kind; 19] = [
        Kind::Action,
        Kind::Send,
        Kind::Handle,
        Kind::Spawn,
        Kind::Settled,
        Kind::Cancelled,
        Kind::Source,
        Kind::Arrived,
        Kind::Closed,
        Kind::Publish,
        Kind::Deliver,
        Kind::Push,
        Kind::Navigate,
        Kind::Store,
        Kind::Render,
        Kind::Span,
        Kind::Log,
        Kind::Tick,
        Kind::Note,
    ];

    pub fn of(point: &TracePoint) -> Kind {
        match point {
            TracePoint::Action { .. } => Kind::Action,
            TracePoint::Send { .. } => Kind::Send,
            TracePoint::Handle { .. } => Kind::Handle,
            TracePoint::Spawn { .. } => Kind::Spawn,
            TracePoint::Settled { .. } => Kind::Settled,
            TracePoint::Cancelled { .. } => Kind::Cancelled,
            TracePoint::Source { .. } => Kind::Source,
            TracePoint::Arrived { .. } => Kind::Arrived,
            TracePoint::Closed { .. } => Kind::Closed,
            TracePoint::Publish { .. } => Kind::Publish,
            TracePoint::Deliver { .. } => Kind::Deliver,
            TracePoint::Push { .. } => Kind::Push,
            TracePoint::Navigate { .. } => Kind::Navigate,
            TracePoint::Render { .. } => Kind::Render,
            TracePoint::Tick { .. } => Kind::Tick,
            TracePoint::Store { .. } => Kind::Store,
            TracePoint::Log { .. } => Kind::Log,
            TracePoint::Pull { .. } => Kind::Pull,
            TracePoint::Note { .. } | TracePoint::Unknown => Kind::Note,
            TracePoint::Span { .. } => Kind::Span,
        }
    }

    /// As `TracePoint::kind` names it: `publish`.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Action => "action",
            Kind::Send => "send",
            Kind::Handle => "handle",
            Kind::Spawn => "spawn",
            Kind::Settled => "settled",
            Kind::Cancelled => "cancelled",
            Kind::Source => "source",
            Kind::Arrived => "arrived",
            Kind::Pull => "pull",
            Kind::Closed => "closed",
            Kind::Publish => "publish",
            Kind::Deliver => "deliver",
            Kind::Push => "push",
            Kind::Navigate => "navigate",
            Kind::Store => "store",
            Kind::Render => "render",
            Kind::Span => "span",
            Kind::Log => "log",
            Kind::Tick => "tick",
            Kind::Note => "note",
        }
    }
}

/// A `tracing` level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Level {
    /// As `tracing` spells it, `INFO`; anything else reads as `TRACE`.
    pub fn named(level: &str) -> Level {
        match level {
            "ERROR" => Level::Error,
            "WARN" => Level::Warn,
            "INFO" => Level::Info,
            "DEBUG" => Level::Debug,
            _ => Level::Trace,
        }
    }

    pub const ALL: [Level; 5] = [
        Level::Error,
        Level::Warn,
        Level::Info,
        Level::Debug,
        Level::Trace,
    ];

    /// Lowercase, as a stream names it: `warn`.
    pub fn name(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Debug => "debug",
            Level::Trace => "trace",
        }
    }

    pub fn parse(name: &str) -> Option<Level> {
        Level::ALL.into_iter().find(|level| level.name() == name)
    }

    /// The level of a record the application's own code wrote, a log line or
    /// a span; a span from an application that did not say is `INFO`, as
    /// `info_span!` is. `None` for the framework's records.
    pub fn of(point: &TracePoint) -> Option<Level> {
        match point {
            TracePoint::Log { level, .. } => Some(Level::named(level)),
            TracePoint::Span { level, .. } if level.is_empty() => Some(Level::Info),
            TracePoint::Span { level, .. } => Some(Level::named(level)),
            _ => None,
        }
    }
}

/// Where a word leads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "to", content = "name", rename_all = "snake_case")]
pub enum Target {
    /// An actor, by its type as the trace names it.
    Actor(String),
    /// A reducer, by its type as the trace names it.
    Reducer(String),
    /// A store key, by its path.
    Key(String),
    /// Records about this, whatever it is: a message type.
    Records(String),
    /// A stream of chains, by its id: `timer/src/startup.rs:48:9`.
    Stream(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    pub tone: Tone,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<Target>,
}

impl Word {
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            link: None,
        }
    }

    fn link(shown: &str, tone: Tone, target: Target) -> Self {
        Self {
            text: shown.to_string(),
            tone,
            link: Some(target),
        }
    }
}

/// The words, joined.
pub fn text(words: &[Word]) -> String {
    words.iter().map(|word| word.text.as_str()).collect()
}

/// A record's colour: its kind's, or for a log line its level's.
pub fn tone(point: &TracePoint) -> Tone {
    match point {
        TracePoint::Log { level, .. } => Tone::Level(Level::named(level)),
        _ => Tone::Kind(Kind::of(point)),
    }
}

/// A record as a sentence: `ProcessActor handles Kill`, with the actor and
/// the message as links. A timer is named from `timers`.
pub fn sentence(point: &TracePoint, timers: &Timers) -> Vec<Word> {
    let said = tone(point);
    let text = |text: String| Word::new(text, said.clone());
    let name = Tone::Plain;
    let actor = |full: &str| {
        Word::link(
            &type_name(full),
            name.clone(),
            Target::Actor(full.to_string()),
        )
    };
    let message = |full: &str| {
        Word::link(
            &type_name(full),
            name.clone(),
            Target::Records(full.to_string()),
        )
    };

    match point {
        TracePoint::Action { message: m } => vec![text("action ".into()), message(m)],
        TracePoint::Send {
            actor: a,
            message: m,
        } => {
            vec![
                text("send ".into()),
                message(m),
                text(" to ".into()),
                actor(a),
            ]
        }
        TracePoint::Handle {
            actor: a,
            message: m,
        } => {
            vec![actor(a), text(" handles ".into()), message(m)]
        }
        TracePoint::Spawn {
            actor: a, output, ..
        } => {
            vec![actor(a), text(" starts work for ".into()), message(output)]
        }
        // How long it took is the row's own column, not part of what the row
        // says: a sentence carrying a number that changes every time is a
        // different sentence every time, and what groups records by shape
        // groups by the sentence.
        TracePoint::Settled {
            actor: a, output, ..
        } => {
            vec![actor(a), text(" has its ".into()), message(output)]
        }
        TracePoint::Cancelled {
            actor: a, output, ..
        } => {
            vec![
                actor(a),
                text(" is gone without its ".into()),
                message(output),
            ]
        }
        TracePoint::Source {
            actor: a, output, ..
        } => {
            vec![
                actor(a),
                text(" opens a source of ".into()),
                message(output),
            ]
        }
        TracePoint::Arrived {
            actor: a, output, ..
        } => vec![
            Word::link(
                &type_name(output),
                name.clone(),
                Target::Stream(Stream::Source(Stream::source_of(a, output)).to_string()),
            ),
            text(" arrives at ".into()),
            actor(a),
        ],
        TracePoint::Pull {
            actor: a, output, ..
        } => vec![
            actor(a),
            text("'s source makes the next ".into()),
            Word::link(
                &type_name(output),
                name.clone(),
                Target::Stream(Stream::Source(Stream::source_of(a, output)).to_string()),
            ),
        ],
        TracePoint::Closed {
            actor: a,
            output,
            gone: false,
            ..
        } => vec![
            actor(a),
            text("'s source of ".into()),
            message(output),
            text(" ran dry".into()),
        ],
        TracePoint::Closed {
            actor: a,
            output,
            gone: true,
            ..
        } => vec![
            actor(a),
            text(" is gone with its source of ".into()),
            message(output),
        ],
        TracePoint::Publish {
            event,
            bus,
            subscribers,
        } => vec![
            text("publish ".into()),
            message(event),
            text(format!(
                " on {} to {}",
                bus_name(*bus),
                match subscribers {
                    0 => "nobody".to_string(),
                    1 => "one listener".to_string(),
                    many => format!("{many} listeners"),
                }
            )),
        ],
        TracePoint::Deliver { event, bus } => vec![
            text("deliver ".into()),
            message(event),
            text(format!(" from {}", bus_name(*bus))),
        ],
        TracePoint::Push { reducer } => vec![
            text("push into ".into()),
            Word::link(&type_name(reducer), name, Target::Reducer(reducer.clone())),
        ],
        TracePoint::Navigate { root, to } => vec![text(format!("{root} navigates to {to}"))],
        TracePoint::Tick { timer: None } => vec![text("timer".into())],
        TracePoint::Tick { timer: Some(id) } => vec![
            text("timer ".into()),
            Word::link(
                &timers.label(*id),
                name,
                Target::Stream(Stream::Timer(timers.site(*id)).to_string()),
            ),
        ],
        TracePoint::Store {
            op,
            path,
            field,
            outside,
        } => {
            let who = if *outside { "disk " } else { "store " };
            let mut said = vec![
                text(format!("{who}{} ", op.verb())),
                Word::link(path, name, Target::Key(path.clone())),
            ];
            if let Some(field) = field {
                said.push(text(format!(" ({field})")));
            }

            said
        }
        TracePoint::Render { segment, .. } => vec![
            text("render ".into()),
            Word::new(type_name(segment), Tone::Plain),
        ],
        TracePoint::Log {
            level,
            target,
            text: logged,
            ..
        } => vec![text(format!("{} {target}: {logged}", level.to_lowercase()))],
        TracePoint::Note { text: note } => vec![text(note.clone())],
        TracePoint::Span {
            name: spanned,
            fields,
            ..
        } => {
            let mut said = vec![
                text("span ".into()),
                Word::new(spanned.clone(), Tone::Plain),
            ];
            if !fields.is_empty() {
                said.push(Word::new(format!(" {fields}"), Tone::Muted));
            }

            said
        }
        TracePoint::Unknown => vec![text(point.describe())],
    }
}

/// The sentence with whole type paths, to search in.
pub fn searchable(point: &TracePoint, timers: &Timers) -> String {
    sentence(point, timers)
        .iter()
        .map(|word| match &word.link {
            Some(Target::Actor(full) | Target::Reducer(full) | Target::Records(full)) => {
                full.as_str()
            }
            _ => word.text.as_str(),
        })
        .collect()
}

/// The sentence cut after what it is about: `publish ProcessKilled`,
/// `ProcessActor handles Kill`.
pub fn gist(point: &TracePoint, timers: &Timers) -> String {
    text(&gist_words(point, timers)).trim_end().to_string()
}

/// [`gist`], as words.
pub fn gist_words(point: &TracePoint, timers: &Timers) -> Vec<Word> {
    let mut out = Vec::new();
    let mut after_text = false;

    for word in sentence(point, timers) {
        let ends = match word.link {
            None => {
                after_text = true;
                false
            }
            Some(_) => after_text,
        };

        out.push(word);
        if ends {
            break;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea_devtools_protocol::{BusKind, StoreOp};

    #[test]
    fn a_cause_is_named_by_what_it_is_about() {
        let timers = Timers::default();
        let publish = TracePoint::Publish {
            event: "events::ProcessKilled".into(),
            bus: BusKind::Global,
            subscribers: 2,
        };

        assert_eq!(gist(&publish, &timers), "publish ProcessKilled");
        assert_eq!(
            searchable(&publish, &timers),
            "publish events::ProcessKilled on the global bus to 2 listeners"
        );

        let handle = TracePoint::Handle {
            actor: "actor::ProcessActor".into(),
            message: "contracts::Kill".into(),
        };

        assert_eq!(gist(&handle, &timers), "ProcessActor handles Kill");
        assert_eq!(gist(&TracePoint::Tick { timer: None }, &timers), "timer");

        let stored = TracePoint::Store {
            op: StoreOp::Set,
            path: "app.language".into(),
            field: Some("Settings.language".into()),
            outside: false,
        };

        assert_eq!(gist(&stored, &timers), "store sets app.language");
        assert_eq!(
            text(&sentence(&stored, &timers)),
            "store sets app.language (Settings.language)"
        );
    }

    #[test]
    fn a_source_is_opened_arrives_and_closes() {
        let timers = Timers::default();
        let (actor, output) = ("agent::Watcher<w::Win>", "events::Changed");

        let opened = TracePoint::Source {
            actor: actor.into(),
            actor_id: 3,
            output: output.into(),
        };
        assert_eq!(
            gist(&opened, &timers),
            "Watcher<Win> opens a source of Changed"
        );

        let arrived = TracePoint::Arrived {
            actor: actor.into(),
            actor_id: 3,
            output: output.into(),
            source: 9,
        };
        let words = sentence(&arrived, &timers);
        assert_eq!(text(&words), "Changed arrives at Watcher<Win>");
        assert_eq!(
            words[0].link,
            Some(Target::Stream(
                "source/agent::Watcher<w::Win>|events::Changed".into()
            ))
        );

        let closed = |gone| TracePoint::Closed {
            actor: actor.into(),
            actor_id: 3,
            output: output.into(),
            took_us: 5,
            gone,
        };
        assert_eq!(
            text(&sentence(&closed(false), &timers)),
            "Watcher<Win>'s source of Changed ran dry"
        );
        assert_eq!(
            text(&sentence(&closed(true), &timers)),
            "Watcher<Win> is gone with its source of Changed"
        );
        for point in [opened, arrived, closed(true)] {
            assert_eq!(Kind::of(&point).name(), point.kind());
        }
    }

    #[test]
    fn a_span_says_its_name_and_then_its_fields() {
        let timers = Timers::default();
        let span = |fields: &str| TracePoint::Span {
            name: "rows_from_report".into(),
            target: "uniproc::processes".into(),
            fields: fields.into(),
            declared: None,
            level: String::new(),
        };

        assert_eq!(
            text(&sentence(&span("rows=12"), &timers)),
            "span rows_from_report rows=12"
        );
        assert_eq!(text(&sentence(&span(""), &timers)), "span rows_from_report");
        assert_eq!(Kind::of(&span("")).name(), span("").kind());
    }

    #[test]
    fn only_what_the_application_wrote_has_a_level() {
        let span = |level: &str| TracePoint::Span {
            name: "scan".into(),
            target: "app".into(),
            fields: String::new(),
            declared: None,
            level: level.into(),
        };
        let log = TracePoint::Log {
            level: "ERROR".into(),
            target: "app".into(),
            text: "failed".into(),
            written: None,
        };

        assert_eq!(Level::of(&log), Some(Level::Error));
        assert_eq!(Level::of(&span("DEBUG")), Some(Level::Debug));
        assert_eq!(
            Level::of(&span("")),
            Some(Level::Info),
            "a span that did not say"
        );
        assert_eq!(Level::of(&TracePoint::Tick { timer: None }), None);
        assert_eq!(Level::parse("warn"), Some(Level::Warn));
        assert_eq!(Level::parse("loud"), None);
    }

    #[test]
    fn a_tick_is_called_by_its_timer_and_leads_to_it() {
        let mut timers = Timers::default();
        timers.note(&[guinea_devtools_protocol::Timer {
            id: 4,
            name: Some("housekeeping".into()),
            ..guinea_devtools_protocol::Timer::default()
        }]);

        let words = sentence(&TracePoint::Tick { timer: Some(4) }, &timers);

        assert_eq!(text(&words), "timer housekeeping");
        assert_eq!(words[1].link, Some(Target::Stream("timer/#4".into())));
    }
}
