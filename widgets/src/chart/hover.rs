//! Reading a value off the chart at a horizontal position.

use super::geometry::Frame;
use super::model::{HoverInfo, LineChartOptions, Series};
use super::{bounds, nearest_point};

/// The values under `pixel_x`, or `None` when there is nothing to report -
/// no data, no width yet, or a single instant on the time axis.
pub(super) fn hover_at(
    series: &[Series],
    options: &LineChartOptions,
    pixel_x: f32,
    width: f32,
) -> Option<HoverInfo> {
    let (min_t, max_t, _, _) = bounds(series)?;
    if width <= 0.0 || max_t == min_t {
        return None;
    }

    let frame = Frame::of(series, options.x_window, options.y_range)?;
    let target = frame.t_at(pixel_x, width);

    let mut values = Vec::with_capacity(series.len());
    let mut any = false;
    for s in series {
        match nearest_point(&s.points, target) {
            Some((_, v)) => {
                values.push(v);
                any = true;
            }
            None => values.push(0.0),
        }
    }

    any.then_some(HoverInfo { x: target, values })
}
