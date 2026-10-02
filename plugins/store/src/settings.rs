use std::sync::Arc;

use amethystate::Open;
use amethystate::store::OpenStruct;
use guinea::app::PluginBuilder;
use guinea::feature::FeatureInitContext;

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
    fn try_settings<S: Open>(&self) -> Result<S, OpenStruct> {
        let Some(store) = self.store() else {
            panic!(
                "{} is kept in the store, and no store was installed - install StorePlugin",
                std::any::type_name::<S>()
            );
        };
        S::new_with(&store)
    }

    /// `S` opened over the store.
    ///
    /// # Panics
    ///
    /// When [`StorePlugin`](crate::StorePlugin) was not installed, and when
    /// `S` will not open - see [`try_settings`](Self::try_settings) for a
    /// struct whose open can be refused.
    fn settings<S: Open>(&self) -> S {
        self.try_settings::<S>()
            .unwrap_or_else(|refused| panic!("{}: {refused}", std::any::type_name::<S>()))
    }
}

impl StoreAccess for FeatureInitContext {
    fn store(&self) -> Option<Arc<Store>> {
        self.try_require::<Store>()
    }
}

impl StoreAccess for PluginBuilder {
    fn store(&self) -> Option<Arc<Store>> {
        self.try_require::<Store>()
    }
}

#[cfg(feature = "test-utils")]
impl StoreAccess for guinea::app::Segment<'_> {
    fn store(&self) -> Option<Arc<Store>> {
        self.try_require::<Store>()
    }
}
