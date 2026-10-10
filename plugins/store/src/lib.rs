//! Persistent storage for guinea applications, backed by [`amethystate`].
//!
//! ```no_run
//! # use guinea_plugin_store::StorePlugin;
//! guinea::app::GuineaApp::new()
//!     .plugin(StorePlugin::for_app("my-app", "settings"))
//!     # ;
//! ```
//!
//! Features reach the store with `app.require::<Store>()`, and open a struct
//! over it with [`StoreAccess::settings`]. It is not amethystate's global
//! store: nothing reads it through `global_store()` or a struct's `new()`.

mod devtools;
mod settings;

use std::path::PathBuf;

use amethystate::StoreBuilder;
use amethystate::migration::builder::MigrationBuilder;
use amethystate::store::builder::{Backend, Layout};
use guinea::app::{Plugin, PluginBuilder};

/// Re-exported so an application can read and write through the store without
/// depending on amethystate itself.
pub use amethystate;
pub use settings::StoreAccess;

/// Where the store keeps what it is given: in its file, or in memory for this
/// run because the file would not open. Provided when the store was asked to
/// fall back with [`StorePlugin::or_in_memory`].
pub use amethystate::store::Persistence;

/// The store this plugin provides.
pub type Store = amethystate::Store;

type Configure = Box<dyn FnOnce(StoreBuilder) -> StoreBuilder + Send>;
type Steps = Box<dyn FnOnce(&mut MigrationBuilder) + Send>;

enum Open {
    App {
        app: String,
        config: String,
        layout: Option<Layout>,
    },
    Path(PathBuf),
    Memory,
    Custom(Box<dyn FnOnce() -> anyhow::Result<StoreBuilder> + Send>),
}

/// Opens the application's store, runs pending migrations, provides it as a
/// service and closes it on shutdown.
///
/// The store is not amethystate's process-wide one: it is reached through
/// the context, with `require::<Store>()` or [`StoreAccess`], so any number
/// of them can be open in one process.
pub struct StorePlugin {
    open: Open,
    backend: Option<Backend>,
    configure: Option<Configure>,
    steps: Vec<Steps>,
    or_in_memory: bool,
    kept: Vec<Box<dyn Send>>,
}

impl StorePlugin {
    /// Stores under the platform's configuration directory for `app_name`.
    pub fn for_app(app_name: impl Into<String>, config_name: impl Into<String>) -> Self {
        Self::with_open(Open::App {
            app: app_name.into(),
            config: config_name.into(),
            layout: None,
        })
    }

    /// Stores at an explicit path.
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self::with_open(Open::Path(path.into()))
    }

    /// A store with no file: what is written lives until the application
    /// shuts down. For tests, and for an application that keeps nothing.
    /// [`backend`](Self::backend) does not apply to it.
    pub fn in_memory() -> Self {
        Self::with_open(Open::Memory)
    }

    /// Builds the store from a closure. Runs on the UI thread during
    /// installation.
    pub fn with(f: impl FnOnce() -> anyhow::Result<StoreBuilder> + Send + 'static) -> Self {
        Self::with_open(Open::Custom(Box::new(f)))
    }

    /// Which directory convention [`for_app`](Self::for_app) follows. Ignored
    /// for the other constructors.
    pub fn layout(mut self, layout: Layout) -> Self {
        if let Open::App { layout: slot, .. } = &mut self.open {
            *slot = Some(layout);
        }
        self
    }

    /// Which engine backs the store. Defaults to whatever amethystate picks.
    ///
    /// `Backend::Json` when more than one copy of the application may run:
    /// redb locks its file exclusively.
    pub fn backend(mut self, backend: Backend) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Any other [`StoreBuilder`] setting: `disk`, `file_write`, `limits`,
    /// `rules`, `when_it_will_not_open`, `when_it_will_not_read`, `context`,
    /// `provide`. Migration steps go through [`migrations`](Self::migrations).
    pub fn configure(
        mut self,
        f: impl FnOnce(StoreBuilder) -> StoreBuilder + Send + 'static,
    ) -> Self {
        self.configure = Some(match self.configure.take() {
            Some(before) => Box::new(move |builder| f(before(builder))),
            None => Box::new(f),
        });
        self
    }

    /// Migration steps written by hand, run with the `#[migrate]` ones when
    /// the store opens. Called more than once, every call's steps run.
    pub fn migrations(mut self, f: impl FnOnce(&mut MigrationBuilder) + Send + 'static) -> Self {
        self.steps.push(Box::new(f));
        self
    }

    /// Where the store will not open - a file that will not read, one
    /// another process holds, a directory it cannot write, a migration that
    /// fails - the application starts anyway on an empty store in memory,
    /// which writes nothing and leaves the file as it was. amethystate logs
    /// why, and it is provided as [`Persistence`] so the application can tell
    /// the user. A configuration directory [`for_app`](Self::for_app) cannot
    /// find, or an error from [`with`](Self::with)'s closure, still fails the
    /// install: there is no file yet to fall back from.
    pub fn or_in_memory(mut self) -> Self {
        self.or_in_memory = true;
        self
    }

    /// Keeps `guard` until the store is closed on shutdown, and drops it
    /// right after. A test hands it the `TempDir` the store lives in: the
    /// store writes itself out when it closes, so a directory removed
    /// before that comes back.
    pub fn keeping(mut self, guard: impl Send + 'static) -> Self {
        self.kept.push(Box::new(guard));
        self
    }

    fn with_open(open: Open) -> Self {
        Self {
            open,
            backend: None,
            configure: None,
            steps: Vec::new(),
            or_in_memory: false,
            kept: Vec::new(),
        }
    }
}

