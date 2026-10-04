#![cfg(all(windows, feature = "winui"))]

//! The Direct2D bindings the chart clips with are what their filter writes.
//!
//! They stand in for what windows-canvas does not wrap - a layer with a
//! geometric mask - until it does. To write them again after the filter
//! changes: `cargo test -p guinea-widgets --test bindings -- --ignored`.

use std::path::{Path, PathBuf};

fn chart() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/chart")
}

fn generate(output: &Path) {
    windows_bindgen::builder()
        .input_default()
        .flat()
        .filter_file(chart().join("d2d.txt"))
        .output(output)
        .write();
}

fn squeezed(text: &str) -> String {
    text.split_whitespace().collect()
}

#[test]
fn the_direct2d_bindings_are_what_their_filter_writes() {
    let fresh = std::env::temp_dir().join(format!("guinea-widgets-d2d-{}.rs", std::process::id()));
    generate(&fresh);
    let written = std::fs::read_to_string(&fresh).unwrap_or_default();
    let _ = std::fs::remove_file(&fresh);
    let held = std::fs::read_to_string(chart().join("d2d.rs")).unwrap_or_default();

    assert!(
        !written.is_empty() && squeezed(&held) == squeezed(&written),
        "src/chart/d2d.rs is not what src/chart/d2d.txt writes: run \
         `cargo test -p guinea-widgets --test bindings -- --ignored`"
    );
}

#[test]
#[ignore = "writes src/chart/d2d.rs"]
fn write_the_direct2d_bindings() {
    generate(&chart().join("d2d.rs"));
}
