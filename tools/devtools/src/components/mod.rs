//! What the pages are built from: blocks and headings, tabs, fields, trees
//! and links to source. The colours they draw with are [`crate::theme`]'s.

mod block;
mod fields;
mod input;
mod panel;
mod runner;
pub mod source;
mod tabs;
mod text;
pub mod tree;

pub use block::{PADDING, block, head, heading, rule};
pub use fields::fields;
pub use input::{FIELD_HEIGHT, field_look, search, select};
pub use panel::{bare, side};
pub use runner::{run_cycle, runner};
pub use tabs::{TABS_HEIGHT, tabs};
pub use text::{dim, line, mono};
