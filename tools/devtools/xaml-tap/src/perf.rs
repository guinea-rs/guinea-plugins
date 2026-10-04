//! The UI thread's frames, from WinUI's own performance events.
//!
//! WinUI 3 writes frames, measure and arrange to the `Microsoft-Windows-XAML`
//! ETW provider. An ordinary session needs an administrator; a private one
//! does not, but only sees its own process - which is why this lives in the
//! tap. While devtools are connected the session writes into a ring file of
//! a few megabytes, so it always holds the last seconds. A capture stops it,
//! reads the ring into frames, and starts it again.

use std::path::PathBuf;
use std::sync::Mutex;

use guinea_devtools_protocol::native::{Frame, Pass};
use uniproc_etw::{Enable, Options, Session, Timestamps};

const XAML: u128 = 0x531a35ab_63ce_4bcf_aa98_f88c7a89e455;
const VERBOSE: u8 = 5;
const RING_MB: u32 = 32;
const KEPT_FRAMES: usize = 600;

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

fn name() -> String {
    format!("guinea-xaml-perf-{}", std::process::id())
}

fn ring() -> PathBuf {
    std::env::temp_dir().join(format!("{}.etl", name()))
}

/// Starts recording, unless it already is.
pub fn start() -> Result<(), String> {
    let mut session = SESSION.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.is_some() {
        return Ok(());
    }

    let started = Session::start(&name(), &Options::private_in_proc(ring(), RING_MB))
        .map_err(|error| format!("the ETW session did not start: {error}"))?;
    started
        .enable(
            XAML,
            Enable {
                keywords: u64::MAX,
                level: VERBOSE,
            },
        )
        .map_err(|error| format!("the XAML provider was not enabled: {error}"))?;

    *session = Some(started);
    Ok(())
}

/// Stops recording, when devtools go away, and removes the ring - up to 32
/// MB that nobody will read.
pub fn stop() {
    halt();
    let _ = std::fs::remove_file(ring());
}

fn halt() {
    drop(SESSION.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).take());
}

/// What the ring holds now, as frames; recording goes on afterwards.
pub fn capture() -> Result<Vec<Frame>, String> {
    halt();
    let frames = read(&ring());
    start()?;
    frames
}

fn read(path: &std::path::Path) -> Result<Vec<Frame>, String> {
    let mut frames = Frames::default();
    let clock = uniproc_etw::read_file(path, Timestamps::Raw, |event| {
        if event.provider() != XAML || event.id() == 0 {
            return;
        }
        let Some(task) = event.task_name().or(event.event_name()) else {
            return;
        };
        frames.see(Seen {
            at: event.timestamp(),
            thread: event.thread_id(),
            task: task.trim(),
            opcode: event.opcode_name().unwrap_or_default().trim(),
            element: event.number("ElementId"),
        });
    })
    .map_err(|error| format!("reading the recording failed: {error}"))?;

    Ok(frames.finish(clock.frequency))
}

/// One XAML event, as much of it as frames are made of.
struct Seen<'a> {
    /// Raw QPC ticks.
    at: i64,
    thread: u32,
    task: &'a str,
    opcode: &'a str,
    element: Option<u64>,
}

#[derive(Default)]
struct Frames {
    open: Option<Open>,
    done: Vec<Drawn>,
}

