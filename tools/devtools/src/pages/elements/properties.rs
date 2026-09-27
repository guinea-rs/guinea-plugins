//! The sidebar for a native element: its properties, the way the browser's
//! devtools show one node's styles.
//!
//! Tabs across the top - what was set and where from, then one per cluster.
//! Layout draws the box model, Text a sample of the font, Appearance the
//! colours; every value gets the field its type calls for.

use std::cell::RefCell;
use std::collections::HashMap;

use guinea_core::feature::Dispatch;
use guinea_devtools_model::native::NativeTree;
use guinea_devtools_model::properties::{self, Cluster};
use guinea_devtools_protocol::Command;
use guinea_devtools_protocol::native::Property;

use crate::features::native::contracts::Select;
use crate::{components, theme};

#[derive(Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    Set,
    Cluster(Cluster),
}

#[derive(Default)]
struct Sidebar {
    tab: Tab,
    filter: String,
    set_only: bool,
    /// Values being typed, by property index, until they are sent.
    editing: HashMap<u32, String>,
}

thread_local! {
    static SIDEBAR: RefCell<Sidebar> = RefCell::new(Sidebar::default());
}

fn sidebar<R>(job: impl FnOnce(&mut Sidebar) -> R) -> R {
    SIDEBAR.with(|sidebar| job(&mut sidebar.borrow_mut()))
}

/// What the sidebar needs besides the element.
pub struct Context<'a> {
    pub tree: &'a NativeTree,
    pub enums: &'a HashMap<String, Vec<(i32, String)>>,
    pub edits: bool,
    pub send: &'a dyn Fn(Command),
    pub dispatch: &'a Dispatch,
}

pub fn show(ui: &mut egui::Ui, selected: u64, properties: Option<&[Property]>, cx: &Context) {
    let Some(element) = cx.tree.get(selected) else {
        return;
    };

    components::block(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading(short(&element.kind));
            if !element.name.is_empty() {
                ui.label(components::dim(format!("#{}", element.name)));
            }
        });
        ui.label(components::dim(&element.kind));
    });

    let Some(properties) = properties else {
        components::rule(ui);
        components::block(ui, |ui| ui.label(components::dim("asking for its properties…")));
        return;
    };

    tabs(ui);

    let (tab, filter, set_only) = sidebar(|sidebar| {
        components::block(ui, |ui| {
            ui.horizontal(|ui| {
                components::field_look(ui);
                components::search(ui, &mut sidebar.filter, "filter", 200.0);
                if matches!(sidebar.tab, Tab::Cluster(_)) {
                    ui.checkbox(&mut sidebar.set_only, "set only");
                }
            });
        });
        (sidebar.tab, sidebar.filter.to_lowercase(), sidebar.set_only)
    });

    let matches = |property: &&Property| filter.is_empty() || property.name.to_lowercase().contains(&filter);

    egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
        components::block(ui, |ui| match tab {
            Tab::Set => set_here(ui, selected, properties, cx, &matches),
            Tab::Cluster(cluster) => {
                match cluster {
                    Cluster::Layout => box_model(ui, properties),
                    Cluster::Text => text_sample(ui, properties),
                    Cluster::Appearance => swatches(ui, properties),
                    _ => {}
                }

                let listed: Vec<&Property> = properties::winning(properties)
                    .filter(|property| Cluster::of(property) == cluster)
                    .filter(|property| !set_only || property.source != "default")
                    .filter(matches)
                    .collect();

                sections(ui, cluster, &listed, selected, cx, !filter.is_empty());
            }
        });
    });
}

fn tabs(ui: &mut egui::Ui) {
    let titles: Vec<&str> = std::iter::once("Set")
        .chain(Cluster::ALL.iter().map(|cluster| cluster.title()))
        .collect();

    sidebar(|sidebar| {
        let open = match sidebar.tab {
            Tab::Set => 0,
            Tab::Cluster(cluster) => 1 + Cluster::ALL.iter().position(|each| *each == cluster).unwrap_or(0),
        };

        if let Some(index) = components::tabs(ui, "native-property-tabs", &titles, open) {
            sidebar.tab = match index {
                0 => Tab::Set,
                at => Tab::Cluster(Cluster::ALL[at - 1]),
            };
        }
    });
}

