//! The scatter chart widget: a [`Painted`] view the page owns.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use windows_canvas::{DrawingSession, GpuDevice};
use windows_reactor::{IntoPayloadCallback, View};

use super::super::live;
use super::gesture::Gesture;
use super::model::{
    Hit, IntoScatterData, Key, ScatterData, ScatterEvent, ScatterOptions, ScatterSeries,
};
use super::paint::{self, Labels, Pointing};
use super::plot::Plot;
use super::progressive::{ScatterRenderStats, State};
use crate::painted::{Metrics, Paint, Painted, Pointer, SurfaceTiming};

/// The span `x` has moved on to `since` after it was published, at
/// `per_second` units a second; where it was, for a chart that is not live.
fn moved(x: (u64, u64), per_second: Option<f64>, since: Duration) -> (u64, u64) {
    let Some(per_second) = per_second else {
        return x;
    };
    let by = (since.as_secs_f64() * per_second).round() as u64;
    (x.0.saturating_add(by), x.1.saturating_add(by))
}

/// What a scatter chart draws with, and what the pointer is doing over it.
struct Plotting<K> {
    /// The page's own data, read where it is kept.
    series: RefCell<Rc<dyn ScatterData<K>>>,
    options: RefCell<ScatterOptions>,
    gesture: RefCell<Gesture<K>>,
    /// When the span shown was last handed over.
    anchored: Cell<Option<Instant>>,
    labels: RefCell<Option<Option<Labels>>>,
    budget: Cell<Option<Duration>>,
    revision: Cell<u64>,
    progressive: RefCell<State<K>>,
    overlay_dirty: Cell<bool>,
}

impl<K: Key> Plotting<K> {
    fn new() -> Self {
        Self {
            series: RefCell::new(Rc::new(Vec::<ScatterSeries<K>>::new())),
            options: RefCell::new(ScatterOptions::default()),
            gesture: RefCell::new(Gesture::default()),
            anchored: Cell::new(None),
            labels: RefCell::new(None),
            budget: Cell::new(None),
            revision: Cell::new(0),
            progressive: RefCell::new(State::default()),
            overlay_dirty: Cell::new(false),
        }
    }

    fn plot(&self, width: f32, height: f32, at: Instant) -> Plot {
        let options = self.options.borrow();
        let since = self.anchored.get().map_or(Duration::ZERO, |anchored| {
            at.saturating_duration_since(anchored)
        });
        let x = moved(options.x, options.live, since);
        Plot::of(
            width,
            height,
            &ScatterOptions {
                x,
                ..options.clone()
            },
        )
    }

    fn interaction(&self, metrics: Metrics) -> Option<(Rc<dyn ScatterData<K>>, Plot, f32)> {
        if self.budget.get().is_some() {
            let state = self.progressive.borrow();
            if !state.eager() {
                return state
                    .front
                    .as_ref()
                    .map(|front| (front.data.clone(), front.plot.clone(), front.options.reach));
            }
        }
        Some((
            self.series.borrow().clone(),
            self.plot(metrics.width, metrics.height, Instant::now()),
            self.options.borrow().reach,
        ))
    }
}

impl<K: Key> Paint for Plotting<K> {
    fn frame_driven(&self) -> bool {
        self.budget.get().is_some()
    }
    fn frame_pending(&self) -> bool {
        self.progressive.borrow().pending()
    }
    fn released(&self) {
        self.progressive.borrow_mut().release();
        *self.gesture.borrow_mut() = Gesture::default();
    }
    fn surface_timing(&self, timing: SurfaceTiming) {
        self.progressive.borrow_mut().surface_timing(timing);
    }
    fn prepare(&self, device: &GpuDevice, metrics: Metrics) -> windows_canvas::Result<bool> {
        let Some(budget) = self.budget.get() else {
            return Ok(true);
        };
        let mut options = self.options.borrow().clone();
        options.x = self.plot(metrics.width, metrics.height, Instant::now()).x;
        self.progressive.borrow_mut().hold_front = self.gesture.borrow().brush().is_some();
        let (ready, swapped) = self.progressive.borrow_mut().prepare(
            device,
            metrics,
            self.series.borrow().clone(),
            options,
            self.revision.get(),
            budget,
        )?;
        if swapped {
            *self.gesture.borrow_mut() = Gesture::default();
        }
        let overlay = self.overlay_dirty.replace(false);
        Ok(ready || overlay)
    }

