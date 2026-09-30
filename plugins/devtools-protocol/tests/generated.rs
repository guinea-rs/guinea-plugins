//! The compiled schema in the tree is the schema as it is written now.
//!
//! Needs the `capnp` binary. After changing `schema/devtools.capnp`, compile
//! it into `src/generated/` with `capnp compile -orust:src/generated
//! --src-prefix=schema schema/devtools.capnp`, run from the crate's directory.

use std::path::Path;

/// The generated code without its comments, which name the versions of the
/// tools that wrote it rather than anything the schema says.
fn code(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_compiled_schema_in_the_tree_is_current() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = tempfile::tempdir().expect("a directory");

    capnpc::CompilerCommand::new()
        .src_prefix(here.join("schema"))
        .file(here.join("schema/devtools.capnp"))
        .output_path(out.path())
        .run()
        .expect("compiling the schema needs the `capnp` binary on PATH");

    assert!(
        code(&out.path().join("devtools_capnp.rs"))
            == code(&here.join("src/generated/devtools_capnp.rs")),
        "src/generated/devtools_capnp.rs is not what schema/devtools.capnp compiles to; compile it again"
    );
}
