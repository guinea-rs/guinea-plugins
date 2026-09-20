//! A tree drawn like lines of code: one row per node, a fixed step per
//! level, the arrow in a column of its own and a guide down every level.
//! Which nodes there are and which are open is the caller's.

use egui::text::LayoutJob;
use egui::{Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::theme;

use super::block::PADDING;

/// How far one level sits from the one above it; also the arrow's column.
const STEP: f32 = 14.0;
/// The side of the arrow.
const SIDE: f32 = 7.0;

/// One row as the tree draws it.
pub struct Line {
    pub depth: usize,
    /// It has children, so it has an arrow.
    pub branch: bool,
    pub open: bool,
    pub selected: bool,
    pub text: LayoutJob,
}

/// What the pointer did to the rows, by index.
#[derive(Default)]
pub struct Answer {
    /// The arrow was clicked or the row double-clicked.
    pub toggled: Option<usize>,
    pub clicked: Option<usize>,
    pub hovered: Option<usize>,
}

/// An equilateral triangle in the middle of `cell`, pointing right when
/// closed and down when open.
fn arrow(painter: &egui::Painter, cell: Rect, open: bool) {
    let center = cell.center();
    let half = SIDE / 2.0;
    let height = SIDE * 3f32.sqrt() / 2.0;

    let points = if open {
        vec![
            center + vec2(-half, -height / 3.0),
            center + vec2(half, -height / 3.0),
            center + vec2(0.0, height * 2.0 / 3.0),
        ]
    } else {
        vec![
            center + vec2(-height / 3.0, -half),
            center + vec2(height * 2.0 / 3.0, 0.0),
            center + vec2(-height / 3.0, half),
        ]
    };

    painter.add(Shape::convex_polygon(points, theme::MUTED, Stroke::NONE));
}

/// Draws `count` rows, asking `line` only for the ones in view, and scrolls
/// `reveal` into view.
pub fn show(
    ui: &mut egui::Ui,
    count: usize,
    reveal: Option<usize>,
    line: impl Fn(&egui::Ui, usize) -> Line,
) -> Answer {
    let height = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;
    let padding = f32::from(PADDING);
    ui.spacing_mut().item_spacing.y = 0.0;

    let mut area = egui::ScrollArea::both().auto_shrink(false);
    if let Some(index) = reveal {
        area = area.vertical_scroll_offset((index as f32 * height - 6.0 * height).max(0.0));
    }

    let mut answer = Answer::default();
    area.show_rows(ui, height, count, |ui, range| {
        ui.spacing_mut().item_spacing.y = 0.0;

        for index in range {
            let line = line(ui, index);
            let galley = ui.painter().layout_job(line.text);
            let indent = padding + line.depth as f32 * STEP;
            let width = (indent + STEP + galley.size().x + padding).max(ui.available_width());

            let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, line.selected, galley.text())
            });

            let painter = ui.painter();
            if line.selected {
                painter.rect_filled(rect, 0, theme::SELECTED);
            } else if response.hovered() {
                painter.rect_filled(rect, 0, theme::FIELD);
            }

            for level in 0..line.depth {
                let x = rect.left() + padding + level as f32 * STEP + STEP / 2.0;
                painter.vline(x, rect.y_range(), Stroke::new(1.0, theme::DIVIDER));
            }

            let cell = Rect::from_min_size(pos2(rect.left() + indent, rect.top()), vec2(STEP, height));
            let mut on_arrow = false;
            if line.branch {
                arrow(painter, cell, line.open);
                if ui.interact(cell, ui.id().with(("arrow", index)), Sense::click()).clicked() {
                    on_arrow = true;
                    answer.toggled = Some(index);
                }
            }

            ui.painter().galley(
                pos2(cell.right(), rect.center().y - galley.size().y / 2.0),
                galley,
                theme::TEXT,
            );

            if response.hovered() {
                answer.hovered = Some(index);
            }
            if response.double_clicked() && line.branch {
                answer.toggled = Some(index);
            } else if response.clicked() && !on_arrow {
                answer.clicked = Some(index);
            }
        }
    });

    answer
}
