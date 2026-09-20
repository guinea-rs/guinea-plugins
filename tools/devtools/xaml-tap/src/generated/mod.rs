//! What a generator wrote, and what a header dictated.
//!
//! Nothing here is edited to read better - it is regenerated. `bindings.rs`
//! is `windows-bindgen` over `bindings.txt`, which is the list of what the
//! tap actually calls. `diag.rs` is transcribed by hand from `xamlom.h` and
//! WinUI's `XamlOM.WinUI.idl`, because no metadata carries the XAML
//! diagnostics interfaces; it is generated in the sense that matters, which
//! is that the header decides what is in it.
//!
//! `winui.rs` is `windows-bindgen` output for the two WinUI methods at the
//! top of `bindings.txt`. Nothing declares it, so nothing compiles it.

#[allow(non_camel_case_types, non_snake_case, dead_code, clippy::all)]
pub mod bindings;

#[allow(non_snake_case, clippy::upper_case_acronyms)]
pub mod diag;
