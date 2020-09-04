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
					return find_eq(txt);
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
					return find_eq(txt);
				}
			}
		}

		Self::None
	}
}

#[cfg(not(feature = "simd"))]
/// # Find First `=`
///
/// This is used solely for deciding between [`KeyKind::Long`] and
/// [`KeyKind::LongV`] variants. It will always be one of the two.
fn find_eq(txt: &[u8]) -> KeyKind {
	txt.iter()
		.position(|x| *x == b'=')
		.map_or(KeyKind::Long, KeyKind::LongV)
}

#[cfg(feature = "simd")]
/// # Find First `=`
///
/// This is used solely for deciding between [`KeyKind::Long`] and
/// [`KeyKind::LongV`] variants. It will always be one of the two.
///
/// This method leverages SIMD to search for that pesky `=` sign in chunks of
/// up to 8 bytes at a time.
fn find_eq(txt: &[u8]) -> KeyKind {
	let len: usize = txt.len();
	let mut offset: usize = 3;

	// We're checking lengths all along the way so this isn't really unsafe.
	unsafe {
		// For long strings, we can check 8 bytes at a time, returning the first
		// match, if any.
		while len - offset >= 8 {
			use packed_simd::u8x8;
			let res = u8x8::from_slice_unaligned_unchecked(&txt[offset..offset+8])
				.eq(u8x8::splat(b'='));
			if res.any() {
				return KeyKind::LongV(
					res.select(
						u8x8::new(0, 1, 2, 3, 4, 5, 6, 7),
						u8x8::splat(9)
					).min_element() as usize + offset
				);
			}

			offset += 8;
		}

		// We can use the same trick again if the remainder is at least four
		// bytes.
		if len - offset >= 4 {
			use packed_simd::u8x4;

			let res = u8x4::from_slice_unaligned_unchecked(&txt[offset..offset+4])
				.eq(u8x4::splat(b'='));
			if res.any() {
				return KeyKind::LongV(
					res.select(
						u8x4::new(0, 1, 2, 3),
						u8x4::splat(9)
					).min_element() as usize + offset
				);
			}

			offset += 4;
		}
	}

	// And a sad manual check for the remainder.
	while offset < len {
		if txt[offset] == b'=' { return KeyKind::LongV(offset); }
		offset += 1;
	}

	KeyKind::Long
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_from() {
		assert_eq!(KeyKind::from(&b"Your Mom"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"--"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"-"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"-0"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"-y"[..]), KeyKind::Short);
		assert_eq!(KeyKind::from(&b"-yp"[..]), KeyKind::ShortV);
		assert_eq!(KeyKind::from(&b"--0"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"--yes"[..]), KeyKind::Long);
		assert_eq!(KeyKind::from(&b"--y-p"[..]), KeyKind::Long);
		assert_eq!(KeyKind::from(&b"--yes=no"[..]), KeyKind::LongV(5));
		assert_eq!(KeyKind::from(&b"--yes="[..]), KeyKind::LongV(5));
	}
}
