/*!
# FYI Core
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "progress")]
extern crate ansi_escapes;

extern crate ansi_term;
extern crate chrono;
extern crate dialoguer;
extern crate num_format;
extern crate strip_ansi_escapes;
extern crate term_size;

pub mod misc;
mod msg;
mod prefix;

#[cfg(feature = "progress")]
mod progress;

/// Append a timestamp to the message.
pub const MSG_TIMESTAMP: u8 = 1;

/// Print compact.
pub const PRINT_COMPACT: u8 = 2;

/// Message should not print in color.
pub const PRINT_NO_COLOR: u8 = 4;

/// Append a new line while printing.
pub const PRINT_NEWLINE: u8 = 8;

/// Print to STDERR instead of STDOUT.
pub const PRINT_STDERR: u8 = 16;

/// Just clear the bar and call it a day.
pub const PROGRESS_NO_ELAPSED: u8 = 32;

/// Exports.
pub use crate::msg::Msg;
pub use crate::prefix::Prefix;

#[cfg(feature = "progress")]
pub use crate::progress::arc as progress_arc;

#[cfg(feature = "progress")]
pub use crate::progress::Progress;
