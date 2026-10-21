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
| Profiler | The UI thread's frames second by second on the trace's timeline, the layout passes in them, every running thread's sampled stacks, what the process holds in memory, and how many of the async runtime's workers are busy |
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

## Checks and debug symbols

This workspace uses the repository's pinned Rust 1.99.1 toolchain. Its declared
MSRV remains 1.95. Ordinary dev, test, release and bench builds disable debug
information. Windows CI also passes `-C link-arg=/DEBUG:NONE`, because `debug = 0`
alone does not stop the MSVC linker from writing a PDB.

Two Windows sampler tests check resolution of private function names and need
symbols. They are ignored in the ordinary test profile and run explicitly in
Windows CI with the `sampling` profile and `-C link-arg=/DEBUG`. Only
`guinea-xaml-tap` enables limited debug information in that profile; its
dependencies stay at `debug = 0`:

```sh
cargo test --workspace --locked
```

For the symbol tests on Windows, override the normal linker flags for that
invocation. This PowerShell example preserves any existing flags and restores
the environment afterwards:

```powershell
$previousFlags = $env:RUSTFLAGS
try {
    $env:RUSTFLAGS = "$previousFlags -C link-arg=/DEBUG"
    cargo test -p guinea-xaml-tap --profile sampling --locked --lib -- --ignored --test-threads=1
} finally {
    $env:RUSTFLAGS = $previousFlags
}
```

An environment `RUSTFLAGS` replaces Cargo's configured rustflags. If your Cargo
configuration supplies additional required flags, include those in the opt-in
invocation too.
