use std::collections::VecDeque;

/// One reading: when it was taken, and what it was.
pub type Sample = (u64, f32);

#[derive(Clone, Debug, PartialEq)]
pub struct RingSeries {
    capacity: usize,
    points: VecDeque<(u64, f32)>,
}

impl RingSeries {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            points: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, point: (u64, f32)) {
        if self.points.len() >= self.capacity {
            self.points.pop_front();
        }
        self.points.push_back(point);
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// The points as they sit in the ring: everything from the read position to
    /// the end, then everything that wrapped around. Either half may be empty.
    ///
    /// For reading without giving anything up. A chart wants one contiguous
    /// slice, so it takes [`into_points`](Self::into_points) instead.
    pub fn as_slices(&self) -> (&[Sample], &[Sample]) {
        self.points.as_slices()
    }

    /// Hands the points over as a contiguous `Vec`, consuming the ring.
    ///
    /// This moves rather than copies - `Vec::from(VecDeque)` reuses the ring's
    /// own allocation, at worst shifting it into place. Prefer it to
    /// [`as_points`](Self::as_points) wherever the ring is owned and done with;
    /// a reducer's state is shared, so read that one with `as_points`.
    pub fn into_points(self) -> Vec<(u64, f32)> {
        Vec::from(self.points)
    }

    /// Copies the points out, leaving the ring alone.
    pub fn as_points(&self) -> Vec<(u64, f32)> {
        self.points.iter().copied().collect()
    }
}

impl Default for RingSeries {
    fn default() -> Self {
        Self::new(120)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_under_capacity_keeps_everything_in_order() {
        let mut ring = RingSeries::new(3);
        ring.push((1, 1.0));
        ring.push((2, 2.0));

        assert_eq!(ring.as_points(), vec![(1, 1.0), (2, 2.0)]);
    }

    #[test]
    fn push_past_capacity_evicts_the_oldest_point() {
        let mut ring = RingSeries::new(3);
        ring.push((1, 1.0));
        ring.push((2, 2.0));
        ring.push((3, 3.0));
        ring.push((4, 4.0));

        assert_eq!(ring.as_points(), vec![(2, 2.0), (3, 3.0), (4, 4.0)]);
    }

    #[test]
    fn into_points_agrees_with_the_slices_it_replaces() {
        let mut ring = RingSeries::new(3);
        for t in 1..=4 {
            ring.push((t, t as f32));
        }

        // Wrapped, so the halves are genuinely split - the interesting case.
        let (front, back) = ring.as_slices();
        assert!(
            !front.is_empty() && !back.is_empty(),
            "expected a wrapped ring"
        );

        let joined: Vec<_> = front.iter().chain(back).copied().collect();
        assert_eq!(ring.clone().into_points(), joined);
        assert_eq!(ring.as_points(), joined);
        assert_eq!(ring.len(), 3);
    }
}