/// Everything not left at its default, grouped by where the value came from,
/// with what it overrides struck through beneath it.
fn set_here(ui: &mut egui::Ui, selected: u64, properties: &[Property], cx: &Context, matches: &dyn Fn(&&Property) -> bool) {
    let mut by_source: Vec<(&str, Vec<&Property>)> = Vec::new();
    for property in properties.iter().filter(|property| !property.overridden && property.source != "default") {
        if !matches(&property) {
            continue;
        }
        match by_source.iter_mut().find(|(source, _)| *source == property.source) {
            Some((_, listed)) => listed.push(property),
            None => by_source.push((&property.source, vec![property])),
        }
    }

    if by_source.is_empty() {
        ui.label(components::dim("everything is at its default"));
        return;
    }

    for (source, listed) in by_source {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(source.replace('_', " ")).strong());
        rows(ui, &listed, selected, cx);

        let overridden: Vec<&Property> = properties
            .iter()
            .filter(|property| property.overridden && listed.iter().any(|set| set.index == property.index))
            .collect();
        for property in overridden {
            ui.label(components::dim(format!("    {} = {} ({})", property.name, shown_value(property, cx), property.source)).strikethrough());
        }
    }
}

/// A cluster's properties under its section headings, in the order they
/// matter; whatever no section names folded at the end, open while
/// `searching`.
fn sections(ui: &mut egui::Ui, cluster: Cluster, listed: &[&Property], element: u64, cx: &Context, searching: bool) {
    for (at, section) in cluster.sections().iter().enumerate() {
        let mut shown: Vec<(usize, &Property)> = listed
            .iter()
            .filter_map(|property| match cluster.place(property) {
                Some((this, within)) if this == at => Some((within, *property)),
                _ => None,
            })
            .collect();
        if shown.is_empty() {
            continue;
        }
        shown.sort_by_key(|(within, _)| *within);

        heading(ui, section.title);
        rows(ui, &shown.into_iter().map(|(_, property)| property).collect::<Vec<_>>(), element, cx);
    }

    let mut rest: Vec<&Property> = listed
        .iter()
        .copied()
        .filter(|property| cluster.place(property).is_none())
        .collect();
    if rest.is_empty() {
        return;
    }
    rest.sort_by(|a, b| (a.source == "default").cmp(&(b.source == "default")).then(a.name.cmp(&b.name)));

    if cluster.sections().is_empty() {
        rows(ui, &rest, element, cx);
        return;
    }

    ui.add_space(6.0);
    egui::CollapsingHeader::new(egui::RichText::new(format!("More · {}", rest.len())).color(theme::MUTED))
        .id_salt(("native-more", cluster.title()))
        .default_open(false)
        .open(searching.then_some(true))
        .show_unindented(ui, |ui| rows(ui, &rest, element, cx));
}

fn heading(ui: &mut egui::Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(title).strong().color(theme::MUTED));
    ui.add_space(2.0);
}

/// How wide the column of sources is, on the right of every row.
const SOURCE_WIDTH: f32 = 90.0;

/// One property to a row: its name, its value as wide as the sidebar lets
/// it be, and where the value came from.
fn rows(ui: &mut egui::Ui, shown: &[&Property], element: u64, cx: &Context) {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let names = shown
        .iter()
        .map(|property| {
            ui.painter()
                .layout_no_wrap(property.name.clone(), font.clone(), egui::Color32::PLACEHOLDER)
                .size()
                .x
        })
        .fold(0.0, f32::max)
        .min(ui.available_width() / 3.0);
    let height = components::FIELD_HEIGHT;

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;

        for (index, property) in shown.iter().enumerate() {
            let stripe = ui.painter().add(egui::Shape::Noop);

            let size = egui::vec2(ui.available_width(), height);
            let row = ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let set_here = property.source != "default";
                let name = egui::RichText::new(&property.name)
                    .color(if set_here { theme::TEXT } else { theme::MUTED });
                ui.add_sized([names, height], egui::Label::new(name).truncate())
                    .on_hover_text(&property.declaring_type);

                let spacing = ui.spacing().item_spacing.x;
                let value = (ui.available_width() - SOURCE_WIDTH - spacing).max(80.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(value, height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_width(value);
                        components::field_look(ui);
                        editor(ui, element, property, cx);
                    },
                );

                let source = egui::RichText::new(property.source.replace('_', " "))
                    .color(source_color(&property.source));
                ui.add_sized([SOURCE_WIDTH, height], egui::Label::new(source).truncate())
                    .on_hover_text(&property.value_type);
            });

            if index % 2 == 1 {
                let rect = row.response.rect.expand2(egui::vec2(4.0, 2.0));
                ui.painter().set(stripe, egui::epaint::RectShape::filled(rect, 4, ui.visuals().faint_bg_color));
            }
        }
    });
}

