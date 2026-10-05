//! Where bars go on the timeline, and the stretch of time it shows.

/// The row of each interval, so that none overlaps another in its row:
/// each takes the first row free when it starts.
pub fn rows(spans: &[(i64, i64)]) -> Vec<usize> {
    let mut row = vec![0; spans.len()];
    let mut busy_until: Vec<i64> = Vec::new();

    for index in by_start(spans) {
        let (from, to) = spans[index];
        let free = busy_until.iter().position(|until| *until <= from);
        let at = free.unwrap_or(busy_until.len());
        if at == busy_until.len() {
            busy_until.push(to);
        } else {
            busy_until[at] = to;
        }
        row[index] = at;
    }

    row
}

/// How many of the others each interval lies inside.
pub fn nesting(spans: &[(i64, i64)]) -> Vec<usize> {
    let mut depth = vec![0; spans.len()];
    let mut open: Vec<i64> = Vec::new();

    for index in by_start(spans) {
        let (from, to) = spans[index];
        while open.last().is_some_and(|until| *until <= from) {
            open.pop();
        }
        depth[index] = open.len();
        open.push(to);
    }

    depth
}

/// The intervals' indices, earliest first, the longer first of two that
/// start together.
fn by_start(spans: &[(i64, i64)]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..spans.len()).collect();
    order.sort_by_key(|&index| (spans[index].0, std::cmp::Reverse(spans[index].1)));
    order
}

/// How long something was running within a stretch, and how many of the
/// intervals that make it up ran there.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Busy {
    pub us: i64,
    pub count: usize,
}

/// How long within `range` at least one of `spans` was running, and how
/// many of them overlap it.
pub fn busy(range: View, spans: &[(i64, i64)]) -> Busy {
    let mut inside: Vec<(i64, i64)> = spans
        .iter()
        .filter(|(from, to)| *from < range.to && *to > range.from)
        .map(|(from, to)| (*from.max(&range.from), *to.min(&range.to)))
        .collect();
    inside.sort_unstable();

    let mut us = 0;
    let mut reached = range.from;
    for (from, to) in &inside {
        let from = (*from).max(reached);
        if *to > from {
            us += to - from;
            reached = *to;
        }
    }

    Busy {
        us,
        count: inside.len(),
    }
}

/// A function that stayed on the sampled stack for samples in a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    /// How many calls lie outside it; the outermost is at 0.
    pub depth: usize,
    pub function: u32,
    pub from: i64,
    pub to: i64,
    pub samples: usize,
}

/// The runs of `samples`, each a time and a stack innermost call first, by
/// depth and then by time. A sample lasts until the next one, or `period`
/// when the next is more than two periods away; samples in a row that agree
/// from the outermost call down make one run at each depth they agree to.
pub fn runs(samples: &[(i64, &[u32])], period: i64) -> Vec<Run> {
    let mut open: Vec<Run> = Vec::new();
    let mut done: Vec<Run> = Vec::new();

    for (index, (at, stack)) in samples.iter().enumerate() {
        let to = match samples.get(index + 1) {
            Some((next, _)) if next - at <= 2 * period => *next,
            _ => at + period,
        };

        let mut agrees = true;
        for (depth, function) in stack.iter().rev().enumerate() {
            agrees &= open
                .get(depth)
                .is_some_and(|run| run.function == *function && run.to == *at);
            if agrees {
                open[depth].to = to;
                open[depth].samples += 1;
            } else {
                done.extend(open.drain(depth.min(open.len())..));
                open.push(Run {
                    depth,
                    function: *function,
                    from: *at,
                    to,
                    samples: 1,
                });
            }
        }
        if open.len() > stack.len() {
            done.extend(open.drain(stack.len()..));
        }
    }

    done.extend(open);
    done.sort_by_key(|run| (run.depth, run.from));
    done
}

