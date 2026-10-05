use std::sync::Arc;

use amethystate::Open;
use amethystate::store::OpenStruct;
use guinea::Services;

use crate::Store;

/// Structs with a place in the store, opened over the one [`StorePlugin`]
/// installed, from wherever a service can be required.
///
/// [`StorePlugin`]: crate::StorePlugin
pub trait StoreAccess {
    /// The store, if [`StorePlugin`](crate::StorePlugin) was installed.
    fn store(&self) -> Option<Arc<Store>>;

    /// `S` opened over the store.
    ///
    /// # Panics
    ///
    /// When [`StorePlugin`](crate::StorePlugin) was not installed: that is
    /// how the application was put together, not something to recover from.
    /// And where `S` will not open, with what it said - [`Self::try_settings`]
    /// returns that instead.
    #[track_caller]
    fn settings<S: Open>(&self) -> S {
        S::new_with(&self.installed::<S>())
    }

    /// `S` opened over the store, or what stopped it opening.
    ///
    /// # Panics
    ///
    /// When [`StorePlugin`](crate::StorePlugin) was not installed.
    #[track_caller]
    fn try_settings<S: Open>(&self) -> Result<S, OpenStruct> {
        S::try_new_with(&self.installed::<S>())
    }

    /// The store `S` is kept in.
    #[track_caller]
    fn installed<S>(&self) -> Arc<Store> {
        self.store().unwrap_or_else(|| {
            panic!(
                "{} is kept in the store, and no store was installed - install StorePlugin",
                std::any::type_name::<S>()
            )
        })
    }
}

impl<C: Services + ?Sized> StoreAccess for C {
    fn store(&self) -> Option<Arc<Store>> {
        self.try_require::<Store>()
    }
}
