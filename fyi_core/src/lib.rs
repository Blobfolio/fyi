/*!
# FYI Core
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

extern crate ansi_term;
extern crate chrono;
extern crate strip_ansi_escapes;

mod msg;
mod prefix;

/// Flag: No Color.
pub const NO_COLOR: u8 = 1;

/// Flag: Append Timestamp.
pub const TIMESTAMP: u8 = 2;

pub use crate::msg::Msg;
pub use crate::prefix::Prefix;
