![guinea-plugin-window-state](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-plugin-window-state.png)

Remembers where a guinea application's windows were, and opens them there
again.

```rust
use guinea_plugin_store::{StoreConfig, StorePlugin};
use guinea_plugin_window_state::{WindowStateConfig, WindowStatePlugin};

let StorePlugin(store, _) = app.plugin::<StorePlugin>(StoreConfig::for_app("app", "settings"))?;
let WindowStatePlugin(saved) = app.plugin::<WindowStatePlugin>(WindowStateConfig::new(store))?;
```

It keeps the geometries in [`guinea-plugin-store`](../store), which has to be
passed explicitly in WindowStateConfig.

Two halves, going opposite ways. Writing is a subscription: the shell
publishes `WindowChanged` whenever a window moves, and the plugin stores it.
Reading is a service: the shell asks `SavedGeometry` *before* it shows a
window, because a size applied afterwards is a visible jump - so here the
plugin is asked rather than telling.

Windows are keyed by their label, not by `RootId`: ids last one run, and the
whole point is to answer for the next one. A window without a label is not
remembered.

The geometries live under `window` in the store, one entry per label;
`.under("…")` picks another prefix for an application that already uses
`window` for something of its own.

Pass the returned `SavedGeometry` to the Slint window's `remembered_by(saved)`
before running it. Returning the value does not register it as a service. In guinea
0.31, WinUI does not consume this contract; this migration does not add that backend.
