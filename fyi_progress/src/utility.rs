/*!
# FYI Progress: Utility

This mod contains miscellaneous utility functions for the crate.
*/

use smallvec::SmallVec;



#[must_use]
/// Num as Bytes
///
/// Convert an integer into a vector of `u8`.
pub fn int_as_bytes(num: u64) -> SmallVec<[u8; 8]> {
	let mut buf = SmallVec::<[u8; 8]>::new();
	itoa::write(&mut buf, num).unwrap();
	buf
}

#[must_use]
/// Term Width
///
/// This is a simple wrapper around `term_size::dimensions()` to provide
/// the current terminal column width. We don't have any use for height,
/// so that property is ignored.
pub fn term_width() -> usize {
	// Reserve one space at the end "just in case".
	if let Some((w, _)) = term_size::dimensions() { w.saturating_sub(1) }
	else { 0 }
}



#[cfg(test)]
mod tests {
	use super::*;


	#[test]
	fn t_int_as_bytes() {
		assert_eq!(&int_as_bytes(1)[..], &b"1"[..]);
		assert_eq!(&int_as_bytes(10)[..], &b"10"[..]);
		assert_eq!(&int_as_bytes(1000)[..], &b"1000"[..]);
		assert_eq!(&int_as_bytes(1000000)[..], &b"1000000"[..]);
	}
}
