//! The element tree, drawn like lines of code: one row per element, a fixed
//! step per level, the arrow in a column of its own and a guide down every
//! level. What the rows are is the model's.

use std::collections::HashSet;

use egui::{Rect, Sense, Shape, Stroke, TextStyle, pos2, vec2};
use guinea_core::feature::Dispatch;
use guinea_devtools_model::elements::{self, Element};
use guinea_devtools_model::protocol::Snapshot;
use guinea_devtools_model::words;

use super::elements::Pick;
use crate::{style, theme};

/// How far one level sits from the one above it; also the arrow's column.
const STEP: f32 = 14.0;

/// The rows a click closed; every other row is open.
fn closed_id() -> egui::Id {
    egui::Id::new("elements-closed")
}

fn arrow(painter: &egui::Painter, cell: Rect, open: bool) {
    let center = cell.center();
    let size = 3.5;

    let points = if open {
        vec![
            pos2(center.x - size, center.y - size * 0.6),
            pos2(center.x + size, center.y - size * 0.6),
            pos2(center.x, center.y + size * 0.8),
        ]
    } else {
        vec![
            pos2(center.x - size * 0.6, center.y - size),
            pos2(center.x + size * 0.8, center.y),
            pos2(center.x - size * 0.6, center.y + size),
        ]
    };

    painter.add(Shape::convex_polygon(points, theme::MUTED, Stroke::NONE));
}

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &Snapshot,
    picked: Option<&Element>,
    reveal: Option<&Element>,
    dispatch: &Dispatch,
) {
    let mut closed: HashSet<Element> = ui.data(|data| data.get_temp(closed_id())).unwrap_or_default();
    let lines = elements::lines(snapshot, &closed, reveal);

    let height = ui.text_style_height(&TextStyle::Monospace) + 2.0;
    let padding = f32::from(style::PADDING);
    ui.spacing_mut().item_spacing.y = 0.0;

    let mut area = egui::ScrollArea::both().auto_shrink(false);
    if let Some(index) = reveal.and_then(|wanted| lines.iter().position(|line| &line.element == wanted)) {
        area = area.vertical_scroll_offset((index as f32 * height - 6.0 * height).max(0.0));
    }

    let mut toggled = None;
    let mut clicked = None;
    area.show_rows(ui, height, lines.len(), |ui, range| {
        ui.spacing_mut().item_spacing.y = 0.0;

        for line in &lines[range] {
            let galley = ui.painter().layout_job(style::line(ui, &line.words));
            let indent = padding + line.depth as f32 * STEP;
            let width = (indent + STEP + galley.size().x + padding).max(ui.available_width());

            let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
            let on = picked == Some(&line.element);
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    true,
                    on,
                    words::text(&line.words),
                )
            });

            let painter = ui.painter();
            if on {
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
                let id = ui.id().with(("arrow", line.element.to_string()));
                if ui.interact(cell, id, Sense::click()).clicked() {
                    on_arrow = true;
                    toggled = Some(line.element.clone());
                }
            }

            ui.painter().galley(
                pos2(cell.right(), rect.center().y - galley.size().y / 2.0),
                galley,
                theme::TEXT,
            );

            if response.double_clicked() && line.branch {
                toggled = Some(line.element.clone());
            } else if response.clicked() && !on_arrow {
                clicked = Some(line.element.clone());
            }
        }
    });

    if let Some(element) = toggled {
        if !closed.remove(&element) {
            closed.insert(element);
        }
        ui.data_mut(|data| data.insert_temp(closed_id(), closed));
    }

    if let Some(element) = clicked {
        dispatch.emit(Pick(element));
    }
}
