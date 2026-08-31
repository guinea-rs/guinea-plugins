use guinea::winui::{Refreshable, Refresher};
use windows_reactor::ViewContext;

use crate::store::L10n;

/// Reads the current strings, and asks this segment to draw again whenever the
/// language changes.
///
/// The strings are read fresh rather than mirrored into the component: they
/// live process-wide and change from under the view, so a copy held here would
/// be one more thing to keep in step. The subscription only says *when*.
///
/// [`Refreshable`] is every segment of a guinea route tree - pages and layouts
/// both are - so this reads as `use_l10n(cx)` from inside any `view`, and the
/// plugin never names the node type or its messages.
pub fn use_l10n<S, C>(cx: &mut ViewContext<C>) -> S
where
    S: Clone + PartialEq + Default + 'static,
    C: Refreshable,
{
    let refresher = Refresher::of(cx);

    cx.use_effect("guinea::l10n", (), move || {
        let subscription = L10n::<S>::subscribe(move |_| refresher.refresh());
        // Ends with this component instance: one that had been unmounted and
        // kept listening would be asking a slot nobody renders to publish.
        Some(Box::new(move || drop(subscription)) as Box<dyn FnOnce()>)
    });

    L10n::<S>::current()
}
