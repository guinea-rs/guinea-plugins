//! Puts the guinea pig on the executable itself, where Explorer and a pinned
//! taskbar button look for it. The window's own icon is set at runtime.

fn main() {
    println!("cargo:rerun-if-changed=assets/guinea.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    winresource::WindowsResource::new()
        .set_icon("assets/guinea.ico")
        .compile()
        .expect("the icon is compiled into the executable");
}