fn short(kind: &str) -> &str {
    kind.rsplit('.').next().unwrap_or(kind)
}

fn color32(rgba: [u8; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
}

/// A value as it reads best: an enumeration's name, a boolean as a word.
fn shown_value(property: &Property, cx: &Context) -> String {
    if property.binding {
        return "{binding}".to_string();
    }
    if let Some(name) = properties::enum_name(cx.enums, property) {
        return name.to_string();
    }
    match (property.value_type.as_str(), property.value.as_str()) {
        ("Windows.Foundation.Boolean", "1") => "true".to_string(),
        ("Windows.Foundation.Boolean", "0") => "false".to_string(),
        (_, "0") if property.object => "null".to_string(),
        _ => property.value.clone(),
    }
}

/// Where a value came from, coloured the way the trace colours what a record
/// is: what the author wrote stands out, what the framework left is quiet.
fn source_color(source: &str) -> egui::Color32 {
    match source {
        "local" => theme::CLAY,
        "animation" | "visual_state" => theme::VIOLET,
        "style" | "built_in_style" | "implicit_style_reference" => theme::SKY,
        "parent_template" | "template_trigger" | "parent_template_trigger" => theme::LILAC,
        "inherited" => theme::PEACH,
        _ => theme::MUTED,
    }
}

/// A value's colour, by what kind of value it is - the way code reads in an
/// editor.
fn value_color(property: &Property, cx: &Context) -> egui::Color32 {
    if property.binding {
        return theme::CLAY;
    }
    if properties::enum_name(cx.enums, property).is_some() {
        return theme::VIOLET;
    }

    match property.value_type.as_str() {
        _ if property.object && property.value == "0" => theme::MUTED,
        "Windows.Foundation.Boolean" => theme::CYAN,
        "Windows.Foundation.Double" | "Windows.Foundation.Single" | "Windows.Foundation.Int32"
        | "Windows.Foundation.UInt32" => theme::PEACH,
        "Microsoft.UI.Xaml.Thickness" | "Microsoft.UI.Xaml.CornerRadius" => theme::PEACH,
        "Windows.Foundation.String" => theme::GREEN,
        _ => theme::TEXT,
    }
}

fn swatch(ui: &mut egui::Ui, rgba: [u8; 4]) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    let painter = ui.painter();
    let checker = ui.visuals().weak_text_color();
    painter.rect_filled(rect, 2.0, checker);
    painter.rect_filled(rect, 2.0, color32(rgba));
    painter.rect_stroke(rect, 2.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Inside);
}

