# TODO

## Publishing to crates.io

guinea is on crates.io since 0.13.4, without WinUI: `guinea-winui` waits for
windows-rs to release a `windows-reactor` with `AppProxy`, `open_window` and
`proxy()`, and crates.io takes no git dependency, not even an optional one.

What can go now, in this order:

1. `guinea-plugin-l10n-build`, `guinea-plugin-single-instance` - nothing from
   git.
2. `guinea-plugin-store` - if the amethystate it needs is on crates.io.
3. `guinea-plugin-window-state` - after the store.

What cannot yet:

- `guinea-plugin-l10n`: its `winui` feature pulls `windows-reactor` from git.
  Either the feature moves into a crate of its own, or it waits for
  windows-rs.
- `guinea-devtools-protocol`, `guinea-plugin-devtools`: ogurpchik comes from
  git; it has to be published first.
- `guinea-widgets`: windows-rs from git.

To do it:

- The guinea pins become `version = "0.13"` with `default-features = false`
  where only the agnostic part is wanted (the published facade's `default` is
  `own-runtime` alone); every `path` dependency gets a `version` beside it.
- A workflow like guinea's `.github/workflows/publish.yml`: in order, with
  retries and a wait on 429. crates.io takes five new crates at once, then one
  new crate every ten minutes.
- The token is the user's, in the repository's secrets; the user starts the
  first run.
