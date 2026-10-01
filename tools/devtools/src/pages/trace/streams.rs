//! The trace as chains: what started them, down the side, and one stream's
//! chains in the middle, those of one shape as one line.

use std::collections::BTreeSet;

use guinea_devtools_model::chains::{self, GroupLine, Section, Stream, StreamLine, TimerView};
use guinea_devtools_model::sessions::Session;
use guinea_devtools_model::words::Level;

use super::lines::{Go, Line, row, row_height};
use crate::components;
use crate::components::source;
use crate::features::editor::contracts::Editor;
use crate::icons;
use crate::theme;

/// Which groups are open, by their keys.
pub type Open = BTreeSet<String>;

pub fn open_id() -> egui::Id {
    egui::Id::new("trace-open-groups")
}

fn section_title(section: Section) -> &'static str {
    match section {
        Section::Actions => "Actions",
        Section::Timers => "Timers",
        Section::Sources => "Sources",
        Section::Loops => "Loops",
        Section::Other => "Other",
    }
}

/// The streams, by section.
pub fn rail(ui: &mut egui::Ui, session: &Session, current: &Stream, go: &mut Option<Go>) {
    let lines = chains::streams(&session.chains, session.reading());
    let chains: usize = lines.iter().map(|line| line.chains).sum();
    let height = row_height(ui);

    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        let reading = session.reading();
        let counted = |stream: &Stream| stream.class().map_or(0, |class| session.classes.count(class));

        components::block(ui, |ui| {
            entry(ui, height, current, &Stream::All, "Everything", chains, go);
            entry(ui, height, current, &Stream::Records, "Records", counted(&Stream::Records), go);
        });

        components::heading(ui, "Levels");
        components::block(ui, |ui| {
            for level in Level::ALL {
                let stream = Stream::Level(level);
                let title = chains::title(&session.chains, reading, &stream);
                entry(ui, height, current, &stream, &title, counted(&stream), go);
            }
        });

        components::heading(ui, "Performance");
        components::block(ui, |ui| {
            let title = chains::title(&session.chains, reading, &Stream::Slow);
            entry(ui, height, current, &Stream::Slow, &title, counted(&Stream::Slow), go);
        });

        let mut sections: Vec<(Section, Vec<&StreamLine>)> = Vec::new();
        for line in &lines {
            match sections.last_mut() {
                Some((section, members)) if *section == line.section => members.push(line),
                _ => sections.push((line.section, vec![line])),
            }
        }

        for (section, members) in sections {
            components::heading(ui, section_title(section));
            components::block(ui, |ui| {
                for line in members {
                    entry(ui, height, current, &line.stream, &line.title, line.chains, go);
                }
            });
        }
    });
}

/// One stream in the list: its title, and how many it holds on the right.
fn entry(
    ui: &mut egui::Ui,
    height: f32,
    current: &Stream,
    stream: &Stream,
    title: &str,
    count: usize,
    go: &mut Option<Go>,
) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height + 2.0),
        egui::Sense::click(),
    );
    let on = current == stream;
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, title)
    });

    let visuals = ui.visuals();
    let fill = if on {
        Some(visuals.selection.bg_fill)
    } else if response.hovered() {
        Some(visuals.widgets.hovered.weak_bg_fill)
    } else {
        None
    };
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, egui::CornerRadius::same(3), fill);
    }

    let font = egui::TextStyle::Body.resolve(ui.style());
    let inner = rect.shrink2(egui::vec2(6.0, 0.0));
    let glyph = font.size;
    let tint = if on { visuals.strong_text_color() } else { visuals.weak_text_color() };

    let icon = egui::Rect::from_min_size(
        egui::pos2(inner.left(), inner.center().y - glyph / 2.0),
        egui::vec2(glyph, glyph),
    );
    icons::image(icons::stream(stream), glyph, tint).paint_at(ui, icon);

    let text_left = icon.right() + 6.0;
    let number = ui.painter().layout_no_wrap(count.to_string(), font.clone(), visuals.weak_text_color());
    let room = (inner.right() - text_left - number.size().x - 8.0).max(0.0);

    let mut job = egui::text::LayoutJob::simple_singleline(title.to_string(), font, visuals.text_color());
    job.wrap = egui::text::TextWrapping::truncate_at_width(room);
    let label = ui.painter().layout_job(job);

    ui.painter()
        .galley(egui::pos2(text_left, inner.center().y - label.size().y / 2.0), label, visuals.text_color());
    ui.painter().galley(
        egui::pos2(inner.right() - number.size().x, inner.center().y - number.size().y / 2.0),
        number,
        visuals.weak_text_color(),
    );

    if response.clicked() && !on {
        *go = Some(Go::Open(stream.clone()));
    }
}

