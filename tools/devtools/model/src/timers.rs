//! Every timer a session reported, kept after it stopped so that its ticks
//! still have a name.

use std::collections::{BTreeMap, BTreeSet};

use guinea_devtools_protocol::{Declared, Timer};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Timers {
    by_id: BTreeMap<u64, Timer>,
    running: BTreeSet<u64>,
}

impl Timers {
    /// Takes in the timers a snapshot says are running.
    pub fn note(&mut self, running: &[Timer]) {
        self.running = running.iter().map(|timer| timer.id).collect();

        for timer in running {
            self.by_id.insert(timer.id, timer.clone());
        }
    }

    /// Learns of timer `id` from one of its ticks, unless a snapshot already
    /// told more of it: a timer that started and stopped between two of them
    /// is known by its ticks alone.
    pub fn hear(&mut self, id: u64, name: Option<&str>, declared: Option<&Declared>) {
        self.by_id.entry(id).or_insert_with(|| Timer {
            id,
            name: name.map(str::to_string),
            declared: declared.cloned(),
            ..Timer::default()
        });
    }

    pub fn get(&self, id: u64) -> Option<&Timer> {
        self.by_id.get(&id)
    }

    /// Whether the last snapshot said it runs.
    pub fn runs(&self, id: u64) -> bool {
        self.running.contains(&id)
    }

    /// Which timers are the same one: the place they were set up, or the id
    /// when that is not known.
    pub fn site(&self, id: u64) -> String {
        match self.get(id) {
            Some(timer) => site(timer),
            None => format!("#{id}"),
        }
    }

    /// What a timer is called on screen: its name, or where it was set up.
    pub fn label(&self, id: u64) -> String {
        match self.get(id) {
            Some(timer) => label(timer),
            None => format!("#{id}"),
        }
    }

    /// The timers set up at `site`, running or not.
    pub fn at<'a>(&'a self, wanted: &'a str) -> impl Iterator<Item = &'a Timer> {
        self.by_id
            .values()
            .filter(move |timer| site(timer) == wanted)
    }
}

pub fn site(timer: &Timer) -> String {
    match &timer.declared {
        Some(declared) => format!("{}:{}:{}", declared.file, declared.line, declared.column),
        None => format!("#{}", timer.id),
    }
}

pub fn label(timer: &Timer) -> String {
    if let Some(name) = &timer.name {
        return name.clone();
    }

    match &timer.declared {
        Some(declared) => {
            let file = std::path::Path::new(&declared.file)
                .file_name()
                .map_or_else(
                    || declared.file.clone(),
                    |name| name.to_string_lossy().into_owned(),
                );
            format!("{file}:{}", declared.line)
        }
        None => format!("#{}", timer.id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_timer_known_by_its_ticks_alone_is_one_place_under_its_name_and_not_running() {
        let mut timers = Timers::default();
        let place = Declared {
            file: "C:/app/src/pages/activity.rs".into(),
            line: 7,
            column: 1,
            found: true,
        };

        timers.hear(43, Some("activity-flush"), Some(&place));
        timers.hear(45, Some("activity-flush"), Some(&place));
        timers.note(&[timer(1, Some("housekeeping"), 48)]);
        timers.hear(1, Some("renamed"), None);

        assert_eq!(
            (timers.label(43), timers.label(45)),
            ("activity-flush".to_string(), "activity-flush".to_string())
        );
        assert_eq!(timers.site(43), timers.site(45));
        assert_eq!(timers.at(&timers.site(43)).count(), 2);
        assert!(!timers.runs(43));
        assert_eq!(
            timers.label(1),
            "housekeeping",
            "a snapshot knows more than a tick"
        );
    }

    fn timer(id: u64, name: Option<&str>, line: u32) -> Timer {
        Timer {
            id,
            name: name.map(str::to_string),
            declared: Some(Declared {
                file: "C:/app/src/startup.rs".into(),
                line,
                column: 9,
                found: true,
            }),
            period_ms: 5_000,
            ..Timer::default()
        }
    }

    #[test]
    fn a_timer_is_known_by_its_place_and_called_by_its_name() {
        let mut timers = Timers::default();
        timers.note(&[
            timer(1, Some("housekeeping"), 48),
            timer(2, None, 60),
            timer(3, None, 60),
        ]);

        assert_eq!(timers.label(1), "housekeeping");
        assert_eq!(timers.label(2), "startup.rs:60");
        assert_eq!(timers.site(2), timers.site(3), "two windows, one place");
        assert_ne!(timers.site(1), timers.site(2));
        assert_eq!(timers.at(&timers.site(2)).count(), 2);

        assert!(timers.runs(1));

        timers.note(&[]);
        assert!(!timers.runs(1));
        assert_eq!(
            timers.label(1),
            "housekeeping",
            "a stopped timer keeps its name"
        );
        assert_eq!(timers.label(9), "#9");
    }
}
