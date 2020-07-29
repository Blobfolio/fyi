/*!
# FYI Menu: Table of Contents
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
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unknown_clippy_lints)]

pub mod arglist;
pub use arglist::ArgList;
pub mod utility;



use fyi_msg::MsgKind;
use std::{
	cmp::Ordering,
	env,
	process,
};



/// Flag: No Flag
pub const FLAG_NONE: u8 =          0b0000;

/// Flag: All Flags
pub const FLAG_ALL: u8 =           0b1111;

/// Flag: (Re)Glue Post-Separator Arguments.
///
/// A "--" entry will be replaced by a recompiled-to-arg-string of all
/// remaining values.
pub const FLAG_GLUE_SEP_ARGS: u8 = 0b0001;

/// Flag: Something Required
///
/// If the parsed result is empty, an error will be printed and the program
/// will `exit(1)`.
pub const FLAG_REQUIRED: u8 =      0b0010;

/// Flag: Short Arguments Are Chars.
///
/// Short arguments may only be a char, e.g. "-a" or "-b". If something like
/// "-lossless" is encountered, the entry will be shortened to "-l", and an
/// adjoining entry for the value "ossless" will be inserted.
pub const FLAG_SHORT_IS_CHAR: u8 = 0b0100;

/// Flag: Trim Start
///
/// Remove all leading empty (or whitespace-only) entries during parse.
pub const FLAG_TRIM_START: u8 =    0b1000;



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// The Kind of Key.
pub enum KeyKind {
	/// Not a key.
	None,
	/// A short one.
	Short,
	/// A long one.
	Long,
}

impl Default for KeyKind {
	fn default() -> Self { Self::None }
}

impl From<&[u8]> for KeyKind {
	fn from(txt: &[u8]) -> Self {
		match txt.len().cmp(&2) {
			// This could be a short option.
			Ordering::Equal =>
				if txt[0] == b'-' && utility::byte_is_letter(txt[1]) {
					Self::Short
				}
				else { Self::None },
			// This could be anything!
			Ordering::Greater if txt[0] == b'-' =>
				if txt[1] == b'-' && utility::byte_is_letter(txt[2]) {
					Self::Long
				}
				else if utility::byte_is_letter(txt[1]) {
					Self::Short
				}
				else {
					Self::None
				}
			_ => Self::None,
		}
	}
}



/// Print an Error and Exit.
pub fn die(msg: &[u8]) {
	MsgKind::Error.as_msg(unsafe { std::str::from_utf8_unchecked(msg) }).eprintln();
	process::exit(1);
}

#[must_use]
/// Parse Env Args
///
/// This is a convenience method for `parse_args()` that draws from whatever is
/// in `std::env` (minus the command path).
pub fn parse_env_args(flags: u8) -> Vec<String> {
	let mut out: Vec<String> = env::args().skip(1).collect::<Vec<String>>();
	parse_args(&mut out, flags);
	out
}

/// Parse Args
///
/// This standardizes the formatting of a vec of args produced by e.g.
/// `std::env` to make it easier to loop through later.
///
/// See the crate flags for additional options.
pub fn parse_args(out: &mut Vec<String>, flags: u8) {
	// Give it a haircut.
	if 0 != flags & FLAG_TRIM_START {
		utility::vec_trim_start(out);
	}

	let mut len: usize = out.len();
	let mut idx: usize = 0;

	while idx < len {
		if out[idx] == "--" {
			// Replace the separator with recompiled args?
			if 0 != flags & FLAG_GLUE_SEP_ARGS {
				if idx + 1 == len {
					out[idx].truncate(0);
				}
				else {
					out[idx] = out.drain(idx+1..len)
						.map(utility::esc_arg)
						.collect::<Vec<String>>()
						.join(" ");
				}
			}

			break;
		}

		let bytes: &[u8] = out[idx].as_bytes();
		match KeyKind::from(bytes) {
			KeyKind::Short =>
				if 0 != flags & FLAG_SHORT_IS_CHAR && bytes.len() != 2 {
					out.insert(idx + 1, String::from(&out[idx][2..]));
					out[idx].truncate(2);

					idx += 2;
					len += 1;
				}
				else {
					idx += 1;
				},
			KeyKind::Long =>
				// Split on equal sign.
				if let Some(x) = bytes.iter().position(|b| *b == b'=') {
					// Insert the value.
					if x + 1 < bytes.len() {
						out.insert(idx + 1, String::from(&out[idx][x+1..]));
					}
					// Otherwise insert an empty value.
					else {
						out.insert(idx + 1, String::new());
					}

					// Shorten the key.
					out[idx].truncate(x);

					idx += 2;
					len += 1;
				}
				else {
					idx += 1;
				},
			KeyKind::None => {
				idx += 1;
			},
		}
	}

	// Make sure we aren't empty.
	if 0 != flags & FLAG_REQUIRED && out.is_empty() {
		die(b"Missing options, flags, arguments, and/or ketchup.");
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_parse_args() {
		let base: Vec<String> = vec![
			String::from(""),
			String::from("hey"),
			String::from("-k"),
			String::from("-kVal"),
			String::from("--key"),
			String::from("--key=Val"),
			String::from("--"),
			String::from("stuff"),
			String::from("and things")
		];

		// No Flags.
		let mut args = base.clone();
		parse_args(&mut args, FLAG_NONE);
		assert_eq!(
			args,
			vec![
				String::from(""),
				String::from("hey"),
				String::from("-k"),
				String::from("-kVal"),
				String::from("--key"),
				String::from("--key"),
				String::from("Val"), // Value split off.
				String::from("--"),
				String::from("stuff"),
				String::from("and things")
			]
		);

		// Test Trim.
		args = base.clone();
		parse_args(&mut args, FLAG_TRIM_START);
		assert_eq!(
			args,
			vec![ // Missing first bit.
				String::from("hey"),
				String::from("-k"),
				String::from("-kVal"),
				String::from("--key"),
				String::from("--key"),
				String::from("Val"),  // Value split off.
				String::from("--"),
				String::from("stuff"),
				String::from("and things")
			]
		);

		// Short Char Keys.
		let mut args = base.clone();
		parse_args(&mut args, FLAG_SHORT_IS_CHAR);
		assert_eq!(
			args,
			vec![
				String::from(""),
				String::from("hey"),
				String::from("-k"),
				String::from("-k"),
				String::from("Val"), // Value split off.
				String::from("--key"),
				String::from("--key"),
				String::from("Val"), // Value split off.
				String::from("--"),
				String::from("stuff"),
				String::from("and things")
			]
		);

		// Glue Separator Bits.
		let mut args = base.clone();
		parse_args(&mut args, FLAG_GLUE_SEP_ARGS);
		assert_eq!(
			args,
			vec![
				String::from(""),
				String::from("hey"),
				String::from("-k"),
				String::from("-kVal"),
				String::from("--key"),
				String::from("--key"),
				String::from("Val"), // Value split off.
				String::from("stuff 'and things'"), // Values glued.
			]
		);

		// All Flags.
		let mut args = base.clone();
		parse_args(&mut args, FLAG_ALL);
		assert_eq!(
			args,
			vec![ // Missing first bit.
				String::from("hey"),
				String::from("-k"),
				String::from("-k"),
				String::from("Val"), // Value split off.
				String::from("--key"),
				String::from("--key"),
				String::from("Val"), // Value split off.
				String::from("stuff 'and things'"), // Values glued.
			]
		);
	}
}
