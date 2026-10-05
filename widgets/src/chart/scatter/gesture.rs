//! What the pointer means on a scatter chart: a point hovered, a rectangle
//! dragged out, a click.

use super::model::{Hit, Key, ScatterData, ScatterEvent};
use super::plot::Plot;
use crate::painted::Pointer;

/// How far a press may move, in DIPs, and still be a click.
pub(super) const CLICK_SLOP: f32 = 4.0;

/// What the pointer is doing, between its events.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Gesture<K> {
    /// Where the button went down, while it is down.
    pressed: Option<(f32, f32)>,
    now: (f32, f32),
    dragging: bool,
    hovered: Option<Hit<K>>,
}

impl<K> Default for Gesture<K> {
    fn default() -> Self {
        Self {
            pressed: None,
            now: (0.0, 0.0),
            dragging: false,
            hovered: None,
        }
    }
}

/// What one pointer event came to.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Took<K> {
    pub events: Vec<ScatterEvent<K>>,
    /// Whether what is drawn changed: the rectangle, or the ring.
    pub redraw: bool,
}

impl<K: Key> Gesture<K> {
    pub fn take(
        &mut self,
        pointer: &Pointer,
        plot: &Plot,
        series: &dyn ScatterData<K>,
        reach: f32,
    ) -> Took<K> {
        let mut took = Took {
            events: Vec::new(),
            redraw: false,
        };
        match pointer {
            Pointer::Pressed(info) => {
                let at = (info.x as f32, info.y as f32);
                self.pressed = Some(at);
                self.now = at;
                self.dragging = false;
            }
            Pointer::Moved(info) => {
                let at = (info.x as f32, info.y as f32);
                match self.pressed {
                    Some(_) if !info.is_left_button_pressed => {
                        took.redraw = self.dragging;
                        self.pressed = None;
                        self.dragging = false;
                        self.hover(at, plot, series, reach, &mut took);
                    }
                    Some(from) => {
                        self.now = at;
                        if !self.dragging && far(from, at) {
                            self.dragging = true;
                            self.unhover(&mut took);
                        }
                        took.redraw |= self.dragging;
                    }
                    None => self.hover(at, plot, series, reach, &mut took),
                }
            }
            Pointer::Released(info) => {
                let at = (info.x as f32, info.y as f32);
                if let Some(from) = self.pressed.take() {
                    if self.dragging {
                        took.events.push(ScatterEvent::Brushed(plot.area(from, at)));
                        took.redraw = true;
                    } else {
                        let under = plot.nearest(series, at.0, at.1, reach);
                        took.events.push(ScatterEvent::Clicked(under));
                    }
                    self.dragging = false;
                }
            }
            Pointer::Exited => {
                if self.pressed.is_none() {
                    self.unhover(&mut took);
                }
            }
            Pointer::Lost => {}
        }
        took
    }

    fn hover(
        &mut self,
        at: (f32, f32),
        plot: &Plot,
        series: &dyn ScatterData<K>,
        reach: f32,
        took: &mut Took<K>,
    ) {
        let under = plot.nearest(series, at.0, at.1, reach);
        if under != self.hovered {
            self.hovered = under.clone();
            took.events.push(ScatterEvent::Hovered(under));
            took.redraw = true;
        }
    }

    fn unhover(&mut self, took: &mut Took<K>) {
        if self.hovered.take().is_some() {
            took.events.push(ScatterEvent::Hovered(None));
            took.redraw = true;
        }
    }

    /// The rectangle being dragged out, corner to corner.
    pub fn brush(&self) -> Option<((f32, f32), (f32, f32))> {
        let from = self.pressed.filter(|_| self.dragging)?;
        Some((from, self.now))
    }

    pub fn hovered(&self) -> Option<&Hit<K>> {
        self.hovered.as_ref()
    }
}

fn far(from: (f32, f32), to: (f32, f32)) -> bool {
    (to.0 - from.0).powi(2) + (to.1 - from.1).powi(2) > CLICK_SLOP * CLICK_SLOP
}

#[cfg(test)]
mod tests {
    use windows_reactor::PointerEventInfo;

    use super::super::model::{
        Area, Level, Marker, Scale, ScatterOptions, ScatterPoint, ScatterSeries,
    };
    use super::*;

    fn plot() -> Plot {
        Plot::of(
            100.0,
            100.0,
            &ScatterOptions {
                x: (0, 100),
                y: Scale::Linear {
                    from: 0.0,
                    to: 100.0,
                },
                ..ScatterOptions::default()
            },
        )
    }

