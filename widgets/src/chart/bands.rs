use super::geometry::Frame;
use super::{BandInterpolation, ChartBand, XSelection};
use crate::painted::Metrics;
use kurbo::{CubicBez, ParamCurve, Point};
use std::rc::Rc;
use windows_canvas::{Brush, DrawingSession, GpuDevice, Path, PathBuilder, Vector2};
use windows_core::Interface;
use windows_numerics::Matrix3x2;

#[derive(Clone, Default, PartialEq)]
pub(super) struct Bands {
    pub layers: Vec<ChartBand>,
    pub clip: Option<XSelection>,
    pub interpolation: BandInterpolation,
}

// Clip the domain before building geometry; never scale values to a total.
fn interval(
    a: (u64, f32, f32),
    b: (u64, f32, f32),
    clip: Option<XSelection>,
) -> Option<[(u64, f32, f32); 2]> {
    if b.0 <= a.0 || ![a.1, a.2, b.1, b.2].iter().all(|v| v.is_finite()) || a.1 > a.2 || b.1 > b.2 {
        return None;
    }
    let (from, to) = clip.unwrap_or((a.0, b.0));
    let (from, to) = (a.0.max(from), b.0.min(to));
    if to <= from {
        return None;
    }
    let sample = |t| {
        let k = (t - a.0) as f64 / (b.0 - a.0) as f64;
        (
            t,
            (a.1 as f64 + (b.1 as f64 - a.1 as f64) * k) as f32,
            (a.2 as f64 + (b.2 as f64 - a.2 as f64) * k) as f32,
        )
    };
    Some([sample(from), sample(to)])
}

pub(super) fn hit(bands: &Bands, frame: Frame, metrics: Metrics, x: f32, y: f32) -> Option<u64> {
    if !x.is_finite()
        || !y.is_finite()
        || metrics.width <= 0.0
        || metrics.height <= 0.0
        || x < 0.0
        || y < 0.0
        || x > metrics.width
        || y > metrics.height
    {
        return None;
    }
    let t = frame.from + x as f64 / metrics.width as f64 * (frame.to - frame.from);
    if bands
        .clip
        .is_some_and(|(a, b)| t < a as f64 || t > b as f64)
    {
        return None;
    }
    bands.layers.iter().rev().find_map(|layer| {
        segments(layer, frame, metrics, bands.interpolation)
            .into_iter()
            .find_map(|s| {
                if t < s.from as f64 || t > s.to as f64 {
                    return None;
                }
                let k = (t - s.from as f64) / (s.to - s.from) as f64;
                let lo = s.lower.eval(k).y;
                let hi = s.upper.eval(k).y;
                (lo > hi && y as f64 >= hi && y as f64 <= lo).then_some(layer.id)
            })
    })
}

struct Segment {
    from: u64,
    to: u64,
    lower: CubicBez,
    upper: CubicBez,
}

