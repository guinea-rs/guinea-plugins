//! What is typed into and picked from: one box, whatever sits in it.

use crate::theme;

/// How tall every field is, and so a row of them.
pub const FIELD_HEIGHT: f32 = 24.0;

/// A search field the way Claude draws one: a soft rounded box with a
/// magnifier, a border that is always there and brightens on hover and
/// focus. `width` is the whole box's.
pub fn search(ui: &mut egui::Ui, text: &mut String, hint: &str, width: f32) -> egui::Response {
    let background = ui.painter().add(egui::Shape::Noop);

    let framed = egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.set_width(width - 16.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                let size = ui.text_style_height(&egui::TextStyle::Body);
                ui.add(crate::icons::image(
                    crate::icons::search(),
                    size,
                    theme::MUTED,
                ));

                let edit = egui::TextEdit::singleline(text)
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::ZERO)
                    .hint_text(egui::RichText::new(hint).color(theme::MUTED))
                    .desired_width(ui.available_width());
                ui.add(edit)
            })
            .inner
        });

    let field = framed.inner;
    let rect = framed.response.rect;

    let border = if field.has_focus() {
        theme::SCROLL
    } else if ui.rect_contains_pointer(rect) {
        theme::CHIP_HOVERED
    } else {
        theme::FIELD_BORDER
    };
    ui.painter().set(
        background,
        egui::epaint::RectShape::new(
            rect,
            8,
            theme::FIELD,
            egui::Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        ),
    );

    field
}

/// Makes every field in `ui` - text, numbers, drop-downs, checkboxes - the
/// box [`search`] is: the field colour, rounded, a border that is always
/// there and brightens on hover and focus.
pub fn field_look(ui: &mut egui::Ui) {
    let visuals = ui.visuals_mut();
    visuals.text_edit_bg_color = Some(theme::FIELD);
    visuals.selection.stroke = egui::Stroke::new(1.0, theme::SCROLL);

    let widgets = &mut visuals.widgets;
    for (widget, border) in [
        (&mut widgets.inactive, theme::FIELD_BORDER),
        (&mut widgets.hovered, theme::CHIP_HOVERED),
        (&mut widgets.active, theme::SCROLL),
        (&mut widgets.open, theme::SCROLL),
    ] {
        widget.bg_fill = theme::FIELD;
        widget.weak_bg_fill = theme::FIELD;
        widget.bg_stroke = egui::Stroke::new(1.0, border);
        widget.corner_radius = egui::CornerRadius::same(8);
        widget.expansion = 0.0;
    }

    ui.spacing_mut().button_padding = egui::vec2(8.0, 3.0);
    ui.spacing_mut().interact_size.y = FIELD_HEIGHT;
}

/// A drop-down in the same box as [`search`]: always bordered, brighter on
/// hover and while open.
pub fn select(
    ui: &mut egui::Ui,
    id: impl egui::AsIdSalt,
    selected: impl Into<egui::WidgetText>,
    add: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    ui.scope(|ui| {
        field_look(ui);

        egui::ComboBox::from_id_salt(id)
            .selected_text(selected)
            .show_ui(ui, add)
            .response
    })
    .inner
}
