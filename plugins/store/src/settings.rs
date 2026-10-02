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
    fn settings<S: Open>(&self) -> Result<S, OpenStruct> {
        let Some(store) = self.store() else {
            panic!(
                "{} is kept in the store, and no store was installed - install StorePlugin",
                std::any::type_name::<S>()
            );
        };
        S::new_with(&store)
    }
}

impl<C: Services + ?Sized> StoreAccess for C {
    fn store(&self) -> Option<Arc<Store>> {
        self.try_require::<Store>()
    }
}
