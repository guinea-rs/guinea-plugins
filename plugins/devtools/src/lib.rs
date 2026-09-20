//! Sends what the application is doing to guinea devtools.
//!
//! ```no_run
//! # use guinea_plugin_devtools::DevToolsPlugin;
//! guinea::app::GuineaApp::new()
//!     .plugin(DevToolsPlugin::new())
//!     # ;
//! ```
//!
//! Devtools listen; the application connects, and keeps trying while devtools
//! are not running. See `guinea_devtools_protocol` for where and how.
//!
//! Nothing is traced or snapshotted until devtools answer, and it stops when
//! they go: an application that has the plugin and no devtools pays for one
//! atomic load per tick.

mod collect;
mod link;
mod profiler;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use guinea::app::{AppMeta, Plugin, PluginBuilder};
use guinea::feature::ContextTimersExt;
use guinea_devtools_protocol::{AppInfo, Capability, Report};

pub struct DevToolsPlugin {
    every_ms: u64,
}

impl Default for DevToolsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DevToolsPlugin {
    pub fn new() -> Self {
        Self { every_ms: 250 }
    }

    /// How often a snapshot is taken. 250 ms unless said otherwise.
    pub fn every(mut self, millis: u64) -> Self {
        self.every_ms = millis.max(16);
        self
    }
}

impl Plugin for DevToolsPlugin {
    const ID: &'static str = "guinea.devtools";

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        let info = app
            .try_require::<AppMeta>()
            .map(|meta| AppInfo {
                name: meta.name.to_string(),
                identifier: meta.identifier.to_string(),
                version: meta.version.to_string(),
                ..AppInfo::default()
            })
            .unwrap_or_default();
        let info = Arc::new(Mutex::new(AppInfo {
            pid: std::process::id(),
            plugins: vec![Self::ID.to_string()],
            capabilities: vec![Capability::Snapshot, Capability::Trace, Capability::Profiler],
            trace_epoch_ms: guinea_core::trace::started_at()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |since| since.as_millis() as u64),
            ..info
        }));

        let connected = Arc::new(AtomicBool::new(false));
        let mut outbox = link::spawn(info.clone(), connected.clone());
        let started = Instant::now();

        let traces = Rc::new(RefCell::new(collect::Traces::default()));

        app.repeat(Duration::from_millis(self.every_ms), move || {
            if !connected.load(Ordering::Relaxed) {
                if guinea_core::trace::is_observed() {
                    guinea_core::trace::stop_observing();
                    traces.borrow_mut().take();
                }
                return;
            }

            if !guinea_core::trace::is_observed() {
                let sink = traces.clone();
                guinea_core::trace::observe(move |record| sink.borrow_mut().push(record));
            }

            let snapshot = collect::snapshot(started);

            {
                let mut info = info.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                if let Some(backend) = snapshot.backend {
                    info.backend = backend.to_string();
                }
                if !snapshot.plugins.is_empty() {
                    info.plugins = snapshot.plugins.iter().map(|id| id.to_string()).collect();
                }
            }

            outbox.send(Report::Snapshot(snapshot.report));

            let batch = traces.borrow_mut().take();
            if !batch.spans.is_empty() || !batch.ends.is_empty() || batch.dropped > 0 {
                outbox.send(Report::Trace(batch));
            }
        })
        .untraced();

        app.on_cleanup(|_| {
            guinea_core::trace::stop_observing();
            Ok(())
        });
        Ok(())
    }
}
