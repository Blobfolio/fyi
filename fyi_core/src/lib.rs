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

#[cfg(any(feature = "witcher", feature = "progress"))]
#[macro_use]
extern crate defaults;

#[cfg(feature = "interactive")]
extern crate dialoguer;

#[cfg(feature = "witcher")]
extern crate nix;

#[cfg(feature = "witcher")]
extern crate rayon;

#[cfg(feature = "witcher")]
extern crate regex;

extern crate bytecount;
extern crate chrono;
extern crate num_traits;
extern crate num_format;
extern crate strip_ansi_escapes;
extern crate term_size;

/// Traits.
pub mod traits;

/// Utilities.
pub mod util;
mod msg;
mod prefix;

#[cfg(feature = "progress")]
mod progress;

#[cfg(feature = "witcher")]
mod witch;

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
pub const PROGRESS_CLEAR_ON_FINISH: u8 = 32;

/// Exports.
pub use crate::msg::Msg;
pub use crate::prefix::Prefix;

#[cfg(feature = "witcher")]
pub use crate::witch::Witch;

#[cfg(feature = "progress")]
/// Arc helpers.
pub mod arc {
	pub use crate::progress::arc as progress;
}

#[cfg(feature = "progress")]
pub use crate::progress::Progress;
