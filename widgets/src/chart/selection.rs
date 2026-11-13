//! Horizontal brushing of the last successfully drawn snapshot.
use super::geometry::Frame;
use super::model::{LineChartOptions, Series, XSelection, XSelectionEvent};
use crate::painted::{Metrics, Pointer};
use std::rc::Rc;

#[derive(Clone)]
pub(super) struct Snapshot {
    pub series: Rc<Vec<Series>>,
    pub options: Rc<LineChartOptions>,
    pub frame: Frame,
    pub metrics: Metrics,
    pub bands: Rc<super::bands::Bands>,
    pub highlighted: Option<u64>,
}

struct Drag {
    snapshot: Snapshot,
    start: f32,
    now: f32,
    previous: Option<XSelection>,
    lost: bool,
}

#[derive(Default)]
pub(super) struct Selection {
    pub enabled: bool,
    pub range: Option<XSelection>,
    drag: Option<Drag>,
}

impl Selection {
    pub fn pinned(&self) -> Option<&Snapshot> {
        self.drag.as_ref().map(|d| &d.snapshot)
    }
    pub fn cancel(&mut self) -> Option<XSelectionEvent> {
        let drag = self.drag.take()?;
        self.range = drag.previous;
        Some(XSelectionEvent::Cancelled)
    }
    pub fn cancel_lost(&mut self) -> Option<XSelectionEvent> {
        if self.drag.as_ref().is_some_and(|d| d.lost) {
            self.cancel()
        } else {
            None
        }
    }
    pub fn take(&mut self, pointer: &Pointer, front: Option<&Snapshot>) -> Option<XSelectionEvent> {
        if !self.enabled {
            return None;
        }
        match pointer {
            Pointer::Pressed(info) if info.is_left_button_pressed => {
                if self.drag.is_some() {
                    return None;
                }
                let snapshot = front?.clone();
                let x = info.x as f32;
                if !x.is_finite() || snapshot.metrics.width <= 0.0 {
                    return None;
                }
                let start = x.clamp(0.0, snapshot.metrics.width);
                let t = snapshot.frame.t_at(start, snapshot.metrics.width);
                self.drag = Some(Drag {
                    snapshot,
                    start,
                    now: start,
                    previous: self.range,
                    lost: false,
                });
                self.range = Some((t, t));
                Some(XSelectionEvent::Began((t, t)))
            }
            Pointer::Moved(info) => {
                if !info.is_left_button_pressed {
                    return self.cancel();
                }
                let drag = self.drag.as_mut()?;
                if !info.x.is_finite() {
                    return None;
                }
                drag.now = (info.x as f32).clamp(0.0, drag.snapshot.metrics.width);
                let range = drag.range();
                if self.range.replace(range) == Some(range) {
                    None
                } else {
                    Some(XSelectionEvent::Changed(range))
                }
            }
            Pointer::Released(info) if !info.is_left_button_pressed => {
                let mut drag = self.drag.take()?;
                if info.x.is_finite() {
                    drag.now = (info.x as f32).clamp(0.0, drag.snapshot.metrics.width);
                }
                if (drag.now - drag.start).abs() <= 4.0 {
                    self.range = None;
                    Some(XSelectionEvent::Cleared)
                } else {
                    let range = drag.range();
                    self.range = Some(range);
                    Some(XSelectionEvent::Ended(range))
                }
            }
            // Painted emits Lost immediately before a normal Released. Defer
            // cancellation until Rendering; Released in this turn wins.
            Pointer::Lost => {
                if let Some(drag) = &mut self.drag {
                    drag.lost = true;
                }
                None
            }
            _ => None,
        }
    }
}

