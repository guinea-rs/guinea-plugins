//! Text the way devtools write it: quiet, monospaced, or a line of words the
//! model has already coloured.

use egui::RichText;
use guinea_devtools_model::words::Word;

use crate::theme;

pub fn dim(text: impl Into<String>) -> RichText {
    RichText::new(text).weak()
}

pub fn mono(text: impl Into<String>) -> RichText {
    RichText::new(text).monospace()
}

/// Words laid out on one line in the monospace font.
pub fn line(ui: &egui::Ui, words: &[Word]) -> egui::text::LayoutJob {
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let mut job = egui::text::LayoutJob::default();

    for word in words {
        job.append(
            &word.text,
            0.0,
            egui::text::TextFormat::simple(font.clone(), theme::tone_color(&word.tone)),
        );
    }

    job
}
