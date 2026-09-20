//! Names and what they hold, read only.

use egui::Color32;

use super::text::{dim, mono};

/// Names and values in two columns; a long value wraps.
pub fn fields(ui: &mut egui::Ui, rows: &[(String, String)]) {
    let names = rows
        .iter()
        .map(|(name, _)| {
            let font = egui::TextStyle::Body.resolve(ui.style());
            ui.painter()
                .layout_no_wrap(name.clone(), font, Color32::PLACEHOLDER)
                .size()
                .x
        })
        .fold(0.0, f32::max)
        .min(ui.available_width() / 3.0);

    for (name, value) in rows {
        ui.horizontal_top(|ui| {
            ui.add_sized([names, 0.0], egui::Label::new(dim(name)).truncate());
            ui.add(egui::Label::new(mono(value)).wrap());
        });
    }
}
