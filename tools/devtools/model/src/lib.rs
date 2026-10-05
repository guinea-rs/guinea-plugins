//! What devtools make of what applications report, with no UI in it.
//!
//! The window and the HTTP API both show what this returns, and nothing they
//! show is worked out anywhere else: a window only draws it, the API only
//! serialises it.

pub mod access;
pub mod chains;
pub mod clock;
pub mod elements;
pub mod graph;
pub mod names;
pub mod native;
pub mod panels;
pub mod profile;
pub mod properties;
pub mod samples;
pub mod sessions;
pub mod tasks;
pub mod timers;
pub mod trace;
pub mod trace_log;
pub mod words;

pub use guinea_devtools_protocol as protocol;