    fn paint(&self, session: &DrawingSession<'_>, _device: &GpuDevice, metrics: Metrics) {
        let plot = self.plot(metrics.width, metrics.height, Instant::now());
        let gesture = self.gesture.borrow();
        let pointing = Pointing {
            brush: gesture.brush(),
            hovered: gesture.hovered(),
        };
        let mut labels = self.labels.borrow_mut();
        let labels = labels.get_or_insert_with(Labels::new).as_ref();

        if self.budget.get().is_some() && !self.progressive.borrow().eager() {
            let copy = {
                let state = self.progressive.borrow();
                match state.front.as_ref() {
                    Some(front) => {
                        let mut options = front.options.clone();
                        options.selection = self.options.borrow().selection;
                        paint::render_bitmap(
                            session,
                            metrics,
                            &front.plot,
                            (&*front.data, Some(&front.bitmap)),
                            &options,
                            &pointing,
                            labels,
                        )
                    }
                    None => paint::render_bitmap(
                        session,
                        metrics,
                        &plot,
                        (&**self.series.borrow(), None),
                        &self.options.borrow(),
                        &Pointing {
                            brush: None,
                            hovered: None,
                        },
                        labels,
                    ),
                }
            };
            let mut state = self.progressive.borrow_mut();
            state.stats.max_copy = state.stats.max_copy.max(copy);
            return;
        }

        paint::render(
            session,
            metrics,
            &plot,
            &**self.series.borrow(),
            &self.options.borrow(),
            &pointing,
            labels,
        );
    }

    fn moving(&self) -> bool {
        self.options.borrow().live.is_some()
    }

    fn next_frame(&self, metrics: Metrics) -> Duration {
        let options = self.options.borrow();
        let span = options.x.1.saturating_sub(options.x.0);
        let per_second = options.live.unwrap_or(1.0);
        live::step(span, per_second, (metrics.width * metrics.scale).max(1.0))
    }
}

/// A scatter chart of points in time against a value, and what it needs
/// between draws. Held by the page as a field, as [`super::super::Chart`]
/// is.
///
/// `K` is what the page knows a point by - its own id, whatever its type -
/// and what a hovered or clicked point is handed back as.
pub struct Scatter<K: Key = u64> {
    painted: Rc<Painted<Plotting<K>>>,
}

impl<K: Key> Default for Scatter<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Key> Scatter<K> {
    pub fn new() -> Self {
        Self {
            painted: Rc::new(Painted::new(Plotting::new())),
        }
    }

    fn plotting(&self) -> &Plotting<K> {
        self.painted.painter()
    }

    /// Opt-in progressive rendering for data providing ScatterData::cursor.
    /// Work is submitted on WinUI Rendering, with a soft per-pass CPU budget.
    /// Native EndDraw/final copy can exceed it; inspect render_stats(). None or
    /// zero disables this mode. Providers without a cursor keep eager rendering.
    pub fn set_frame_budget(&self, budget: Option<Duration>) {
        let budget = budget.filter(|budget| !budget.is_zero());
        if self.plotting().budget.replace(budget) != budget {
            self.plotting().progressive.borrow_mut().release();
            self.painted.redraw();
        }
    }

    /// Diagnostics of this chart's progressive jobs and surface passes.
    pub fn render_stats(&self) -> ScatterRenderStats {
        let mut stats = self.plotting().progressive.borrow().stats;
        stats.requested_revision = self.plotting().revision.get();
        stats
    }

    /// Hands the chart the page's data and options; redraws only when either
    /// changed.
    ///
    /// The chart keeps the page's `Rc` and no copy of what is behind it, so
    /// handing the same `Rc` again is no change, and new data is a new `Rc`.
    /// Series handed over as a `Vec` are always new.
    ///
    /// A live chart's span moves on from where this puts it, so a page
    /// hands over `(now - length, now)` and need not publish again for the
    /// chart to keep up with the clock. Handed the same span again, the
    /// chart keeps moving from where it was.
    pub fn publish(&self, data: impl IntoScatterData<K>, options: ScatterOptions) {
        let data = data.into_scatter_data();
        let plotting = self.plotting();
        let changed = self.changed(&data, &options);

        if plotting.anchored.get().is_none() || plotting.options.borrow().x != options.x {
            plotting.anchored.set(Some(Instant::now()));
        }
        *plotting.series.borrow_mut() = data;
        *plotting.options.borrow_mut() = options;

        if changed {
            plotting
                .revision
                .set(plotting.revision.get().wrapping_add(1));
            self.painted.redraw();
        }
    }

    /// Whether `data` and `options` draw anything other than what the chart
    /// holds.
    fn changed(&self, data: &Rc<dyn ScatterData<K>>, options: &ScatterOptions) -> bool {
        let plotting = self.plotting();
        !Rc::ptr_eq(&plotting.series.borrow(), data) || *plotting.options.borrow() != *options
    }

    /// The point under the pointer, where it is drawn now: the chart moves
    /// under a resting pointer, and what the page shows beside the point
    /// should follow it.
    pub fn hovered(&self) -> Option<Hit<K>> {
        let hit = self.plotting().gesture.borrow().hovered()?.clone();
        let metrics = self.painted.metrics()?;
        let (series, plot, _) = self.plotting().interaction(metrics)?;
        if hit.series >= series.series() {
            return None;
        }
        let mut found = None;
        series.points(hit.series, &mut |point| {
            if found.is_none() && point.key == hit.key {
                found = Some((point.at, point.value));
            }
        });
        let (at, value) = found?;
        Some(Hit {
            x: plot.x(at),
            y: plot.y(value),
            ..hit
        })
    }