/// The field for one value, by what the value is.
fn editor(ui: &mut egui::Ui, element: u64, property: &Property, cx: &Context) {
    let set = |value: String| {
        (cx.send)(Command::NativeSetProperty {
            element,
            property: property.index,
            type_name: property.value_type.clone(),
            value,
        })
    };
    let editable = cx.edits && !property.read_only;

    if property.binding {
        ui.label(components::mono("{binding}").italics().color(theme::CLAY))
            .on_hover_text("bound; the value it gives is the next one in the chain");
        return;
    }

    if property.object {
        ui.horizontal(|ui| {
            if let Some(rgba) = property.color.as_deref().and_then(properties::color) {
                swatch(ui, rgba);
                ui.label(components::mono(property.color.clone().unwrap_or_default()));
            }

            let handle: u64 = property.value.parse().unwrap_or(0);
            if handle == 0 {
                ui.label(components::dim("null"));
            } else if cx.tree.get(handle).is_some() {
                if ui.link(short(&property.value_type)).clicked() {
                    cx.dispatch.emit(Select(handle));
                }
            } else {
                ui.label(components::dim(short(&property.value_type)));
            }
        });
        return;
    }

    if !editable {
        ui.label(components::mono(shown_value(property, cx)).color(value_color(property, cx)));
        return;
    }

    if let Some(values) = cx.enums.get(&property.value_type) {
        let current = properties::enum_name(cx.enums, property).unwrap_or(&property.value).to_string();
        ui.spacing_mut().combo_width = ui.available_width();
        let shown = egui::RichText::new(&current).color(theme::VIOLET);
        components::select(ui, ("native-enum", element, property.index), shown, |ui| {
            for (_, name) in values {
                if ui.selectable_label(*name == current, name).clicked() && *name != current {
                    set(name.clone());
                }
            }
        });
        return;
    }

    match property.value_type.as_str() {
        "Windows.Foundation.Boolean" => {
            let mut on = property.value == "1";
            if ui.checkbox(&mut on, "").changed() {
                set(if on { "True" } else { "False" }.to_string());
            }
        }
        "Windows.Foundation.Double" | "Windows.Foundation.Single" | "Windows.Foundation.Int32" => {
            number(ui, property, set);
        }
        "Microsoft.UI.Xaml.Thickness" | "Microsoft.UI.Xaml.CornerRadius" => {
            sides(ui, property, set);
        }
        "Windows.UI.Color" => {
            ui.horizontal(|ui| {
                if let Some(rgba) = properties::color(&property.value) {
                    swatch(ui, rgba);
                }
                text(ui, property, set);
            });
        }
        _ => text(ui, property, set),
    }
}

fn number(ui: &mut egui::Ui, property: &Property, set: impl Fn(String)) {
    let Ok(mut value) = property.value.parse::<f64>() else {
        text(ui, property, set);
        return;
    };

    let integer = property.value_type == "Windows.Foundation.Int32";
    let mut drag = egui::DragValue::new(&mut value).speed(if integer { 1.0 } else { 0.5 });
    if integer {
        drag = drag.fixed_decimals(0);
    }

    let response = ui.add_sized([ui.available_width(), components::FIELD_HEIGHT], drag);
    if response.drag_stopped() || (response.changed() && !response.dragged()) {
        set(if integer { format!("{}", value as i64) } else { format!("{value}") });
    }
}

fn sides(ui: &mut egui::Ui, property: &Property, set: impl Fn(String)) {
    let Some(mut four) = properties::thickness(&property.value) else {
        text(ui, property, set);
        return;
    };

    let mut done = false;
    let row = egui::vec2(ui.available_width(), components::FIELD_HEIGHT);
    ui.allocate_ui_with_layout(row, egui::Layout::left_to_right(egui::Align::Center), |ui| {
        let spacing = ui.spacing().item_spacing.x;
        let each = ((ui.available_width() - spacing * 3.0) / 4.0).max(28.0);

        for side in &mut four {
            let drag = egui::DragValue::new(side).speed(0.5).max_decimals(1);
            let response = ui.add_sized([each, components::FIELD_HEIGHT], drag);
            done |= response.drag_stopped() || (response.changed() && !response.dragged());
        }
    });
    if done {
        set(four.map(|side| side.to_string()).join(","));
    }
}

fn text(ui: &mut egui::Ui, property: &Property, set: impl Fn(String)) {
    let submitted = sidebar(|sidebar| {
        let typed = sidebar.editing.entry(property.index).or_insert_with(|| property.value.clone());
        let width = ui.available_width();
        let edit = egui::TextEdit::singleline(typed)
            .font(egui::TextStyle::Monospace)
            .text_color(theme::GREEN);
        let response = ui.add_sized([width, components::FIELD_HEIGHT], edit);

        let done = response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
        if response.lost_focus() || !response.has_focus() && !done {
            let typed = sidebar.editing.remove(&property.index);
            return done.then_some(typed).flatten();
        }
        None
    });

    if let Some(value) = submitted.filter(|value| *value != property.value) {
        set(value);
    }
}

