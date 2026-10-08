//! Frames on the trace's timeline, by the second they began in.
//!
//! The backend's frames come stamped with QPC; the trace counts microseconds
//! since it began. The application read both at one moment
//! ([`ClockAnchor`]), which is all it takes to put a frame among the records
//! that led to it.

use guinea_devtools_protocol::native::{Frame as Captured, Pass, Sample};
use guinea_devtools_protocol::{ClockAnchor, Span, TracePoint};
use serde::Serialize;

use crate::clock::Clock;
use crate::samples::{Origin, Sampled};
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

    /// Where the frame ended on the trace.
    pub fn end_us(&self) -> i64 {
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

/// What a thread ran in a stretch, as its samples tell it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cpu {
    /// The samples taken on it.
    pub samples: usize,
    /// How long it ran on a processor, in microseconds; `None` from a tap
    /// that counts no cycles.
    pub ran_us: Option<u64>,
}

/// The frames of one capture, placed on the trace's timeline, and the
/// stacks sampled meanwhile.
#[derive(Clone, Debug, Default)]
pub struct Profile<'a> {
    /// Oldest first.
    pub frames: Vec<Frame>,
    clock: Clock,
    timeline: Option<Timeline>,
    sampled: Option<&'a Sampled>,
}

impl<'a> Profile<'a> {
    /// With the stacks `sampled` holds.
    pub fn with_samples(self, sampled: &'a Sampled) -> Self {
        Self {
            sampled: Some(sampled),
            ..self
        }
    }

    /// The samples taken on `thread` from `from_us` up to `to_us`, oldest
    /// first: when each was taken, and its stack by function, the innermost
    /// call first.
    pub fn samples_between(&self, thread: u32, from_us: i64, to_us: i64) -> Vec<(i64, &'a [u32])> {
        let (Some(sampled), Some(timeline)) = (self.sampled, self.timeline) else {
            return Vec::new();
        };
        self.taken_on(thread, from_us, to_us)
            .filter_map(|sample| {
                let stack = sampled.stacks.get(sample.stack as usize)?;
                Some((timeline.trace_us(sample.qpc), stack.as_slice()))
            })
            .collect()
    }

