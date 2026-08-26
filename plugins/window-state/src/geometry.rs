//! What is written down, and what is worth writing down.

use guinea::app::windows::{Geometry, Position, Size};
use serde::{Deserialize, Serialize};

/// One window's remembered state.
///
/// Its own type rather than [`Geometry`] itself: what goes in a file is a
/// format, and a format that is also a contract type starts deciding how the
/// contract may change.
#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Debug)]
pub(crate) struct Saved {
    /// The size the window has when it is neither maximised nor fullscreen -
    /// the one worth restoring, because the others are a flag away.
    size: Option<(f64, f64)>,
    position: Option<(f64, f64)>,
    maximized: bool,
    fullscreen: bool,
}

impl Saved {
    /// Folds in what the window looks like now, or answers `None` when there
    /// is nothing to learn from it.
    pub(crate) fn update(mut self, geometry: Geometry) -> Option<Self> {
        // A minimised window is nowhere: its size and position say only that
        // it is minimised, and writing them down loses the ones that matter.
        if geometry.minimized {
            return None;
        }

        self.maximized = geometry.maximized;
        self.fullscreen = geometry.fullscreen;

        // Restoring a maximised window means setting the flag, not the size it
        // happened to have while maximised - what has to survive is the size
        // it goes back to.
        if !geometry.maximized && !geometry.fullscreen {
            self.size = geometry.size.map(|size| (size.width, size.height));
            self.position = geometry.position.map(|at| (at.x, at.y));
        }

        (self != Self::default()).then_some(self)
    }

    pub(crate) fn geometry(self) -> Geometry {
        Geometry {
            size: self.size.map(|(width, height)| Size { width, height }),
            position: self.position.map(|(x, y)| Position { x, y }),
            maximized: self.maximized,
            fullscreen: self.fullscreen,
            minimized: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(width: f64, height: f64, x: f64, y: f64) -> Geometry {
        Geometry {
            size: Some(Size { width, height }),
            position: Some(Position { x, y }),
            ..Geometry::default()
        }
    }

    #[test]
    fn an_ordinary_window_is_remembered_whole() {
        let saved = Saved::default().update(at(800.0, 600.0, 100.0, 50.0));

        let geometry = saved.expect("worth remembering").geometry();
        assert_eq!(geometry.size.unwrap().width, 800.0);
        assert_eq!(geometry.position.unwrap().y, 50.0);
        assert!(!geometry.maximized);
    }

    #[test]
    fn a_minimised_window_teaches_nothing() {
        let saved = Saved::default()
            .update(at(800.0, 600.0, 100.0, 50.0))
            .expect("worth remembering");

        let minimised = Geometry {
            minimized: true,
            ..Geometry::default()
        };

        assert_eq!(
            saved.update(minimised),
            None,
            "the size of a minimised window is not a size to keep"
        );
    }

    #[test]
    fn maximising_is_a_flag_and_leaves_the_size_to_come_back_to() {
        let saved = Saved::default()
            .update(at(800.0, 600.0, 100.0, 50.0))
            .expect("worth remembering");

        let maximised = Geometry {
            size: Some(Size {
                width: 1920.0,
                height: 1080.0,
            }),
            position: Some(Position { x: -8.0, y: -8.0 }),
            maximized: true,
            ..Geometry::default()
        };

        let after = saved.update(maximised).expect("worth remembering");
        let geometry = after.geometry();

        assert!(geometry.maximized);
        assert_eq!(
            geometry.size.unwrap().width,
            800.0,
            "restoring down has to land where the window was, not on the screen's size"
        );
        assert_eq!(geometry.position.unwrap().x, 100.0);
    }

    #[test]
    fn a_window_that_says_nothing_is_not_written_down() {
        assert_eq!(Saved::default().update(Geometry::default()), None);
    }

    #[test]
    fn what_is_written_is_what_comes_back() {
        let saved = Saved::default()
            .update(at(1024.0, 768.0, 12.0, 34.0))
            .expect("worth remembering");

        let json = serde_json::to_string(&saved).expect("serialise");
        let read: Saved = serde_json::from_str(&json).expect("deserialise");

        assert_eq!(read, saved);
    }
}
