//! What the process held, as a line under the seconds and on the timeline.

use egui::{Pos2, Rect, Stroke, pos2};
use guinea_devtools_model::memory::MemorySample;

use crate::theme;

/// How tall a memory strip is.
pub const TALL: f32 = 30.0;

/// `bytes` as a person reads it: `640 KB`, `312 MB`, `1.25 GB`.
pub fn bytes(bytes: u64) -> String {
    let Some((unit, size)) = [("GB", 30), ("MB", 20), ("KB", 10)]
        .into_iter()
        .find(|(_, shift)| bytes >= 1 << shift)
    else {
        return format!("{bytes} B");
    };
    let value = bytes as f64 / (1u64 << size) as f64;
    let digits = if value >= 100.0 {
        0
    } else if value >= 10.0 {
        1
    } else {
        2
    };
    format!("{value:.digits$} {unit}")
}

/// What a reading says, for a tooltip.
pub fn said(sample: MemorySample) -> String {
    format!(
        "private {} · working set {}",
        bytes(sample.private),
        bytes(sample.working_set)
    )
}

/// Draws `points` - where each reading is across, and what it held - as a
/// line stepping from one reading to the next, the least of them at the
/// bottom of `rect` and the most at the top.
pub fn line(painter: &egui::Painter, rect: Rect, points: &[(f32, u64)]) {
    let (Some(low), Some(high)) = (
        points.iter().map(|(_, held)| *held).min(),
        points.iter().map(|(_, held)| *held).max(),
    ) else {
        return;
    };
    let range = (high - low).max(1) as f32;
    let y = |held: u64| rect.bottom() - 2.0 - (held - low) as f32 / range * (rect.height() - 4.0);

    let mut stepped: Vec<Pos2> = Vec::with_capacity(points.len() * 2);
    for (index, (x, held)) in points.iter().enumerate() {
        let x = x.clamp(rect.left(), rect.right());
        if index > 0 {
            stepped.push(pos2(x, stepped[stepped.len() - 1].y));
        }
        stepped.push(pos2(x, y(*held)));
    }
    if let Some(last) = stepped.last().copied() {
        stepped.push(pos2(rect.right(), last.y));
    }
    painter.add(egui::Shape::line(stepped, Stroke::new(1.5, theme::SKY)));
    let font = egui::FontId::proportional(10.0);
    painter.text(
        rect.right_top() + egui::vec2(-4.0, 1.0),
        egui::Align2::RIGHT_TOP,
        bytes(high),
        font.clone(),
        theme::MUTED,
    );
    painter.text(
        rect.right_bottom() + egui::vec2(-4.0, -1.0),
        egui::Align2::RIGHT_BOTTOM,
        bytes(low),
        font,
        theme::MUTED,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_in_the_unit_that_keeps_them_short() {
        assert_eq!(
            [
                bytes(900),
                bytes(640 << 10),
                bytes(312 << 20),
                bytes((1 << 30) + (256 << 20)),
            ],
            ["900 B", "640 KB", "312 MB", "1.25 GB"]
        );
    }
}