/// `samples` with only the calls `keeps` keeps, leaving out the samples
/// whose innermost call `idles` says is waiting.
pub fn shown(
    samples: &[(i64, &[u32])],
    keeps: impl Fn(u32) -> bool,
    idles: impl Fn(u32) -> bool,
) -> Vec<(i64, Vec<u32>)> {
    samples
        .iter()
        .filter(|(_, stack)| !stack.first().is_some_and(|innermost| idles(*innermost)))
        .map(|(at, stack)| {
            let kept = stack.iter().copied().filter(|function| keeps(*function));
            (*at, kept.collect())
        })
        .collect()
}

/// How long a function was sampled as the innermost call, and on the stack
/// at all, in samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weight {
    pub function: u32,
    pub own: usize,
    pub total: usize,
}

/// Every function in `samples`, the most often innermost first.
pub fn heaviest(samples: &[(i64, &[u32])]) -> Vec<Weight> {
    let mut weights: std::collections::HashMap<u32, Weight> = std::collections::HashMap::new();
    let mut seen: Vec<u32> = Vec::new();

    for (_, stack) in samples {
        seen.clear();
        for (depth, function) in stack.iter().enumerate() {
            let weight = weights.entry(*function).or_insert(Weight {
                function: *function,
                own: 0,
                total: 0,
            });
            if depth == 0 {
                weight.own += 1;
            }
            if !seen.contains(function) {
                weight.total += 1;
                seen.push(*function);
            }
        }
    }

    let mut weights: Vec<Weight> = weights.into_values().collect();
    weights.sort_by_key(|weight| {
        (
            std::cmp::Reverse(weight.own),
            std::cmp::Reverse(weight.total),
            weight.function,
        )
    });
    weights
}

/// The last segment of a function's path, its type's and generics' paths
/// left whole: `Vec<T>::push` of `alloc::vec::Vec<T>::push`.
pub fn last_segment(name: &str) -> &str {
    let mut depth = 0i32;
    let mut cuts: Vec<usize> = Vec::new();
    let bytes = name.as_bytes();
    for (at, byte) in bytes.iter().enumerate() {
        match byte {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b':' if depth == 0 && bytes.get(at + 1) == Some(&b':') => cuts.push(at + 2),
            _ => {}
        }
    }

    match cuts.as_slice() {
        [.., before, last] if name[*before..*last].contains('<') => &name[*before..],
        [.., last] => &name[*last..],
        [] => name,
    }
}

/// A stretch of time in microseconds, `from` before `to`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View {
    pub from: i64,
    pub to: i64,
}

/// The narrowest stretch a view zooms in to.
pub const NARROWEST_US: i64 = 20;

impl View {
    pub fn length(self) -> i64 {
        self.to - self.from
    }

    /// Narrowed by `factor` (above one zooms in) around `at`, which stays
    /// where it was; never wider than `within`.
    pub fn zoomed(self, at: i64, factor: f32, within: View) -> View {
        let length = ((self.length() as f64 / f64::from(factor.max(f32::EPSILON))).round() as i64)
            .clamp(NARROWEST_US.min(within.length()), within.length());
        let share = (at - self.from) as f64 / self.length().max(1) as f64;
        let from = at - (share * length as f64).round() as i64;
        View {
            from,
            to: from + length,
        }
        .panned(0, within)
    }

