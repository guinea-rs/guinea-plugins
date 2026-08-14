use crate::store::L10n;

/// Reads the current strings and re-renders this component whenever the
/// language changes.
pub fn use_l10n<S>(cx: &mut windows_reactor::RenderCx) -> S
where
    S: Clone + PartialEq + Default + 'static,
{
    let (current, set_current) = cx.use_state(L10n::<S>::current());

    cx.use_effect_with_cleanup((), move || {
        let set_current = set_current.clone();
        let subscription = L10n::<S>::subscribe(move |latest| set_current.call(latest));
        Some(move || drop(subscription))
    });

    current
}
