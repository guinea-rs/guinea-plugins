//! Two made-up applications talking to these devtools, for `--demo`.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use guinea_devtools_protocol::devtools_capnp::peer;
use guinea_devtools_protocol::{
    Actor, AppInfo, BusKind, BusSubscription, Channel, Flow, Handled, Listener, Node, Panel,
    ReducerState, Report, Root, Segment, Snapshot, Span, TraceBatch, TracePoint, key, wire,
};
use ogurpchik::rpc::connect_session;

pub fn spawn() {
    for app in [processes(), terminal()] {
        std::thread::spawn(move || {
            let Ok(runtime) = compio::runtime::Runtime::new() else {
                return;
            };

            runtime.block_on(async move {
                for _ in 0..50 {
                    if let Ok(secret) = key::read() {
                        let endpoint = guinea_devtools_protocol::endpoint();
                        if let Ok(session) =
                            connect_session::<peer::Client, _>(&endpoint, &key::handshake(secret), Deaf)
                                .await
                        {
                            let _ = run(session.remote(), &app).await;
                            return;
                        }
                    }

                    compio::time::sleep(Duration::from_millis(100)).await;
                }
            });
        });
    }
}

struct Deaf;

impl peer::Server for Deaf {
    async fn send(
        self: capnp::capability::Rc<Self>,
        _params: peer::SendParams,
        _results: peer::SendResults,
    ) -> Result<(), capnp::Error> {
        Ok(())
    }
}

