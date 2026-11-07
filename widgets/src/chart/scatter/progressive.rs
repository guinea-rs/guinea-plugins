//! Immutable front and a bounded-work back job. No CPU pixel readback.
use std::rc::Rc;
use std::time::{Duration, Instant};

use windows_canvas::{Bitmap, Brush, ColorF, GpuDevice, RenderTarget};
use windows_core::Interface;

use super::model::{Key, ScatterCursor, ScatterData, ScatterOptions, SeriesStyle};
use super::paint;
use super::plot::Plot;
use crate::painted::{Metrics, SurfaceTiming};

/// CPU wall-time diagnostics. Offscreen finish is the tail of RenderTarget::draw
/// after the submission closure, including EndDraw and target restoration.
/// Surface finish likewise includes EndDraw; neither is GPU execution time.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScatterRenderStats {
    pub chunks: u64,
    pub points: u64,
    pub completed: u64,
    pub cancelled: u64,
    pub front_revision: u64,
    pub requested_revision: u64,
    pub max_job_init: Duration,
    pub max_submit: Duration,
    pub max_offscreen_begin: Duration,
    pub max_offscreen_finish: Duration,
    pub max_chunk: Duration,
    pub max_surface_paint: Duration,
    pub max_surface_begin: Duration,
    pub max_copy: Duration,
    pub max_surface_finish: Duration,
    pub max_device: Duration,
    pub max_surface_init: Duration,
    pub last_job_wall: Duration,
}

pub(super) struct Front<K> {
    pub data: Rc<dyn ScatterData<K>>,
    pub options: ScatterOptions,
    pub plot: Plot,
    pub bitmap: Bitmap,
    revision: u64,
}

struct Job<K> {
    front: Front<K>,
    target: RenderTarget,
    brushes: Vec<(SeriesStyle, Brush)>,
    cursor: ScatterCursor<K>,
    started: Instant,
}

// Ordinary live clock/tick label updates belong to the next snapshot, not a
// cancellation of the job currently being completed. Manual panning disables live.
fn configuration(options: &ScatterOptions) -> ScatterOptions {
    let mut key = options.clone();
    key.selection = None;
    key.reach = 0.0;
    if key.live.is_some() {
        key.x = (0, key.x.1.saturating_sub(key.x.0));
        if !key.x_ticks.is_empty() {
            key.x_ticks = vec![(0, String::new())];
        }
    }
    key
}

pub(super) struct State<K> {
    pub front: Option<Front<K>>,
    pub hold_front: bool,
    job: Option<Job<K>>,
    key: Option<(Metrics, usize, ScatterOptions)>,
    blank: bool,
    pending: bool,
    eager_revision: Option<u64>,
    pub stats: ScatterRenderStats,
}

impl<K: Key> Default for State<K> {
    fn default() -> Self {
        Self {
            front: None,
            hold_front: false,
            job: None,
            key: None,
            blank: false,
            pending: false,
            eager_revision: None,
            stats: ScatterRenderStats::default(),
        }
    }
}

impl<K: Key> State<K> {
    pub fn release(&mut self) {
        if self.job.take().is_some() {
            self.stats.cancelled += 1;
        }
        self.front = None;
        self.stats.front_revision = 0;
        self.key = None;
        self.blank = false;
        self.pending = false;
        self.eager_revision = None;
    }

    pub fn pending(&self) -> bool {
        self.pending && !(self.hold_front && self.front.is_some())
    }
    pub fn eager(&self) -> bool {
        self.eager_revision.is_some()
    }

    pub fn surface_timing(&mut self, timing: SurfaceTiming) {
        self.stats.max_device = self.stats.max_device.max(timing.device);
        self.stats.max_surface_init = self.stats.max_surface_init.max(timing.surface);
        self.stats.max_surface_begin = self.stats.max_surface_begin.max(timing.begin);
        self.stats.max_surface_paint = self.stats.max_surface_paint.max(timing.paint);
        self.stats.max_surface_finish = self.stats.max_surface_finish.max(timing.finish);
    }

