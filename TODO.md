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