    /// The samples taken on `thread` from `from_us` up to `to_us`; one from
    /// a tap that names no thread was taken on the UI thread.
    fn taken_on(&self, thread: u32, from_us: i64, to_us: i64) -> impl Iterator<Item = &'a Sample> {
        let ui = self.ui_thread();
        let taken: &'a [Sample] = match (self.sampled, self.timeline) {
            (Some(sampled), Some(timeline)) => {
                let at = |sample: &Sample| timeline.trace_us(sample.qpc);
                let first = sampled
                    .samples
                    .partition_point(|sample| at(sample) < from_us);
                let past = sampled.samples.partition_point(|sample| at(sample) < to_us);
                &sampled.samples[first..past.max(first)]
            }
            _ => &[],
        };
        taken
            .iter()
            .filter(move |sample| sample.thread == thread || sample.thread == 0 && thread == ui)
    }

    /// The thread the application runs its UI on, as its clock said.
    pub fn ui_thread(&self) -> u32 {
        self.timeline
            .map_or(0, |timeline| timeline.anchor.ui_thread as u32)
    }

    /// Every thread sampled, the UI thread first, with its name or none.
    pub fn threads(&self) -> Vec<(u32, &'a str)> {
        self.sampled.map_or_else(Vec::new, |sampled| {
            crate::samples::ui_first(sampled.threads(), self.ui_thread())
        })
    }

    /// What `thread` ran from `from_us` up to `to_us`, by its samples.
    pub fn cpu(&self, thread: u32, from_us: i64, to_us: i64) -> Cpu {
        let (samples, cycles) = self
            .taken_on(thread, from_us, to_us)
            .fold((0, 0u64), |(samples, cycles), sample| {
                (samples + 1, cycles + sample.cycles)
            });
        let per_second = self.sampled.map_or(0, Sampled::cycles_per_second);
        Cpu {
            samples,
            ran_us: (per_second > 0)
                .then(|| (u128::from(cycles) * 1_000_000 / u128::from(per_second)) as u64),
        }
    }

    /// The name of function `id` of a sampled stack.
    pub fn function(&self, id: u32) -> &'a str {
        self.sampled.map_or("", |sampled| sampled.function(id))
    }

    /// Whose code the sampled function `id` is.
    pub fn origin(&self, id: u32) -> Origin {
        self.sampled
            .map_or(Origin::Unknown, |sampled| sampled.origin(id))
    }

    /// The path of the module the sampled function `id` is in.
    pub fn module(&self, id: u32) -> Option<&'a str> {
        self.sampled.and_then(|sampled| sampled.module(id))
    }

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

        Self {
            frames,
            clock,
            timeline: Some(timeline),
            sampled: None,
        }
    }

    /// The second `at_us` falls in.
    pub fn second_of(&self, at_us: i64) -> i64 {
        let wall_us = self.clock.epoch_ms as i64 * 1_000 + at_us;
        wall_us.div_euclid(1_000_000)
    }

    /// Where second `t` begins on the trace, in microseconds.
    pub fn second_starts(&self, t: i64) -> i64 {
        t * 1_000_000 - self.clock.epoch_ms as i64 * 1_000
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
        self.frames
            .iter()
            .filter(move |frame| self.second_of(frame.at_us) == t)
    }

    /// Second `t` as a person reads it: `14:03:27`, or `27 s` into the trace.
    pub fn second_named(&self, t: i64) -> String {
        if self.clock.epoch_ms == 0 {
            return format!("{t} s");
        }
        self.local(self.second_starts(t), "%H:%M:%S")
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
            .map(|when| {
                when.with_timezone(&chrono::Local)
                    .format(format)
                    .to_string()
            })
            .unwrap_or_default()
    }

    pub fn frame(&self, at_us: i64) -> Option<&Frame> {
        self.frames.iter().find(|frame| frame.at_us == at_us)
    }

    /// What the application recorded towards `frame`: from where the frame
    /// before it ended to where it ends, oldest first. The first frame of a
    /// capture has nothing before it, and gets what happened during it.
    /// A source waiting for its next item is left out: the wait is not work,
    /// though what the source does under it is.
    pub fn work<'log>(&self, frame: &Frame, log: &'log TraceLog) -> Vec<&'log Span> {
        let since = self
            .frames
            .iter()
            .take_while(|earlier| earlier.at_us < frame.at_us)
            .last()
            .map_or(frame.at_us, Frame::end_us);
        let until = frame.end_us();
        if until < 0 {
            return Vec::new();
        }

        log.during(since.saturating_add(1).max(0) as u64, until as u64)
            .filter(|span| !matches!(span.point, TracePoint::Pull { .. }))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use guinea_devtools_protocol::TraceBatch;

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
        assert_eq!(profile.second_starts(start + 1), 500_000);
        assert_eq!(
            profile.second_of(profile.second_starts(start + 4)),
            start + 4
        );
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
            profile
                .frames_in(start + 1)
                .map(|frame| frame.at_us)
                .collect::<Vec<_>>(),
            [600_000, 700_000]
        );
    }

    #[test]
    fn without_a_wall_clock_seconds_count_from_the_trace() {
        let profile = Profile::new(timeline(), Clock::default(), &[captured(2_500_000, 1_000)]);

        assert_eq!(profile.seconds()[0].t, 2);
        assert_eq!(profile.second_starts(2), 2_000_000);
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

        assert_eq!(
            profile.second_named(1_700_000_003),
            local(3_000_000, "%H:%M:%S")
        );
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
            point: TracePoint::Note {
                text: format!("{id}"),
            },
            thread: 0,
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

        let ids = |frame: &Frame| {
            profile
                .work(frame, &log)
                .iter()
                .map(|span| span.id)
                .collect::<Vec<_>>()
        };

        assert_eq!(
            ids(&profile.frames[0]),
            [2],
            "the first frame: only what happened during it"
        );
        assert_eq!(ids(&profile.frames[1]), [3, 4]);
    }

    #[test]
    fn samples_are_placed_on_the_trace_and_picked_by_when_they_were_taken() {
        use guinea_devtools_protocol::native::{Sample, Stacks};

        let mut sampled = Sampled::default();
        sampled.absorb(Stacks {
            functions: vec!["main".into(), "draw".into()],
            stacks: vec![vec![0], vec![1, 0]],
            samples: [(100_000, 0), (101_000, 1), (102_000, 1), (103_000, 0)]
                .into_iter()
                .map(|(at, stack)| Sample {
                    qpc: qpc_at(at),
                    stack,
                    ..Sample::default()
                })
                .collect(),
            ..Stacks::default()
        });
        let profile = Profile::new(timeline(), Clock::default(), &[]).with_samples(&sampled);

        let picked: Vec<(i64, Vec<&str>)> = profile
            .samples_between(7, 101_000, 103_000)
            .into_iter()
            .map(|(at, stack)| (at, stack.iter().map(|id| profile.function(*id)).collect()))
            .collect();

        assert_eq!(
            picked,
            [
                (101_000, vec!["draw", "main"]),
                (102_000, vec!["draw", "main"])
            ]
        );
    }

    #[test]
    fn each_thread_has_its_own_samples_and_what_it_ran_and_a_tap_that_names_none_samples_the_ui() {
        use guinea_devtools_protocol::native::{Sample, SampledThread, Stacks};

        let mut sampled = Sampled::default();
        sampled.absorb(Stacks {
            functions: vec!["main".into()],
            stacks: vec![vec![0]],
            samples: [
                (101_000, 7, 2_000),
                (102_000, 9, 3_000),
                (103_000, 9, 1_000),
                (104_000, 0, 0),
            ]
            .into_iter()
            .map(|(at, thread, cycles)| Sample {
                qpc: qpc_at(at),
                stack: 0,
                thread,
                cycles,
            })
            .collect(),
            threads: vec![SampledThread {
                id: 9,
                name: "worker".into(),
            }],
            cycles_per_second: 1_000_000,
            ..Stacks::default()
        });
        let profile = Profile::new(timeline(), Clock::default(), &[]).with_samples(&sampled);
        let at = |thread: u32| -> Vec<i64> {
            profile
                .samples_between(thread, 100_000, 110_000)
                .into_iter()
                .map(|(at, _)| at)
                .collect()
        };

        assert_eq!(at(9), [102_000, 103_000]);
        assert_eq!(at(7), [101_000, 104_000]);
        assert_eq!(profile.threads(), [(7, ""), (9, "worker")]);
        assert_eq!(
            profile.cpu(9, 100_000, 110_000),
            Cpu {
                samples: 2,
                ran_us: Some(4_000)
            }
        );
        assert_eq!(
            profile.cpu(9, 100_000, 102_500),
            Cpu {
                samples: 1,
                ran_us: Some(3_000)
            }
        );
    }

    #[test]
    fn a_source_waiting_for_its_next_item_is_not_work_but_what_it_does_under_the_wait_is() {
        let profile = Profile::new(timeline(), Clock::default(), &[captured(500_000, 10_000)]);
        let mut log = TraceLog::default();
        log.absorb(TraceBatch {
            spans: vec![
                Span {
                    id: 1,
                    parent: None,
                    at: 501_000,
                    took: Some(800_000),
                    point: TracePoint::Pull {
                        actor: "Agent".into(),
                        actor_id: 1,
                        output: "Streamed".into(),
                        source: 9,
                    },
                    thread: 0,
                },
                Span {
                    id: 2,
                    parent: Some(1),
                    at: 502_000,
                    took: Some(3_000),
                    point: TracePoint::Note {
                        text: "scan".into(),
                    },
                    thread: 0,
                },
            ],
            ends: Vec::new(),
            dropped: 0,
        });

        let ids: Vec<u64> = profile
            .work(&profile.frames[0], &log)
            .iter()
            .map(|span| span.id)
            .collect();

        assert_eq!(ids, [2]);
    }
}
