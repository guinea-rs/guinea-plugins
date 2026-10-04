//! Where things fall on a scatter chart, and back: times and levels to DIPs,
//! a pointer to the point under it, a dragged rectangle to an area.

use super::model::{Area, Hit, Key, Level, Scale, ScatterOptions, ScatterSeries};

/// How wide the labels at the left are given, when there are any.
pub(super) const LABELS_WIDE: f32 = 56.0;
/// How tall the labels underneath are given, when there are any.
pub(super) const TICKS_TALL: f32 = 16.0;

/// The chart laid out at one size.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Plot {
    pub left: f32,
    pub right: f32,
    /// The top of the highest band, or of the scale without one.
    pub top: f32,
    /// The bottom of the lowest band, or of the scale without one.
    pub bottom: f32,
    pub scale_top: f32,
    pub scale_bottom: f32,
    pub band_height: f32,
    /// How many bands there are above the scale and below it.
    pub above: usize,
    pub below: usize,
    pub x: (u64, u64),
    pub y: Scale,
}

impl Plot {
    pub fn of(width: f32, height: f32, options: &ScatterOptions) -> Self {
        let labelled = !options.y_lines.is_empty()
            || !options.above.is_empty()
            || !options.below.is_empty();
        let left = if labelled { LABELS_WIDE } else { 0.0 };
        let bottom = height - if options.x_ticks.is_empty() { 0.0 } else { TICKS_TALL };
        let band_height = options.band_height.max(0.0);

        Self {
            left: left.min(width),
            right: width,
            top: 0.0,
            bottom,
            scale_top: options.above.len() as f32 * band_height,
            scale_bottom: bottom - options.below.len() as f32 * band_height,
            band_height,
            above: options.above.len(),
            below: options.below.len(),
            x: options.x,
            y: options.y,
        }
    }

    fn span(&self) -> f64 {
        self.x.1.saturating_sub(self.x.0).max(1) as f64
    }

    pub fn x(&self, at: u64) -> f32 {
        let part = (at as f64 - self.x.0 as f64) / self.span();
        self.left + (part * (self.right - self.left) as f64) as f32
    }

    /// The time at `x`, within the span shown.
    pub fn at(&self, x: f32) -> u64 {
        let wide = (self.right - self.left).max(f32::EPSILON);
        let part = ((x - self.left) / wide).clamp(0.0, 1.0) as f64;
        self.x.0 + (part * self.span()).round() as u64
    }

