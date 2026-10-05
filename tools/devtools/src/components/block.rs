//! The blocks a panel is stacked from: a rule between them, a heading bar,
//! the room inside one, and the head of a side panel.

use crate::theme;

/// The room inside every block, the same on every side.
pub const PADDING: i8 = 8;

/// A line between two blocks, with no room of its own: it runs edge to edge.
pub fn rule(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().spacing(1.0));
}

/// A section's title as a bar of its own, lined above and below.
pub fn heading(ui: &mut egui::Ui, title: &str) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        rule(ui);

        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::hover());

        ui.painter().rect_filled(rect, 0, theme::FIELD);
        ui.painter().text(
            rect.left_center() + egui::vec2(f32::from(PADDING), 0.0),
            egui::Align2::LEFT_CENTER,
            title,
            egui::TextStyle::Body.resolve(ui.style()),
            theme::TEXT,
        );

        rule(ui);
    });
}

/// Content as a block, with [`PADDING`] around it.
pub fn block<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(PADDING))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// The top of a side panel: what it shows, and a button that closes it;
/// whether it was closed, by the button or by Escape.
pub fn head(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) -> bool {
    let clicked = block(ui, |ui| {
        ui.horizontal_top(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let size = ui.text_style_height(&egui::TextStyle::Body);
                let cross = crate::icons::image(
                    crate::icons::close(),
                    size,
                    ui.visuals().weak_text_color(),
                );
                let close = ui
                    .add(egui::Button::image(cross).frame(false))
                    .on_hover_text("close (Esc)")
                    .clicked();

                ui.with_layout(egui::Layout::top_down(egui::Align::Min), add);

                close
            })
            .inner
        })
        .inner
    });

    let escaped = ui.memory(|memory| memory.focused().is_none())
        && ui.input(|input| input.key_pressed(egui::Key::Escape));

    clicked || escaped
}
