#![cfg(all(windows, feature = "winui"))]

/// GUINEA_WINUI_WINMD points to the reactor-metadata/winmd directory of the
/// windows-rs checkout. Generated bindings are checked in, never built at startup.
#[test]
#[ignore = "regenerates CompositionTarget.Rendering bindings; requires WinUI metadata"]
fn write_frame_bindings() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let metadata = std::env::var_os("GUINEA_WINUI_WINMD").expect("WinUI metadata directory");
    windows_bindgen::builder()
        .input(std::path::PathBuf::from(metadata))
        .input_default()
        .flat()
        .minimal()
        .filter_file(root.join("src/painted/frame.txt"))
        .output(root.join("src/painted/frame_bindings.rs"))
        .write();
}
