//! Localisation for guinea applications: the language the application shows,
//! the Fluent resolver, and the plugin that loads it at startup.
//!
//! ```no_run
//! use guinea_plugin_l10n::{L10nConfig, L10nPlugin};
//! # #[derive(Clone, Default)] struct Strings;
//! # impl guinea_plugin_l10n::Localization for Strings {
//! #     fn for_tag(_: &str) -> Option<Self> { Some(Self) }
//! #     fn tag(&self) -> String { "en".into() }
//! # }
//! fn app(app: &mut guinea::app::FeatureBuilder) -> anyhow::Result<()> {
//!     app.plugin::<L10nPlugin<Strings>>(L10nConfig::new("en"))?;
//!     Ok(())
//! }
//! ```
//!
//! The plugin claims [`Language`] for the application and exports it, so
//! every page of every window reads it - `cx.l10n::<L10n>()` through
//! [`L10nAccess`] on WinUI, eframe, ratatui and iced - and is drawn again
//! when it changes. A switch is an action on it, [`SwitchLanguage`] with a
//! tag. With the `persist` feature the plugin remembers the choice.

mod devtools;
#[cfg(feature = "fluent")]
pub mod fluent;
mod store;

pub use store::{
    Key, L10nAccess, L10nSetup, Language, LanguageSwitch, Localization, SwitchLanguage,
};

use guinea::app::PluginBuilder;

#[cfg(feature = "persist")]
const KEY: [&str; 2] = ["app", "language"];

/// Loads the application's strings before the first render.
pub struct L10nConfig {
    default_tag: String,
    #[cfg(feature = "persist")]
    store: Option<guinea_plugin_store::Store>,
}

impl L10nConfig {
    /// `default_tag` is a BCP-47 language tag - `"en"`, `"ru"`, `"pt-BR"`.
    pub fn new(default_tag: impl Into<String>) -> Self {
        Self {
            default_tag: default_tag.into(),
            #[cfg(feature = "persist")]
            store: None,
        }
    }

    /// Restores the saved language and remembers later switches in this store.
    /// Without an explicitly selected store, language changes are not persisted.
    #[cfg(feature = "persist")]
    pub fn remembered_in(mut self, store: guinea_plugin_store::Store) -> Self {
        self.store = Some(store);
        self
    }
}

guinea::plugin! {
    pub L10nPlugin<S: Localization> {
        exports { Language<S> }
    }
}

#[guinea::builds]
fn l10n<S: Localization>(
    app: &mut PluginBuilder,
    config: L10nConfig,
) -> anyhow::Result<L10nPlugin<S>> {
    let tag = saved_tag(&config).unwrap_or_else(|| config.default_tag.clone());

    let strings = S::for_tag(&tag)
        .filter(|strings| has_locale::<S>(&strings.tag()))
        .or_else(|| {
            tracing::warn!(%tag, "no strings for this language, falling back");
            S::for_tag(&config.default_tag)
        })
        .ok_or_else(|| anyhow::anyhow!("`{}` is not a language tag", config.default_tag))?;
    if !has_locale::<S>(&strings.tag()) {
        anyhow::bail!(
            "the default language `{}` has no strings in this application",
            config.default_tag
        );
    }

    let language = app
        .state::<Language<S>>()
        .seed(Language::new(strings))
        .plain();
    app.answers(move |SwitchLanguage(tag)| {
        match S::for_tag(&tag).filter(|strings| has_locale::<S>(&strings.tag())) {
            Some(strings) => language.push(strings),
            None => tracing::warn!(%tag, "no strings for this language, it stays"),
        }
    });

    #[cfg(feature = "persist")]
    if let Some(store) = config.store {
        remember_changes::<S>(app, store);
    }

    let panel = devtools::watch::<S>((**app).clone());
    app.on_cleanup(move || {
        drop(panel);
        Ok(())
    });

    Ok(L10nPlugin(language))
}

/// Whether the application has strings for `tag`. A resolver that does not
/// list its locales is taken at its word for any tag it accepts. A locale
/// directory may separate the subtags with `_`, as a language id parses it.
fn has_locale<S: Localization>(tag: &str) -> bool {
    let subtag = |byte: u8| match byte {
        b'_' => b'-',
        other => other.to_ascii_lowercase(),
    };
    let languages = S::languages();
    languages.is_empty()
        || languages.iter().any(|known| {
            known.len() == tag.len()
                && known
                    .bytes()
                    .zip(tag.bytes())
                    .all(|(known, asked)| subtag(known) == subtag(asked))
        })
}

#[cfg(feature = "persist")]
fn saved_tag(config: &L10nConfig) -> Option<String> {
    let store = config.store.as_ref()?;
    match store.get::<String>(KEY) {
        Ok(tag) => tag,
        Err(e) => {
            tracing::warn!(error = %e, "could not read the saved language");
            None
        }
    }
}

