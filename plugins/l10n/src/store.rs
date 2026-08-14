use std::marker::PhantomData;

use guinea_core::scope::{GlobalScope, NoopActions, Reducer, Subscription};

struct Marker<S>(PhantomData<S>);

impl<S: Clone + Default + 'static> Reducer for Marker<S> {
    type State = S;
    type Push = S;
    type Group = ();
    type Actions = NoopActions;

    fn reduce(state: &mut Self::State, msg: Self::Push) {
        *state = msg;
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
        GlobalScope::instance().push::<Marker<S>>(strings);
    }

    pub fn current() -> S {
        GlobalScope::instance().state::<Marker<S>>().borrow().clone()
    }

    pub fn subscribe(callback: impl Fn(S) + 'static) -> Subscription {
        let scope = GlobalScope::instance();
        let scope_for_cb = scope.clone();
        scope.subscribe::<Marker<S>>(move || {
            callback(scope_for_cb.state::<Marker<S>>().borrow().clone());
        })
    }
}