/// One stream's chains; a timer's own numbers above them.
pub fn stream(
    ui: &mut egui::Ui,
    session: &Session,
    stream: &Stream,
    text: &str,
    selected: Option<u64>,
    editor: Editor,
    go: &mut Option<Go>,
) {
    let view = chains::view(&session.chains, session.reading(), stream, text);
    let open: Open = ui.data(|data| data.get_temp(open_id())).unwrap_or_default();

    match &view.timer {
        Some(timer) => timer_head(ui, timer, editor),
        None => {
            components::block(ui, |ui| {
                ui.heading(&view.title);
            });
        }
    }

    components::heading(ui, "Chains");

    let height = row_height(ui);
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        components::block(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            if view.groups.is_empty() {
                ui.label(components::dim(if text.is_empty() {
                    "nothing started here yet"
                } else {
                    "no chain says that"
                }));
            }

            for group in &view.groups {
                chain(ui, height, group, open.contains(&group.key), selected, go);
            }
        });
    });
}

fn chain(
    ui: &mut egui::Ui,
    height: f32,
    group: &GroupLine,
    open: bool,
    selected: Option<u64>,
    go: &mut Option<Go>,
) {
    let mut line = Line::new(ui);
    line.weak(&format!("{}  ", group.time));
    line.weak(match (&group.opens, open) {
        (Some(_), _) => "→ ",
        (None, true) => "▾ ",
        (None, false) => "▸ ",
    });
    line.count(group.count);
    line.words(&group.words);
    if let Some(longest) = &group.longest {
        line.weak(&format!("  up to {longest}"));
    }

    let click = match &group.opens {
        Some(stream) => Go::Open(stream.clone()),
        None => Go::Toggle(group.key.clone()),
    };
    row(ui, height, false, go, click, line);

    if !open || group.opens.is_some() {
        return;
    }

    for run in &group.runs {
        let mut line = Line::new(ui);
        line.indent(36.0);
        line.weak(&format!("{}  ", run.time));
        line.text(run.took.as_deref().unwrap_or("–"), ui.visuals().text_color());

        row(ui, height, selected == Some(run.root), go, Go::Select(run.root), line);
    }

    let older = group.count.saturating_sub(group.runs.len());
    if older > 0 {
        ui.horizontal(|ui| {
            ui.add_space(40.0);
            ui.label(components::dim(format!("… and {older} older")));
        });
    }
}

fn timer_head(ui: &mut egui::Ui, timer: &TimerView, editor: Editor) {
    components::block(ui, |ui| {
        ui.horizontal(|ui| {
            let size = ui.text_style_height(&egui::TextStyle::Heading);
            ui.add(icons::image(guicons::icon!(timer), size, ui.visuals().weak_text_color()));
            ui.heading(&timer.title);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(declared) = &timer.declared {
                    source::link(ui, declared, editor);
                }
            });
        });

        let mut said: Vec<String> = Vec::new();
        if let Some(period) = &timer.period {
            said.push(format!("every {period}"));
        }
        said.push(format!("{} ticks", timer.fires));
        if let Some(average) = &timer.average {
            said.push(format!("average {average}"));
        }
        if let Some(longest) = &timer.longest {
            said.push(format!("longest {longest}"));
        }
        if let Some(feature) = &timer.feature {
            said.push(format!("set up by {feature}"));
        }
        said.push(match timer.running {
            0 => "stopped".to_string(),
            1 => "running".to_string(),
            many => format!("{many} running"),
        });
        ui.label(components::dim(said.join(" · ")));

        chart(ui, &timer.recent);
    });
}

/// The newest ticks' durations as bars, the longest in the accent colour.
fn chart(ui: &mut egui::Ui, recent: &[u64]) {
    if recent.is_empty() {
        return;
    }

    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::hover());
    let most = recent.iter().copied().max().unwrap_or(1).max(1);
    let step = (rect.width() / chains::TICKS_DRAWN as f32).min(8.0);

    for (at, took) in recent.iter().enumerate() {
        let height = (rect.height() * *took as f32 / most as f32).max(1.0);
        let left = rect.left() + at as f32 * step;
        let bar = egui::Rect::from_min_max(
            egui::pos2(left, rect.bottom() - height),
            egui::pos2(left + step - 2.0, rect.bottom()),
        );
        let color = if *took == most { theme::ACCENT } else { theme::SCROLL };

        ui.painter().rect_filled(bar, 0, color);
    }
}
