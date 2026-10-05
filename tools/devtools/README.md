![guinea-devtools](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-devtools.png)

Looks inside running guinea applications.

```sh
cargo install --locked --path tools/devtools
guinea-devtools
```

An application with [`guinea-plugin-devtools`](../../plugins/devtools)
connects on its own, as many as are running, and keeps trying while devtools
are not.

| Tab | What it shows |
|---|---|
| Elements | The application's routes, segments, actors and state, and - through the native inspector - the UI tree with its properties, editable, and a picker on the window |
| Trace | Every record of what set off what: a click, the action it sent, the state it moved, the work it spawned and how that ended |
| Profiler | The UI thread's frames second by second on the trace's timeline, the layout passes in them, and the thread's sampled stacks |
| Application | Whatever the backends and plugins contribute as panels - the l10n plugin's strings in every language, for one |

The native inspector for WinUI is a tap devtools inject into the application:
it reads the live XAML tree, the frames WinUI writes to ETW, and the UI
thread's stack a thousand times a second while asked to, all without an
administrator.

## HTTP and MCP

Devtools serve what they see as an HTTP API described by its own OpenAPI
document; `guinea-devtools --help` lists the routes. `--headless` serves it
without a window.

`guinea-devtools-mcp` exposes the same routes as MCP tools over stdio, one
tool per route, proxying to a running devtools - window or headless - found
through the access file they write at startup.

Only one copy runs per user.
