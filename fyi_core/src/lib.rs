/*!
# FYI Core
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "interactive")]
extern crate casual;

extern crate bytecount;
extern crate bytes;
extern crate chrono;
extern crate itoa;
extern crate lazy_static;
extern crate num_format;
extern crate num_traits;
extern crate regex;
extern crate term_size;

mod error;
mod msg;

/// Traits.
pub mod traits;

/// Utilities.
pub mod util;

#[cfg(feature = "progress")]
mod progress;

/// Append a timestamp to the message.
pub const MSG_TIMESTAMP: u8 = 1;

/// Message should not print in color.
pub const PRINT_NO_COLOR: u8 = 2;

/// Do not print anything. This is mostly just for debugging.
pub const PRINT_NOTHING: u8 = 4;

/// Append a new line while printing.
pub const PRINT_NEWLINE: u8 = 8;

/// Print to STDERR instead of STDOUT.
pub const PRINT_STDERR: u8 = 16;

#[cfg(feature = "progress")]
/// Progress Active.
pub const PROGRESSING: u8 = 32;

/// Exports.
pub use crate::msg::{
	Msg,
	Prefix,
};
pub use crate::error::{
	Error,
	Result,
};

#[cfg(feature = "progress")]
pub use crate::progress::Progress;

#[cfg(feature = "progress")]
pub use crate::progress::ProgressInner;
