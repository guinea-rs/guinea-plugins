//! What a page draws from a session, built when the session changed rather
//! than every frame.

use std::sync::Arc;

/// What `make` built for `key` under `id`; built again only when `key` is
/// not what it was built for last time.
pub fn memo<K, T>(ctx: &egui::Context, id: egui::Id, key: K, make: impl FnOnce() -> T) -> Arc<T>
where
    K: PartialEq + Clone + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    if let Some((built_for, built)) = ctx.data(|data| data.get_temp::<(K, Arc<T>)>(id))
        && built_for == key
    {
        return built;
    }

    let built = Arc::new(make());
    ctx.data_mut(|data| data.insert_temp(id, (key, built.clone())));
    built
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn it_is_built_again_only_for_another_key() {
        let ctx = egui::Context::default();
        let id = egui::Id::new("memo-test");
        let built = Cell::new(0);
        let make = || {
            built.set(built.get() + 1);
            built.get()
        };

        assert_eq!(*memo(&ctx, id, (1u64, 7u64), make), 1);
        assert_eq!(*memo(&ctx, id, (1u64, 7u64), make), 1, "the same key");
        assert_eq!(*memo(&ctx, id, (2u64, 7u64), make), 2, "the session moved");
        assert_eq!(built.get(), 2);
    }
}
