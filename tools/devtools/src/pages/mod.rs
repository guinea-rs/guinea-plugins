pub mod elements;
pub mod graph;
pub mod home;
pub mod native;
pub mod panels;
pub mod trace;

use crate::features::focus::contracts::Focus;

/// Whether the page has yet to bring the target `focus` holds into view; it
/// counts as brought after this call.
///
/// `seen` is the page's own memory of the last ask it answered - a field of
/// the page, because only that page knows what it has already shown.
pub fn fresh(focus: &Focus, seen: &mut Option<u64>) -> bool {
    seen.replace(focus.asked) != Some(focus.asked)
}
