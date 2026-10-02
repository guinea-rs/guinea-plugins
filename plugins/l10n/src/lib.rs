//! Localisation for guinea applications: the language the application shows,
//! the Fluent resolver, and the plugin that loads it at startup.
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
//! The plugin claims [`Language`] for the application and exports it, so
//! every page of every window reads it - `cx.l10n::<L10n>()` through
//! [`L10nAccess`] on WinUI, eframe and ratatui, `state::<Language<L10n>, _>()`
//! on iced - and is drawn again when it changes. A switch is an action on it, [`SwitchLanguage`] with a
//! tag. With the `persist` feature the plugin remembers the choice.

mod devtools;
#[cfg(feature = "fluent")]
pub mod fluent;
mod store;

pub use store::{Key, L10nAccess, Language, Localization, SwitchLanguage};

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
            .filter(|strings| has_locale::<S>(&strings.tag()))
            .or_else(|| {
                tracing::warn!(%tag, "no strings for this language, falling back");
                S::for_tag(&self.default_tag)
            })
            .ok_or_else(|| anyhow::anyhow!("`{}` is not a language tag", self.default_tag))?;

        let language = app.state::<Language<S>>().seed(Language::new(strings)).plain();
        app.export::<Language<S>>()?;
        app.answers(move |SwitchLanguage(tag)| {
            match S::for_tag(&tag).filter(|strings| has_locale::<S>(&strings.tag())) {
                Some(strings) => language.push(strings),
                None => tracing::warn!(%tag, "no strings for this language, it stays"),
            }
        });

        if self.persist {
            remember_changes::<S>(app);
        }

        let panel = devtools::watch::<S>(app.scope);
        app.on_cleanup(move |_| {
            drop(panel);
            Ok(())
        });

        Ok(())
    }
}

/// Whether the application has strings for `tag`. A resolver that does not
/// list its locales is taken at its word for any tag it accepts.
fn has_locale<S: Localization>(tag: &str) -> bool {
    let languages = S::languages();
    languages.is_empty()
        || languages
            .iter()
            .any(|known| known.eq_ignore_ascii_case(tag))
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

    app.observe::<Language<S>>(move |strings: &S| {
        if let Err(e) = store.set(KEY, &strings.tag()) {
            tracing::warn!(error = %e, "could not save the language");
        }
    });
}

#[cfg(not(feature = "persist"))]
fn remember_changes<S: Localization>(_app: &PluginBuilder) {}

#[cfg(test)]
mod tests {
    use super::*;
    use guinea::app::{Harness, TestApp};

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

    #[derive(Clone, Default, Debug, PartialEq)]
    struct Listed(String);

    impl Localization for Listed {
        fn for_tag(tag: &str) -> Option<Self> {
            Some(Self(tag.to_string()))
        }

        fn tag(&self) -> String {
            self.0.clone()
        }

        fn languages() -> &'static [&'static str] {
            &["en", "ru"]
        }
    }

    fn shown<S: Localization>(h: &Harness) -> S {
        h.state::<Language<S>>().strings().clone()
    }

    #[test]
    fn the_application_shows_the_default_language() {
        let mut h = Harness::new(1);
        h.plugin(L10nPlugin::<Strings>::new("en").persist(false))
            .expect("install");

        assert_eq!(shown::<Strings>(&h), Strings("en".into()));
    }

    #[test]
    fn a_language_is_switched_by_its_tag() {
        let mut h = Harness::new(1);
        h.plugin(L10nPlugin::<Listed>::new("en").persist(false))
            .expect("install");

        h.act::<Language<Listed>>(SwitchLanguage("ru".into())).settle();
        assert_eq!(shown::<Listed>(&h), Listed("ru".into()));
    }

    #[test]
    fn a_tag_the_application_has_no_strings_for_changes_nothing() {
        let mut h = Harness::new(1);
        h.plugin(L10nPlugin::<Listed>::new("en").persist(false))
            .expect("install");

        h.act::<Language<Listed>>(SwitchLanguage("de".into())).settle();
        assert_eq!(shown::<Listed>(&h), Listed("en".into()));
    }

    #[test]
    fn a_locale_counts_only_when_the_application_has_it() {
        #[derive(Clone, Default)]
        struct Listed;

        impl Localization for Listed {
            fn for_tag(_: &str) -> Option<Self> {
                Some(Self)
            }

            fn tag(&self) -> String {
                String::new()
            }

            fn languages() -> &'static [&'static str] {
                &["en", "ru"]
            }
        }

        assert!(has_locale::<Listed>("ru"));
        assert!(has_locale::<Listed>("EN"));
        assert!(
            !has_locale::<Listed>("de"),
            "a tag that parses is not enough"
        );
        assert!(
            has_locale::<Strings>("de"),
            "an unlisted resolver is taken at its word"
        );
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
        use guinea_plugin_store::amethystate::store::builder::Backend;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("store");

        {
            let mut h = Harness::new(1);
            h.plugin(StorePlugin::at(&path).backend(Backend::Json)).expect("store");
            h.plugin(L10nPlugin::<Listed>::new("en")).expect("l10n");
            assert_eq!(shown::<Listed>(&h), Listed("en".into()));

            h.act::<Language<Listed>>(SwitchLanguage("ru".into())).settle();
        }

        let mut h = Harness::new(1);
        h.plugin(StorePlugin::at(&path).backend(Backend::Json)).expect("store");
        h.plugin(L10nPlugin::<Listed>::new("en")).expect("l10n");
        assert_eq!(shown::<Listed>(&h), Listed("ru".into()), "the switch was saved and restored");
    }
}
