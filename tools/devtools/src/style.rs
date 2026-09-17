use egui::{Color32, RichText};
use guinea_devtools_model::words::{Tone, Word};

use crate::theme;

pub const LIVE: Color32 = theme::GREEN;
pub const GONE: Color32 = theme::MUTED;
pub const ACCENT: Color32 = theme::CLAY;

pub fn dim(text: impl Into<String>) -> RichText {
    RichText::new(text).weak()
}

pub fn mono(text: impl Into<String>) -> RichText {
    RichText::new(text).monospace()
}

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

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 22.0),
            egui::Sense::hover(),
        );

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
                let cross = crate::icons::image(crate::icons::close(), size, ui.visuals().weak_text_color());
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

/// A panel split by lines into blocks, which bring their own padding.
pub fn bare(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::NONE.fill(ui.visuals().panel_fill)
}

/// A side panel: [`bare`], on the sidebar's colour.
pub fn side() -> egui::Frame {
    egui::Frame::NONE.fill(theme::SIDEBAR)
}

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

/// The colour of a word, by what the model says it is.
pub fn tone_color(tone: &Tone) -> Color32 {
    match tone {
        Tone::Plain => theme::TEXT,
        Tone::Muted => theme::MUTED,
        Tone::Accent => ACCENT,
        Tone::Kind(kind) => kind_color(kind),
        Tone::Level(level) => level_color(level),
        Tone::Quote => theme::PINK,
    }
}

/// Words laid out on one line in the monospace font.
pub fn line(ui: &egui::Ui, words: &[Word]) -> egui::text::LayoutJob {
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let mut job = egui::text::LayoutJob::default();

    for word in words {
        job.append(
            &word.text,
            0.0,
            egui::text::TextFormat::simple(font.clone(), tone_color(&word.tone)),
        );
    }

    job
}

/// The colour of a log line, by its `tracing` level.
pub fn level_color(level: &str) -> Color32 {
    match level {
        "ERROR" => theme::RED,
        "WARN" => theme::CLAY,
        "INFO" => theme::TEXT,
        _ => theme::MUTED,
    }
}

/// The colour of a trace record's kind, as `TracePoint::kind` names it.
pub fn kind_color(kind: &str) -> Color32 {
    match kind {
        "action" => theme::CLAY,
        "send" => theme::LINK,
        "handle" => theme::SKY,
        "spawn" => theme::VIOLET,
        "publish" => theme::PINK,
        "deliver" => theme::PEACH,
        "push" => theme::GREEN,
        "navigate" => theme::CYAN,
        "store" => theme::LILAC,
        "log" => theme::TEXT,
        _ => theme::MUTED,
    }
}
