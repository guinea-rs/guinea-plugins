use super::Series;

pub fn bounds(series: &[Series]) -> Option<(u64, u64, f32, f32)> {
    let mut points = series.iter().flat_map(|s| s.points.iter().copied());
    let (first_t, first_v) = points.next()?;
    let mut acc = (first_t, first_t, first_v, first_v);
    for (t, v) in points {
        acc.0 = acc.0.min(t);
        acc.1 = acc.1.max(t);
        acc.2 = acc.2.min(v);
        acc.3 = acc.3.max(v);
    }
    Some(acc)
}

/// What the chart spans: the stretch of time across it and the values up it.
///
/// Time is kept as `f64` rather than `u64`: with a window the left edge can
/// fall before the first sample, or before zero, and the samples older than
/// the edge still have to land somewhere - off the left side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Frame {
    pub from: f64,
    pub to: f64,
    pub min_v: f32,
    pub max_v: f32,
}

impl Frame {
    /// The frame for `series`: the last `x_window` of time up to `end`, or up
    /// to the newest sample, or all of it; `y_range`, or the values of the
    /// samples in view. `end` counts only with a window.
    pub fn of(
        series: &[Series],
        x_window: Option<u64>,
        y_range: Option<(f32, f32)>,
        end: Option<f64>,
    ) -> Option<Frame> {
        let (min_t, max_t, all_min_v, all_max_v) = bounds(series)?;
        let (from, to) = match x_window {
            Some(window) => {
                let to = end.unwrap_or(max_t as f64);
                (to - window.max(1) as f64, to)
            }
            None => (min_t as f64, max_t as f64),
        };

        let (min_v, max_v) = y_range.unwrap_or_else(|| {
            let mut shown = series
                .iter()
                .flat_map(|s| s.points.iter())
                .filter(|(t, _)| (from..=to).contains(&(*t as f64)))
                .map(|&(_, v)| v);
            match shown.next() {
                Some(first) => shown.fold((first, first), |(lo, hi), v| (lo.min(v), hi.max(v))),
                None => (all_min_v, all_max_v),
            }
        });

        Some(Frame {
            from,
            to: to.max(from + 1.0),
            min_v,
            max_v,
        })
    }

    pub fn x(&self, t: u64, width: f32) -> f32 {
        ((t as f64 - self.from) / (self.to - self.from)) as f32 * width
    }

    pub fn y(&self, v: f32, height: f32) -> f32 {
        let span = (self.max_v - self.min_v).max(f32::EPSILON);
        height - ((v - self.min_v) / span) * height
    }

    /// The time at `x` of `width`, kept within the frame.
    pub fn t_at(&self, x: f32, width: f32) -> u64 {
        let ratio = (x / width).clamp(0.0, 1.0) as f64;
        (self.from + (self.to - self.from) * ratio).max(0.0) as u64
    }

    /// Every multiple of `every` in view, oldest first.
    pub fn multiples(&self, every: u64) -> impl Iterator<Item = u64> {
        let every = every.max(1);
        let first = (self.from.max(0.0) / every as f64).ceil() as u64;
        let last = (self.to / every as f64).floor() as u64;
        (first..=last).map(move |k| k * every)
    }
}

