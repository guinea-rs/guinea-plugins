![guinea-plugin-window-state](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-plugin-window-state.png)

Remembers where a guinea application's windows were, and opens them there
again.

```rust
guinea::app::GuineaApp::new()
    .plugin(guinea_plugin_store::StorePlugin::for_app("app", "settings"))
    .plugin(guinea_plugin_window_state::WindowStatePlugin::new())
```

It keeps the geometries in [`guinea-plugin-store`](../store), which has to be
installed first.

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
