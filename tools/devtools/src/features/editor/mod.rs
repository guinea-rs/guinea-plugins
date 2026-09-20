//! The editor source files open in: which ones this machine has, which one
//! was picked, and opening a file at a line in it.

mod actor;
pub mod contracts;
pub mod detect;
pub mod install;
pub mod launch;
mod settings;

pub use install::EditorFeature;
