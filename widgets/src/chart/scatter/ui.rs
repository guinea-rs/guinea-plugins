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
use crate::painted::{Metrics, Paint, Painted, Pointer};

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
}

impl<K: Key> Plotting<K> {
    fn new() -> Self {
        Self {
            series: RefCell::new(Rc::new(Vec::<ScatterSeries<K>>::new())),
            options: RefCell::new(ScatterOptions::default()),
            gesture: RefCell::new(Gesture::default()),
            anchored: Cell::new(None),
            labels: RefCell::new(None),
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
}

impl<K: Key> Paint for Plotting<K> {
    fn paint(&self, session: &DrawingSession<'_>, _device: &GpuDevice, metrics: Metrics) {
        let plot = self.plot(metrics.width, metrics.height, Instant::now());
        let gesture = self.gesture.borrow();
        let pointing = Pointing {
            brush: gesture.brush(),
            hovered: gesture.hovered(),
        };
        let mut labels = self.labels.borrow_mut();
        let labels = labels.get_or_insert_with(Labels::new).as_ref();

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
        let plot = self
            .plotting()
            .plot(metrics.width, metrics.height, Instant::now());
        let series = self.plotting().series.borrow().clone();
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
            let plot = plotting.plot(metrics.width, metrics.height, Instant::now());
            let reach = plotting.options.borrow().reach;
            let series = plotting.series.borrow().clone();
            let took = plotting
                .gesture
                .borrow_mut()
                .take(&pointer, &plot, &*series, reach);
            if took.redraw {
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
