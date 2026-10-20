use crate::Store;
use amethystate::Open;
use amethystate::store::OpenStruct;

/// Typed settings opened over an explicitly selected store.
pub trait StoreAccess {
    /// The store used by this extension.
    fn store(&self) -> &Store;

    /// Opens settings, panicking with the opening error if they cannot be read.
    #[track_caller]
    fn settings<S: Open>(&self) -> S {
        S::new_with(self.store())
    }

    /// Opens settings and returns any schema or reading error.
    fn try_settings<S: Open>(&self) -> Result<S, OpenStruct> {
        S::try_new_with(self.store())
    }
}

impl StoreAccess for Store {
    fn store(&self) -> &Store {
        self
    }
}