#[cfg(not(feature = "persist"))]
fn saved_tag(_config: &L10nConfig) -> Option<String> {
    None
}

#[cfg(feature = "persist")]
fn remember_changes<S: Localization>(app: &PluginBuilder, store: guinea_plugin_store::Store) {
    app.observe::<Language<S>>(move |strings: &S| {
        if let Err(e) = store.set(KEY, &strings.tag()) {
            tracing::warn!(error = %e, "could not save the language");
        }
    });
}

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
        h.plugin::<L10nPlugin<Strings>>(L10nConfig::new("en"))
            .expect("install");

        assert_eq!(shown::<Strings>(&h), Strings("en".into()));
    }

    #[test]
    fn a_language_is_switched_by_its_tag() {
        let mut h = Harness::new(1);
        h.plugin::<L10nPlugin<Listed>>(L10nConfig::new("en"))
            .expect("install");

        h.act::<Language<Listed>>(SwitchLanguage("ru".into()))
            .settle();
        assert_eq!(shown::<Listed>(&h), Listed("ru".into()));
    }

    #[test]
    fn a_feature_reads_the_language_where_it_is_installed() {
        let mut h = Harness::new(1);
        h.plugin::<L10nPlugin<Listed>>(L10nConfig::new("en"))
            .expect("install");
        h.act::<Language<Listed>>(SwitchLanguage("ru".into()))
            .settle();

        let segment = h.segment();
        assert_eq!(segment.context().l10n::<Listed>(), Listed("ru".into()));
    }

    #[test]
    fn reading_the_language_without_the_plugin_says_what_is_missing() {
        let h = Harness::new(1);
        let segment = h.segment();

        let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            segment.context().l10n::<Listed>()
        }));
        let Err(panicked) = read else {
            panic!("a language read with no L10nPlugin installed");
        };
        let said = panicked
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panicked.downcast_ref::<&str>().map(|said| said.to_string()))
            .unwrap_or_default();
        assert!(said.contains("L10nPlugin"), "{said}");
    }

    #[test]
    fn a_tag_the_application_has_no_strings_for_changes_nothing() {
        let mut h = Harness::new(1);
        h.plugin::<L10nPlugin<Listed>>(L10nConfig::new("en"))
            .expect("install");

        h.act::<Language<Listed>>(SwitchLanguage("de".into()))
            .settle();
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
    fn a_locale_named_with_an_underscore_is_the_tag_named_with_a_hyphen() {
        #[derive(Clone, Default)]
        struct Underscored;

        impl Localization for Underscored {
            fn for_tag(_: &str) -> Option<Self> {
                Some(Self)
            }

            fn tag(&self) -> String {
                String::new()
            }

            fn languages() -> &'static [&'static str] {
                &["en", "pt_BR"]
            }
        }

        assert_eq!(
            [
                has_locale::<Underscored>("pt-BR"),
                has_locale::<Underscored>("pt_br"),
                has_locale::<Underscored>("pt"),
            ],
            [true, true, false]
        );
    }

    #[test]
    fn a_default_tag_the_application_has_no_strings_for_is_an_installation_error() {
        let mut app = TestApp::new();

        let installed = app.plugin::<L10nPlugin<Listed>>(L10nConfig::new("de"));

        assert!(installed.is_err());
    }

    #[test]
    fn falls_back_when_the_saved_tag_is_unusable() {
        let mut app = TestApp::new();
        app.plugin::<L10nPlugin<Strings>>(L10nConfig::new("!broken"))
            .map(|_| ())
            .expect_err("a default tag that cannot be resolved is an error");
    }

    #[cfg(feature = "persist")]
    #[test]
    fn restores_and_saves_the_language_through_the_store() {
        use guinea_plugin_store::amethystate::store::builder::Backend;
        use guinea_plugin_store::{StoreConfig, StorePlugin};

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("store");

        {
            let mut h = Harness::new(1);
            let StorePlugin(store, _) = h
                .plugin::<StorePlugin>(StoreConfig::at(&path).backend(Backend::Json))
                .expect("store");
            h.plugin::<L10nPlugin<Listed>>(L10nConfig::new("en").remembered_in(store))
                .expect("l10n");
            assert_eq!(shown::<Listed>(&h), Listed("en".into()));

            h.act::<Language<Listed>>(SwitchLanguage("ru".into()))
                .settle();
        }

        let mut h = Harness::new(1);
        let StorePlugin(store, _) = h
            .plugin::<StorePlugin>(StoreConfig::at(&path).backend(Backend::Json))
            .expect("store");
        h.plugin::<L10nPlugin<Listed>>(L10nConfig::new("en").remembered_in(store))
            .expect("l10n");
        assert_eq!(
            shown::<Listed>(&h),
            Listed("ru".into()),
            "the switch was saved and restored"
        );
    }
}
