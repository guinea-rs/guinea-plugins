//! What the application's async runtime did, as a line on the timeline.

use egui::Rect;
use guinea_devtools_model::runtime::{Busy, RuntimeSample};

use super::memory;
use crate::theme;

/// Draws `points` - where each stretch between two readings begins across,
/// and how many workers were busy in it - none at the bottom of `rect`, all
/// `workers` at the top.
pub fn line(painter: &egui::Painter, rect: Rect, points: &[(f32, f64)], workers: u32) {
    let all = f64::from(workers.max(1));
    let shares: Vec<(f32, f32)> = points
        .iter()
        .map(|(x, busy)| (*x, (busy / all) as f32))
        .collect();
    memory::steps(
        painter,
        rect,
        &shares,
        theme::CYAN,
        (&format!("{workers} workers"), "0"),
    );
}

/// What a reading says, and how many workers were busy around it, for a
/// tooltip.
pub fn said(sample: RuntimeSample, busy: Option<f64>) -> String {
    let workers = match busy {
        Some(busy) => format!("{busy:.1} of {} workers busy", sample.workers),
        None => format!("{} workers", sample.workers),
    };
    format!("{workers}{}", waiting(sample))
}

/// What the runtime did over a selected stretch, `busy` told by the readings
/// around it and `last` the reading at its end.
pub fn summary(busy: Busy, last: RuntimeSample) -> String {
    format!(
        "{:.1} of {} workers busy over the {} ms read around it{}",
        busy.busy_us as f64 / busy.over_us.max(1) as f64,
        busy.workers,
        busy.over_us / 1000,
        waiting(last)
    )
}

fn waiting(sample: RuntimeSample) -> String {
    format!(
        " · {} tasks alive · {} queued",
        sample.alive_tasks, sample.queued
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RuntimeSample {
        RuntimeSample {
            at_us: 0,
            alive_tasks: 12,
            queued: 3,
            workers: 8,
            busy_us: 0,
            parks: 0,
        }
    }

    #[test]
    fn a_reading_says_how_many_workers_were_busy_and_what_waits() {
        assert_eq!(
            [said(sample(), Some(1.375)), said(sample(), None)],
            [
                "1.4 of 8 workers busy · 12 tasks alive · 3 queued",
                "8 workers · 12 tasks alive · 3 queued",
            ]
        );
    }

    #[test]
    fn a_stretch_says_how_busy_the_workers_were_over_the_readings_around_it() {
        assert_eq!(
            summary(
                Busy {
                    workers: 8,
                    busy_us: 330_000,
                    over_us: 300_000,
                },
                sample()
            ),
            "1.1 of 8 workers busy over the 300 ms read around it · 12 tasks alive · 3 queued"
        );
    }
}