impl Plugin for StorePlugin {
    const ID: &'static str = "guinea.store";
    type Exports = ();

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        let in_memory = matches!(self.open, Open::Memory);
        let mut builder = match self.open {
            Open::App {
                app,
                config,
                layout,
            } => StoreBuilder::located(|at| match layout {
                Some(layout) => at.app_under(layout, &app, &config),
                None => at.app(&app, &config),
            })
            .map_err(|error| anyhow::anyhow!("opening the store: {error:?}"))?,
            Open::Path(path) => StoreBuilder::new(path),
            Open::Memory => StoreBuilder::in_memory(),
            Open::Custom(f) => f()?,
        };
        if let Some(backend) = self.backend.filter(|_| !in_memory) {
            builder = builder.backend(backend);
        }
        if let Some(configure) = self.configure {
            builder = configure(builder);
        }

        let steps = self.steps;
        let migrating = builder.migrations(move |migrations| {
            for step in steps {
                step(migrations);
            }
        });
        let (store, report, persistence) = if self.or_in_memory {
            let (store, report, persistence) = migrating.or_in_memory().migrate();
            (store, report, Some(persistence))
        } else {
            let (store, report) = migrating.migrate().map_err(|refused| match refused {
                amethystate::store::OpenStore::Migrating { why, report } => {
                    let failed = report
                        .as_ref()
                        .map(|report| report.failures().count())
                        .unwrap_or_default();

                    anyhow::anyhow!("the store's migration did not finish ({failed} failed): {why}")
                }
                other => anyhow::anyhow!("opening the store: {other:?}"),
            })?;
            (store, report, None)
        };

        let watching = devtools::Watching::start(&store, &report);
        let closing = store.clone();
        let kept = self.kept;

        app.on_cleanup(move |_| {
            drop(watching);
            let closed = closing
                .close()
                .map_err(|error| anyhow::anyhow!("closing the store: {error:?}"));
            drop(kept);
            closed
        });

        app.provide(store);
        if let Some(persistence) = persistence {
            app.provide(persistence);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea::app::TestApp;

    #[test]
    fn a_directory_kept_until_the_store_closes_is_gone_after_shutdown() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("store");
        let root = dir.path().to_path_buf();
        let mut app = TestApp::new();

        let installed = app
            .install(StorePlugin::at(&path).backend(Backend::Json).keeping(dir))
            .map(|_| ())
            .map_err(|error| format!("{error:#}"));
        let written = app
            .require::<Store>()
            .map_err(|error| format!("{error:#}"))
            .and_then(|store| {
                store
                    .kv()
                    .set("greeting", &"hello")
                    .map_err(|error| format!("{error:?}"))
            });
        let nothing_leaked = app.shutdown().is_empty();

        assert_eq!((installed, written), (Ok(()), Ok(())));
        assert!(nothing_leaked, "no actors should leak");
        assert!(!root.exists(), "{} is still there", root.display());
    }

    #[test]
    fn provides_a_working_store_and_closes_it_on_shutdown() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("store");
        let mut app = TestApp::new();

        app.install(StorePlugin::at(&path).backend(Backend::Json))
            .expect("install");

        let store = app.require::<Store>().expect("store provided");
        store.kv().set("greeting", &"hello").expect("set");
        drop(store);

        assert!(app.shutdown().is_empty(), "no actors should leak");

        let reopened = StoreBuilder::new(&path)
            .backend(Backend::Json)
            .build()
            .expect("reopen");
        assert_eq!(
            reopened.kv().get::<String>("greeting").expect("get"),
            Some("hello".to_string()),
        );
    }
}