impl Drag {
    fn range(&self) -> XSelection {
        let frame = self.snapshot.frame;
        let width = self.snapshot.metrics.width;
        let a = frame.t_at(self.start, width);
        let b = frame.t_at(self.now, width);
        (a.min(b), a.max(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_reactor::PointerEventInfo;
    fn info(x: f64, held: bool) -> PointerEventInfo {
        PointerEventInfo {
            x,
            is_left_button_pressed: held,
            ..Default::default()
        }
    }
    fn snapshot() -> Snapshot {
        Snapshot {
            series: Rc::default(),
            options: Rc::default(),
            frame: Frame {
                from: 1000.0,
                to: 2000.0,
                min_v: 0.0,
                max_v: 100.0,
            },
            metrics: Metrics::new(100.0, 40.0, 1.0),
            bands: Rc::default(),
            highlighted: None,
        }
    }
    #[test]
    fn reverse_drag_clamps_to_pinned_display_despite_new_epoch() {
        let mut s = Selection {
            enabled: true,
            ..Default::default()
        };
        let front = snapshot();
        assert_eq!(
            s.take(&Pointer::Pressed(info(80.0, true)), Some(&front)),
            Some(XSelectionEvent::Began((1800, 1800)))
        );
        let mut newer = snapshot();
        newer.frame.from = 9000.0;
        newer.frame.to = 10_000.0;
        assert_eq!(
            s.take(&Pointer::Moved(info(-10.0, true)), Some(&newer)),
            Some(XSelectionEvent::Changed((1000, 1800)))
        );
        assert_eq!(s.take(&Pointer::Released(info(40.0, true)), None), None);
        assert!(s.pinned().is_some());
        assert_eq!(
            s.take(&Pointer::Released(info(-20.0, false)), Some(&newer)),
            Some(XSelectionEvent::Ended((1000, 1800)))
        );
        assert!(s.pinned().is_none());
    }
    #[test]
    fn normal_capture_loss_before_release_finishes_but_unexpected_loss_cancels() {
        let mut s = Selection {
            enabled: true,
            range: Some((1100, 1200)),
            ..Default::default()
        };
        let front = snapshot();
        s.take(&Pointer::Pressed(info(10.0, true)), Some(&front));
        s.take(&Pointer::Moved(info(60.0, true)), None);
        s.take(&Pointer::Lost, None);
        assert_eq!(
            s.take(&Pointer::Released(info(70.0, false)), None),
            Some(XSelectionEvent::Ended((1100, 1700)))
        );
        assert_eq!(s.cancel_lost(), None);
        s.take(&Pointer::Pressed(info(20.0, true)), Some(&front));
        s.take(&Pointer::Lost, None);
        assert_eq!(s.cancel_lost(), Some(XSelectionEvent::Cancelled));
        assert_eq!(s.range, Some((1100, 1700)));
        assert!(s.pinned().is_none());
    }
    #[test]
    fn click_clears_and_missing_button_cancels_without_replacing_previous_range() {
        let mut s = Selection {
            enabled: true,
            range: Some((1100, 1500)),
            ..Default::default()
        };
        let front = snapshot();
        s.take(&Pointer::Pressed(info(30.0, true)), Some(&front));
        assert_eq!(
            s.take(&Pointer::Moved(info(40.0, false)), None),
            Some(XSelectionEvent::Cancelled)
        );
        assert_eq!(s.range, Some((1100, 1500)));
        s.take(&Pointer::Pressed(info(30.0, true)), Some(&front));
        assert_eq!(
            s.take(&Pointer::Released(info(32.0, false)), None),
            Some(XSelectionEvent::Cleared)
        );
        assert_eq!(s.range, None);
    }
    #[test]
    fn disabled_no_snapshot_and_secondary_press_do_not_start_selection() {
        let front = snapshot();
        let mut s = Selection::default();
        assert_eq!(
            s.take(&Pointer::Pressed(info(20.0, true)), Some(&front)),
            None
        );
        s.enabled = true;
        assert_eq!(s.take(&Pointer::Pressed(info(20.0, true)), None), None);
        assert_eq!(
            s.take(&Pointer::Pressed(info(20.0, false)), Some(&front)),
            None
        );
        assert!(s.pinned().is_none());
    }
}
