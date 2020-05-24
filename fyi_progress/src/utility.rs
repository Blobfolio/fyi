/*!
# FYI Progress: Utility

This mod contains miscellaneous utility functions for the crate.
*/

use num_format::{
	Buffer,
	Locale,
};
use smallvec::SmallVec;
use unicode_width::UnicodeWidthChar;



#[must_use]
/// Byte Index At Width
///
/// Go char-by-char through a string, counting the widths of printable
/// characters, and return the byte length of the substring that fits.
pub fn chopped_len(buf: &[u8], width: usize) -> usize {
	let len: usize = buf.len();
	// Width can't exceed length, so we can quit early if it is short.
	if len <= width { len }
	else {
		let mut cur_len: usize = 0;
		let mut cur_width: usize = 0;
		let mut in_ansi: bool = false;

		for c in unsafe { std::str::from_utf8_unchecked(buf) }.chars() {
			let char_len: usize = c.len_utf8();

			// If we're in the middle of an ANSI command, normal printability
			// does not apply. But if we hit an exit, we should note that for
			// the next loop.
			if in_ansi {
				if c == 'A' || c == 'K' || c == 'm' {
					in_ansi = false;
				}

				cur_len += char_len;
				continue;
			}
			// Enter an ANSI command.
			else if c == '\x1b' {
				in_ansi = true;
				cur_len += char_len;
				continue;
			}

			// For everything else, trust the `UnicodeWidth` crate to come up
			// with a decent size estimation.
			let char_width: usize = UnicodeWidthChar::width(c).unwrap_or(0);
			cur_len += char_len;
			cur_width += char_width;

			// We've reached the end of what can fit.
			if cur_width >= width {
				// If we went over, go back one step, length-wise.
				if cur_width > width {
					cur_len -= char_len;
				}

				break;
			}
		}

		cur_len
	}
}

#[must_use]
/// Num as Bytes
///
/// Convert an integer into a vector of `u8`.
pub fn int_as_bytes(num: u64) -> SmallVec<[u8; 8]> {
	if num < 1000 {
		let mut buf = SmallVec::<[u8; 8]>::new();
		itoa::write(&mut buf, num).unwrap();
		buf
	}
	// Handle commas and whatnot for big numbers.
	else {
		let mut buf = Buffer::default();
		buf.write_formatted(&num, &Locale::en);
		SmallVec::<[u8; 8]>::from_slice(buf.as_bytes())
	}
}

#[must_use]
/// Chunked Seconds
///
/// This method converts seconds into hours, minutes, and seconds, returning
/// a fixed-length array with each value in order, e.g. `[h, m, s]`.
///
/// As with the rest of the methods in this module, days and beyond are not
/// considered. Large values are simply truncated to `86399`, i.e. one second
/// shy of a full day.
pub fn secs_chunks(num: u32) -> [u32; 3] {
	let mut out: [u32; 3] = [0, 0, u32::min(86399, num)];

	// Hours.
	if out[2] >= 3600 {
		out[0] = num_integer::div_floor(out[2], 3600);
		out[2] -= out[0] * 3600;
	}

	// Minutes.
	if out[2] >= 60 {
		out[1] = num_integer::div_floor(out[2], 60);
		out[2] -= out[1] * 60;
	}

	out
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
		assert_eq!(&int_as_bytes(1000)[..], &b"1,000"[..]);
		assert_eq!(&int_as_bytes(1000000)[..], &b"1,000,000"[..]);
	}
}
