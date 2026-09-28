//! How things are called on screen.

use std::borrow::Cow;

use guinea_devtools_protocol::BusKind;

/// A type without its path, and its parameters without theirs:
/// `MetricsActor`, `GenericAgentActor<WindowsAgent>`, `List<Recent>`.
pub fn type_name(full: &str) -> Cow<'_, str> {
    if !full.contains('<') {
        let start = full.rfind("::").map_or(0, |at| at + 2);
        return Cow::Borrowed(&full[start..]);
    }

    let mut out = String::with_capacity(full.len());
    let mut path = 0;
    for (at, c) in full.char_indices() {
        if c.is_alphanumeric() || c == '_' || c == ':' {
            continue;
        }
        out.push_str(last_segment(&full[path..at]));
        out.push(c);
        path = at + c.len_utf8();
    }
    out.push_str(last_segment(&full[path..]));

    Cow::Owned(out)
}

fn last_segment(path: &str) -> &str {
    path.rfind("::").map_or(path, |at| &path[at + 2..])
}

/// Whether `full`, a type path from a snapshot, is the type `name` names.
pub fn names(name: &str, full: &str) -> bool {
    full == name
        || full
            .strip_suffix(name)
            .is_some_and(|head| head.ends_with("::"))
}

/// What a window is called: its label, or its place among the windows.
pub fn window_name(label: Option<&str>, index: usize) -> String {
    match label {
        Some(label) => label.to_string(),
        None => format!("window {}", index + 1),
    }
}

pub fn bus_name(bus: BusKind) -> &'static str {
    match bus {
        BusKind::Global => "the global bus",
        BusKind::Window => "the window bus",
    }
}

/// A duration given in microseconds.
///
/// Seconds once it is seconds: work that waits - a request, a retry, a poll
/// that found nothing - runs for tens of them, and `30252.9 ms` is a number
/// nobody reads as half a minute.
pub fn took(micros: u64) -> String {
    match micros {
        seconds if seconds >= 1_000_000 => format!("{:.1} s", seconds as f64 / 1_000_000.0),
        millis if millis >= 1_000 => format!("{:.1} ms", millis as f64 / 1_000.0),
        micros => format!("{micros} µs"),
    }
}

/// How often something happens, given in milliseconds.
pub fn period(millis: u64) -> String {
    if millis >= 1_000 && millis.is_multiple_of(1_000) {
        format!("{} s", millis / 1_000)
    } else {
        format!("{millis} ms")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_period_reads_in_seconds_when_it_is_whole_ones() {
        assert_eq!(period(5_000), "5 s");
        assert_eq!(period(250), "250 ms");
        assert_eq!(period(1_500), "1500 ms");
    }

    #[test]
    fn a_type_is_named_by_its_last_segment() {
        assert_eq!(type_name("a::b::List<c::Recent>"), "List<Recent>");
        assert_eq!(
            type_name("agent::GenericAgentActor<uniproc_agent::windows::WindowsAgent>"),
            "GenericAgentActor<WindowsAgent>"
        );
        assert_eq!(
            type_name("m::Pair<a::A, std::vec::Vec<b::B>, &'static str, [u8; 4]>"),
            "Pair<A, Vec<B>, &'static str, [u8; 4]>"
        );
        assert_eq!(type_name("contracts::Kill"), "Kill");
        assert_eq!(type_name("Kill"), "Kill");
        assert!(names("actor::Worker", "crate::actor::Worker"));
        assert!(!names("Worker", "crate::actor::BigWorker"));
    }
}
