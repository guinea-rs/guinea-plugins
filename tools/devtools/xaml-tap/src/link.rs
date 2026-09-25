//! The connection to devtools, on a thread and a runtime of its own.
//!
//! The tap is a peer like any application: the same endpoint, the same key,
//! a [`Report::Hello`] first. It lists the native capabilities, sends the whole
//! tree and then what changes, and answers commands. When devtools go away it
//! takes the highlight down and waits for them to come back.

use std::sync::Mutex;
use std::time::Duration;

use guinea_devtools_protocol::devtools_capnp::peer;
use guinea_devtools_protocol::{AppInfo, Capability, Command, Report, key, wire};
use ogurpchik::rpc::connect_session;

use crate::{highlight, inspect, perf, tree, ui};

const RETRY: Duration = Duration::from_secs(1);
const FLUSH: Duration = Duration::from_millis(50);

static OUTBOX: Mutex<Vec<Report>> = Mutex::new(Vec::new());

fn post(report: Report) {
    OUTBOX.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push(report);
}

fn take_posted() -> Vec<Report> {
    std::mem::take(&mut *OUTBOX.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
}

pub fn spawn() {
    let spawned = std::thread::Builder::new()
        .name("guinea-xaml-tap".into())
        .spawn(|| match compio::runtime::Runtime::new() {
            Ok(runtime) => runtime.block_on(run()),
            Err(error) => tracing::warn!(%error, "the xaml tap has no runtime"),
        });

    if let Err(error) = spawned {
        tracing::warn!(%error, "the xaml tap link did not start");
    }
}

fn hello() -> AppInfo {
    let name = std::env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
        .unwrap_or_default();

    AppInfo {
        name,
        identifier: "guinea.xaml-tap".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        backend: "WinUI XAML".into(),
        pid: std::process::id(),
        capabilities: vec![
            Capability::NativeTree,
            Capability::NativeProperties,
            Capability::NativeEdit,
            Capability::NativeHitTest,
            Capability::NativeHighlight,
            Capability::NativePerf,
        ],
        ..AppInfo::default()
    }
}

async fn run() {
    let endpoint = guinea_devtools_protocol::endpoint();

    loop {
        let session = match key::read() {
            Ok(secret) => connect_session::<peer::Client, _>(
                &endpoint,
                &key::handshake(secret),
                guinea_devtools_protocol::schema(),
                Inbound,
            )
            .await
            .ok(),
            Err(_) => None,
        };
        let Some(session) = session else {
            compio::time::sleep(RETRY).await;
            continue;
        };

        let remote = session.remote();
        take_posted();

        if let Err(error) = perf::start() {
            tracing::debug!(%error, "no performance recording");
        }

        let enums = inspect::with(|inspector| inspector.enums()).unwrap_or_default();
        let opening = [
            Report::Hello(hello()),
            Report::NativeTree {
                changes: tree::whole(),
            },
            Report::NativeEnums { enums },
        ];
        let mut open = true;
        for report in &opening {
            open &= wire::send(remote, report).await.is_ok();
        }

        while open {
            compio::time::sleep(FLUSH).await;

            let changes = tree::take_changes();
            if !changes.is_empty() {
                open &= wire::send(remote, &Report::NativeTree { changes }).await.is_ok();
            }

            for report in take_posted() {
                open &= wire::send(remote, &report).await.is_ok();
            }
        }

        perf::stop();
        ui::on_ui(highlight::hide);
    }
}

/// Carries out what devtools ask, on the UI thread, and posts the answer.
struct Inbound;

impl peer::Server for Inbound {
    async fn send(
        self: capnp::capability::Rc<Self>,
        params: peer::SendParams,
        _results: peer::SendResults,
    ) -> Result<(), capnp::Error> {
        let command: Command = wire::received(&params)?;
        let name = format!("{:?}", command.needs());

        match answer(command) {
            Ok(Some(report)) => post(report),
            Ok(None) => {}
            Err(reason) => post(Report::Refused { command: name, reason }),
        }
        Ok(())
    }
}

fn answer(command: Command) -> Result<Option<Report>, String> {
    match command {
        Command::NativeProperties { element } => {
            let properties = inspect::with(|inspector| inspector.properties(element))?;
            Ok(Some(Report::NativeProperties { element, properties }))
        }
        Command::NativeSetProperty {
            element,
            property,
            type_name,
            value,
        } => {
            let properties = inspect::with(|inspector| {
                inspector.set_property(element, property, &type_name, &value)?;
                inspector.properties(element)
            })?;
            Ok(Some(Report::NativeProperties { element, properties }))
        }
        Command::NativeHitTest { x, y } => {
            let (chain, bounds) = inspect::with(|inspector| inspector.hit_test(x, y))?;
            Ok(Some(Report::NativePicked { chain, bounds }))
        }
        Command::NativeHighlight { element } => {
            inspect::with(|inspector| {
                match element.and_then(|element| inspector.bounds(element)) {
                    Some((bounds, window)) => highlight::show(bounds, window),
                    None => highlight::hide(),
                }
                Ok(())
            })?;
            Ok(None)
        }
        Command::NativePerfCapture => Ok(Some(Report::NativePerf { frames: perf::capture()? })),
        // The tap reads someone else's XAML tree; it draws no frames of its
        // own, so there is nothing here to profile.
        Command::Profiler { .. } => Err("the XAML tap has no profiler".to_string()),
    }
}
