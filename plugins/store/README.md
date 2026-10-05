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

The store is not amethystate's process-wide one: `amethystate::global_store()`
and a struct's `new()` do not reach it. Any number of applications - tests
running side by side, say - each open their own.

A struct with a place in the store opens over it from any context that can
require a service - a feature's, a plugin's, a harness segment's with the
`test-utils` feature:

```rust
use guinea_plugin_store::StoreAccess;

let general = cx.settings::<GeneralSettings>();
let general = cx.try_settings::<GeneralSettings>()?;
```

`settings` panics where the struct will not open, with what stopped it;
`try_settings` returns that as amethystate's `OpenStruct`. Without
`StorePlugin` both panic: that is how the application was put together.

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
