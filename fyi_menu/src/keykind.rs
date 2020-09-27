/*!
# FYI Menu: Key Kind

**Note:** This is not intended for external use and is subject to change.
*/

#[cfg(target_arch = "x86")]
use std::arch::x86::{
	_mm_cmpeq_epi8,
	_mm_loadu_si128,
	_mm_movemask_epi8,
	_mm_set1_epi8
};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
	_mm_cmpeq_epi8,
	_mm_loadu_si128,
	_mm_movemask_epi8,
	_mm_set1_epi8
};



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
	fn from(txt: &[u8]) -> Self {
		let len: usize = txt.len();
		if len >= 2 && txt[0] == b'-' {
			// Could be long.
			if txt[1] == b'-' {
				// Is a long.
				if len > 2 && is_letter(txt[2]) {
					return find_eq(txt);
				}
			}
			// Is short.
			else if is_letter(txt[1]) {
				if len == 2 { return Self::Short; }
				else { return Self::ShortV; }
			}
		}

		Self::None
	}
}

#[must_use]
#[inline]
/// # Is Letter.
const fn is_letter(byte: u8) -> bool {
	(b'a' <= byte && byte <= b'z') || (b'A' <= byte && byte <= b'Z')
}

#[must_use]
/// # Find First `=`
///
/// This is used solely for deciding between [`KeyKind::Long`] and
/// [`KeyKind::LongV`] variants. It will always be one of the two.
///
/// This method leverages SIMD to search for that pesky `=` sign in chunks of
/// up to 16 bytes at a time.
fn find_eq(txt: &[u8]) -> KeyKind {
	if 16 <= txt.len() && is_x86_feature_detected!("sse2") {
		unsafe { find_eq_sse2(txt) }
	}
	else {
		txt.iter().position(|x| *x == b'=').map_or(KeyKind::Long, KeyKind::LongV)
	}
}

#[allow(clippy::cast_possible_wrap)] // It's fine.
#[allow(clippy::cast_ptr_alignment)] // It's fine.
#[target_feature(enable = "sse2")]
/// # Find First `=` (SSE2).
///
/// This is an SSE2/SIMD-optimized implementation of `find_eq` used for strings
/// that are at least 16 bytes.
unsafe fn find_eq_sse2(txt: &[u8]) -> KeyKind {
	let len: usize = txt.len();
	let ptr = txt.as_ptr();
	let needle = _mm_set1_epi8(b'=' as i8);
	let mut offset: usize = 3;

	// Check for matches 16 bytes at a time.
	while offset + 16 <= len {
		let haystack = _mm_loadu_si128(ptr.add(offset) as *const _);
		let eq = _mm_cmpeq_epi8(needle, haystack);
		let res = _mm_movemask_epi8(eq).trailing_zeros();
		if res < 16 {
			return KeyKind::LongV(res as usize + offset);
		}

		offset += 16;
	}

	// If there's a remainder, recheck from the end (to fill the
	// registers).
	if offset < len {
		offset = len - 16;
		let haystack = _mm_loadu_si128(ptr.add(offset) as *const _);
		let eq = _mm_cmpeq_epi8(needle, haystack);
		let res = _mm_movemask_epi8(eq).trailing_zeros();
		if res < 16 {
			return KeyKind::LongV(res as usize + offset);
		}
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

		// Test in and around the 16-char boundary.
		assert_eq!(KeyKind::from(&b"--yes_="[..]), KeyKind::LongV(6));
		assert_eq!(KeyKind::from(&b"--yes__="[..]), KeyKind::LongV(7));
		assert_eq!(KeyKind::from(&b"--yes___="[..]), KeyKind::LongV(8));
		assert_eq!(KeyKind::from(&b"--yes____="[..]), KeyKind::LongV(9));
		assert_eq!(KeyKind::from(&b"--yes_____="[..]), KeyKind::LongV(10));
		assert_eq!(KeyKind::from(&b"--yes______="[..]), KeyKind::LongV(11));
		assert_eq!(KeyKind::from(&b"--yes_______="[..]), KeyKind::LongV(12));
		assert_eq!(KeyKind::from(&b"--yes________="[..]), KeyKind::LongV(13));
		assert_eq!(KeyKind::from(&b"--yes_________="[..]), KeyKind::LongV(14));
		assert_eq!(KeyKind::from(&b"--yes__________="[..]), KeyKind::LongV(15));
		assert_eq!(KeyKind::from(&b"--yes___________="[..]), KeyKind::LongV(16));
		assert_eq!(KeyKind::from(&b"--yes____________="[..]), KeyKind::LongV(17));
		assert_eq!(KeyKind::from(&b"--yes_____________"[..]), KeyKind::Long);

		// Does this work?
		assert_eq!(
			KeyKind::from("--BjörkGuðmundsdóttir".as_bytes()),
			KeyKind::Long
		);
		assert_eq!(
			KeyKind::from("--BjörkGuðmunds=dóttir".as_bytes()),
			KeyKind::LongV(17)
		);
	}
}