    pub fn prepare(
        &mut self,
        device: &GpuDevice,
        metrics: Metrics,
        data: Rc<dyn ScatterData<K>>,
        options: ScatterOptions,
        revision: u64,
        budget: Duration,
    ) -> windows_canvas::Result<(bool, bool)> {
        self.stats.requested_revision = revision;
        let key = (
            metrics,
            device.d2d_device().as_raw() as usize,
            configuration(&options),
        );
        if self.key.as_ref() != Some(&key) {
            self.release();
            self.key = Some(key);
        }
        // Give native initialization/axes their own frame before starting the job.
        if !self.blank {
            self.blank = true;
            self.pending = true;
            return Ok((true, false));
        }
        if self.hold_front && self.front.is_some() {
            return Ok((true, false));
        }
        let plot = Plot::of(metrics.width, metrics.height, &options);
        if self.job.is_none() {
            if self
                .front
                .as_ref()
                .is_some_and(|front| front.revision == revision && front.plot == plot)
            {
                self.pending = false;
                return Ok((true, false)); // overlay only
            }
            if self.eager_revision == Some(revision) {
                self.pending = false;
                return Ok((true, false));
            }
            let radius = (0..data.series())
                .map(|i| data.style(i).size / 2.0)
                .fold(0.0_f32, f32::max);
            let (from, to) = plot.times_within(
                (plot.left + plot.right) / 2.0,
                (plot.right - plot.left) / 2.0 + radius,
            );
            let started = Instant::now();
            let Some(cursor) = data.clone().cursor(from, to) else {
                self.eager_revision = Some(revision);
                self.pending = false;
                return Ok((true, false));
            };
            let target = device.create_render_target(
                (metrics.width * metrics.scale).round().max(1.0) as u32,
                (metrics.height * metrics.scale).round().max(1.0) as u32,
            )?;
            let mut bitmap = None;
            let mut brushes = Vec::new();
            target.draw(|draw| {
                let context: super::super::d2d::ID2D1DeviceContext = draw.raw().cast()?;
                unsafe {
                    context.SetDpi(96.0 * metrics.scale, 96.0 * metrics.scale);
                }
                let image = draw.create_bitmap_target()?;
                draw.with_target(&image, || draw.clear(ColorF::TRANSPARENT));
                for index in 0..data.series() {
                    let style = data.style(index);
                    brushes.push((style, draw.create_solid_brush(style.color)?));
                }
                bitmap = Some(image);
                Ok(())
            })?;
            self.job = Some(Job {
                front: Front {
                    data,
                    options,
                    plot,
                    bitmap: bitmap.unwrap(),
                    revision,
                },
                target,
                brushes,
                cursor,
                started,
            });
            self.stats.max_job_init = self.stats.max_job_init.max(started.elapsed());
        }
        let mut job = self.job.take().unwrap();
        let started = Instant::now();
        let mut submitted = None;
        let mut began = None;
        let mut count = 0;
        let mut complete = false;
        job.target.draw(|draw| {
            began = Some(Instant::now());
            draw.with_target(&job.front.bitmap, || {
                loop {
                    let Some((series, point)) = job.cursor.next() else {
                        complete = true;
                        break;
                    };
                    if let Some((style, brush)) = job.brushes.get(series) {
                        paint::point(draw, &job.front.plot, *style, brush, &point);
                    }
                    count += 1;
                    if count % 64 == 0 && started.elapsed() >= budget {
                        break;
                    }
                }
            });
            submitted = Some(Instant::now());
            Ok(())
        })?;
        let finished = Instant::now();
        let submitted = submitted.unwrap();
        let began = began.unwrap();
        self.stats.chunks += 1;
        self.stats.points += count;
        self.stats.max_submit = self.stats.max_submit.max(submitted.duration_since(began));
        self.stats.max_offscreen_begin = self
            .stats
            .max_offscreen_begin
            .max(began.duration_since(started));
        self.stats.max_offscreen_finish = self
            .stats
            .max_offscreen_finish
            .max(finished.duration_since(submitted));
        self.stats.max_chunk = self.stats.max_chunk.max(finished.duration_since(started));
        if complete {
            self.stats.completed += 1;
            self.stats.front_revision = job.front.revision;
            self.stats.last_job_wall = job.started.elapsed();
            self.pending = revision != job.front.revision;
            self.front = Some(job.front);
            self.eager_revision = None;
            Ok((true, true))
        } else {
            self.job = Some(job);
            self.pending = true;
            Ok((false, false)) // no clear, copy, attach or partial presentation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::{Level, Marker, Scale, ScatterPoint};
    use super::super::paint::{Labels, Pointing};
    use super::*;
    use std::cell::Cell;

    struct Data {
        count: usize,
        cursors: Cell<usize>,
        visited: Cell<usize>,
    }
    impl Data {
        fn new(count: usize) -> Rc<Self> {
            Rc::new(Self {
                count,
                cursors: Cell::new(0),
                visited: Cell::new(0),
            })
        }
        fn point(index: usize) -> ScatterPoint<usize> {
            ScatterPoint {
                key: index,
                at: (index % 100) as u64,
                value: Level::Value((index % 53) as f32),
            }
        }
    }
    impl ScatterData<usize> for Data {
        fn series(&self) -> usize {
            3
        }
        fn style(&self, index: usize) -> SeriesStyle {
            SeriesStyle {
                color: ColorF {
                    r: 0.3,
                    g: 0.8,
                    b: 0.5,
                    a: 0.65,
                },
                marker: [Marker::Dot, Marker::Ring, Marker::Tick][index],
                size: 4.0,
            }
        }
        fn points(&self, index: usize, each: &mut dyn FnMut(&ScatterPoint<usize>)) {
            for point in (index..self.count).step_by(3).map(Self::point) {
                each(&point);
            }
        }
        fn cursor(self: Rc<Self>, from: u64, to: u64) -> Option<ScatterCursor<usize>> {
            self.cursors.set(self.cursors.get() + 1);
            let data = self.clone();
            Some(Box::new((0..3).flat_map(move |series| {
                let data = data.clone();
                (series..data.count).step_by(3).filter_map(move |i| {
                    data.visited.set(data.visited.get() + 1);
                    let point = Self::point(i);
                    (from..=to).contains(&point.at).then_some((series, point))
                })
            })))
        }
    }

    fn options() -> ScatterOptions {
        ScatterOptions {
            x: (0, 100),
            y: Scale::Linear {
                from: 0.0,
                to: 60.0,
            },
            ..ScatterOptions::default()
        }
    }

    #[test]
    fn frequent_data_and_live_clock_updates_finish_frozen_jobs_then_take_only_latest() {
        let device = GpuDevice::new_warp().unwrap();
        let metrics = Metrics::new(120.0, 80.0, 1.0);
        let original = Data::new(512);
        let latest = Data::new(512);
        let mut state = State::default();
        let budget = Duration::from_nanos(1); // deterministic slices of at most 64 points
        let mut opts = options();
        opts.live = Some(1000.0);
        assert!(
            state
                .prepare(&device, metrics, original.clone(), opts.clone(), 1, budget)
                .unwrap()
                .0
        );
        state
            .prepare(&device, metrics, original.clone(), opts.clone(), 1, budget)
            .unwrap();
        let mut revision = 2;
        while state.stats.completed == 0 {
            opts.x = (revision, revision + 100);
            state
                .prepare(
                    &device,
                    metrics,
                    latest.clone(),
                    opts.clone(),
                    revision,
                    budget,
                )
                .unwrap();
            revision += 1;
            assert!(revision < 100);
        }
        assert_eq!(state.stats.cancelled, 0);
        assert_eq!(state.stats.front_revision, 1);
        assert!(state.pending());
        assert!(Rc::ptr_eq(
            &state.front.as_ref().unwrap().data,
            &(original.clone() as Rc<dyn ScatterData<usize>>)
        ));
        assert_eq!(original.visited.get(), 512, "no skip(N) or repeated visits");
        assert_eq!(
            latest.cursors.get(),
            0,
            "no job restarted for intermediate snapshots"
        );
        state.hold_front = true;
        state
            .prepare(
                &device,
                metrics,
                latest.clone(),
                opts.clone(),
                revision,
                budget,
            )
            .unwrap();
        assert_eq!(
            latest.cursors.get(),
            0,
            "a brush pins its displayed snapshot"
        );
        assert!(
            !state.pending(),
            "a pinned front does not keep a frame subscription busy"
        );
        state.hold_front = false;
        while state.stats.completed < 2 {
            state
                .prepare(
                    &device,
                    metrics,
                    latest.clone(),
                    opts.clone(),
                    revision,
                    budget,
                )
                .unwrap();
        }
        assert_eq!(latest.cursors.get(), 1);
        assert_eq!(state.stats.front_revision, revision);
        assert!(!state.pending());
        state.release();
        assert!(state.front.is_none() && !state.pending());
    }

    #[test]
    fn viewport_changes_cancel_back_and_never_expose_a_partial_front() {
        let device = GpuDevice::new_warp().unwrap();
        let data = Data::new(512);
        let metrics = Metrics::new(100.0, 60.0, 1.0);
        let mut state = State::default();
        for _ in 0..2 {
            state
                .prepare(
                    &device,
                    metrics,
                    data.clone(),
                    options(),
                    1,
                    Duration::from_nanos(1),
                )
                .unwrap();
        }
        assert!(state.front.is_none());
        let changed = Metrics::new(110.0, 60.0, 2.0);
        state
            .prepare(
                &device,
                changed,
                data,
                options(),
                2,
                Duration::from_nanos(1),
            )
            .unwrap();
        assert_eq!(state.stats.cancelled, 1);
        assert!(state.front.is_none());
    }

    #[test]
    fn progressive_pixels_match_eager_markers_at_native_dpi_with_bounded_compositing_roundoff() {
        let device = GpuDevice::new_warp().unwrap();
        for scale in [1.0, 1.5, 2.0] {
            let metrics = Metrics::new(120.0, 80.0, scale);
            let data = Data::new(100);
            let options = ScatterOptions {
                corner_radius: Some(3.0),
                ..options()
            };
            let mut state = State::default();
            while state.stats.completed == 0 {
                state
                    .prepare(
                        &device,
                        metrics,
                        data.clone(),
                        options.clone(),
                        1,
                        Duration::from_nanos(1),
                    )
                    .unwrap();
            }
            let front = state.front.as_ref().unwrap();
            let render = |progressive: bool| {
                let target = device
                    .create_render_target(
                        (metrics.width * metrics.scale).round() as u32,
                        (metrics.height * metrics.scale).round() as u32,
                    )
                    .unwrap();
                target
                    .draw(|draw| {
                        let context: super::super::super::d2d::ID2D1DeviceContext =
                            draw.raw().cast()?;
                        unsafe {
                            context.SetDpi(96.0 * metrics.scale, 96.0 * metrics.scale);
                        }
                        draw.clear(ColorF::TRANSPARENT);
                        let labels = Labels::new();
                        let pointing = Pointing {
                            hovered: None,
                            brush: None,
                        };
                        if progressive {
                            paint::render_bitmap(
                                draw,
                                metrics,
                                &front.plot,
                                (&*front.data, Some(&front.bitmap)),
                                &front.options,
                                &pointing,
                                labels.as_ref(),
                            );
                        } else {
                            paint::render(
                                draw,
                                metrics,
                                &front.plot,
                                &*data,
                                &options,
                                &pointing,
                                labels.as_ref(),
                            );
                        }
                        Ok(())
                    })
                    .unwrap();
                target.read_pixels().unwrap()
            };
            let eager = render(false);
            let progressive = render(true);
            let differences: Vec<u8> = eager
                .iter()
                .zip(&progressive)
                .map(|(a, b)| a.abs_diff(*b))
                .collect();
            let max = differences.iter().copied().max().unwrap();
            let mean = differences.iter().map(|&n| n as u64).sum::<u64>() as f64
                / differences.len() as f64;
            assert!(
                max <= 4 && mean <= 0.25,
                "scale {scale}, max {max}, mean {mean}"
            );
        }
    }
}