pub fn nearest_point(points: &[(u64, f32)], target: u64) -> Option<(u64, f32)> {
    if points.is_empty() {
        return None;
    }

    let idx = points.partition_point(|&(t, _)| t < target);
    Some(match idx {
        0 => points[0],
        n if n == points.len() => points[n - 1],
        n => {
            let (before, after) = (points[n - 1], points[n]);
            if target - before.0 <= after.0 - target {
                before
            } else {
                after
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::Interpolation;
    use windows_canvas::ColorF;

    fn series(points: &[(u64, f32)]) -> Series {
        Series {
            color: ColorF::BLACK,
            interpolation: Interpolation::Linear,
            fill: None,
            points: points.to_vec(),
        }
    }

    #[test]
    fn bounds_spans_every_series() {
        let all = bounds(&[
            series(&[(0, 1.0), (10, 5.0)]),
            series(&[(5, -2.0), (20, 3.0)]),
        ])
        .unwrap();
        assert_eq!(all, (0, 20, -2.0, 5.0));
    }

    #[test]
    fn bounds_is_none_when_every_series_is_empty() {
        assert_eq!(bounds(&[series(&[]), series(&[])]), None);
    }

    #[test]
    fn nearest_point_picks_the_closer_neighbor() {
        let points = [(0, 0.0), (10, 1.0), (20, 2.0)];
        assert_eq!(nearest_point(&points, 4), Some((0, 0.0)));
        assert_eq!(nearest_point(&points, 6), Some((10, 1.0)));
        assert_eq!(nearest_point(&points, 10), Some((10, 1.0)));
    }

    #[test]
    fn nearest_point_clamps_to_the_ends() {
        let points = [(10, 0.0), (20, 1.0)];
        assert_eq!(nearest_point(&points, 0), Some((10, 0.0)));
        assert_eq!(nearest_point(&points, 100), Some((20, 1.0)));
    }

    #[test]
    fn a_frame_spans_the_data_or_the_window_up_to_the_newest_sample() {
        let data = [series(&[(1_000, 1.0), (2_000, 9.0), (4_000, 3.0)])];

        let all = Frame::of(&data, None, None, None).unwrap();
        assert_eq!(
            (all.from, all.to, all.min_v, all.max_v),
            (1_000.0, 4_000.0, 1.0, 9.0)
        );
        assert_eq!(all.x(2_500, 300.0), 150.0);

        let window = Frame::of(&data, Some(10_000), None, None).unwrap();
        assert_eq!((window.from, window.to), (-6_000.0, 4_000.0));
        assert_eq!(window.x(4_000, 100.0), 100.0);
        assert_eq!(window.x(1_000, 100.0), 70.0);

        let last = Frame::of(&data, Some(1_500), None, None).unwrap();
        assert_eq!(
            (last.min_v, last.max_v),
            (3.0, 3.0),
            "only the samples in view"
        );
        assert!(
            last.x(1_000, 100.0) < 0.0,
            "older samples fall off the left"
        );

        let fixed = Frame::of(&data, Some(1_500), Some((0.0, 100.0)), None).unwrap();
        assert_eq!((fixed.min_v, fixed.max_v), (0.0, 100.0));
        assert_eq!(fixed.y(50.0, 200.0), 100.0);
    }

    #[test]
    fn a_frame_given_an_end_ends_there_whatever_the_newest_sample() {
        let data = [series(&[(1_000, 1.0), (2_000, 9.0), (4_000, 3.0)])];

        let ahead = Frame::of(&data, Some(2_000), None, Some(4_500.0)).unwrap();
        assert_eq!((ahead.from, ahead.to), (2_500.0, 4_500.0));
        assert_eq!(
            ahead.x(4_000, 100.0),
            75.0,
            "the newest sample short of the edge"
        );
        assert_eq!((ahead.min_v, ahead.max_v), (3.0, 3.0));

        let behind = Frame::of(&data, Some(2_000), None, Some(3_000.0)).unwrap();
        assert!(
            behind.x(4_000, 100.0) > 100.0,
            "a sample past the edge waits off the right"
        );
        assert_eq!(
            (behind.min_v, behind.max_v),
            (1.0, 9.0),
            "and is not fitted until it is in view"
        );

        let unwindowed = Frame::of(&data, None, None, Some(9_000.0)).unwrap();
        assert_eq!(unwindowed.to, 4_000.0, "an end needs a window");
    }

    #[test]
    fn a_frame_names_the_multiples_in_view_and_the_time_under_a_point() {
        let frame = Frame::of(&[series(&[(1_500, 0.0), (4_200, 0.0)])], None, None, None).unwrap();
        assert_eq!(
            frame.multiples(1_000).collect::<Vec<_>>(),
            [2_000, 3_000, 4_000]
        );
        assert_eq!(frame.t_at(0.0, 270.0), 1_500);
        assert_eq!(frame.t_at(135.0, 270.0), 2_850);
        assert_eq!(frame.t_at(999.0, 270.0), 4_200);

        let early = Frame::of(&[series(&[(500, 0.0)])], Some(2_000), None, None).unwrap();
        assert_eq!(early.multiples(1_000).collect::<Vec<_>>(), [0]);
        assert_eq!(early.t_at(0.0, 100.0), 0);
    }

    #[test]
    fn nearest_point_of_empty_series_is_none() {
        assert_eq!(nearest_point(&[], 5), None);
    }
}