    /// The chart, drawn.
    pub fn view(&self, on_event: impl IntoPayloadCallback<ScatterEvent<K>>) -> View {
        let on_event = on_event.into_payload_callback();
        let painted = self.painted.clone();

        self.painted.view(move |pointer: Pointer| {
            let Some(metrics) = painted.metrics() else {
                return;
            };
            let plotting = painted.painter();
            let Some((series, plot, reach)) = plotting.interaction(metrics) else {
                return;
            };
            let took = plotting
                .gesture
                .borrow_mut()
                .take(&pointer, &plot, &*series, reach);
            if took.redraw {
                plotting.overlay_dirty.set(true);
                painted.redraw();
            }
            for event in took.events {
                on_event.call(event);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_testing_uses_completed_front_until_the_next_snapshot_is_swapped() {
        use super::super::model::{Level, Marker, ScatterCursor, ScatterPoint, SeriesStyle};
        struct Data(u64);
        impl ScatterData<u64> for Data {
            fn series(&self) -> usize {
                1
            }
            fn style(&self, _: usize) -> SeriesStyle {
                SeriesStyle {
                    color: windows_canvas::ColorF::TRANSPARENT,
                    marker: Marker::Dot,
                    size: 4.0,
                }
            }
            fn points(&self, _: usize, each: &mut dyn FnMut(&ScatterPoint<u64>)) {
                for i in 0..512 {
                    each(&ScatterPoint {
                        key: self.0 + i,
                        at: 50,
                        value: Level::Value(0.5),
                    });
                }
            }
            fn cursor(self: Rc<Self>, _: u64, _: u64) -> Option<ScatterCursor<u64>> {
                Some(Box::new((0..512).map(move |i| {
                    (
                        0,
                        ScatterPoint {
                            key: self.0 + i,
                            at: 50,
                            value: Level::Value(0.5),
                        },
                    )
                })))
            }
        }
        let device = GpuDevice::new_warp().unwrap();
        let metrics = Metrics::new(100.0, 60.0, 1.0);
        let plotting = Plotting::new();
        plotting.budget.set(Some(Duration::from_nanos(1)));
        plotting.options.borrow_mut().x = (0, 100);
        *plotting.series.borrow_mut() = Rc::new(Data(1));
        plotting.revision.set(1);
        while plotting.progressive.borrow().stats.completed == 0 {
            plotting.prepare(&device, metrics).unwrap();
        }
        *plotting.series.borrow_mut() = Rc::new(Data(1000));
        plotting.revision.set(2);
        plotting.prepare(&device, metrics).unwrap();
        let (data, plot, reach) = plotting.interaction(metrics).unwrap();
        let hit = plot
            .nearest(&*data, plot.x(50), plot.y(Level::Value(0.5)), reach)
            .unwrap();
        assert!(
            hit.key < 1000,
            "pending data must not replace hit-test data"
        );
        while plotting.progressive.borrow().stats.completed < 2 {
            plotting.prepare(&device, metrics).unwrap();
        }
        let (data, plot, reach) = plotting.interaction(metrics).unwrap();
        let hit = plot
            .nearest(&*data, plot.x(50), plot.y(Level::Value(0.5)), reach)
            .unwrap();
        assert!(
            hit.key >= 1000,
            "completed swap changes both image and hit-test data"
        );
    }

    #[test]
    fn a_live_span_moves_on_with_the_clock() {
        let x = (1_000, 4_000);
        assert_eq!(
            moved(x, Some(1_000.0), Duration::from_millis(1_500)),
            (2_500, 5_500)
        );
        assert_eq!(moved(x, None, Duration::from_secs(9)), x, "not live");
    }

    #[test]
    fn the_same_data_handed_again_is_no_change_and_another_rc_is() {
        let scatter = Scatter::<u64>::new();
        let options = ScatterOptions::default();
        let data: Rc<Vec<ScatterSeries>> = Rc::new(Vec::new());
        scatter.publish(data.clone(), options.clone());

        let again: Rc<dyn ScatterData<u64>> = data.clone();
        let other: Rc<dyn ScatterData<u64>> = Rc::new(Vec::<ScatterSeries>::new());
        let restyled = ScatterOptions {
            reach: 20.0,
            ..options.clone()
        };

        assert_eq!(
            (
                scatter.changed(&again, &options),
                scatter.changed(&other, &options),
                scatter.changed(&again, &restyled),
            ),
            (false, true, true)
        );
        assert_eq!(Rc::strong_count(&data), 3, "the page's, the chart's, again");
    }

    #[test]
    fn the_same_span_handed_again_keeps_moving_from_where_it_was() {
        let options = ScatterOptions {
            x: (1_000, 4_000),
            live: Some(1_000.0),
            ..ScatterOptions::default()
        };
        let scatter = Scatter::<u64>::new();
        scatter.publish(Vec::new(), options.clone());
        let anchored = scatter.plotting().anchored.get();

        std::thread::sleep(Duration::from_millis(2));
        scatter.publish(Vec::new(), options.clone());
        assert_eq!(scatter.plotting().anchored.get(), anchored);

        scatter.publish(
            Vec::new(),
            ScatterOptions {
                x: (1_100, 4_100),
                ..options
            },
        );
        assert_ne!(
            scatter.plotting().anchored.get(),
            anchored,
            "a new span is a new anchor"
        );
    }
}
