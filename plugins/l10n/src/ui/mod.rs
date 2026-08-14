//! Per-backend ways of consuming the current strings.
//!
//! The backend is a feature, not a target: an application can build for one OS
//! and still choose - or combine - toolkits. Everything outside this module is
//! backend-agnostic.

#[cfg(feature = "winui")]
mod winui;

#[cfg(feature = "winui")]
pub use winui::use_l10n;
