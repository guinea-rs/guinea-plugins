![guinea-plugins](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-plugins.png)

Plugins for [guinea](https://github.com/guinea-rs/guinea) 0.31 applications.

| Crate | Purpose |
|---|---|
| [guinea-plugin-store](plugins/store) | Persistent storage, migrations and optional in-memory fallback |
| [guinea-plugin-l10n](plugins/l10n) | Typed localisation and an explicitly remembered language |
| [guinea-plugin-l10n-build](plugins/l10n-build) | Fluent accessors generated in build.rs |
| [guinea-plugin-single-instance](plugins/single-instance) | One running copy per application identifier and user |
| [guinea-plugin-window-state](plugins/window-state) | Window geometry persisted by label |
| [guinea-plugin-devtools](plugins/devtools) | Connection to devtools |
| [guinea-devtools-protocol](plugins/devtools-protocol) | Messages shared by applications and devtools |

[guinea-widgets](widgets) contains tables, charts and drag handles.
[guinea-devtools](tools/devtools) is the inspector application, HTTP API and MCP server.

## Application wiring

```rust
use guinea::app::FeatureBuilder;
use guinea_plugin_store::{StoreConfig, StorePlugin};
use guinea_plugin_l10n::{L10nConfig, L10nPlugin};

fn app(app: &mut FeatureBuilder) -> anyhow::Result<guinea::winui::Window> {
    let StorePlugin(store, persistence) =
        app.plugin::<StorePlugin>(StoreConfig::for_app("my-app", "settings").or_in_memory())?;
    app.plugin::<L10nPlugin<L10n>>(L10nConfig::new("en").remembered_in(store.clone()))?;
    app.provide(store);
    if let Some(persistence) = persistence {
        app.provide(persistence);
    }
    Ok(guinea::winui::Window::of::<Shell>().title("My app"))
}
```

The application selects the instances and the order. Every call builds a new plugin.
A returned service is not provided automatically: pass it in another plugin's
configuration, or call `app.provide` so feature installation can inject it.
Reducer exports, such as `Language<L10n>`, are published by the plugin declaration.

Configuration is separate from the installed value: `StoreConfig`, `L10nConfig`,
`SingleInstanceConfig`, `WindowStateConfig` and `DevToolsConfig`.
Plugin resources belong to the application scope; dropping a returned value does not
uninstall them. Dropping the application tears down its resources and runs cleanup.

## Writing a plugin

```rust
use guinea::prelude::*;

plugin! {
    pub MyPlugin {
        provides { MyService }
    }
}

#[builds]
fn my_plugin(app: &mut PluginBuilder, config: MyConfig) -> anyhow::Result<MyPlugin> {
    let service = MyService::open(config)?;
    let closing = service.clone();
    app.on_cleanup(move || closing.close());
    Ok(MyPlugin(service))
}
```

Validate configuration before creating resources. A failed installation does not
undo external side effects or release resources already registered in the application
scope. Propagating the error out of the application's startup drops that scope.

Test installation without a window using guinea's `test-utils` feature:

```rust
let mut app = guinea::app::TestApp::new();
let StorePlugin(store, _) = app.plugin::<StorePlugin>(StoreConfig::in_memory())?;
drop(app);
```

## Compatibility

Dependencies use guinea 0.31.1 or a compatible patch release. The WinUI stack is
`windows-reactor-pre` 0.105.0 and `windows-canvas-pre` 0.105.0,
with `windows-window-pre` 0.104.0.
Consumers must use the same guinea and reactor types; depending on an older
plugin tag alongside guinea 0.31 creates incompatible copies.

The store re-exports amethystate 0.28 and has its own package version, 0.28.0.
The other workspace packages use version 0.31.0. Plugins are distributed by
repository git tags, not through crates.io:

```toml
guinea-plugin-store = { git = "https://github.com/guinea-rs/guinea-plugins", tag = "v0.31.0" }
```

The older `v0.30.2` tag targets guinea 0.29 and must not be used for the API
shown here. The devtools JSON protocol is 3.9; rebuild the inspector to display
the source and actor trace points introduced with this migration.

## Checks

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo test --manifest-path tools/devtools/Cargo.toml --workspace --locked
```
