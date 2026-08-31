use std::marker::PhantomData;

use guinea_core::scope::{GlobalScope, Reducer, Subscription};

/// The strings themselves, as a reducer - the state is the reducer now, so
/// this is a newtype around `S` rather than a marker beside it.
#[derive(Clone, Default)]
struct Strings<S>(S);

impl<S: Clone + Default + 'static> Reducer for Strings<S> {
    type Update = S;

    fn reduce(&mut self, next: S) {
        self.0 = next;
    }
}

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
}

/// The application's current strings, process-wide.
///
/// Backed by the global scope, so every window sees the same language and
/// every subscriber is told when it changes.
pub struct L10n<S>(PhantomData<S>);

impl<S: Clone + Default + 'static> L10n<S> {
    pub fn load(strings: S) {
        GlobalScope::instance().push::<Strings<S>>(strings);
    }

    pub fn current() -> S {
        GlobalScope::instance().binding::<Strings<S>>().get().0
    }

    pub fn subscribe(callback: impl Fn(S) + 'static) -> Subscription {
        GlobalScope::instance()
            .binding::<Strings<S>>()
            .on_change(move |strings| callback(strings.0.clone()))
    }
}
