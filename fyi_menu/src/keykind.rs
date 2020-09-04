/*!
# FYI Menu: Key Kind

**Note:** This is not intended for external use and is subject to change.
*/



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// The `KeyKind` enum is used to differentiate between the types of CLI argument
/// keys [`Argue`](crate::Argue) might encounter during parsing (and `None` in the case of a
/// non-key-looking entry).
///
/// In keeping with the general ethos of this crate, speed is the name of the game,
/// which is achieved primarily through simplicity:
/// * If an entry begins with a single `-`, it is assumed to be a short key.
/// * If a short key consists of more than two characters, `2..` is assumed to be a value.
/// * If an entry begins with two `--`, it is assumed to be a long key.
/// * If a long key contains an `=`, everything after that is assumed to be a value.
pub enum KeyKind {
	/// Not a key.
	None,
	/// A short key.
	Short,
	/// A short key with a value.
	ShortV,
	/// A long key.
	Long,
	/// A long key with a value chunk. The `usize` indicates the position of
	/// the `=` character, with everything before being the key, and everything
	/// after being the value.
	LongV(usize),
}

impl Default for KeyKind {
	fn default() -> Self { Self::None }
}

impl From<&[u8]> for KeyKind {
	#[cfg(not(feature = "simd"))]
	fn from(txt: &[u8]) -> Self {
		let len: usize = txt.len();
		if len >= 2 {
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
		}

		Self::None
	}

	#[cfg(feature = "simd")]
	fn from(txt: &[u8]) -> Self {
		let len: usize = txt.len();
		if len >= 2 {
			let dashes: usize = unsafe {
				use packed_simd::u8x2;
				let res = u8x2::from_slice_unaligned_unchecked(&txt[0..2])
					.eq(u8x2::splat(b'-'));
				if res.all() { 2 }
				else if res.extract_unchecked(0) { 1 }
				else { 0 }
			};

			if 0 < dashes && dashes < len && matches!(txt[dashes], b'A'..=b'Z' | b'a'..=b'z') {
				if dashes == 1 {
					if len == 2 { return Self::Short; }
					else { return Self::ShortV; }
				}
				else if dashes == 2 {
					return memchr::memchr(b'=', txt).map_or(Self::Long, Self::LongV);
				}
			}
		}

		Self::None
	}
}
