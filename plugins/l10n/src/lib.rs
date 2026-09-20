//! Localisation for guinea applications: the store the whole process reads,
//! the Fluent resolver, the per-backend hooks, and the plugin that loads it all
//! at startup.
//!
//! ```no_run
//! # use guinea_plugin_l10n::L10nPlugin;
//! # #[derive(Clone, Default)] struct Strings;
//! # impl guinea_plugin_l10n::Localization for Strings {
//! #     fn for_tag(_: &str) -> Option<Self> { Some(Self) }
//! #     fn tag(&self) -> String { "en".into() }
//! # }
//! guinea::app::GuineaApp::new()
//!     .plugin(L10nPlugin::<Strings>::new("en"))
//!     # ;
//! ```
//!
//! Views read the current strings through a backend hook (`ui::use_l10n` under
//! the `winui` feature); anything switching the language calls
//! [`L10n::load`]. The plugin owns startup and, with the `persist` feature,
//! remembering the choice.

mod devtools;
#[cfg(feature = "fluent")]
pub mod fluent;
mod store;
pub mod ui;

pub use store::{Key, L10n, Localization};

use std::marker::PhantomData;

use guinea::app::{Plugin, PluginBuilder};

#[cfg(feature = "persist")]
const KEY: [&str; 2] = ["app", "language"];

/// Loads the application's strings before the first render.
pub struct L10nPlugin<S> {
    default_tag: String,
    persist: bool,
    resolver: PhantomData<fn() -> S>,
}

impl<S: Localization> L10nPlugin<S> {
    /// `default_tag` is a BCP-47 language tag - `"en"`, `"ru"`, `"pt-BR"`.
    pub fn new(default_tag: impl Into<String>) -> Self {
        Self {
            default_tag: default_tag.into(),
            persist: cfg!(feature = "persist"),
            resolver: PhantomData,
        }
    }

    /// Whether to restore the language chosen last run and save every change.
    /// On by default when the `persist` feature is enabled.
    ///
    /// Persistence needs `guinea-plugin-store` installed; without it the
    /// plugin warns and falls back to the default tag.
    pub fn persist(mut self, yes: bool) -> Self {
        self.persist = yes;
        self
    }
}

impl<S: Localization> Plugin for L10nPlugin<S> {
    const ID: &'static str = "guinea.l10n";

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        let tag = self
            .persist
            .then(|| saved_tag(app))
            .flatten()
            .unwrap_or_else(|| self.default_tag.clone());

        let strings = S::for_tag(&tag)
            .or_else(|| {
                tracing::warn!(%tag, "not a usable language tag, falling back");
                S::for_tag(&self.default_tag)
            })
            .ok_or_else(|| anyhow::anyhow!("`{}` is not a language tag", self.default_tag))?;

        L10n::<S>::load(strings);

        if self.persist {
            remember_changes::<S>(app);
        }

        let panel = devtools::watch::<S>();
        app.on_cleanup(move |_| {
            drop(panel);
            Ok(())
        });

        Ok(())
    }
}

#[cfg(feature = "persist")]
fn saved_tag(app: &PluginBuilder) -> Option<String> {
    use guinea_plugin_store::Store;

    let store = app.try_require::<Store>()?;
    match store.get::<String>(KEY) {
        Ok(tag) => tag,
        Err(e) => {
            tracing::warn!(error = %e, "could not read the saved language");
            None
        }
    }
}

#[cfg(not(feature = "persist"))]
fn saved_tag(_app: &PluginBuilder) -> Option<String> {
    None
}

#[cfg(feature = "persist")]
fn remember_changes<S: Localization>(app: &PluginBuilder) {
    use guinea_plugin_store::Store;

    let Some(store) = app.try_require::<Store>() else {
        tracing::warn!(
            "language persistence is on, but no store service is installed - \
             install `guinea-plugin-store` before this plugin"
        );
        return;
    };

    let subscription = L10n::<S>::subscribe(move |strings| {
        if let Err(e) = store.set(KEY, &strings.tag()) {
            tracing::warn!(error = %e, "could not save the language");
        }
    });

    app.on_cleanup(move |_| {
        drop(subscription);
        Ok(())
    });
}

#[cfg(not(feature = "persist"))]
fn remember_changes<S: Localization>(_app: &PluginBuilder) {}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea::app::TestApp;

    #[derive(Clone, Default, Debug, PartialEq)]
    struct Strings(String);

    impl Localization for Strings {
        fn for_tag(tag: &str) -> Option<Self> {
            (!tag.contains('!')).then(|| Self(tag.to_string()))
        }

        fn tag(&self) -> String {
            self.0.clone()
        }
    }

    #[test]
    fn loads_the_default_language() {
        let mut app = TestApp::new();
        app.install(L10nPlugin::<Strings>::new("en").persist(false))
            .expect("install");

        assert_eq!(L10n::<Strings>::current(), Strings("en".into()));
    }

    #[test]
    fn falls_back_when_the_saved_tag_is_unusable() {
        let mut app = TestApp::new();
        app.install(L10nPlugin::<Strings>::new("!broken").persist(false))
            .map(|_| ())
            .expect_err("a default tag that cannot be resolved is an error");
    }

    #[cfg(feature = "persist")]
    #[test]
    fn restores_and_saves_the_language_through_the_store() {
        use guinea_plugin_store::StorePlugin;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("store");
        let store = StorePlugin::at(&path);

        let mut app = TestApp::new();
        app.install(store).expect("store");
        app.install(L10nPlugin::<Strings>::new("en")).expect("l10n");

        assert_eq!(L10n::<Strings>::current(), Strings("en".into()));

        L10n::<Strings>::load(Strings("ru".into()));
        app.shutdown();

        let saved = guinea_plugin_store::amethystate::StoreBuilder::new(&path)
            .build()
            .expect("reopen")
            .get::<String>(KEY)
            .expect("read");
        assert_eq!(saved.as_deref(), Some("ru"));
    }
}