    fn series() -> Vec<ScatterSeries> {
        vec![ScatterSeries {
            color: windows_canvas::ColorF {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            marker: Marker::Dot,
            size: 4.0,
            points: vec![ScatterPoint {
                key: 7,
                at: 50,
                value: Level::Value(50.0),
            }],
        }]
    }

    fn info(x: f32, y: f32, held: bool) -> PointerEventInfo {
        PointerEventInfo {
            x: x as f64,
            y: y as f64,
            is_left_button_pressed: held,
            ..Default::default()
        }
    }

    struct Run {
        gesture: Gesture<u64>,
        plot: Plot,
        series: Vec<ScatterSeries>,
        heard: Vec<ScatterEvent>,
    }

    impl Run {
        fn new() -> Self {
            Self {
                gesture: Gesture::default(),
                plot: plot(),
                series: series(),
                heard: Vec::new(),
            }
        }

        fn pointer(&mut self, pointer: Pointer) -> &mut Self {
            let took = self.gesture.take(&pointer, &self.plot, &self.series, 8.0);
            self.heard.extend(took.events);
            self
        }

        fn heard(&mut self) -> Vec<ScatterEvent> {
            std::mem::take(&mut self.heard)
        }
    }

    fn hit() -> Hit {
        Hit {
            series: 0,
            key: 7,
            x: 50.0,
            y: 50.0,
        }
    }

    #[test]
    fn a_point_is_hovered_once_and_left_once() {
        let mut run = Run::new();
        run.pointer(Pointer::Moved(info(20.0, 20.0, false)));
        assert_eq!(run.heard(), [], "nothing near, and nothing was hovered");

        run.pointer(Pointer::Moved(info(52.0, 49.0, false)))
            .pointer(Pointer::Moved(info(51.0, 51.0, false)));
        assert_eq!(run.heard(), [ScatterEvent::Hovered(Some(hit()))]);
        assert_eq!(run.gesture.hovered(), Some(&hit()));

        run.pointer(Pointer::Moved(info(80.0, 80.0, false)))
            .pointer(Pointer::Exited);
        assert_eq!(run.heard(), [ScatterEvent::Hovered(None)]);
    }

    #[test]
    fn a_press_that_hardly_moved_is_a_click_on_what_is_under_it() {
        let mut run = Run::new();
        run.pointer(Pointer::Pressed(info(50.0, 50.0, true)))
            .pointer(Pointer::Moved(info(52.0, 51.0, true)))
            .pointer(Pointer::Lost)
            .pointer(Pointer::Released(info(52.0, 51.0, false)));
        assert_eq!(run.heard(), [ScatterEvent::Clicked(Some(hit()))]);

        run.pointer(Pointer::Pressed(info(10.0, 10.0, true)))
            .pointer(Pointer::Released(info(10.0, 10.0, false)));
        assert_eq!(run.heard(), [ScatterEvent::Clicked(None)], "empty space");
    }

    #[test]
    fn a_drag_draws_its_rectangle_and_is_an_area_when_let_go() {
        let mut run = Run::new();
        run.pointer(Pointer::Pressed(info(80.0, 10.0, true)))
            .pointer(Pointer::Moved(info(40.0, 30.0, true)));
        assert_eq!(run.gesture.brush(), Some(((80.0, 10.0), (40.0, 30.0))));

        run.pointer(Pointer::Moved(info(20.0, 60.0, true)))
            .pointer(Pointer::Lost)
            .pointer(Pointer::Released(info(20.0, 60.0, false)));
        assert_eq!(
            run.heard(),
            [ScatterEvent::Brushed(Area {
                x: (20, 80),
                y: (Level::Value(40.0), Level::Value(90.0)),
            })]
        );
        assert_eq!(run.gesture.brush(), None);
    }

    #[test]
    fn a_drag_hides_the_hovered_point() {
        let mut run = Run::new();
        run.pointer(Pointer::Moved(info(50.0, 50.0, false)));
        run.heard();

        run.pointer(Pointer::Pressed(info(50.0, 50.0, true)))
            .pointer(Pointer::Moved(info(10.0, 10.0, true)));
        assert_eq!(run.heard(), [ScatterEvent::Hovered(None)]);
    }

    #[test]
    fn a_drag_taken_away_is_dropped_at_the_next_move_without_a_button() {
        let mut run = Run::new();
        run.pointer(Pointer::Pressed(info(80.0, 10.0, true)))
            .pointer(Pointer::Moved(info(40.0, 30.0, true)))
            .pointer(Pointer::Lost)
            .pointer(Pointer::Moved(info(30.0, 30.0, false)));
        assert_eq!(run.heard(), []);
        assert_eq!(run.gesture.brush(), None);
    }
}
