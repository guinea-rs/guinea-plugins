//! Claude Desktop's dark palette, applied to egui.

use egui::{Color32, CornerRadius, Stroke, Visuals};
use guinea_devtools_model::words::{Kind, Level, Tone};

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// Where content sits.
pub const BACKGROUND: Color32 = rgb(0x151515);
/// Side panels and the status bar.
pub const SIDEBAR: Color32 = rgb(0x111111);
/// Lines between blocks.
pub const DIVIDER: Color32 = rgb(0x292929);
/// A picked row.
pub const SELECTED: Color32 = rgb(0x343434);
/// Text fields.
pub const FIELD: Color32 = rgb(0x20201f);
pub const FIELD_BORDER: Color32 = rgb(0x373736);
/// Buttons and chips.
pub const CHIP: Color32 = rgb(0x383838);
pub const CHIP_HOVERED: Color32 = rgb(0x424241);
pub const CODE: Color32 = rgb(0x1a1a19);
pub const TEXT: Color32 = rgb(0xf0efec);
pub const MUTED: Color32 = rgb(0x898781);
/// Claude's own orange.
pub const CLAY: Color32 = rgb(0xd97757);
pub const LINK: Color32 = rgb(0x6da7ec);
pub const GREEN: Color32 = rgb(0x32d74b);
pub const RED: Color32 = rgb(0xff2c56);
pub const CYAN: Color32 = rgb(0x5ce6e6);
pub const PINK: Color32 = rgb(0xec7e7e);
/// A scroll bar's thumb; also what checkboxes and sliders fill with.
pub const SCROLL: Color32 = rgb(0x5d5c5a);

/// Kinds of trace records the palette has no colour for, kept as soft as it.
pub const SKY: Color32 = rgb(0x9cc3f2);
pub const VIOLET: Color32 = rgb(0xb59cf0);
pub const PEACH: Color32 = rgb(0xf0a58a);
pub const LILAC: Color32 = rgb(0xcdb8f2);

/// A session that is still connected.
pub const LIVE: Color32 = GREEN;
/// A session that is not.
pub const GONE: Color32 = MUTED;
/// What is picked out: the open tab, a link, a chosen row.
pub const ACCENT: Color32 = CLAY;

/// The colour of a word, by what the model says it is.
pub fn tone_color(tone: &Tone) -> Color32 {
    match tone {
        Tone::Plain => TEXT,
        Tone::Muted => MUTED,
        Tone::Accent => ACCENT,
        Tone::Kind(kind) => kind_color(*kind),
        Tone::Level(level) => level_color(*level),
        Tone::Quote => PINK,
    }
}

/// The colour of a log line, by its `tracing` level.
pub fn level_color(level: Level) -> Color32 {
    match level {
        Level::Error => RED,
        Level::Warn => CLAY,
        Level::Info => TEXT,
        Level::Debug | Level::Trace => MUTED,
    }
}

/// The colour of a trace record's kind.
pub fn kind_color(kind: Kind) -> Color32 {
    match kind {
        Kind::Action => CLAY,
        Kind::Send => LINK,
        Kind::Handle => SKY,
        Kind::Spawn => VIOLET,
        Kind::Settled => LILAC,
        Kind::Cancelled => MUTED,
        Kind::Source | Kind::Pull => VIOLET,
        Kind::Arrived => PEACH,
        Kind::Closed => MUTED,
        Kind::Publish => PINK,
        Kind::Deliver => PEACH,
        Kind::Push => GREEN,
        Kind::Navigate => CYAN,
        Kind::Store => LILAC,
        Kind::Render => SKY,
        Kind::Span => CYAN,
        Kind::Log => TEXT,
        Kind::Tick | Kind::Note => MUTED,
    }
}

fn stroke(color: Color32) -> Stroke {
    Stroke::new(1.0, color)
}

pub fn visuals() -> Visuals {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = FIELD;
    visuals.window_stroke = stroke(FIELD_BORDER);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.extreme_bg_color = FIELD;
    visuals.text_edit_bg_color = Some(FIELD);
    visuals.faint_bg_color = CODE;
    visuals.code_bg_color = CODE;

    visuals.hyperlink_color = LINK;
    visuals.warn_fg_color = CLAY;
    visuals.error_fg_color = RED;
    visuals.weak_text_color = Some(MUTED);

    visuals.selection.bg_fill = SELECTED;
    visuals.selection.stroke = stroke(TEXT);
    visuals.text_cursor.stroke = stroke(TEXT);

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = BACKGROUND;
    widgets.noninteractive.weak_bg_fill = BACKGROUND;
    widgets.noninteractive.bg_stroke = stroke(DIVIDER);
    widgets.noninteractive.fg_stroke = stroke(TEXT);

    widgets.inactive.bg_fill = SCROLL;
    widgets.inactive.weak_bg_fill = CHIP;
    widgets.inactive.bg_stroke = Stroke::NONE;
    widgets.inactive.fg_stroke = stroke(TEXT);

    widgets.hovered.bg_fill = CHIP_HOVERED;
    widgets.hovered.weak_bg_fill = SELECTED;
    widgets.hovered.bg_stroke = stroke(FIELD_BORDER);
    widgets.hovered.fg_stroke = stroke(TEXT);

    widgets.active.bg_fill = CHIP_HOVERED;
    widgets.active.weak_bg_fill = CHIP_HOVERED;
    widgets.active.bg_stroke = stroke(FIELD_BORDER);
    widgets.active.fg_stroke = stroke(TEXT);

    widgets.open.bg_fill = FIELD;
    widgets.open.weak_bg_fill = FIELD;
    widgets.open.bg_stroke = stroke(FIELD_BORDER);
    widgets.open.fg_stroke = stroke(TEXT);

    for widget in [
        &mut widgets.noninteractive,
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.open,
    ] {
        widget.corner_radius = CornerRadius::same(6);
    }

    visuals
}

/// [`visuals`], the first time a frame asks.
pub fn once(ctx: &egui::Context) {
    let done = egui::Id::new("claude-theme");
    if ctx.data(|data| data.get_temp::<bool>(done)).is_none() {
        ctx.data_mut(|data| data.insert_temp(done, true));

        ctx.set_visuals_of(egui::Theme::Dark, visuals());
        ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.spacing.scroll.foreground_color = false;
        });
        ctx.set_theme(egui::Theme::Dark);
    }
}
