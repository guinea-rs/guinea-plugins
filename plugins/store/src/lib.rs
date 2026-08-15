//! Persistent storage for guinea applications, backed by [`amethystate`].
//!
//! ```no_run
//! # use guinea_plugin_store::StorePlugin;
//! guinea::app::GuineaApp::new()
//!     .plugin(StorePlugin::for_app("my-app", "settings"))
//!     # ;
//! ```
//!
//! Features reach the store with `app.require::<Store>()`; view code that
//! already talks to `amethystate` directly can keep using
//! [`amethystate::global_store`] - the plugin initialises the same global.

use std::path::PathBuf;

use amethystate::StoreBuilder;
use guinea::app::{Plugin, PluginBuilder};

/// Re-exported so an application can read and write through the store - the
/// `get`/`set` methods live on [`amethystate::Store`] - without depending on
/// amethystate itself.
pub use amethystate;

/// The store this plugin provides. `app.require::<Store>()` hands out an
/// `Arc` of it; the store itself is cheap to clone and safe to share.
pub type Store = amethystate::Store;

enum Open {
    App { app: String, config: String },
    Path(PathBuf),
    Custom(Box<dyn FnOnce() -> anyhow::Result<StoreBuilder> + Send>),
}

/// Opens the application's store, runs pending migrations, and provides the
/// result as a service.
///
/// The underlying global can only be initialised once per process, so this
/// plugin must be the only thing calling [`amethystate::init_global`].
pub struct StorePlugin {
    open: Open,
    save_on_exit: bool,
}

impl StorePlugin {
    /// Stores under the platform's configuration directory for `app_name`.
    pub fn for_app(app_name: impl Into<String>, config_name: impl Into<String>) -> Self {
        Self::with_open(Open::App {
            app: app_name.into(),
            config: config_name.into(),
        })
    }

    /// Stores at an explicit path.
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self::with_open(Open::Path(path.into()))
    }

    /// Builds the store from a closure, for migrations and other
    /// [`StoreBuilder`] configuration.
    ///
    /// The closure runs on the UI thread during installation; `StoreBuilder`
    /// itself never crosses threads, which is why this takes a factory rather
    /// than a built builder.
    pub fn with(f: impl FnOnce() -> anyhow::Result<StoreBuilder> + Send + 'static) -> Self {
        Self::with_open(Open::Custom(Box::new(f)))
    }

    /// Flushes the store during shutdown. On by default.
    pub fn save_on_exit(mut self, yes: bool) -> Self {
        self.save_on_exit = yes;
        self
    }

    fn with_open(open: Open) -> Self {
        Self {
            open,
            save_on_exit: true,
        }
    }
}

impl Plugin for StorePlugin {
    const ID: &'static str = "guinea.store";

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        let builder = match self.open {
            Open::App { app, config } => StoreBuilder::for_app(&app, &config)?,
            Open::Path(path) => StoreBuilder::new(path),
            Open::Custom(f) => f()?,
        };

        let report = amethystate::init_global(builder);
        report.log_to_tracing();
        if report.has_failures() {
            anyhow::bail!("store migration failed - see the report above");
        }

        let store = amethystate::global_store();

        if self.save_on_exit {
            let store = store.clone();
            app.on_cleanup(move |_| {
                store.save_now()?;
                Ok(())
            });
        }

        app.provide(store);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea::app::TestApp;

    #[test]
    fn provides_a_working_store_and_flushes_it_on_shutdown() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut app = TestApp::new();

        app.install(StorePlugin::at(dir.path().join("store")))
            .expect("install");

        let store = app.require::<Store>().expect("store provided");
        store.set("greeting", &"hello").expect("set");

        assert!(app.shutdown().is_empty(), "no actors should leak");

        assert_eq!(
            amethystate::global_store()
                .get::<String>("greeting")
                .expect("get"),
            Some("hello".to_string()),
        );
    }
}
