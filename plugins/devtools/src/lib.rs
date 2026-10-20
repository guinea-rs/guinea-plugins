//! Sends what the application is doing to guinea devtools.
//!
//! ```no_run
//! use guinea_plugin_devtools::{DevToolsConfig, DevToolsPlugin};
//! fn app(app: &mut guinea::app::FeatureBuilder) -> anyhow::Result<()> {
//!     app.plugin::<DevToolsPlugin>(DevToolsConfig::new())?;
//!     Ok(())
//! }
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
//! [`in_release`](DevToolsConfig::in_release) says otherwise: devtools can
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
use std::time::Duration;

use guinea::app::{AppMeta, PluginBuilder};
use guinea_devtools_protocol::{AppInfo, Capability};

pub struct DevToolsConfig {
    meta: Option<AppMeta>,
    wait_ms: u64,
    reread_ms: u64,
    launch: bool,
    in_release: bool,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DevToolsConfig {
    pub fn new() -> Self {
        Self {
            meta: None,
            wait_ms: 50,
            reread_ms: 1000,
            launch: false,
            in_release: false,
        }
    }

    pub fn for_app(mut self, meta: AppMeta) -> Self {
        self.meta = Some(meta);
        self
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

guinea::plugin! { pub DevToolsPlugin {} }

#[guinea::builds]
fn devtools(app: &mut PluginBuilder, config: DevToolsConfig) -> anyhow::Result<DevToolsPlugin> {
    let observed = app.observed();
    anyhow::ensure!(
        !live::is_started(),
        "devtools is already installed on this UI thread"
    );
    if !cfg!(debug_assertions) && !config.in_release {
        tracing::info!(
            "devtools stay off in a release build; DevToolsConfig::in_release turns them on"
        );
        return Ok(DevToolsPlugin);
    }

    let info = config
        .meta
        .map(|meta| AppInfo {
            name: meta.name.to_string(),
            identifier: meta.identifier.to_string(),
            version: meta.version.to_string(),
            ..AppInfo::default()
        })
        .unwrap_or_default();
    let info = Arc::new(Mutex::new(AppInfo {
        pid: std::process::id(),
        plugins: vec![std::any::type_name::<DevToolsPlugin>().to_string()],
        capabilities: {
            let mut capabilities = vec![Capability::Snapshot, Capability::Trace, Capability::Act];
            if cfg!(feature = "profiling") {
                capabilities.push(Capability::Profiler);
            }
            capabilities
        },
        trace_epoch_ms: guinea::observability::trace::started_at()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_millis() as u64),
        actions: observed
            .actions()
            .iter()
            .map(|name| name.to_string())
            .collect(),
        events: observed
            .events()
            .iter()
            .map(|name| name.to_string())
            .collect(),
        clock: Some(clock::anchor()),
        ..info
    }));

    let outbox = link::spawn(info.clone(), config.launch);
    live::start(
        outbox,
        info,
        observed,
        Duration::from_millis(config.wait_ms),
        Duration::from_millis(config.reread_ms),
    );

    app.on_cleanup(|| {
        live::stop();
        Ok(())
    });
    Ok(DevToolsPlugin)
}