fn boundary(points: &[Vector2], i: usize, mode: BandInterpolation) -> CubicBez {
    let a = points[i];
    let b = points[i + 1];
    let valid = |p: Vector2| p.x.is_finite() && p.y.is_finite();
    let (c1, c2) = if mode == BandInterpolation::Monotone {
        let start = if i > 0 && valid(points[i - 1]) && points[i - 1].x < a.x {
            i - 1
        } else {
            i
        };
        let end = if i + 2 < points.len() && valid(points[i + 2]) && points[i + 2].x > b.x {
            i + 3
        } else {
            i + 2
        };
        super::paint::monotone_controls(&points[start..end], i - start)
    } else {
        (
            Vector2 {
                x: a.x + (b.x - a.x) / 3.0,
                y: a.y + (b.y - a.y) / 3.0,
            },
            Vector2 {
                x: b.x - (b.x - a.x) / 3.0,
                y: b.y - (b.y - a.y) / 3.0,
            },
        )
    };
    let p = |p: Vector2| Point::new(p.x as f64, p.y as f64);
    CubicBez::new(p(a), p(c1), p(c2), p(b))
}
fn segments(
    layer: &ChartBand,
    frame: Frame,
    metrics: Metrics,
    mode: BandInterpolation,
) -> Vec<Segment> {
    let lower: Vec<_> = layer
        .points
        .iter()
        .map(|&(t, lo, _)| Vector2 {
            x: frame.x(t, metrics.width),
            y: frame.y(lo, metrics.height),
        })
        .collect();
    let upper: Vec<_> = layer
        .points
        .iter()
        .map(|&(t, _, hi)| Vector2 {
            x: frame.x(t, metrics.width),
            y: frame.y(hi, metrics.height),
        })
        .collect();
    layer
        .points
        .windows(2)
        .enumerate()
        .filter_map(|(i, p)| {
            interval(p[0], p[1], None)?;
            if lower[i + 1].x <= lower[i].x {
                return None;
            }
            let lo = boundary(&lower, i, mode);
            let hi = boundary(&upper, i, mode);
            if ![lo.p0, lo.p1, lo.p2, lo.p3, hi.p0, hi.p1, hi.p2, hi.p3]
                .iter()
                .all(|p| p.x.is_finite() && p.y.is_finite())
            {
                return None;
            }
            Some(Segment {
                from: p[0].0,
                to: p[1].0,
                lower: lo,
                upper: hi,
            })
        })
        .collect()
}
// kurbo handles cubic roots and exact subdivision. A reversed interpolated area
// is a gap, never a rescaled or fabricated contribution to the total.
fn visible_parts(s: &Segment, clip: Option<XSelection>) -> Vec<(CubicBez, CubicBez)> {
    let (from, to) = clip.unwrap_or((s.from, s.to));
    let a = (from.max(s.from) - s.from) as f64 / (s.to - s.from) as f64;
    let b = (to.min(s.to).saturating_sub(s.from)) as f64 / (s.to - s.from) as f64;
    if b <= a || a >= 1.0 {
        return vec![];
    }
    let d = [
        s.lower.p0.y - s.upper.p0.y,
        s.lower.p1.y - s.upper.p1.y,
        s.lower.p2.y - s.upper.p2.y,
        s.lower.p3.y - s.upper.p3.y,
    ];
    let roots = kurbo::common::solve_cubic(
        d[0],
        3.0 * (d[1] - d[0]),
        3.0 * (d[0] - 2.0 * d[1] + d[2]),
        d[3] - d[0] + 3.0 * (d[1] - d[2]),
    );
    let mut splits = vec![a, b];
    splits.extend(roots.into_iter().filter(|r| *r > a && *r < b));
    splits.sort_by(f64::total_cmp);
    splits.dedup();
    splits
        .windows(2)
        .filter_map(|p| {
            let mid = (p[0] + p[1]) / 2.0;
            (s.lower.eval(mid).y > s.upper.eval(mid).y).then(|| {
                (
                    s.lower.subsegment(p[0]..p[1]),
                    s.upper.subsegment(p[0]..p[1]),
                )
            })
        })
        .collect()
}

