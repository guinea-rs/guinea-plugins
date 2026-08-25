//! Reading a value off the chart at a horizontal position.

use super::model::{HoverInfo, Series};
use super::{bounds, nearest_point};

/// The values under `pixel_x`, or `None` when there is nothing to report -
/// no data, no width yet, or a single instant on the time axis.
pub(super) fn hover_at(series: &[Series], pixel_x: f32, width: f32) -> Option<HoverInfo> {
    let (min_t, max_t, _, _) = bounds(series)?;
    if width <= 0.0 || max_t == min_t {
        return None;
    }

    let ratio = (pixel_x / width).clamp(0.0, 1.0);
    let target = min_t + ((max_t - min_t) as f32 * ratio) as u64;

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
