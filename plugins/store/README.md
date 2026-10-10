![guinea-plugin-store](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-plugin-store.png)

Persistent key-value storage for guinea applications, backed by
[amethystate](https://crates.io/crates/amethystate).

```rust
guinea::app::GuineaApp::new()
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

`keeping(guard)` holds anything until the store has closed, and drops it
right after. A test whose store lives in a temporary directory hands it the
`TempDir`, so the directory is removed after the store wrote itself out
rather than before, when the write would bring it back:

```rust
let dir = tempfile::tempdir()?;
h.plugin(StorePlugin::at(dir.path().join("settings")).backend(Backend::Json).keeping(dir))?;
```

`or_in_memory()` starts the application anyway where the file will not open -
it will not read, another process holds it, a migration fails: on an empty
store in memory that writes nothing and leaves the file as it was. Why is
logged, and `Persistence` is provided so the application can tell the user.
