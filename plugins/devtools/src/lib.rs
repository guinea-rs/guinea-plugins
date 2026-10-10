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
//! Nothing is traced or read until devtools answer, and it stops when they
//! go: an application that has the plugin and no devtools runs nothing for
//! it. While they are there, they are sent everything once and then only
//! what guinea says moved: the trace a short wait after it happened, the
//! state that moved at most once a second, however often it moves - reading
//! state prints it whole, and an application that streams would otherwise
//! spend more on its debugger than on itself.
//!
//! In a release build the plugin does nothing unless
//! [`in_release`](DevToolsPlugin::in_release) says otherwise: devtools can
//! call the application's remote actions, and a build shipped to users should
//! not listen for that because a line was left in.

mod clock;
mod collect;
mod launch;
mod link;
mod live;
mod profiler;
mod runtime;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use guinea::app::{AppMeta, Plugin, PluginBuilder};
use guinea_devtools_protocol::{AppInfo, Capability};

pub struct DevToolsPlugin {
    wait_ms: u64,
    reread_ms: u64,
    launch: bool,
    in_release: bool,
}

impl Default for DevToolsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DevToolsPlugin {
    pub fn new() -> Self {
        Self {
            wait_ms: 50,
            reread_ms: 1000,
            launch: false,
            in_release: false,
        }
    }

    /// Connects in a release build too. Off unless asked for.
    pub fn in_release(mut self, on: bool) -> Self {
        self.in_release = on;
        self
    }

    /// How long after something moves devtools are told, so that a burst
    /// goes as one report. 50 ms unless said otherwise.
    pub fn every(mut self, millis: u64) -> Self {
        self.wait_ms = millis.max(16);
        self
    }

    /// How often at most state that keeps moving is read again and sent.
    /// The trace is not held back by it. 1000 ms unless said otherwise.
    pub fn reread(mut self, millis: u64) -> Self {
        self.reread_ms = millis;
        self
    }

    /// Starts devtools when they are not running, once, as the application
    /// starts. The binary is `GUINEA_DEVTOOLS_BIN`, or `guinea-devtools` from
    /// the `PATH`. Off unless asked for.
    pub fn launch(mut self, on: bool) -> Self {
        self.launch = on;
        self
    }
}

impl Plugin for DevToolsPlugin {
    const ID: &'static str = "guinea.devtools";
    type Exports = ();

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        if !cfg!(debug_assertions) && !self.in_release {
            tracing::info!(
                "devtools stay off in a release build; DevToolsPlugin::in_release turns them on"
            );
            return Ok(());
        }

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
            capabilities: vec![
                Capability::Snapshot,
                Capability::Trace,
                Capability::Profiler,
                Capability::Act,
            ],
            trace_epoch_ms: guinea_core::trace::started_at()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |since| since.as_millis() as u64),
            actions: guinea_core::remote::actions()
                .iter()
                .map(|name| name.to_string())
                .collect(),
            events: guinea_core::remote::events()
                .iter()
                .map(|name| name.to_string())
                .collect(),
            clock: Some(clock::anchor()),
            ..info
        }));

        let outbox = link::spawn(info.clone(), self.launch);
        live::start(
            outbox,
            info,
            Instant::now(),
            Duration::from_millis(self.wait_ms),
            Duration::from_millis(self.reread_ms),
        );

        app.on_cleanup(|_| {
            live::stop();
            Ok(())
        });
        Ok(())
    }
}
