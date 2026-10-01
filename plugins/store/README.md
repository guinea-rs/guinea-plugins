# guinea-plugin-store

Persistent key-value storage for guinea applications, backed by
[amethystate](https://crates.io/crates/amethystate).

```rust
guinea::app::App::new()
    .plugin(guinea_plugin_store::StorePlugin::for_app("my-app", "settings"))
```

The plugin resolves the platform's configuration directory, runs pending
migrations - amethystate logs their report, schema drift included, through
`tracing` - and provides the store as a service:

```rust
use guinea_plugin_store::{Store, amethystate::Store as _};

let store = app.require::<Store>()?;
let launches: u64 = store.get("app.launches")?.unwrap_or(0) + 1;
store.set("app.launches", &launches)?;
```

amethystate is re-exported, so an application needs no dependency on it of its
own.

View code that already talks to amethystate directly keeps working -
`amethystate::global_store()` returns the same store the plugin initialised.

## Configuration

| Constructor | Location |
|---|---|
| `StorePlugin::for_app(app, config)` | platform configuration directory |
| `StorePlugin::at(path)` | explicit path |
| `StorePlugin::in_memory()` | no file: gone at shutdown, for tests |
| `StorePlugin::with(\|\| ..)` | a `StoreBuilder` you open yourself |

`migrations(|m| ..)` adds steps written by hand, which run with the
`#[migrate]` ones when the store opens; a step that fails refuses the open.
`configure(|builder| ..)` takes any other `StoreBuilder` setting, `backend(..)`
picks the engine. The store is closed, and so written out, on shutdown.

## Caveats

amethystate's global store can only be initialised once per process, so this
plugin must be the only caller of `amethystate::init_global`. Since plugin
installation is idempotent per `Plugin::ID`, listing it more than once is
harmless - calling `init_global` yourself as well is not.
