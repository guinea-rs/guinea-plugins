//! The frames panels are drawn in.

use crate::theme;

/// A panel split by lines into blocks, which bring their own padding.
pub fn bare(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::NONE.fill(ui.visuals().panel_fill)
}

/// A side panel: [`bare`], on the sidebar's colour.
pub fn side() -> egui::Frame {
    egui::Frame::NONE.fill(theme::SIDEBAR)
}
