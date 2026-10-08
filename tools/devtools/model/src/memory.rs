//! How much memory an application's process held, read by devtools from
//! outside it a few times a second.

use std::collections::VecDeque;

/// How many readings are kept: at ten a second, an hour of them.
pub const KEPT_READINGS: usize = 36_000;

/// What the process held at one moment, in bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemorySample {
    /// `QueryPerformanceCounter` when it was read.
    pub qpc: u64,
    /// Committed to the process alone: what it allocated.
    pub private: u64,
    /// Of the process in physical memory.
    pub working_set: u64,
}

/// One process's readings, oldest first.
#[derive(Clone, Debug, Default)]
pub struct Memory {
    samples: VecDeque<MemorySample>,
}

impl Memory {
    /// Keeps `sample`, unless it is no newer than the newest kept.
    pub fn push(&mut self, sample: MemorySample) {
        if self
            .samples
            .back()
            .is_some_and(|newest| newest.qpc >= sample.qpc)
        {
            return;
        }
        if self.samples.len() >= KEPT_READINGS {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn samples(&self) -> &VecDeque<MemorySample> {
        &self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(qpc: u64) -> MemorySample {
        MemorySample {
            qpc,
            private: qpc * 2,
            working_set: qpc,
        }
    }

    #[test]
    fn readings_are_kept_oldest_first_and_only_the_newest_of_them() {
        let mut memory = Memory::default();

        memory.push(at(1));
        memory.push(at(3));
        memory.push(at(2));
        memory.push(at(3));
        for qpc in 4..KEPT_READINGS as u64 + 4 {
            memory.push(at(qpc));
        }

        assert_eq!(memory.samples().len(), KEPT_READINGS);
        assert_eq!(memory.samples().front(), Some(&at(4)));
        assert_eq!(memory.samples().back(), Some(&at(KEPT_READINGS as u64 + 3)));
    }
}