    /// Moved later by `by`, kept inside `within`.
    pub fn panned(self, by: i64, within: View) -> View {
        let length = self.length().min(within.length());
        let from = (self.from + by).clamp(within.from, within.to - length);
        View {
            from,
            to: from + length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_interval_takes_the_first_row_free_when_it_starts() {
        let spans = [(0, 100), (10, 20), (30, 40), (15, 35), (100, 120)];

        assert_eq!(rows(&spans), [0, 1, 1, 2, 0]);
    }

    #[test]
    fn nesting_counts_the_intervals_around_each() {
        let spans = [(0, 100), (10, 60), (20, 30), (40, 50), (70, 90), (100, 110)];

        assert_eq!(nesting(&spans), [0, 1, 2, 2, 1, 0]);
    }

    #[test]
    fn nesting_does_not_care_what_order_the_intervals_came_in() {
        let spans = [(20, 30), (0, 100), (10, 60)];

        assert_eq!(nesting(&spans), [2, 0, 1]);
    }

    #[test]
    fn busy_time_counts_overlapping_intervals_once_and_only_inside_the_range() {
        let range = View { from: 10, to: 50 };
        let spans = [(0, 10), (0, 20), (15, 30), (40, 60), (70, 80)];

        assert_eq!(busy(range, &spans), Busy { us: 30, count: 3 });
    }

    const A: u32 = 0;
    const B: u32 = 1;
    const C: u32 = 2;

    #[test]
    fn samples_that_agree_from_the_outermost_call_make_one_run() {
        let samples: [(i64, &[u32]); 4] =
            [(0, &[B, A]), (1, &[C, A]), (2, &[C, A]), (10, &[C, A])];

        let run = |depth, function, from, to, samples| Run {
            depth,
            function,
            from,
            to,
            samples,
        };
        assert_eq!(
            runs(&samples, 1),
            [
                run(0, A, 0, 3, 3),
                run(0, A, 10, 11, 1),
                run(1, B, 0, 1, 1),
                run(1, C, 1, 3, 2),
                run(1, C, 10, 11, 1),
            ]
        );
    }

    #[test]
    fn hidden_calls_leave_the_stack_and_waiting_samples_leave_altogether() {
        const SYSTEM: u32 = 9;
        const WAIT: u32 = 8;
        let samples: [(i64, &[u32]); 4] = [
            (0, &[SYSTEM, B, SYSTEM, A]),
            (1, &[WAIT, SYSTEM, A]),
            (2, &[SYSTEM, WAIT]),
            (3, &[C, A]),
        ];

        let shown = shown(
            &samples,
            |function| function != SYSTEM && function != WAIT,
            |function| function == WAIT,
        );

        assert_eq!(
            shown,
            [(0, vec![B, A]), (2, vec![]), (3, vec![C, A])],
            "a wait under other calls is not the thread waiting"
        );
    }

    #[test]
    fn the_heaviest_function_is_the_one_most_often_innermost() {
        let samples: [(i64, &[u32]); 3] = [(0, &[B, A]), (1, &[C, A]), (2, &[C, A])];

        let weight = |function, own, total| Weight {
            function,
            own,
            total,
        };
        assert_eq!(
            heaviest(&samples),
            [weight(C, 2, 2), weight(B, 1, 1), weight(A, 0, 3)]
        );
    }

    #[test]
    fn a_function_is_called_by_its_last_segment_with_its_type() {
        assert_eq!(last_segment("uniproc::ui::Activity::render"), "render");
        assert_eq!(
            last_segment("alloc::vec::Vec<u8,alloc::alloc::Global>::push"),
            "Vec<u8,alloc::alloc::Global>::push"
        );
        assert_eq!(
            last_segment("core::ptr::drop_in_place<alloc::string::String>"),
            "drop_in_place<alloc::string::String>"
        );
        assert_eq!(
            last_segment("Microsoft.UI.Xaml.dll+0x1a2b"),
            "Microsoft.UI.Xaml.dll+0x1a2b"
        );
    }

    #[test]
    fn zooming_keeps_the_moment_under_the_pointer_in_place() {
        let within = View { from: 0, to: 1000 };
        let view = View { from: 0, to: 1000 };

        let zoomed = view.zoomed(250, 2.0, within);

        assert_eq!(zoomed, View { from: 125, to: 625 });
    }

    #[test]
    fn zooming_out_stops_at_everything_and_in_at_the_narrowest() {
        let within = View { from: 0, to: 1000 };

        assert_eq!(View { from: 400, to: 600 }.zoomed(500, 0.01, within), within);
        assert_eq!(
            View { from: 400, to: 600 }.zoomed(500, 1000.0, within).length(),
            NARROWEST_US
        );
    }

    #[test]
    fn panning_stops_at_either_end() {
        let within = View { from: 0, to: 1000 };
        let view = View { from: 100, to: 300 };

        assert_eq!(view.panned(50, within), View { from: 150, to: 350 });
        assert_eq!(view.panned(-500, within), View { from: 0, to: 200 });
        assert_eq!(view.panned(5000, within), View { from: 800, to: 1000 });
    }
}
