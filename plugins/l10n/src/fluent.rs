/// Re-exported so an application needs no Fluent dependencies of its own.
///
/// `static_loader!` and the generated accessors refer to `fluent_templates`
/// and `fluent_bundle` by bare crate name, which is why [`fluent_loader!`]
/// brings both into scope rather than spelling out paths.
#[doc(hidden)]
pub mod __deps {
    pub use fluent_bundle;
    pub use fluent_templates;
    pub use unic_langid;
}

/// Declares the application's Fluent resolver: a `L10n` type with one typed
/// accessor per message, built from the `.ftl` files that
/// `guinea-plugin-l10n-build` compiled in `build.rs`.
///
/// ```ignore
/// guinea_plugin_l10n::fluent_loader! {
///     locales: "./locales",
///     fallback_language: "en",
/// }
/// ```
#[macro_export]
macro_rules! fluent_loader {
    (locales: $locales:literal, fallback_language: $fallback:literal $(,)?) => {
        use $crate::fluent::__deps::{fluent_bundle, fluent_templates};

        fluent_templates::static_loader! {
            static LOCALES = {
                locales: $locales,
                fallback_language: $fallback,
            };
        }

        #[derive(Default)]
        struct Args(::std::collections::HashMap<::std::borrow::Cow<'static, str>, fluent_bundle::FluentValue<'static>>);

        impl Args {
            fn new() -> Self {
                Self::default()
            }

            fn set(&mut self, key: &'static str, value: fluent_bundle::FluentValue<'static>) {
                self.0.insert(::std::borrow::Cow::Borrowed(key), value);
            }
        }

        #[derive(Clone, PartialEq, Default)]
        pub struct L10n($crate::fluent::__deps::unic_langid::LanguageIdentifier);

        impl L10n {
            pub fn new(lang: $crate::fluent::__deps::unic_langid::LanguageIdentifier) -> Self {
                Self(lang)
            }

            pub fn language(&self) -> &$crate::fluent::__deps::unic_langid::LanguageIdentifier {
                &self.0
            }

            /// The strings the application is showing right now.
            pub fn current() -> Self {
                $crate::L10n::<Self>::current()
            }

            fn get_raw(&self, id: &str, args: &Args) -> String {
                use fluent_templates::Loader;
                LOCALES.lookup_with_args(&self.0, id, &args.0)
            }
        }

        include!(concat!(env!("OUT_DIR"), "/l10n_keys.rs"));

        impl $crate::Localization for L10n {
            fn for_tag(tag: &str) -> ::std::option::Option<Self> {
                tag.parse::<$crate::fluent::__deps::unic_langid::LanguageIdentifier>()
                    .ok()
                    .map(Self::new)
            }

            fn tag(&self) -> ::std::string::String {
                self.0.to_string()
            }

            fn keys() -> &'static [$crate::Key] {
                L10N_KEYS
            }

            fn value(&self, id: &str) -> ::std::option::Option<::std::string::String> {
                ::std::option::Option::Some(self.get_raw(id, &Args::new()))
            }
        }

        include!(concat!(env!("OUT_DIR"), "/l10n_accessors.rs"));
    };
}
