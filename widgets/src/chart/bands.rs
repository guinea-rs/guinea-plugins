use super::geometry::Frame;
use super::{ChartBand, XSelection};
use crate::painted::Metrics;
use std::rc::Rc;
use windows_canvas::{Brush, DrawingSession, GpuDevice, Path, PathBuilder, Vector2};
use windows_core::Interface;
use windows_numerics::Matrix3x2;

#[derive(Default, PartialEq)]
pub(super) struct Bands {
    pub layers: Vec<ChartBand>,
    pub clip: Option<XSelection>,
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
    let value =
        frame.min_v + (1.0 - y / metrics.height) * (frame.max_v - frame.min_v).max(f32::EPSILON);
    bands.layers.iter().rev().find_map(|layer| {
        layer.points.windows(2).find_map(|pair| {
            let [a, b] = interval(pair[0], pair[1], bands.clip)?;
            if t < a.0 as f64 || t > b.0 as f64 {
                return None;
            }
            let k = (t - a.0 as f64) / (b.0 - a.0) as f64;
            let lower = a.1 as f64 + (b.1 as f64 - a.1 as f64) * k;
            let upper = a.2 as f64 + (b.2 as f64 - a.2 as f64) * k;
            (upper > lower && value as f64 >= lower && value as f64 <= upper).then_some(layer.id)
        })
    })
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
                        for pair in layer.points.windows(2) {
                            if let Some([a, b]) = interval(pair[0], pair[1], source.clip) {
                                let p = |t, v| Vector2 {
                                    x: frame.x(t, metrics.width),
                                    y: frame.y(v, metrics.height),
                                };
                                builder = builder
                                    .begin(p(a.0, a.1))
                                    .line_to(p(b.0, b.1))
                                    .line_to(p(b.0, b.2))
                                    .line_to(p(a.0, a.2))
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
    fn identity_hit_respects_clip_and_keeps_overshoot_and_gaps() {
        let bands = Bands {
            layers: vec![layer(1, 0.0, 40.0), layer(42, 40.0, 90.0)],
            clip: Some((25, 75)),
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
