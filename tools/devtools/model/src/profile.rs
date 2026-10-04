//! Frames on the trace's timeline, by the second they began in.
//!
//! The backend's frames come stamped with QPC; the trace counts microseconds
//! since it began. The application read both at one moment
//! ([`ClockAnchor`]), which is all it takes to put a frame among the records
//! that led to it.

use guinea_devtools_protocol::native::{Frame as Captured, Pass};
use guinea_devtools_protocol::{ClockAnchor, Span};
use serde::Serialize;

use crate::clock::Clock;
use crate::trace_log::TraceLog;

/// A frame longer than this missed a 60 Hz vsync.
pub const BUDGET_US: u64 = 16_667;

/// QPC ticks to the trace's microseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timeline {
    anchor: ClockAnchor,
}

impl Timeline {
    /// `None` when the application read no QPC: off Windows, or before it
    /// sent a clock at all.
    pub fn new(anchor: ClockAnchor) -> Option<Self> {
        (anchor.qpc_frequency > 0).then_some(Self { anchor })
    }

    /// Microseconds since the trace began; negative before it.
    pub fn trace_us(&self, qpc: u64) -> i64 {
        let ticks = qpc as i128 - self.anchor.qpc as i128;
        let micros = ticks * 1_000_000 / self.anchor.qpc_frequency as i128;
        (self.anchor.trace_us as i128 + micros) as i64
    }
}

/// One frame, on the trace's timeline.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Frame {
    /// When it began, in microseconds since the trace did; also what names
    /// it.
    pub at_us: i64,
    pub took_us: u64,
    pub measure_us: u64,
    pub arrange_us: u64,
    /// The operating system's id of the thread that drew it.
    pub thread: u32,
    /// Every layout pass, costliest first.
    pub passes: Vec<Pass>,
}

impl Frame {
    pub fn over(&self) -> bool {
        self.took_us > BUDGET_US
    }

    fn end_us(&self) -> i64 {
        self.at_us + self.took_us as i64
    }
}

/// A second in which something was drawn.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Second {
    /// Unix seconds; seconds since the trace began when the application did
    /// not say when that was.
    pub t: i64,
    pub frames: usize,
    /// How many took longer than [`BUDGET_US`].
    pub over: usize,
    pub worst_us: u64,
    /// The `at_us` of the longest frame.
    pub worst_frame: i64,
}

/// The frames of one capture, placed on the trace's timeline.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Profile {
    /// Oldest first.
    pub frames: Vec<Frame>,
    clock: Clock,
}

impl Profile {
    /// Frames the tap stamped with no QPC are left out: nothing places them.
    pub fn new(timeline: Timeline, clock: Clock, captured: &[Captured]) -> Self {
        let mut frames: Vec<Frame> = captured
            .iter()
            .filter(|frame| frame.qpc != 0)
            .map(|frame| Frame {
                at_us: timeline.trace_us(frame.qpc),
                took_us: frame.took_us,
                measure_us: frame.measure_us,
                arrange_us: frame.arrange_us,
                thread: frame.thread,
                passes: frame.passes.clone(),
            })
            .collect();
        frames.sort_by_key(|frame| frame.at_us);

        Self { frames, clock }
    }

    /// The second `at_us` falls in.
    pub fn second_of(&self, at_us: i64) -> i64 {
        let wall_us = self.clock.epoch_ms as i64 * 1_000 + at_us;
        wall_us.div_euclid(1_000_000)
    }

    /// Every second with a frame in it, oldest first; idle ones are not
    /// listed.
    pub fn seconds(&self) -> Vec<Second> {
        let mut seconds: Vec<Second> = Vec::new();
        for frame in &self.frames {
            let t = self.second_of(frame.at_us);
            let second = match seconds.last_mut() {
                Some(second) if second.t == t => second,
                _ => {
                    seconds.push(Second {
                        t,
                        frames: 0,
                        over: 0,
                        worst_us: 0,
                        worst_frame: frame.at_us,
                    });
                    seconds.last_mut().expect("just pushed")
                }
            };

            second.frames += 1;
            second.over += usize::from(frame.over());
            if frame.took_us > second.worst_us {
                second.worst_us = frame.took_us;
                second.worst_frame = frame.at_us;
            }
        }
        seconds
    }

    /// The frames that began in second `t`.
    pub fn frames_in(&self, t: i64) -> impl Iterator<Item = &Frame> {
        self.frames.iter().filter(move |frame| self.second_of(frame.at_us) == t)
    }

    /// Second `t` as a person reads it: `14:03:27`, or `27 s` into the trace.
    pub fn second_named(&self, t: i64) -> String {
        if self.clock.epoch_ms == 0 {
            return format!("{t} s");
        }
        self.local(t * 1_000_000 - self.clock.epoch_ms as i64 * 1_000, "%H:%M:%S")
    }

    /// When `at_us` was, to the millisecond.
    pub fn when(&self, at_us: i64) -> String {
        if self.clock.epoch_ms == 0 {
            return format!("{:.3} s", at_us as f64 / 1_000_000.0);
        }
        self.local(at_us, "%H:%M:%S%.3f")
    }

    fn local(&self, at_us: i64, format: &str) -> String {
        chrono::DateTime::from_timestamp_micros(self.clock.epoch_ms as i64 * 1_000 + at_us)
            .map(|when| when.with_timezone(&chrono::Local).format(format).to_string())
            .unwrap_or_default()
    }

    pub fn frame(&self, at_us: i64) -> Option<&Frame> {
        self.frames.iter().find(|frame| frame.at_us == at_us)
    }

