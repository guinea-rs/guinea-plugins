![guinea-plugin-single-instance](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-plugin-single-instance.png)

Keeps one copy of a guinea application running per user.

```rust
guinea::app::GuineaApp::new()
    .meta(guinea::app::AppMeta::new("app", "dev.example.app", "0.1.0", "example"))
    .plugin(guinea_plugin_single_instance::SingleInstancePlugin::new())
```

The first copy holds a lock on a file named after the application's
identifier, in the user's own profile, for as long as it runs. A copy that
finds the lock taken stops before it opens a window: the plugin's `build`
returns `Stop`, the plugins installed before it are cleaned up again, and
`run` returns `Ok`. The operating system drops the lock when the holder exits,
however it exits, so a crash leaves nothing to clean up.

The identifier is the one in the application's `AppMeta`; `.named(..)` locks
under another.

Another program can ask whether the application is up - to start it only
when it is not:

```rust
if !guinea_plugin_single_instance::running("dev.example.app") {
    // start it
}
```

Outside an application, `claim(identifier)` takes the lock directly and holds
it for as long as the returned `Instance` lives.

Where it sits among the plugins is up to the application. What the ones
before it do that their cleanup does not undo - a migration run - a second
copy does too.
