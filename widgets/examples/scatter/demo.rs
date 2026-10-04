//! A scatter chart of process lifetimes over the last fifteen minutes, on a
//! coloured card it should let show through: points hovered, rectangles
//! dragged out, clicks, and the span moving with the clock.

use std::time::Instant;

use guinea_widgets::chart::scatter::{
    Level, Marker, Scale, Scatter, ScatterEvent, ScatterOptions, ScatterPoint, ScatterSeries,
};
use guinea_widgets::color::hex;
use windows_reactor::{
    App, Border, Color, Component, ComponentContext, Grid, StackPanel, TextBlock, View, ViewContext,
};

const SPAN: u64 = 15 * 60 * 1000;

struct Demo {
    scatter: Scatter,
    started: Instant,
    heard: String,
}

enum Message {
    Heard(ScatterEvent),
}

fn noise(seed: &mut u64) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed % 10_000) as f32 / 10_000.0
}

fn points(now: u64) -> Vec<ScatterSeries> {
    let mut seed = 0x2545_f491_4f6c_dd1d;
    let mut came = Vec::new();
    let mut running = Vec::new();
    let mut unknown = Vec::new();
    for key in 0..3_000u64 {
        let at = now.saturating_sub((noise(&mut seed) * SPAN as f32) as u64);
        let roll = noise(&mut seed);
        let point = |value| ScatterPoint { key, at, value };
        if roll < 0.08 {
            running.push(point(Level::Above(0)));
        } else if roll < 0.12 {
            unknown.push(point(Level::Below(0)));
        } else {
            let lifetime = 10f32.powf(noise(&mut seed) * 4.6 - 1.0);
            came.push(point(Level::Value(lifetime)));
        }
    }

    let series = |color, marker, points| ScatterSeries {
        color,
        marker,
        size: 5.0,
        points,
    };
    vec![
        series(hex(0x6ea8fe), Marker::Dot, came),
        series(hex(0x6ea8fe), Marker::Ring, running),
        series(hex(0xff6b6b), Marker::Tick, unknown),
    ]
}

fn options(now: u64) -> ScatterOptions {
    let ticks = (0..5)
        .map(|step| {
            let at = now - SPAN + step * SPAN / 4;
            (at, format!("-{} min", (now - at) / 60_000))
        })
        .collect();

    ScatterOptions {
        background: None,
        border: None,
        corner_radius: Some(8.0),
        x: (now - SPAN, now),
        live: Some(1_000.0),
        y: Scale::Log {
            from: 0.1,
            to: 3_600.0,
        },
        above: vec!["running".into()],
        below: vec!["unknown".into()],
        y_lines: vec![
            (0.1, "100 ms".into()),
            (1.0, "1 s".into()),
            (10.0, "10 s".into()),
            (60.0, "1 min".into()),
            (600.0, "10 min".into()),
            (3_600.0, "1 h".into()),
        ],
        x_ticks: ticks,
        ..ScatterOptions::default()
    }
}

impl Component for Demo {
    type Input = ();
    type Message = Message;

    fn create(_input: &(), _cx: &ComponentContext<Self>) -> Self {
        let started = Instant::now();
        let scatter = Scatter::new();
        let now = SPAN;
        scatter.publish(points(now), options(now));

        Self {
            scatter,
            started,
            heard: "move over a point, drag out a rectangle, click".into(),
        }
    }

    fn update(&mut self, message: Message, _cx: &ComponentContext<Self>) {
        let Message::Heard(event) = message;
        if let ScatterEvent::Brushed(area) = &event {
            let now = SPAN + self.started.elapsed().as_millis() as u64;
            self.scatter.publish(
                points(SPAN),
                ScatterOptions {
                    selection: Some(*area),
                    ..options(now)
                },
            );
        }
        self.heard = format!("{event:?}");
    }

    fn view(&self, _input: &(), cx: &mut ViewContext<Self>) -> View {
        let chart = Border::new()
            .background(Color::rgb(38, 70, 83))
            .corner_radius(8.0)
            .padding(12.0)
            .height(320.0)
            .content(self.scatter.view(cx.callback(Message::Heard)));

        let content = StackPanel::new().spacing(12.0).margin(16.0).children((
            chart,
            TextBlock::new().text(self.heard.clone()),
            Grid::new(),
        ));

        cx.window_frame("Scatter", content)
    }
}

pub fn run() {
    App::run_component::<Demo>(()).unwrap();
}
