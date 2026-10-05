//! A row of tabs, like a browser's devtools.

use egui::Color32;

use crate::theme;

/// How tall a row of [`tabs`] is.
pub const TABS_HEIGHT: f32 = 30.0;

/// A row of tabs; the one clicked, if another.
///
/// When they do not fit, the wheel scrolls them sideways, with no bar. Left
/// and right, with the pointer over them, open the neighbouring tab.
pub fn tabs(ui: &mut egui::Ui, id: &str, titles: &[&str], open: usize) -> Option<usize> {
    let mut clicked = None;

    let (bar, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), TABS_HEIGHT),
        egui::Sense::hover(),
    );
    let line = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter()
        .hline(bar.x_range(), bar.bottom() - line.width / 2.0, line);

    let typing = ui.memory(|memory| memory.focused().is_some());
    if ui.rect_contains_pointer(bar) && !typing {
        let (left, right) = ui.input_mut(|input| {
            (
                input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft),
                input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight),
            )
        });
        if left && open > 0 {
            clicked = Some(open - 1);
        }
        if right && open + 1 < titles.len() {
            clicked = Some(open + 1);
        }
    }

    let shown = egui::Id::new((id, "shown"));
    let moved = ui.data(|data| data.get_temp::<usize>(shown)) != Some(open);
    ui.data_mut(|data| data.insert_temp(shown, open));

    let mut strip = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(bar)
            .layout(egui::Layout::left_to_right(egui::Align::Min)),
    );
    strip.style_mut().always_scroll_the_only_direction = true;

    let area = egui::ScrollArea::horizontal()
        .id_salt(id)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden);

    area.show(&mut strip, |ui| {
        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

        for (index, title) in titles.iter().enumerate() {
            let on = index == open;
            let response = tab(ui, title, on);

            if on && moved {
                response.scroll_to_me(None);
            }
            if response.clicked() && !on {
                clicked = Some(index);
            }
        }
    });

    clicked
}

/// One of [`tabs`], underlined when it is the open one.
fn tab(ui: &mut egui::Ui, title: &str, on: bool) -> egui::Response {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(title.to_string(), font, Color32::PLACEHOLDER);

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(galley.size().x + 24.0, TABS_HEIGHT),
        egui::Sense::click(),
    );
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, on, title));

    let visuals = ui.visuals();
    if response.hovered() && !on {
        ui.painter()
            .rect_filled(rect, 0, visuals.widgets.hovered.weak_bg_fill);
    }
    let color = if on {
        visuals.strong_text_color()
    } else if response.hovered() {
        visuals.text_color()
    } else {
        visuals.weak_text_color()
    };

    if on {
        let underline = egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - 2.0),
            rect.right_bottom(),
        );
        ui.painter().rect_filled(underline, 0, theme::ACCENT);
    }
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, color);

    response
}
