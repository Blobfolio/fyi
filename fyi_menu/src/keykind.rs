/*!
# FYI: `KeyKind`

This is a very simple CLI argument "key" parser, meant to identify things like
"-s" or "--long", etc.

It is only really used by `Argue` during construction, but might find other
uses.
*/



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
		let len: usize = txt.len();
		let dashes: usize = txt.iter().take_while(|x| **x == b'-').count();

		if 0 < dashes && dashes < len && matches!(txt[dashes], b'A'..=b'Z' | b'a'..=b'z') {
			if dashes == 1 {
				if len == 2 { return Self::Short; }
				else { return Self::ShortV; }
			}
			else if dashes == 2 {
				return memchr::memchr(b'=', txt).map_or(Self::Long, Self::LongV);
			}
		}

		Self::None
	}
}