struct Fake {
    info: AppInfo,
    windows: usize,
    winui: bool,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

fn processes() -> Fake {
    Fake {
        info: AppInfo {
            name: "Processes".into(),
            identifier: "dev.uniproc.guinea.processes".into(),
            version: "0.1.0".into(),
            backend: "winui".into(),
            pid: 18244,
            plugins: vec![
                "guinea.store".into(),
                "guinea.l10n".into(),
                "guinea.devtools".into(),
            ],
            trace_epoch_ms: now_ms(),
        },
        windows: 2,
        winui: true,
    }
}

fn terminal() -> Fake {
    Fake {
        info: AppInfo {
            name: "Processes (terminal)".into(),
            identifier: "guinea-processes".into(),
            version: "0.1.0".into(),
            backend: "ratatui".into(),
            pid: 20931,
            plugins: vec!["guinea.store".into(), "guinea.devtools".into()],
            trace_epoch_ms: now_ms(),
        },
        windows: 1,
        winui: false,
    }
}

async fn run(remote: &peer::Client, app: &Fake) -> Result<(), capnp::Error> {
    let start = Instant::now();
    wire::send(remote, &Report::Hello(app.info.clone())).await?;

    let mut tick = 0u64;
    loop {
        let at = start.elapsed().as_millis() as u64;
        wire::send(remote, &Report::Snapshot(snapshot(app, at, tick))).await?;
        wire::send(remote, &Report::Trace(trace(at, tick))).await?;

        tick += 1;
        compio::time::sleep(Duration::from_millis(250)).await;
    }
}

fn snapshot(app: &Fake, at: u64, tick: u64) -> Snapshot {
    let cpu = 50.0 + 35.0 * (tick as f32 / 6.0).sin();

    let roots = (0..app.windows)
        .map(|window| {
            let page = if window == 0 { "Metrics" } else { "Processes" };

            Root {
                id: window as u64 + 1,
                label: Some(if window == 0 { "main".into() } else { format!("window {window}") }),
                route: Some(format!("{page} {{ context: \"ubuntu\" }}")),
                back: 2 - window.min(2),
                forward: 0,
                pending: None,
                chain: vec![
                    Segment {
                        name: "TabsLayout".into(),
                        params: "TabsLayoutParams { context: \"ubuntu\" }".into(),
                        features: vec!["TabsFeature".into()],
                        reducers: vec![ReducerState {
                            type_name: "tabs::Tabs".into(),
                            state: Some(format!(
                                "Tabs {{\n    context: \"ubuntu\",\n    install_count: 1,\n    kills_this_window: {},\n    kills_all_windows: {},\n    last_killed: None,\n}}",
                                tick / 40,
                                tick / 30
                            )),
                            feature: Some("TabsFeature".into()),
                        }],
                        listeners: vec![
                            listener("events::ProcessKilled", BusKind::Window),
                            listener("events::ProcessKilled", BusKind::Global),
                        ],
                    },
                    Segment {
                        name: page.into(),
                        params: format!("{page}Params {{ context: \"ubuntu\" }}"),
                        features: vec![format!("{page}Feature")],
                        reducers: vec![ReducerState {
                            type_name: format!("{}::{page}", page.to_lowercase()),
                            state: if window == 0 {
                                Some(format!("Metrics {{ cpu: RingSeries(len 60, last {cpu:.1}), memory: RingSeries(len 60) }}"))
                            } else {
                                None
                            },
                            feature: Some(format!("{page}Feature")),
                        }],
                        listeners: Vec::new(),
                    },
                ],
                panels: if app.winui {
                    vec![components(window, page, tick)]
                } else {
                    Vec::new()
                },
                bus: vec![BusSubscription {
                    event: "events::ProcessKilled".into(),
                    subscribers: 1,
                }],
            }
        })
        .collect();

    Snapshot {
        at,
        roots,
        actors: vec![
            Actor {
                id: 3,
                type_name: "metrics::MetricsActor".into(),
                state: format!("MetricsActor {{\n    tick: {tick},\n    start: Instant {{ .. }},\n}}"),
                root: Some(1),
                segment: Some(1),
                feature: Some("MetricsFeature".into()),
                drives: Some("metrics::Metrics".into()),
                handles: vec![handles("actor::Tick", &[(Channel::Bg, "actor::Tick", true)])],
                ..Actor::default()
            },
            Actor {
                id: 7,
                type_name: "processes::ProcessActor".into(),
                state: "ProcessActor {\n    items: [\n        \"systemd (pid 1)\",\n        \"sshd (pid 42)\",\n        \"bash (pid 512)\",\n    ],\n}".into(),
                root: Some(2),
                segment: Some(1),
                feature: Some("ProcessesFeature".into()),
                drives: Some("processes::Processes".into()),
                handles: vec![
                    handles("contracts::Kill", &[(Channel::Send, "contracts::Refresh", false)]),
                    handles("contracts::Refresh", &[(Channel::Ask, "store::Load", false)]),
                ],
                publishes: vec!["events::ProcessKilled".into()],
                ..Actor::default()
            },
            Actor {
                id: 9,
                type_name: "startup::Housekeeping".into(),
                state: format!("Housekeeping {{ sweeps: {} }}", tick / 20),
                root: None,
                handles: vec![
                    handles("startup::Sweep", &[]),
                    handles("events::ProcessKilled", &[(Channel::Send, "startup::Sweep", false)]),
                ],
                subscribes: vec!["events::ProcessKilled".into()],
                ..Actor::default()
            },
        ],
        panels: vec![Panel {
            id: "store".into(),
            title: "Store".into(),
            nodes: vec![Node {
                label: "settings.json".into(),
                kind: "json".into(),
                properties: vec![("path".into(), "%APPDATA%/guinea-processes-app-example/config/settings.json".into())],
                children: vec![
                    leaf("app.launches", "u64", "42"),
                    leaf("app.language", "String", "\"ru\""),
                    leaf("window.main", "Saved", "{ size: [675, 480], position: [205, 189] }"),
                ],
            }],
        }],
        global_bus: vec![BusSubscription {
            event: "events::ProcessKilled".into(),
            subscribers: 2,
        }],
        timers: Vec::new(),
    }
}

fn listener(event: &str, bus: BusKind) -> Listener {
    Listener {
        event: event.into(),
        actor: None,
        bus,
        feature: Some("TabsFeature".into()),
    }
}

fn handles(message: &str, edges: &[(Channel, &str, bool)]) -> Handled {
    Handled {
        message: message.into(),
        edges: Some(
            edges
                .iter()
                .map(|(channel, target, looping)| Flow {
                    channel: *channel,
                    target: target.to_string(),
                    looping: *looping,
                })
                .collect(),
        ),
        declared: None,
    }
}

fn leaf(label: &str, kind: &str, value: &str) -> Node {
    Node {
        label: label.into(),
        kind: kind.into(),
        properties: vec![("value".into(), value.into())],
        children: Vec::new(),
    }
}

fn element(label: &str, kind: &str, properties: &[(&str, String)], children: Vec<Node>) -> Node {
    Node {
        label: label.into(),
        kind: kind.into(),
        properties: properties
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
        children,
    }
}

fn components(window: usize, page: &str, tick: u64) -> Panel {
    let rows = (0..3)
        .map(|row| {
            element(
                &format!("row {row}"),
                "ListViewItem",
                &[("key", format!("\"{}\"", [1, 42, 512][row]))],
                vec![element("TextBlock", "TextBlock", &[("text", format!("\"proc {row}\""))], Vec::new())],
            )
        })
        .collect();

    Panel {
        id: "winui.components".into(),
        title: "Components".into(),
        nodes: vec![element(
            "Root<Route>",
            "component",
            &[("window", format!("{window}")), ("title", "\"guinea · processes\"".into())],
            vec![element(
                "RouterRoot<Route>",
                "component",
                &[("route", format!("{page} {{ context: \"ubuntu\" }}"))],
                vec![element(
                    "LayoutNode<TabsLayout>",
                    "component",
                    &[("renders", tick.to_string())],
                    vec![element(
                        "StackPanel",
                        "StackPanel",
                        &[("orientation", "Vertical".into()), ("spacing", "12".into())],
                        vec![
                            element("tabs", "StackPanel", &[("orientation", "Horizontal".into())], vec![
                                element("Button", "Button", &[("content", "\"Processes\"".into())], Vec::new()),
                                element("Button", "Button", &[("content", "\"Metrics\"".into())], Vec::new()),
                            ]),
                            element(
                                &format!("PageNode<{page}>"),
                                "component",
                                &[("renders", (tick * 3).to_string())],
                                vec![element("table", "ListView", &[("items", "3".into()), ("selected", "None".into())], rows)],
                            ),
                        ],
                    )],
                )],
            )],
        )],
    }
}

/// A made-up but causally sound batch: a metrics loop every tick, and now and
/// then a kill from the UI that publishes and refreshes.
fn trace(at: u64, tick: u64) -> TraceBatch {
    let base = 1 + tick * 100;
    let at = at * 1_000;

    let mut spans = Vec::new();
    let mut push = |offset: u64, parent: Option<u64>, took: Option<u64>, point: TracePoint| {
        spans.push(Span {
            id: base + offset,
            parent,
            at: at + offset * 40,
            took,
            point,
        });

        base + offset
    };
    let text = |s: &str| s.to_string();

    const METRICS: &str = "metrics::MetricsActor";
    const PROCESSES: &str = "processes::ProcessActor";
    const HOUSEKEEPING: &str = "startup::Housekeeping";
    const KILLED: &str = "events::ProcessKilled";

    let previous_spawn = (tick > 0).then(|| base - 100 + 2);
    let send = push(0, previous_spawn, None, TracePoint::Send { actor: text(METRICS), message: text("actor::Tick") });
    let handle = push(1, Some(send), Some(90), TracePoint::Handle { actor: text(METRICS), message: text("actor::Tick") });
    push(2, Some(handle), None, TracePoint::Spawn { actor: text(METRICS), output: text("actor::Tick") });
    push(3, Some(handle), None, TracePoint::Push { reducer: text("metrics::Metrics") });

    if tick.is_multiple_of(9) {
        let action = push(10, None, Some(410), TracePoint::Action { message: text("contracts::Kill") });
        let send = push(11, Some(action), None, TracePoint::Send { actor: text(PROCESSES), message: text("contracts::Kill") });
        let handle = push(12, Some(send), Some(380), TracePoint::Handle { actor: text(PROCESSES), message: text("contracts::Kill") });
        push(13, Some(handle), None, TracePoint::Push { reducer: text("processes::Processes") });

        let local = push(14, Some(handle), Some(60), TracePoint::Publish { event: text(KILLED), bus: BusKind::Window, subscribers: 1 });
        let delivered = push(15, Some(local), Some(30), TracePoint::Deliver { event: text(KILLED), bus: BusKind::Window });
        push(16, Some(delivered), None, TracePoint::Push { reducer: text("tabs::Tabs") });

        let global = push(17, Some(handle), Some(120), TracePoint::Publish { event: text(KILLED), bus: BusKind::Global, subscribers: 2 });
        let delivered = push(18, Some(global), Some(20), TracePoint::Deliver { event: text(KILLED), bus: BusKind::Global });
        push(19, Some(delivered), None, TracePoint::Push { reducer: text("tabs::Tabs") });

        let send = push(20, Some(global), None, TracePoint::Send { actor: text(HOUSEKEEPING), message: text(KILLED) });
        let handled = push(21, Some(send), Some(15), TracePoint::Handle { actor: text(HOUSEKEEPING), message: text(KILLED) });
        push(22, Some(handled), None, TracePoint::Send { actor: text(HOUSEKEEPING), message: text("startup::Sweep") });
    }

    if tick.is_multiple_of(13) {
        let nav = push(30, None, Some(900), TracePoint::Navigate { root: text("main"), to: text("Metrics") });
        let send = push(31, Some(nav), None, TracePoint::Send { actor: text(METRICS), message: text("actor::Tick") });
        push(32, Some(send), Some(70), TracePoint::Handle { actor: text(METRICS), message: text("actor::Tick") });
    }

    TraceBatch {
        spans,
        ends: Vec::new(),
        dropped: 0,
    }
}
