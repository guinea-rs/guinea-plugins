//! A record's time, by the clock of the machine devtools run on.

/// What an application's `at` counts from.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clock {
    /// Milliseconds since the Unix epoch; zero when the application did not
    /// say.
    pub epoch_ms: u64,
}

impl Clock {
    fn local(self, at: u64) -> Option<chrono::DateTime<chrono::Local>> {
        if self.epoch_ms == 0 {
            return None;
        }

        let epoch = chrono::DateTime::from_timestamp_millis(self.epoch_ms as i64)?;
        let when = epoch + chrono::Duration::microseconds(at as i64);
        Some(when.with_timezone(&chrono::Local))
    }

    /// Hour and minute, for a list.
    pub fn short(self, at: u64) -> String {
        match self.local(at) {
            Some(when) => when.format("%H:%M").to_string(),
            None => since_start(at),
        }
    }

    /// To the millisecond, with how long the application had been running.
    pub fn long(self, at: u64) -> String {
        match self.local(at) {
            Some(when) => format!(
                "at {} · {} since the application started",
                when.format("%H:%M:%S%.3f"),
                since_start(at)
            ),
            None => format!("{} since the application started", since_start(at)),
        }
    }
}

pub fn since_start(at: u64) -> String {
    format!("{:.3} s", at as f64 / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_time_reads_by_the_clock_when_the_application_gave_one() {
        let unknown = Clock { epoch_ms: 0 };
        assert_eq!(unknown.short(1_500_000), "1.500 s");
        assert_eq!(unknown.long(1_500_000), "1.500 s since the application started");

        let epoch_ms = 1_700_000_000_000;
        let known = Clock { epoch_ms };
        let expected = chrono::DateTime::from_timestamp_millis(epoch_ms as i64 + 61_000)
            .expect("in range")
            .with_timezone(&chrono::Local);

        assert_eq!(known.short(61_000_000), expected.format("%H:%M").to_string());
        assert_eq!(
            known.long(61_000_000),
            format!(
                "at {} · 61.000 s since the application started",
                expected.format("%H:%M:%S%.3f")
            )
        );
    }
}
