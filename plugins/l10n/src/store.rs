use guinea_core::scope::Reducer;

/// The strings the application shows: claimed by [`crate::L10nPlugin`] for
/// the application, and exported to every window.
#[derive(Clone, Default, PartialEq)]
pub struct Language<S>(S);

impl<S> Language<S> {
    pub(crate) fn new(strings: S) -> Self {
        Self(strings)
    }

    pub fn strings(&self) -> &S {
        &self.0
    }
}

impl<S> std::fmt::Debug for Language<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Language<{}>", std::any::type_name::<S>())
    }
}

impl<S: Clone + Default + 'static> Reducer for Language<S> {
    type Update = S;

    fn reduce(&mut self, next: S) {
        self.0 = next;
    }
}

/// The language from a page or a layout of any backend that reads as it
/// draws.
///
/// The segment reaches the language when the application installs
/// `L10nPlugin<S>`: `app! { App { installs { L10nPlugin<S> } } }`.
/// `let strings: Strings = cx.l10n();`, or `cx.l10n::<Strings>()`.
pub trait L10nAccess: guinea::feature::Reads {
    /// The strings the application shows; the segment is drawn again when
    /// they change.
    fn l10n<S: Localization + PartialEq>(&mut self) -> S {
        let (language, _) = self.read::<Language<S>>();
        language.strings().clone()
    }

    /// What switches the language, to keep in a callback. Taking it reads
    /// nothing: the segment is not drawn again for it.
    fn language_switch<S: Localization>(&self) -> LanguageSwitch {
        LanguageSwitch(self.dispatch::<Language<S>>())
    }
}

impl<C: guinea::feature::Reads> L10nAccess for C {}

/// Switches the language the application shows, by tag.
#[derive(Clone)]
pub struct LanguageSwitch(guinea_core::feature::Dispatch);

impl LanguageSwitch {
    /// A tag the application has no strings for leaves the language as it
    /// was.
    pub fn to(&self, tag: impl Into<String>) {
        self.0.emit(SwitchLanguage(tag.into()));
    }
}

/// The language from where a plugin or a feature is installed: read once,
/// not followed - [`observe`](guinea::feature::ScopeContext::observe)
/// follows it.
pub trait L10nSetup {
    /// The strings the application shows now.
    ///
    /// # Panics
    ///
    /// When [`L10nPlugin`](crate::L10nPlugin) for `S` was not installed:
    /// that is how the application was put together.
    fn l10n<S: Localization>(&self) -> S;
}

impl L10nSetup for guinea::feature::ScopeContext {
    fn l10n<S: Localization>(&self) -> S {
        let Some(language) = self.read::<Language<S>>() else {
            panic!(
                "the language is read here and nothing exports {} - install L10nPlugin",
                std::any::type_name::<Language<S>>()
            );
        };
        language.strings().clone()
    }
}

/// Asks for the strings of a BCP-47 tag. A tag the application has no
/// strings for leaves the language as it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchLanguage(pub String);

/// A set of localised strings identified by, and rebuildable from, a BCP-47
/// language tag.
///
/// Implemented by [`crate::fluent_loader!`]. Code that handles languages
/// without knowing the concrete resolver - persisting the choice, offering a
/// picker - goes through this. Tags rather than
/// `unic_langid::LanguageIdentifier` keep that dependency out of the contract,
/// which is what lets a resolver be built on something other than Fluent.
pub trait Localization: Clone + Default + 'static {
    /// Returns `None` if the tag is malformed.
    fn for_tag(tag: &str) -> Option<Self>;

    fn tag(&self) -> String;

    /// Every message the application has, as the reference locale compiled
    /// them. Empty for a resolver that does not say.
    fn keys() -> &'static [Key] {
        &[]
    }

    /// Every locale the application has, as tags: what a picker offers.
    /// Empty for a resolver that does not say.
    fn languages() -> &'static [&'static str] {
        &[]
    }

    /// What `id` reads as in these strings; `None` for a resolver that cannot
    /// be asked by name.
    fn value(&self, id: &str) -> Option<String> {
        let _ = id;
        None
    }
}

/// One message, as the build wrote it down: devtools show these by the file
/// they live in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    /// `process-killed-toast`.
    pub id: &'static str,
    /// The `.ftl` it is written in, relative to the locale's directory.
    pub file: &'static str,
    /// The line it starts on, counting from one; zero when unknown.
    pub line: u32,
    /// What the reference locale says, as written: `Process { $name } was
    /// killed.`
    pub text: &'static str,
    /// What it interpolates: `["name"]`.
    pub variables: &'static [&'static str],
    /// What every other locale says for it, as written: `[("ru", "Процесс {
    /// $name } завершён.")]`.
    pub translations: &'static [(&'static str, &'static str)],
    /// The locales that have no translation for it.
    pub missing: &'static [&'static str],
}
