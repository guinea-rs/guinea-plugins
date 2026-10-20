# guinea-plugin-store

Persistent storage for guinea 0.31, backed by amethystate 0.28.0.
The plugin's own version follows amethystate because it re-exports its public types.

## Installation

```toml
[dependencies]
guinea = "0.31.1"
guinea-plugin-store = { git = "https://github.com/guinea-rs/guinea-plugins", tag = "v0.31.0" }
```

Plugins are distributed by repository git tag, not through crates.io. The store
package has version 0.28.0; its version follows amethystate, not the repository's
tag. The older repository tag `v0.30.2` targets guinea 0.29 and does not contain
this API.

## Application setup

```rust
use guinea::app::FeatureBuilder;
use guinea_plugin_store::{StoreConfig, StorePlugin};

fn app(app: &mut FeatureBuilder) -> anyhow::Result<()> {
    let StorePlugin(store, persistence) = app.plugin::<StorePlugin>(
        StoreConfig::for_app("my-app", "settings").or_in_memory()
    )?;
    app.provide(store);
    if let Some(persistence) = persistence {
        app.provide(persistence);
    }
    Ok(())
}
```

Put this setup before constructing the backend's window in a GUI application's
entry function. `StorePlugin` is declared with `guinea::plugin!` and constructed by
a `#[guinea::builds]` function; the application calls `app.plugin`, not that function.

## Store configuration

`StoreConfig` selects where to open the store:

| Constructor | Location |
|---|---|
| `for_app(app, config)` | platform configuration directory |
| `at(path)` | explicit path |
| `in_memory()` | no file |
| `with(|| ..)` | a custom StoreBuilder |

`backend`, `layout` and `configure` set builder options.
Repeated `migrations` calls accumulate migration steps.
Opening failure is an installation error unless `or_in_memory()` permits an empty,
non-persistent fallback. A failure to find a configuration directory or construct a
custom builder still returns an error.

The returned `Option<Arc<Persistence>>` is present when fallback was enabled and describes
the chosen storage. The application decides whether to publish it.
Each plugin call opens an independent store. There is no process-wide store and no
automatic publication.

## Typed settings

Use the re-exported amethystate so settings and the plugin share the same types:

```rust
use guinea_plugin_store::amethystate::amethystate;
use guinea_plugin_store::{Store, StoreAccess};

#[amethystate(prefix = "general")]
pub struct GeneralSettings {
    #[amestate(default = 8080u16)]
    pub port: u16,
}

fn change_port(store: &Store) -> anyhow::Result<()> {
    let settings = store.try_settings::<GeneralSettings>()?;
    settings.port().set(9000);
    Ok(())
}
```

`StoreAccess` extends `Store`, not guinea contexts:
`store.settings::<S>()` opens an amethystate struct over that store;
`store.try_settings::<S>()` returns opening errors instead of panicking.
To use it in a feature, provide the store in application setup and declare it
after the installation context:

```rust
use guinea::app::Install;
use guinea_plugin_store::{Store, StoreAccess};

guinea::feature! { pub Preferences {} }

#[guinea::installs]
fn preferences(_cx: &Install, store: Store) -> anyhow::Result<Preferences> {
    let settings = store.try_settings::<GeneralSettings>()?;
    tracing::debug!(port = settings.port().get(), "settings loaded");
    Ok(Preferences)
}
```

Here `GeneralSettings` is the declaration above. `#[installs]` injects a clone of
the explicitly provided store. Opening settings does not itself subscribe a feature
to changes; use `amethystate-guinea` 0.28 for guinea sources over reactive fields.

If an application also depends directly on amethystate or its guinea adapter, use
the compatible 0.28 line, not 0.25 or 0.27: types from those versions are distinct.

## Shutdown

The store closes when the application is dropped. `keeping(guard)` keeps a resource
until closing finishes; pass a test's TempDir to prevent a final write from recreating
a directory removed too early. External clones do not postpone that explicit close.