    /// How far up the scale `value` is, from 0 at its bottom to 1 at its top.
    fn up(&self, value: f32) -> f32 {
        let part = match self.y {
            Scale::Linear { from, to } => (value - from) / (to - from),
            Scale::Log { from, to } => {
                let value = value.clamp(from.min(to), from.max(to));
                (value.log10() - from.log10()) / (to.log10() - from.log10())
            }
        };
        if part.is_finite() {
            part.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn value(&self, up: f32) -> f32 {
        match self.y {
            Scale::Linear { from, to } => from + (to - from) * up,
            Scale::Log { from, to } => {
                10f32.powf(from.log10() + (to.log10() - from.log10()) * up)
            }
        }
    }

    pub fn y(&self, level: Level) -> f32 {
        let tall = self.scale_bottom - self.scale_top;
        match level {
            Level::Value(value) => self.scale_bottom - self.up(value) * tall,
            Level::Above(band) => {
                let band = band.min(self.above.saturating_sub(1)) as f32;
                self.scale_top - (band + 0.5) * self.band_height
            }
            Level::Below(band) => {
                let band = band.min(self.below.saturating_sub(1)) as f32;
                self.scale_bottom + (band + 0.5) * self.band_height
            }
        }
    }

    /// The level at `y`: a band past the scale's ends, or a value on it.
    pub fn level(&self, y: f32) -> Level {
        let band = |past: f32, bands: usize| {
            ((past / self.band_height.max(f32::EPSILON)) as usize).min(bands - 1)
        };
        if y < self.scale_top && self.above > 0 {
            return Level::Above(band(self.scale_top - y, self.above));
        }
        if y > self.scale_bottom && self.below > 0 {
            return Level::Below(band(y - self.scale_bottom, self.below));
        }

        let tall = (self.scale_bottom - self.scale_top).max(f32::EPSILON);
        let up = ((self.scale_bottom - y) / tall).clamp(0.0, 1.0);
        Level::Value(self.value(up))
    }

    /// Where an area whose edge is `level` ends: at a value, there; in a
    /// band, at the band's far side, so that a band is taken whole.
    pub fn edge(&self, level: Level, top: bool) -> f32 {
        let half = self.band_height / 2.0;
        match level {
            Level::Value(_) => self.y(level),
            Level::Above(_) | Level::Below(_) if top => self.y(level) - half,
            Level::Above(_) | Level::Below(_) => self.y(level) + half,
        }
    }

    /// The point drawn nearest `(x, y)`, if it is within `reach`.
    pub fn nearest<K: Key>(
        &self,
        series: &[ScatterSeries<K>],
        x: f32,
        y: f32,
        reach: f32,
    ) -> Option<Hit<K>> {
        let mut nearest: Option<(f32, Hit<K>)> = None;
        for (index, line) in series.iter().enumerate() {
            for point in &line.points {
                let (px, py) = (self.x(point.at), self.y(point.value));
                let far = (px - x).powi(2) + (py - y).powi(2);
                if far > reach * reach || nearest.as_ref().is_some_and(|(best, _)| *best <= far) {
                    continue;
                }
                nearest = Some((
                    far,
                    Hit {
                        series: index,
                        key: point.key.clone(),
                        x: px,
                        y: py,
                    },
                ));
            }
        }

        nearest.map(|(_, hit)| hit)
    }

    /// The area between two corners of a dragged rectangle, either way round.
    pub fn area(&self, a: (f32, f32), b: (f32, f32)) -> Area {
        let (left, right) = (a.0.min(b.0), a.0.max(b.0));
        let (top, bottom) = (a.1.min(b.1), a.1.max(b.1));
        Area {
            x: (self.at(left), self.at(right)),
            y: (self.level(bottom), self.level(top)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::{Marker, ScatterPoint};
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    /// Where the scale starts, past the labels.
    const L: f32 = LABELS_WIDE;

    /// 0.1 s to 1000 s by powers of ten, a band above and one below, labels
    /// at the left and underneath: 300 DIPs past the labels by 216 leaves a
    /// scale 300 wide and 168 tall, from y 16 to 184.
    fn lifetimes() -> ScatterOptions {
        ScatterOptions {
            x: (1_000, 4_000),
            y: Scale::Log {
                from: 0.1,
                to: 1_000.0,
            },
            above: vec!["running".into()],
            below: vec!["unknown".into()],
            band_height: 16.0,
            y_lines: vec![(1.0, "1 s".into())],
            x_ticks: vec![(1_000, "12:00".into())],
            ..ScatterOptions::default()
        }
    }

    fn plot() -> Plot {
        Plot::of(L + 300.0, 216.0, &lifetimes())
    }

    #[test]
    fn the_scale_sits_between_its_bands_and_beside_its_labels() {
        let plot = plot();
        assert_eq!((plot.left, plot.right), (L, L + 300.0));
        assert_eq!((plot.top, plot.scale_top), (0.0, 16.0));
        assert_eq!((plot.scale_bottom, plot.bottom), (184.0, 200.0));
    }

    #[test]
    fn time_runs_left_to_right_and_back() {
        let plot = plot();
        assert!(close(plot.x(1_000), L));
        assert!(close(plot.x(2_500), L + 150.0));
        assert!(close(plot.x(4_000), L + 300.0));
        assert_eq!(plot.at(L + 150.0), 2_500);
        assert_eq!(plot.at(0.0), 1_000, "left of the scale is its start");
    }

    #[test]
    fn a_log_scale_gives_each_power_of_ten_the_same_height_and_clamps_at_its_ends() {
        let plot = plot();
        assert!(close(plot.y(Level::Value(0.1)), 184.0));
        assert!(close(plot.y(Level::Value(1.0)), 142.0));
        assert!(close(plot.y(Level::Value(1_000.0)), 16.0));
        assert!(close(plot.y(Level::Value(0.01)), 184.0), "below the bottom");
        assert!(close(plot.y(Level::Value(5_000.0)), 16.0), "past the top");

        match plot.level(142.0) {
            Level::Value(value) => assert!(close(value, 1.0), "{value}"),
            other => panic!("on the scale, not {other:?}"),
        }
    }

    #[test]
    fn a_linear_scale_is_even() {
        let plot = Plot::of(
            100.0,
            100.0,
            &ScatterOptions {
                y: Scale::Linear {
                    from: 0.0,
                    to: 10.0,
                },
                ..ScatterOptions::default()
            },
        );
        assert!(close(plot.y(Level::Value(5.0)), 50.0));
        assert_eq!(plot.level(25.0), Level::Value(7.5));
    }

    #[test]
    fn a_band_is_its_middle_and_its_middle_is_the_band() {
        let plot = plot();
        assert!(close(plot.y(Level::Above(0)), 8.0));
        assert!(close(plot.y(Level::Below(0)), 192.0));
        assert_eq!(plot.level(3.0), Level::Above(0));
        assert_eq!(plot.level(198.0), Level::Below(0));
    }

    fn dots(points: &[(u64, u64, Level)]) -> ScatterSeries {
        ScatterSeries {
            color: windows_canvas::ColorF {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            marker: Marker::Dot,
            size: 4.0,
            points: points
                .iter()
                .map(|&(key, at, value)| ScatterPoint { key, at, value })
                .collect(),
        }
    }

    #[test]
    fn the_nearest_point_in_both_directions_is_hovered_within_reach() {
        let plot = plot();
        let series = [
            dots(&[(1, 2_500, Level::Value(1.0)), (2, 2_520, Level::Value(1.0))]),
            dots(&[(3, 2_500, Level::Above(0))]),
        ];

        let hit = plot.nearest(&series, L + 150.0, 140.0, 8.0).expect("one is near");
        assert_eq!((hit.series, hit.key), (0, 1));
        assert!(close(hit.x, L + 150.0) && close(hit.y, 142.0), "{hit:?}");

        let band = plot.nearest(&series, L + 152.0, 10.0, 8.0).expect("the band's");
        assert_eq!((band.series, band.key), (1, 3));

        assert_eq!(plot.nearest(&series, L + 150.0, 100.0, 8.0), None, "too far below");
    }

    #[test]
    fn a_point_is_handed_back_by_the_pages_own_key() {
        let plot = plot();
        let blank = dots(&[]);
        let series = [ScatterSeries {
            color: blank.color,
            marker: blank.marker,
            size: blank.size,
            points: vec![ScatterPoint {
                key: (4242_u32, "second run".to_string()),
                at: 2_500,
                value: Level::Value(1.0),
            }],
        }];

        let hit = plot.nearest(&series, L + 150.0, 142.0, 8.0).expect("under it");
        assert_eq!(hit.key, (4242, "second run".to_string()));
    }

    #[test]
    fn an_area_reaching_into_a_band_takes_the_whole_band() {
        let plot = plot();
        assert_eq!(plot.edge(Level::Above(0), true), 0.0);
        assert_eq!(plot.edge(Level::Below(0), false), 200.0);
        assert!(close(plot.edge(Level::Value(1.0), true), 142.0));
    }

    #[test]
    fn a_dragged_rectangle_is_an_area_either_way_round() {
        let plot = plot();
        let area = plot.area((L + 300.0, 4.0), (L + 150.0, 142.0));
        assert_eq!(area.x, (2_500, 4_000));
        assert!(matches!(area.y.0, Level::Value(value) if close(value, 1.0)), "{area:?}");
        assert_eq!(area.y.1, Level::Above(0));
    }
}
