# guinea plugins

Plugins for [guinea](https://github.com/uniproc-dev/guinea) applications.

A plugin closes a need a typical desktop application has but that does not
belong in the framework: persistence, localisation, single instance, an
updater, a tray icon. It knows guinea; guinea does not know it.

| Plugin | Crate | What it does |
|---|---|---|
| Store | `guinea-plugin-store` | Persistent key-value storage backed by [amethystate](https://crates.io/crates/amethystate), with migrations |
| L10n | `guinea-plugin-l10n` | Loads the application's strings at startup, and remembers the chosen language |

## Usage

```rust
use guinea_plugin_store::StorePlugin;

guinea::app::App::new()
    .plugin(StorePlugin::for_app("my-app", "settings"))
    .plugin(L10nPlugin::<L10n>::new("en"))
    .feature(Startup)
    .run(window, RouterRoot::at(initial_route()));
```

Installation is idempotent and keyed by `Plugin::ID`, so a feature can install
the plugins it depends on itself, and listing the same plugin twice is a no-op
rather than a conflict.

Order matters only where one plugin reads what another provides - the l10n
plugin above restores the saved language if a store is already installed, and
runs without persistence if it is not.

## Writing a plugin

```rust
use guinea::app::{Plugin, PluginBuilder};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    const ID: &'static str = "vendor.my-plugin";

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        let store = app.require::<guinea_plugin_store::Store>()?;
        app.provide(MyService::new(store));
        app.on_cleanup(|_| Ok(()));
        Ok(())
    }
}
```

`PluginBuilder` is deliberately narrow - services in and out (`provide` /
`require`), actors (`spawn`), timers (`every` / `repeat`), global-bus
subscriptions, and cleanups. Anything a plugin registers is tracked and torn
down for it; subscription ids are never handed out, so an unsubscribe cannot
be forgotten.

Test a plugin with `guinea`'s `test-utils` feature, which runs installation
without a window:

```rust
let mut app = guinea::app::TestApp::new();
app.install(MyPlugin).unwrap();
assert!(app.shutdown().is_empty());
```

## Layout

Each plugin is its own crate under `plugins/`, versioned and released
independently.

guinea and the WinUI stack come from crates.io: `guinea` 0.16, and the
windows-rs reactor and canvas guinea is written against as
`windows-reactor-pre` and `windows-canvas-pre`, under their usual lib names.
An application must take guinea and the `-pre` crates from the same source as
the plugins, otherwise cargo builds two copies and the `PluginBuilder` a plugin
expects is a different type from the one the application has.

Working on a plugin and on guinea at the same time is a `[patch]` pointing at
your guinea checkout:

```toml
[patch.crates-io]
guinea = { path = "../guinea/crates/guinea" }
guinea-core = { path = "../guinea/crates/guinea-core" }
guinea-mark = { path = "../guinea/crates/guinea-mark" }
windows-reactor-pre = { path = "../guinea/crates/vendor/windows-reactor-pre" }
windows-canvas-pre = { path = "../guinea/crates/vendor/windows-canvas-pre" }
```
