![guinea-plugin-l10n-build](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-plugin-l10n-build.png)

The build-time half of [`guinea-plugin-l10n`](../l10n): reads the
application's Fluent `.ftl` files in `build.rs` and writes one typed accessor
per message, so a message that is renamed or loses a variable is a compile
error rather than a missing string at runtime.

```rust
// build.rs
fn main() {
    guinea_plugin_l10n_build::build("locales");
}
```

```rust
// src/l10n.rs
guinea_plugin_l10n::fluent_loader! {
    locales: "./locales",
    fallback_language: "en",
}
```

A message with variables takes them as arguments:

```ftl
# locales/en/main.ftl
greeting = Hello, { $name }!
files-left = { $count } files left
```

```rust
strings.greeting("Ann");
strings.files_left(3);
```

The reference locale - `en` for `build`, any other with
`build_for_locale(dir, "ru")` - is the one the accessors are generated from.
A locale is either `locales/<tag>.ftl` or a directory `locales/<tag>/` of
`.ftl` files, nested as deep as it likes.

It also writes the table devtools read: every message, the file and line it
is written on, what it takes, what each locale says for it, and which have not
translated it yet.