#[derive(Default)]
pub(super) struct Cache {
    source: Option<Rc<Bands>>,
    frame: Option<Frame>,
    metrics: Option<Metrics>,
    device: usize,
    paths: Vec<(u64, Path, Brush, Brush)>,
}
impl Cache {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn paint(
        &mut self,
        draw: &DrawingSession<'_>,
        device: &GpuDevice,
        metrics: Metrics,
        frame: Frame,
        source: &Rc<Bands>,
        highlight: Option<u64>,
    ) {
        let identity = device.d2d_device().as_raw() as usize;
        let valid = self.source.as_ref().is_some_and(|s| Rc::ptr_eq(s, source))
            && self.metrics == Some(metrics)
            && self.device == identity
            && self.frame.is_some_and(|f| {
                f.to - f.from == frame.to - frame.from
                    && f.min_v == frame.min_v
                    && f.max_v == frame.max_v
            });
        if !valid {
            self.clear();
            let build = || -> windows_canvas::Result<Vec<_>> {
                source
                    .layers
                    .iter()
                    .map(|layer| {
                        let mut builder = PathBuilder::new(device)?;
                        for segment in segments(layer, frame, metrics, source.interpolation) {
                            for (lower, upper) in visible_parts(&segment, source.clip) {
                                let v = |p: Point| Vector2 {
                                    x: p.x as f32,
                                    y: p.y as f32,
                                };
                                builder = builder
                                    .begin(v(lower.p0))
                                    .bezier_to(v(lower.p1), v(lower.p2), v(lower.p3))
                                    .line_to(v(upper.p3))
                                    .bezier_to(v(upper.p2), v(upper.p1), v(upper.p0))
                                    .close();
                            }
                        }
                        let path = builder.build()?;
                        let color = layer.color;
                        let mut bright = color;
                        let adjust = |v: f32| {
                            if (color.r + color.g + color.b) / 3.0 > 0.5 {
                                v * 0.65
                            } else {
                                (v + 0.35).min(1.0)
                            }
                        };
                        bright.r = adjust(bright.r);
                        bright.g = adjust(bright.g);
                        bright.b = adjust(bright.b);
                        Ok((
                            layer.id,
                            path,
                            draw.create_solid_brush(color)?,
                            draw.create_solid_brush(bright)?,
                        ))
                    })
                    .collect()
            };
            let paths = match build() {
                Ok(p) => p,
                Err(error) => {
                    tracing::warn!(%error,"chart: band geometry failed");
                    return;
                }
            };
            self.paths = paths;
            self.source = Some(source.clone());
            self.frame = Some(frame);
            self.metrics = Some(metrics);
            self.device = identity;
        }
        let anchor = self.frame.unwrap_or(frame);
        let shift = ((anchor.from - frame.from) / (frame.to - frame.from)) as f32 * metrics.width;
        draw.with_transform(
            &(Matrix3x2::translation(shift, 0.0) * draw.transform()),
            || {
                for (id, path, brush, bright) in &self.paths {
                    draw.fill_path(
                        path,
                        if highlight == Some(*id) {
                            bright
                        } else {
                            brush
                        },
                    );
                }
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame() -> Frame {
        Frame {
            from: 0.0,
            to: 100.0,
            min_v: 0.0,
            max_v: 100.0,
        }
    }
    fn layer(id: u64, lo: f32, hi: f32) -> ChartBand {
        ChartBand {
            id,
            color: crate::color::hex(0xffffff),
            points: vec![(0, lo, hi), (100, lo, hi)],
        }
    }
    #[test]
    fn monotone_top_reuses_total_controls_and_clipping_preserves_curve() {
        let m = Metrics::new(100.0, 100.0, 1.0);
        let data = ChartBand {
            id: 1,
            color: crate::color::hex(0xff0000),
            points: vec![
                (0, 0.0, 20.0),
                (20, 0.0, 90.0),
                (80, 0.0, 30.0),
                (100, 0.0, 70.0),
            ],
        };
        let segments = segments(&data, frame(), m, BandInterpolation::Monotone);
        let points: Vec<_> = data
            .points
            .iter()
            .map(|&(t, _, v)| Vector2 {
                x: frame().x(t, 100.0),
                y: frame().y(v, 100.0),
            })
            .collect();
        let bands = Bands {
            layers: vec![data.clone()],
            clip: Some((25, 75)),
            interpolation: BandInterpolation::Monotone,
        };
        assert_eq!(hit(&bands, frame(), m, 35.0, 22.0), Some(1));
        let linear = Bands {
            interpolation: BandInterpolation::Linear,
            ..bands
        };
        assert_eq!(hit(&linear, frame(), m, 35.0, 22.0), None);
        let (c1, c2) = super::super::paint::monotone_controls(&points, 1);
        assert_eq!(segments[1].upper.p1, Point::new(c1.x as f64, c1.y as f64));
        assert_eq!(segments[1].upper.p2, Point::new(c2.x as f64, c2.y as f64));
        let parts = visible_parts(&segments[1], Some((25, 75)));
        assert_eq!(parts.len(), 1);
        for k in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let original = segments[1].upper.eval((5.0 + 50.0 * k) / 60.0);
            assert!(parts[0].1.eval(k).distance(original) < 1e-10);
        }
        // Ordered knots alone need not keep two independent cubics ordered.
        let crossing = ChartBand {
            id: 2,
            color: data.color,
            points: vec![(0, 0.0, 1.0), (50, 1.0, 1.0), (100, 2.0, 2.0)],
        };
        let curves = super::segments(&crossing, frame(), m, BandInterpolation::Monotone);
        assert!(
            visible_parts(&curves[1], None).is_empty(),
            "no artificial area from reversed cubic bounds"
        );
    }

    #[test]
    fn monotone_bands_are_above_original_fill_below_stroke_and_clipped() {
        use super::super::model::{Interpolation, LineChartOptions, Series};
        let device = GpuDevice::new_warp().unwrap();
        let target = device.create_render_target(100, 100).unwrap();
        let metrics = Metrics::new(100.0, 100.0, 1.0);
        let total = Series {
            color: crate::color::hex(0xffffff),
            interpolation: Interpolation::Monotone,
            fill: Some(crate::color::hex_alpha(0x0088ff, 100)),
            points: vec![(0, 20.0), (20, 90.0), (80, 30.0), (100, 70.0)],
        };
        let options = LineChartOptions {
            border: None,
            y_range: Some((0.0, 100.0)),
            ..Default::default()
        };
        let mut cache = super::super::paint::Cache::default();
        let mut render = |layers| {
            cache.bands = Rc::new(Bands {
                layers,
                clip: Some((25, 75)),
                interpolation: BandInterpolation::Monotone,
            });
            target
                .draw(|d| {
                    d.clear(windows_canvas::ColorF::TRANSPARENT);
                    super::super::paint::render(
                        d,
                        &device,
                        metrics,
                        std::slice::from_ref(&total),
                        &options,
                        None,
                        &mut cache,
                    );
                    Ok(())
                })
                .unwrap();
            target.read_pixels().unwrap()
        };
        let baseline = render(vec![]);
        let composed = render(vec![ChartBand {
            id: 1,
            color: crate::color::hex(0xff0000),
            points: total.points.iter().map(|&(t, v)| (t, 0.0, v)).collect(),
        }]);
        for y in 0..100 {
            for x in 0..100 {
                let i = (y * 100 + x) * 4;
                if !(25..75).contains(&x) {
                    assert_eq!(&baseline[i..i + 4], &composed[i..i + 4]);
                }
                if baseline[i + 1] > 245 {
                    assert!(composed[i + 1] > 220, "stroke covered at {x},{y}");
                }
            }
        }
        let inside = (90 * 100 + 50) * 4;
        assert!(
            composed[inside + 2] > 240 && composed[inside] < 15,
            "original blue fill must be behind red bands"
        );
    }

    #[test]
    fn identity_hit_respects_clip_and_keeps_overshoot_and_gaps() {
        let bands = Bands {
            layers: vec![layer(1, 0.0, 40.0), layer(42, 40.0, 90.0)],
            clip: Some((25, 75)),
            ..Default::default()
        };
        let m = Metrics::new(100.0, 100.0, 1.0);
        assert_eq!(hit(&bands, frame(), m, 50.0, 20.0), Some(42));
        assert_eq!(hit(&bands, frame(), m, 50.0, 80.0), Some(1));
        assert_eq!(hit(&bands, frame(), m, 24.0, 20.0), None);
        assert_eq!(hit(&bands, frame(), m, 76.0, 20.0), None);
        assert_eq!(hit(&bands, frame(), m, 50.0, -1.0), None);
        assert!(interval((0, 0.0, 40.0), (100, f32::NAN, 40.0), None).is_none());
        assert!(interval((0, 50.0, 40.0), (100, 50.0, 40.0), None).is_none());
        assert!(interval((100, 0.0, 40.0), (0, 0.0, 40.0), None).is_none());
        let clipped = interval((0, 0.0, 10.0), (100, 20.0, 50.0), Some((25, 75))).unwrap();
        assert_eq!(clipped, [(25, 5.0, 20.0), (75, 15.0, 40.0)]);
    }
    #[test]
    fn raster_clip_preserves_total_geometry_and_highlight_uses_identity() {
        use super::super::model::{Interpolation, LineChartOptions, Series};
        let device = GpuDevice::new_warp().unwrap();
        let target = device.create_render_target(100, 100).unwrap();
        let metrics = Metrics::new(100.0, 100.0, 1.0);
        let series = [Series {
            color: crate::color::hex(0xffffff),
            interpolation: Interpolation::Linear,
            fill: None,
            points: vec![(0, 80.0), (100, 80.0)],
        }];
        let options = LineChartOptions {
            border: None,
            y_range: Some((0.0, 100.0)),
            ..Default::default()
        };
        let mut cache = super::super::paint::Cache::default();
        let mut render = |bands: Vec<ChartBand>, highlight| {
            cache.bands = Rc::new(Bands {
                layers: bands,
                clip: Some((25, 75)),
                ..Default::default()
            });
            cache.highlighted = highlight;
            target
                .draw(|draw| {
                    super::super::paint::render(
                        draw, &device, metrics, &series, &options, None, &mut cache,
                    );
                    Ok(())
                })
                .unwrap();
            target.read_pixels().unwrap()
        };
        let base = render(vec![], None);
        let band = render(
            vec![layer(42, 20.0, 50.0), layer(77, f32::NAN, f32::NAN)],
            None,
        );
        let bright = render(
            vec![layer(42, 20.0, 50.0), layer(77, f32::NAN, f32::NAN)],
            Some(42),
        );
        for y in 0..100 {
            for x in 0..100 {
                let i = (y * 100 + x) * 4;
                if !(25..75).contains(&x) || y < 40 {
                    assert_eq!(
                        &base[i..i + 4],
                        &band[i..i + 4],
                        "outside selected area {x},{y}"
                    );
                }
            }
        }
        assert!(base != band, "band fill is missing");
        assert!(
            band != bright,
            "white band must still have a visible highlight"
        );
    }
}
