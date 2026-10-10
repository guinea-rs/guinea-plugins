![guinea-plugin-devtools](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-plugin-devtools.png)

Lets [guinea devtools](../../tools/devtools) look inside a running guinea
application: its routes and segments, actors and state, the trace of what set
off what, timers and the global bus, what the async runtime's workers are
busy with by tokio's own counters - and, through the native inspector, the UI
tree and its frames.

```rust
guinea::app::GuineaApp::new()
    .plugin(guinea_plugin_devtools::DevToolsPlugin::new())
```

Devtools listen; the application connects, and keeps trying while devtools
are not running. Nothing is traced or read until devtools answer, and it
stops when they go: an application that has the plugin and no devtools runs
nothing for it. While they are there, they are sent everything once and then
only what guinea says moved: the trace a short wait after it happened, the
state that moved at most once a second, however often it moves.

| Setting | What it does |
|---|---|
| `.every(ms)` | How long after something moves devtools are told, so that a burst goes as one report. 50 ms |
| `.reread(ms)` | How often at most state that keeps moving is read again. 1000 ms |
| `.launch(true)` | Starts devtools when they are not running: `GUINEA_DEVTOOLS_BIN`, or `guinea-devtools` from the `PATH` |
| `.in_release(true)` | Connects in a release build too |

In a release build the plugin does nothing unless `in_release` says
otherwise: devtools can call the application's remote actions, and a build
shipped to users should not listen for that because a line was left in.

What goes over the wire is [`guinea-devtools-protocol`](../devtools-protocol).
