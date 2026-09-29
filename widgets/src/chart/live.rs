//! Where now is on a live chart's time axis, between the samples that say.

use std::time::{Duration, Instant};

/// The fastest a live chart redraws, whatever its width.
const FASTEST: Duration = Duration::from_micros(16_667);

/// The slowest: a chart whose pixels are seconds apart still looks in on its
/// clock now and then.
const SLOWEST: Duration = Duration::from_secs(1);

/// Now on the time axis: a sample's time and the instant it was taken for,
/// moved on by the clock since.
///
/// Set by the newest sample and left alone while the samples keep up, so the
/// chart moves at the clock's pace rather than jumping to each sample's time.
/// A sample ahead of it moves it on; one so far behind that it cannot be the
/// same timeline - a restart, a reset clock - starts it again.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Clock {
    anchor: Option<(f64, Instant)>,
}

impl Clock {
    /// Takes the newest sample, seen at `at`.
    pub fn saw(&mut self, newest: f64, window: f64, per_second: f64, at: Instant) {
        match self.now(per_second, at) {
            Some(now) if newest <= now && now - newest <= window => {}
            _ => self.anchor = Some((newest, at)),
        }
    }

    /// Where on the axis `at` is, once a sample has said.
    pub fn now(&self, per_second: f64, at: Instant) -> Option<f64> {
        self.anchor
            .map(|(t, since)| t + at.saturating_duration_since(since).as_secs_f64() * per_second)
    }

    pub fn forget(&mut self) {
        self.anchor = None;
    }
}

/// How long a window of `window` units spread over `pixels` device pixels
/// takes to move one of them.
pub(super) fn step(window: u64, per_second: f64, pixels: f32) -> Duration {
    if pixels <= 0.0 || per_second <= 0.0 {
        return SLOWEST;
    }
    let seconds = window as f64 / per_second / pixels as f64;
    Duration::from_secs_f64(seconds.max(0.0)).clamp(FASTEST, SLOWEST)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_moves_on_from_the_sample_that_set_it() {
        let start = Instant::now();
        let mut clock = Clock::default();
        assert_eq!(clock.now(1000.0, start), None);

        clock.saw(5_000.0, 60_000.0, 1000.0, start);
        assert_eq!(
            clock.now(1000.0, start + Duration::from_millis(250)),
            Some(5_250.0)
        );

        clock.saw(
            5_200.0,
            60_000.0,
            1000.0,
            start + Duration::from_millis(300),
        );
        assert_eq!(
            clock.now(1000.0, start + Duration::from_millis(300)),
            Some(5_300.0),
            "a sample that keeps up leaves it alone"
        );
    }

    #[test]
    fn a_sample_ahead_moves_it_on_and_one_from_another_timeline_restarts_it() {
        let start = Instant::now();
        let mut clock = Clock::default();
        clock.saw(5_000.0, 60_000.0, 1000.0, start);

        let later = start + Duration::from_millis(100);
        clock.saw(5_400.0, 60_000.0, 1000.0, later);
        assert_eq!(clock.now(1000.0, later), Some(5_400.0));

        clock.saw(1_000.0, 60_000.0, 1000.0, later);
        assert_eq!(
            clock.now(1000.0, later),
            Some(5_400.0),
            "a late sample within a window of it is only late"
        );

        clock.saw(10.0, 1_000.0, 1000.0, later);
        assert_eq!(clock.now(1000.0, later), Some(10.0));
    }

    #[test]
    fn a_step_is_one_pixel_of_the_window_within_bounds() {
        assert_eq!(step(60_000, 1000.0, 400.0), Duration::from_millis(150));
        assert_eq!(step(1_000, 1000.0, 400.0), FASTEST);
        assert_eq!(step(3_600_000, 1000.0, 400.0), SLOWEST);
        assert_eq!(step(60_000, 1000.0, 0.0), SLOWEST);
    }
}
