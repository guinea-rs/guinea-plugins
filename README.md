# guinea plugins

Plugins for [guinea](https://github.com/uniproc-dev/guinea) applications.

A plugin closes a need a typical desktop application has but that does not
belong in the framework: persistence, localisation, single instance, an
updater, a tray icon. It knows guinea; guinea does not know it.

| Plugin | Crate | What it does |
|---|---|---|
| Store | `guinea-plugin-store` | Persistent key-value storage backed by [amethystate](https://crates.io/crates/amethystate), with migrations |

## Usage

```rust
use guinea_plugin_store::StorePlugin;

guinea::app::App::new()
    .plugin(StorePlugin::for_app("my-app", "settings"))
    .feature(Startup)
    .run(window, RouterRoot::at(initial_route()));
```

Installation is idempotent and keyed by `Plugin::ID`, so a feature can install
the plugins it depends on itself, and listing the same plugin twice is a no-op
rather than a conflict.

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
`require`), actors (`spawn`), timers (`spawn_heartbeat`), global-bus
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

`guinea` is pinned to a rev rather than a version, since the plugin API is
still moving. An application that also depends on guinea directly must resolve
both to the same source, otherwise cargo builds two copies of guinea and the
`PluginBuilder` a plugin expects is a different type from the one the
application has:

```toml
[patch."https://github.com/uniproc-dev/guinea"]
guinea = { path = "../guinea/crates/guinea" }
guinea-core = { path = "../guinea/crates/guinea-core" }
```

Working on a plugin and on guinea at the same time is the same `[patch]`,
pointing at your guinea checkout.
