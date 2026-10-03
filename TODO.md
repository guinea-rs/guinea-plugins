# TODO

## Publishing to crates.io

guinea is on crates.io since 0.13.4, and with WinUI since 0.16.0: the
windows-rs reactor and canvas it needs are published from guinea as
`windows-reactor-pre` and `windows-canvas-pre`. crates.io takes no git
dependency, not even an optional one.

What can go now, in this order:

1. `guinea-plugin-l10n-build`, `guinea-plugin-single-instance` - nothing from
   git.
2. `guinea-plugin-store` - if the amethystate it needs is on crates.io.
3. `guinea-plugin-window-state` - after the store.

4. `guinea-plugin-l10n`, `guinea-widgets` - on the `-pre` crates.

What cannot yet:

- `guinea-devtools-protocol`, `guinea-plugin-devtools`: ogurpchik comes from
  git; it has to be published first.

To do it:

- The guinea pins become `version = "0.13"` with `default-features = false`
  where only the agnostic part is wanted (the published facade's `default` is
  `own-runtime` alone); every `path` dependency gets a `version` beside it.
- A workflow like guinea's `.github/workflows/publish.yml`: in order, with
  retries and a wait on 429. crates.io takes five new crates at once, then one
  new crate every ten minutes.
- The token is the user's, in the repository's secrets; the user starts the
  first run.

## A dragged header and a reorder nobody took

Since windows-reactor preview 3 (guinea 0.22, v0.20.0) `Callback::call`
returns `()`; only `RoutedCallback::call` still says whether it was
delivered. The table moved a dragged header's anchor only when the reorder
was taken, and now moves it regardless: when the segment that drew the
table is no longer publishing, the header drifts away from its column.
Rare, and upstream's to fix. guinea does not call `Callback::call`, so it is
not touched.

A draft for microsoft/windows-rs, for the user to post if it is worth it:

> **windows-reactor: `Callback::call` no longer reports whether the call was
> delivered**
>
> In preview 3, `Callback<T>::call` returns `()`; before, it returned `bool`,
> false when the receiver was gone (a component's `LocalSender` closed, or
> its queue full). Only `RoutedCallback::call` still returns `bool`.
>
> A widget that takes a `Callback` from its caller used that result to keep
> its own state in step with the owner's. Our table's header drag moves its
> anchor only when the reorder was accepted. Otherwise the header drifts
> away from its column while the owner never applied the move. Now the
> widget has to assume every call was delivered.
>
> Could `Callback::call` return `bool` again, or could a `Callback` built
> from a component context expose whether it delivered (`try_call`, say)?
