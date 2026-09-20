//! Rows of words that answer the pointer: the trace's records and chains.

use std::ops::Range;

use egui::text::{CCursor, LayoutJob, TextFormat, TextWrapping};
use guinea_devtools_model::chains::Stream;
use guinea_devtools_model::trace::{Cause, Row};
use guinea_devtools_model::words::{Target, Word};

use crate::components;
use crate::theme;

/// Where a click on the trace page leads.
#[derive(Clone)]
pub enum Go {
    Close,
    Select(u64),
    /// Selects a record and brings it into view in the list of records.
    Record(u64),
    Link(Target),
    Open(Stream),
    /// Opens or closes a group of chains, by its key.
    Toggle(String),
}

/// One row: the font's height and a hair.
pub fn row_height(ui: &egui::Ui) -> f32 {
    ui.text_style_height(&egui::TextStyle::Monospace) + 2.0
}

/// Where a linked word leads, said on hover.
pub fn hint(target: &Target) -> String {
    match target {
        Target::Actor(name) => format!("{name}\nshow this actor"),
        Target::Reducer(name) => format!("{name}\nshow this state"),
        Target::Key(path) => format!("{path}\nshow this key"),
        Target::Records(name) => format!("{name}\nshow only records about it"),
        Target::Stream(_) => "show what it started".to_string(),
    }
}

/// Words that wrap, each link its own widget: for the one sentence at the top
/// of a record, not for rows.
pub fn sentence(ui: &mut egui::Ui, words: &[Word], go: &mut Option<Go>) {
    ui.spacing_mut().item_spacing.x = 0.0;

    for word in words {
        let text = components::mono(&word.text);
        match &word.link {
            None => {
                ui.label(text.color(theme::tone_color(&word.tone)));
            }
            Some(target) => {
                let strong = ui.visuals().strong_text_color();
                let response = ui
                    .add(egui::Link::new(text.color(strong).underline()))
                    .on_hover_text(hint(target));
                if response.clicked() {
                    *go = Some(Go::Link(target.clone()));
                }
            }
        }
    }
}

/// A part of a line that answers the pointer.
struct Spot {
    chars: Range<usize>,
    go: Option<Go>,
    hint: String,
}

/// A row's text as one piece of layout, and where in it the pointer means
/// something. One galley and one hit test per row, however many words.
pub struct Line {
    job: LayoutJob,
    chars: usize,
    spots: Vec<Spot>,
    /// Room before whatever is put next.
    room: f32,
    font: egui::FontId,
    strong: egui::Color32,
    weak: egui::Color32,
}

impl Line {
    pub fn new(ui: &egui::Ui) -> Self {
        Self {
            job: LayoutJob::default(),
            chars: 0,
            spots: Vec::new(),
            room: 0.0,
            font: egui::TextStyle::Monospace.resolve(ui.style()),
            strong: ui.visuals().strong_text_color(),
            weak: ui.visuals().weak_text_color(),
        }
    }

    fn format(&self, color: egui::Color32) -> TextFormat {
        TextFormat::simple(self.font.clone(), color)
    }

    fn put(&mut self, text: &str, format: TextFormat) -> Range<usize> {
        let start = self.chars;
        self.chars += text.chars().count();
        self.job.append(text, std::mem::take(&mut self.room), format);

        start..self.chars
    }

    pub fn text(&mut self, text: &str, color: egui::Color32) {
        let format = self.format(color);
        self.put(text, format);
    }

    pub fn weak(&mut self, text: &str) {
        self.text(text, self.weak);
    }

    /// A count on a chip: `×128`, as wide for a thousand as for one.
    pub fn count(&mut self, count: usize) {
        let format = TextFormat {
            background: crate::theme::CHIP,
            ..self.format(self.strong)
        };
        self.put(&format!(" ×{count:<4} "), format);
        self.text("  ", self.weak);
    }

    pub fn indent(&mut self, by: f32) {
        self.room += by;
    }

    pub fn link(&mut self, text: &str, color: egui::Color32, go: Go, hint: String) {
        let format = TextFormat {
            underline: egui::Stroke::new(1.0, color),
            ..self.format(color)
        };
        let chars = self.put(text, format);

        self.spots.push(Spot {
            chars,
            go: Some(go),
            hint,
        });
    }

    pub fn hinted(&mut self, text: &str, color: egui::Color32, hint: String) {
        let format = self.format(color);
        let chars = self.put(text, format);

        self.spots.push(Spot {
            chars,
            go: None,
            hint,
        });
    }

    pub fn words(&mut self, words: &[Word]) {
        for word in words {
            match &word.link {
                None => self.text(&word.text, theme::tone_color(&word.tone)),
                Some(target) => self.link(
                    &word.text,
                    self.strong,
                    Go::Link(target.clone()),
                    hint(target),
                ),
            }
        }
    }

    pub fn took(&mut self, took: Option<&String>) {
        if let Some(took) = took {
            self.weak(&format!("  {took}"));
        }
    }

    pub fn cause(&mut self, cause: Option<&Cause>) {
        match cause {
            None => {}
            Some(Cause::Known { id, gist, tone }) => {
                self.weak(" ← ");
                self.link(
                    gist,
                    theme::tone_color(tone),
                    Go::Record(*id),
                    "go to the cause".to_string(),
                );
            }
            Some(Cause::Forgotten) => {
                self.weak(" ← ");
                self.weak("something devtools no longer keep");
            }
        }
    }

    /// A record as the list shows it.
    pub fn record(ui: &egui::Ui, row: &Row) -> Self {
        let mut line = Self::new(ui);
        line.hinted(&format!("{}  ", row.time), line.weak, row.when.clone());
        line.words(&row.words);
        line.took(row.took.as_ref());
        line.cause(row.cause.as_ref());

        line
    }
}

/// A full-width row that does `click` when clicked anywhere but a link.
pub fn row(ui: &mut egui::Ui, height: f32, selected: bool, go: &mut Option<Go>, click: Go, line: Line) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );
    if !ui.is_rect_visible(rect) {
        return;
    }

    let fill = if selected {
        Some(ui.visuals().selection.bg_fill)
    } else if response.hovered() {
        Some(ui.visuals().widgets.hovered.weak_bg_fill)
    } else {
        None
    };
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, egui::CornerRadius::same(2), fill);
    }

    let text = rect.shrink2(egui::vec2(4.0, 0.0));
    if text.width() < 1.0 {
        return;
    }

    let Line { mut job, spots, .. } = line;
    job.wrap = TextWrapping::truncate_at_width(text.width());
    let galley = ui.painter().layout_job(job);

    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, galley.text())
    });

    let origin = egui::pos2(text.left(), text.center().y - galley.size().y / 2.0);
    let under = response.hover_pos().and_then(|pointer| {
        let at = pointer - origin;
        spots.iter().find(|spot| {
            let from = galley.pos_from_cursor(CCursor::new(spot.chars.start)).min.x;
            let to = galley.pos_from_cursor(CCursor::new(spot.chars.end)).min.x;
            (from..to).contains(&at.x)
        })
    });

    ui.painter()
        .with_clip_rect(rect.intersect(ui.clip_rect()))
        .galley(origin, galley, ui.visuals().text_color());

    match under {
        Some(spot) => {
            let response = response.on_hover_text_at_pointer(&spot.hint);
            if spot.go.is_some() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                *go = Some(spot.go.clone().unwrap_or(click));
            }
        }
        None => {
            if response.clicked() && go.is_none() {
                *go = Some(click);
            }
        }
    }
}