/// Margin, border and padding around the content, the way Chrome draws the
/// box model.
fn box_model(ui: &mut egui::Ui, properties: &[Property]) {
    let four = |name: &str| {
        properties::find(properties, name)
            .and_then(|property| properties::thickness(&property.value))
            .unwrap_or([0.0; 4])
    };
    let size = |name: &str| {
        properties::find(properties, name)
            .and_then(|property| property.value.parse::<f64>().ok())
            .unwrap_or(0.0)
    };

    let layers = [
        ("margin", four("Margin"), egui::Color32::from_rgba_unmultiplied(246, 178, 107, 90)),
        ("border", four("BorderThickness"), egui::Color32::from_rgba_unmultiplied(255, 229, 153, 90)),
        ("padding", four("Padding"), egui::Color32::from_rgba_unmultiplied(147, 196, 125, 90)),
    ];

    let width = ui.available_width().min(320.0);
    let (row, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 150.0), egui::Sense::hover());
    let rect = egui::Rect::from_center_size(row.center(), egui::vec2(width, 150.0));
    let painter = ui.painter_at(rect);
    let font = egui::FontId::monospace(10.0);
    let ink = ui.visuals().text_color();
    let step = 22.0;

    let mut area = rect;
    for (label, [left, top, right, bottom], fill) in layers {
        painter.rect_filled(area, 2.0, fill);
        painter.rect_stroke(area, 2.0, egui::Stroke::new(1.0, ink.gamma_multiply(0.4)), egui::StrokeKind::Inside);
        painter.text(area.left_top() + egui::vec2(4.0, 2.0), egui::Align2::LEFT_TOP, label, font.clone(), ink.gamma_multiply(0.7));

        let side = |value: f64| if value == 0.0 { "-".to_string() } else { format!("{value}") };
        painter.text(egui::pos2(area.center().x, area.top() + step / 2.0), egui::Align2::CENTER_CENTER, side(top), font.clone(), ink);
        painter.text(egui::pos2(area.center().x, area.bottom() - step / 2.0), egui::Align2::CENTER_CENTER, side(bottom), font.clone(), ink);
        painter.text(egui::pos2(area.left() + step / 2.0, area.center().y), egui::Align2::CENTER_CENTER, side(left), font.clone(), ink);
        painter.text(egui::pos2(area.right() - step / 2.0, area.center().y), egui::Align2::CENTER_CENTER, side(right), font.clone(), ink);

        area = area.shrink2(egui::vec2(step * 1.4, step));
    }

    painter.rect_filled(area, 2.0, egui::Color32::from_rgba_unmultiplied(111, 168, 220, 110));
    painter.text(
        area.center(),
        egui::Align2::CENTER_CENTER,
        format!("{:.1} × {:.1}", size("ActualWidth"), size("ActualHeight")),
        font,
        ink,
    );
    ui.add_space(6.0);
}

/// The element's text in its own font, size, weight, style and colour.
fn text_sample(ui: &mut egui::Ui, properties: &[Property]) {
    let value = |name: &str| properties::find(properties, name).map(|property| property.value.as_str());

    let Some(family) = value("FontFamily") else {
        return;
    };
    let size = value("FontSize").and_then(|size| size.parse::<f32>().ok()).unwrap_or(14.0);
    let sample = value("Text").filter(|text| !text.is_empty() && *text != "0").unwrap_or("The quick brown fox");
    let weight = value("FontWeight").unwrap_or("Normal");

    let mut text = egui::RichText::new(sample).size(size.clamp(6.0, 64.0));
    if !matches!(weight, "Normal" | "Light" | "SemiLight" | "ExtraLight" | "Thin") {
        text = text.strong();
    }
    if value("FontStyle").is_some_and(|style| style != "0") {
        text = text.italics();
    }
    if let Some(rgba) = properties::find(properties, "Foreground").and_then(|brush| brush.color.as_deref()).and_then(properties::color) {
        text = text.color(color32(rgba));
    }

    egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.label(text);
            ui.label(components::dim(format!("{family} · {size} · {weight}")));
        });
    ui.add_space(6.0);
}

/// Every brush the element paints with, as a colour.
fn swatches(ui: &mut egui::Ui, properties: &[Property]) {
    let brushes: Vec<&Property> = properties::winning(properties)
        .filter(|property| property.color.is_some())
        .collect();
    if brushes.is_empty() {
        return;
    }

    ui.horizontal_wrapped(|ui| {
        for brush in brushes {
            let Some(rgba) = brush.color.as_deref().and_then(properties::color) else {
                continue;
            };
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(56.0, 28.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, color32(rgba));
                ui.painter().rect_stroke(rect, 4.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Inside);
                ui.label(components::dim(&brush.name));
            });
        }
    });
    ui.add_space(6.0);
}