struct Open {
    at: i64,
    thread: u32,
    open_passes: Vec<(u64, &'static str, i64)>,
    measure: i64,
    arrange: i64,
    passes: Vec<(u64, &'static str, i64, i64)>,
}

struct Drawn {
    at: i64,
    took: i64,
    thread: u32,
    measure: i64,
    arrange: i64,
    passes: Vec<(u64, &'static str, i64, i64)>,
}

impl Frames {
    fn see(&mut self, seen: Seen<'_>) {
        let kind = match seen.task {
            "MeasureElement" => "measure",
            "ArrangeElement" => "arrange",
            _ => "",
        };

        match (seen.task, seen.opcode) {
            ("Frame", "Start") => {
                self.open = Some(Open {
                    at: seen.at,
                    thread: seen.thread,
                    open_passes: Vec::new(),
                    measure: 0,
                    arrange: 0,
                    passes: Vec::new(),
                });
            }
            ("Frame", "Stop") => {
                if let Some(open) = self.open.take() {
                    self.done.push(Drawn {
                        at: open.at,
                        took: seen.at - open.at,
                        thread: open.thread,
                        measure: open.measure,
                        arrange: open.arrange,
                        passes: open.passes,
                    });
                }
            }
            ("MeasureElement" | "ArrangeElement", "Start") => {
                let Some(open) = self.open.as_mut() else { return };
                open.open_passes.push((seen.element.unwrap_or(0), kind, seen.at));
            }
            ("MeasureElement" | "ArrangeElement", "Stop") => {
                let Some(open) = self.open.as_mut() else { return };
                let element = seen.element.unwrap_or(0);

                let Some(at) = open
                    .open_passes
                    .iter()
                    .rposition(|(open_element, open_kind, _)| *open_element == element && *open_kind == kind)
                else {
                    return;
                };
                let (_, _, started) = open.open_passes.remove(at);
                let took = seen.at - started;

                let outermost = !open.open_passes.iter().any(|(_, open_kind, _)| *open_kind == kind);
                if outermost {
                    if kind == "measure" {
                        open.measure += took;
                    } else {
                        open.arrange += took;
                    }
                }

                open.passes.push((element, kind, started, took));
            }
            _ => {}
        }
    }

    /// The frames seen, the last [`KEPT_FRAMES`] of them, with QPC ticks of
    /// `frequency` a second turned into microseconds.
    fn finish(self, frequency: i64) -> Vec<Frame> {
        let frequency = frequency.max(1) as i128;
        let micros = |ticks: i64| (ticks.max(0) as i128 * 1_000_000 / frequency) as u64;
        let first = self.done.first().map_or(0, |frame| frame.at);
        let kept = self.done.len().saturating_sub(KEPT_FRAMES);

        self.done
            .into_iter()
            .skip(kept)
            .map(|drawn| {
                let mut passes: Vec<Pass> = drawn
                    .passes
                    .into_iter()
                    .map(|(element, kind, started, took)| Pass {
                        element,
                        kind: kind.to_string(),
                        took_us: micros(took),
                        at_us: micros(started - drawn.at),
                    })
                    .collect();
                passes.sort_by_key(|pass| std::cmp::Reverse(pass.took_us));

                Frame {
                    at_us: micros(drawn.at - first),
                    took_us: micros(drawn.took),
                    measure_us: micros(drawn.measure),
                    arrange_us: micros(drawn.arrange),
                    passes,
                    qpc: drawn.at.max(0) as u64,
                    thread: drawn.thread,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(frames: &mut Frames, at: i64, task: &str, opcode: &str, element: Option<u64>) {
        frames.see(Seen {
            at,
            thread: 7,
            task,
            opcode,
            element,
        });
    }

    #[test]
    fn a_frame_keeps_its_raw_start_its_thread_and_every_pass_where_it_began() {
        const FREQUENCY: i64 = 3_000_000;
        let start = 81_000_000;
        let mut frames = Frames::default();

        seen(&mut frames, start, "Frame", "Start", None);
        for element in 0..20u64 {
            let began = start + 300 + element as i64 * 30;
            seen(&mut frames, began, "MeasureElement", "Start", Some(element));
            seen(&mut frames, began + 15, "MeasureElement", "Stop", Some(element));
        }
        seen(&mut frames, start + 30_000, "Frame", "Stop", None);

        let frames = frames.finish(FREQUENCY);
        let frame = &frames[0];

        assert_eq!(frame.qpc, start as u64);
        assert_eq!(frame.thread, 7);
        assert_eq!(frame.took_us, 10_000, "30 000 ticks at 3 MHz");
        assert_eq!(frame.measure_us, 100, "twenty passes of 5 µs");
        assert_eq!(frame.passes.len(), 20, "every pass, not the costliest few");

        let last = frame.passes.iter().find(|pass| pass.element == 19).expect("the last pass");
        assert_eq!(last.kind, "measure");
        assert_eq!(last.took_us, 5);
        assert_eq!(last.at_us, 290, "(300 + 19 × 30) ticks after the frame began");
    }
}
