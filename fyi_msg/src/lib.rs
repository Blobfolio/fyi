/*!
# FYI Msg: Table of Contents
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::pedantic)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

#[cfg(feature = "interactive")]
extern crate casual;

extern crate bitflags;
extern crate bytecount;
extern crate bytes;
extern crate chrono;
extern crate itoa;
extern crate lazy_static;
extern crate num_traits;
extern crate regex;
extern crate term_size;

mod msg;

pub mod print;
pub mod traits;

pub use msg::Msg;
pub use print::{
	Flags,
	print,
	prompt,
	term_width,
	whitespace,
};
