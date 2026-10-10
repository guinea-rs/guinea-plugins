//! What an application's async runtime did, as its own counters told it a
//! few times a second.

use std::collections::VecDeque;

use guinea_devtools_protocol::RuntimeReading;

/// How many readings are kept: at ten a second, an hour of them.
pub const KEPT_READINGS: usize = 36_000;

/// The runtime at one moment, its workers summed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSample {
    /// On the trace's clock, in microseconds.
    pub at_us: i64,
    pub alive_tasks: u64,
    /// Waiting in the queue every worker takes from.
    pub queued: u64,
    pub workers: u32,
    /// How long its workers had been busy, together, since it started.
    pub busy_us: u64,
    /// How many times its workers had gone to sleep, together.
    pub parks: u64,
}

/// How busy the workers were over a stretch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Busy {
    pub workers: u32,
    /// Busy, all workers together.
    pub busy_us: u64,
    /// How long the readings it is told by lie apart: the stretch, widened
    /// to the readings around it.
    pub over_us: u64,
}

/// One application's readings, oldest first.
#[derive(Clone, Debug, Default)]
pub struct Runtime {
    samples: VecDeque<RuntimeSample>,
}

impl Runtime {
    /// Keeps each of `readings` that is newer than the newest kept.
    pub fn absorb(&mut self, readings: Vec<RuntimeReading>) {
        for reading in readings {
            let at_us = reading.at as i64;
            if self
                .samples
                .back()
                .is_some_and(|newest| newest.at_us >= at_us)
            {
                continue;
            }
            if self.samples.len() >= KEPT_READINGS {
                self.samples.pop_front();
            }
            let (busy_us, parks) = reading
                .workers
                .iter()
                .fold((0, 0), |(busy, parks), worker| {
                    (busy + worker.busy_us, parks + worker.parks)
                });
            self.samples.push_back(RuntimeSample {
                at_us,
                alive_tasks: reading.alive_tasks,
                queued: reading.queued,
                workers: reading.workers.len() as u32,
                busy_us,
                parks,
            });
        }
    }

    pub fn samples(&self) -> &VecDeque<RuntimeSample> {
        &self.samples
    }

    /// The last reading at or before `at_us`.
    pub fn at(&self, at_us: i64) -> Option<RuntimeSample> {
        let past = self.samples.partition_point(|sample| sample.at_us <= at_us);
        self.samples.get(past.checked_sub(1)?).copied()
    }

    /// How busy the workers were from `from_us` to `to_us`, told by the last
    /// reading at or before the one and the first at or after the other.
    pub fn busy(&self, from_us: i64, to_us: i64) -> Option<Busy> {
        let before = self
            .samples
            .partition_point(|sample| sample.at_us <= from_us);
        let first = self.samples.get(before.checked_sub(1)?)?;
        let after = self.samples.partition_point(|sample| sample.at_us < to_us);
        let last = self.samples.get(after)?;
        (last.at_us > first.at_us).then(|| Busy {
            workers: last.workers,
            busy_us: last.busy_us.saturating_sub(first.busy_us),
            over_us: (last.at_us - first.at_us) as u64,
        })
    }

    /// How many workers were busy on average between each two readings that
    /// lie apart over some of `from_us` up to `to_us`: from when the earlier
    /// of the two was taken, how many.
    pub fn busy_between(&self, from_us: i64, to_us: i64) -> Vec<(i64, f64)> {
        let first = self
            .samples
            .partition_point(|sample| sample.at_us < from_us);
        let past = self.samples.partition_point(|sample| sample.at_us < to_us);
        let past = (past + 1).min(self.samples.len());
        let read = self.samples.range(first.saturating_sub(1)..past.max(first));
        read.clone()
            .zip(read.skip(1))
            .map(|(earlier, later)| {
                let busy = later.busy_us.saturating_sub(earlier.busy_us) as f64;
                (earlier.at_us, busy / (later.at_us - earlier.at_us) as f64)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use guinea_devtools_protocol::WorkerReading;

    use super::*;

    fn reading(at: u64, busy: [u64; 2]) -> RuntimeReading {
        RuntimeReading {
            at,
            alive_tasks: 5,
            queued: 1,
            workers: busy
                .into_iter()
                .map(|busy_us| WorkerReading { busy_us, parks: 3 })
                .collect(),
        }
    }

    #[test]
    fn a_reading_sums_its_workers_and_only_newer_ones_are_kept() {
        let mut runtime = Runtime::default();

        runtime.absorb(vec![reading(100, [10, 20]), reading(200, [30, 40])]);
        runtime.absorb(vec![reading(150, [0, 0]), reading(300, [50, 60])]);

        assert_eq!(
            runtime.samples().iter().copied().collect::<Vec<_>>(),
            [(100, 30), (200, 70), (300, 110)].map(|(at_us, busy_us)| RuntimeSample {
                at_us,
                alive_tasks: 5,
                queued: 1,
                workers: 2,
                busy_us,
                parks: 6,
            })
        );
        assert_eq!(runtime.at(250).map(|sample| sample.at_us), Some(200));
        assert_eq!(runtime.at(300).map(|sample| sample.at_us), Some(300));
        assert_eq!(runtime.at(99), None);
    }

    #[test]
    fn how_busy_the_workers_were_is_told_by_the_readings_around_a_stretch() {
        let mut runtime = Runtime::default();
        runtime.absorb(vec![
            reading(100_000, [0, 0]),
            reading(200_000, [100_000, 50_000]),
            reading(300_000, [200_000, 150_000]),
        ]);

        assert_eq!(
            runtime.busy(220_000, 280_000),
            Some(Busy {
                workers: 2,
                busy_us: 200_000,
                over_us: 100_000,
            })
        );
        assert_eq!(
            runtime.busy(150_000, 200_000),
            Some(Busy {
                workers: 2,
                busy_us: 150_000,
                over_us: 100_000,
            })
        );
        assert_eq!(runtime.busy(50_000, 150_000), None, "nothing read before");
        assert_eq!(runtime.busy(250_000, 350_000), None, "nothing read after");
        assert_eq!(
            runtime.busy_between(150_000, 400_000),
            [(100_000, 1.5), (200_000, 2.0)]
        );
        assert_eq!(runtime.busy_between(0, 150_000), [(100_000, 1.5)]);
        assert_eq!(runtime.busy_between(0, 50_000), []);
    }
}