    /// What the application recorded towards `frame`: from where the frame
    /// before it ended to where it ends, oldest first. The first frame of a
    /// capture has nothing before it, and gets what happened during it.
    pub fn work<'a>(&self, frame: &Frame, log: &'a TraceLog) -> Vec<&'a Span> {
        let since = self
            .frames
            .iter()
            .take_while(|earlier| earlier.at_us < frame.at_us)
            .last()
            .map_or(frame.at_us, Frame::end_us);
        let until = frame.end_us();

        log.iter()
            .filter(|span| {
                let at = span.at as i64;
                at > since && at <= until
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use guinea_devtools_protocol::{TraceBatch, TracePoint};

    use super::*;

    const FREQUENCY: u64 = 10_000_000;
    const ANCHOR_QPC: u64 = 50_000_000_000;

    fn timeline() -> Timeline {
        Timeline::new(ClockAnchor {
            qpc: ANCHOR_QPC,
            qpc_frequency: FREQUENCY,
            trace_us: 5_000,
            puffin_ns: None,
            ui_thread: 7,
        })
        .expect("a clock with QPC")
    }

    fn qpc_at(trace_us: i64) -> u64 {
        (ANCHOR_QPC as i64 + (trace_us - 5_000) * (FREQUENCY as i64 / 1_000_000)) as u64
    }

    fn captured(trace_us: i64, took_us: u64) -> Captured {
        Captured {
            took_us,
            qpc: qpc_at(trace_us),
            thread: 7,
            ..Captured::default()
        }
    }

    #[test]
    fn qpc_lands_on_the_trace_through_the_anchor() {
        let timeline = timeline();

        assert_eq!(timeline.trace_us(ANCHOR_QPC), 5_000);
        assert_eq!(timeline.trace_us(ANCHOR_QPC + 10_000), 6_000);
        assert_eq!(timeline.trace_us(ANCHOR_QPC - 90_000), -4_000);
        assert!(
            Timeline::new(ClockAnchor::default()).is_none(),
            "no QPC frequency, nothing to convert by"
        );
    }

    #[test]
    fn frames_go_to_the_wall_clock_second_they_began_in_and_idle_seconds_are_not_listed() {
        let clock = Clock {
            epoch_ms: 1_700_000_000_500,
        };
        let start = 1_700_000_000;
        let profile = Profile::new(
            timeline(),
            clock,
            &[
                captured(100_000, 4_000),
                captured(400_000, 6_000),
                captured(600_000, 20_000),
                captured(700_000, 31_000),
                captured(3_700_000, 5_000),
                Captured {
                    took_us: 99_000,
                    ..Captured::default()
                },
            ],
        );

        assert_eq!(profile.frames.len(), 5, "a frame with no QPC is left out");
        assert_eq!(
            profile.seconds(),
            [
                Second {
                    t: start,
                    frames: 2,
                    over: 0,
                    worst_us: 6_000,
                    worst_frame: 400_000,
                },
                Second {
                    t: start + 1,
                    frames: 2,
                    over: 2,
                    worst_us: 31_000,
                    worst_frame: 700_000,
                },
                Second {
                    t: start + 4,
                    frames: 1,
                    over: 0,
                    worst_us: 5_000,
                    worst_frame: 3_700_000,
                },
            ]
        );
        assert_eq!(
            profile.frames_in(start + 1).map(|frame| frame.at_us).collect::<Vec<_>>(),
            [600_000, 700_000]
        );
    }

    #[test]
    fn without_a_wall_clock_seconds_count_from_the_trace() {
        let profile = Profile::new(timeline(), Clock::default(), &[captured(2_500_000, 1_000)]);

        assert_eq!(profile.seconds()[0].t, 2);
        assert_eq!(profile.second_named(2), "2 s");
        assert_eq!(profile.when(2_500_000), "2.500 s");
    }

    #[test]
    fn with_a_wall_clock_seconds_read_as_the_time_of_day() {
        let epoch_ms = 1_700_000_000_000;
        let profile = Profile::new(timeline(), Clock { epoch_ms }, &[]);
        let local = |at_us: i64, format: &str| {
            chrono::DateTime::from_timestamp_micros(epoch_ms as i64 * 1_000 + at_us)
                .expect("in range")
                .with_timezone(&chrono::Local)
                .format(format)
                .to_string()
        };

        assert_eq!(profile.second_named(1_700_000_003), local(3_000_000, "%H:%M:%S"));
        assert_eq!(profile.when(3_250_000), local(3_250_000, "%H:%M:%S%.3f"));
    }

    #[test]
    fn a_frame_gets_what_was_recorded_since_the_frame_before_it_ended() {
        let profile = Profile::new(
            timeline(),
            Clock::default(),
            &[captured(500_000, 10_000), captured(600_000, 20_000)],
        );
        let mut log = TraceLog::default();
        let note = |id: u64, at: u64| Span {
            id,
            parent: None,
            at,
            took: None,
            point: TracePoint::Note { text: format!("{id}") },
        };
        log.absorb(TraceBatch {
            spans: vec![
                note(1, 50),
                note(2, 505_000),
                note(3, 550_000),
                note(4, 590_000),
                note(5, 700_000),
            ],
            ends: Vec::new(),
            dropped: 0,
        });

        let ids = |frame: &Frame| profile.work(frame, &log).iter().map(|span| span.id).collect::<Vec<_>>();

        assert_eq!(ids(&profile.frames[0]), [2], "the first frame: only what happened during it");
        assert_eq!(ids(&profile.frames[1]), [3, 4]);
    }
}
