/*!
# FYI: `KeyKind`

This is a very simple CLI argument "key" parser, meant to identify things like
"-s" or "--long", etc.

It is only really used by `Argue` during construction, but might find other
uses.
*/

use crate::utility;
use std::cmp::Ordering;



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// The Kind of Key.
///
/// This is only used during argument parsing. It is made public for the sake
/// of benchmarking.
pub enum KeyKind {
	/// Not a key.
	None,
	/// A short one.
	Short,
	/// A short one with a potential value chunk.
	ShortV,
	/// A long one.
	Long,
	/// A long one with a value chunk. The `usize` indicates the position of
	/// the `=` character.
	LongV(usize),
}

impl Default for KeyKind {
	fn default() -> Self { Self::None }
}

impl From<&[u8]> for KeyKind {
	fn from(txt: &[u8]) -> Self {
		if txt.is_empty() || txt[0] != b'-' { return Self::None; }

		match txt.len().cmp(&2) {
			// This could be a short option.
			Ordering::Equal if utility::byte_is_letter(txt[1]) => Self::Short,
			// This could be anything!
			Ordering::Greater =>
				if txt[1] == b'-' && utility::byte_is_letter(txt[2]) {
					memchr::memchr(b'=', txt)
						.map_or(Self::Long, Self::LongV)
				}
				else if utility::byte_is_letter(txt[1]) {
					Self::ShortV
				}
				else {
					Self::None
				}
			_ => Self::None,
		}
	}
}
