/*!
# FYI Progress: Utility

This mod contains miscellaneous utility functions for the crate.
*/

use std::borrow::Cow;
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

			// If we went over budget, back up a step and exit.
			if cur_width > width {
				cur_len -= char_len;
				break;
			}
		}

		cur_len
	}
}

#[must_use]
/// Full, Human-Readable Time.
///
/// The full format breaks seconds down into hours, minutes, and seconds, then
/// joins the non-zero values with grammatically-appropriate commas and
/// conjunctions.
///
/// For example, if a time matches exactly two hours, it returns "2 hours". If
/// minutes are involved, "2 hours and 13 minutes". If seconds, then you get
/// "2 hours, 13 minutes, and 1 second".
///
/// Etc.
///
/// For times stretching beyond one day, a static value of "1+ days" is
/// returned.
pub fn human_elapsed(num: u32) -> Cow<'static, [u8]> {
	static ONE: [&[u8]; 3] = [b" hour", b" minute", b" second"];
	static MANY: [&[u8]; 3] = [b" hours", b" minutes", b" seconds"];
	static AND: &[u8] = b", and ";

	if 1 == num {
		Cow::Borrowed(b"1 second")
	}
	// Just seconds.
	else if num < 60 {
		Cow::Owned({
			let mut buf: Vec<u8> = Vec::with_capacity(10);
			itoa::write(&mut buf, num).unwrap();
			buf.extend_from_slice(MANY[2]);
			buf
		})
	}
	// Let's build it.
	else if num < 86400 {
		let c = secs_chunks(num);

		// Find out how many non-zero values there are.
		let len: usize = c.iter().filter(|&n| *n != 0).count();

		let mut buf = Vec::with_capacity(64);
		let mut i: usize = 0;
		let mut j: usize = 0;
		loop {
			// Skip empties.
			if c[i] == 0 {
				i += 1;
				continue;
			}

			itoa::write(&mut buf, c[i]).unwrap();
			match c[i] {
				1 => buf.extend_from_slice(ONE[i]),
				_ => buf.extend_from_slice(MANY[i]),
			}

			i += 1;
			j += 1;

			if j == len {
				break;
			}
			else if len - j == 1 {
				if len > 2 {
					buf.extend_from_slice(AND);
				}
				else {
					buf.extend_from_slice(&AND[1..]);
				}
			}
			else {
				buf.extend_from_slice(&AND[..2]);
			}
		}

		Cow::Owned(buf)
	}
	// Too long.
	else {
		Cow::Borrowed(b"1+ days")
	}
}

#[must_use]
/// Num as Bytes
///
/// Convert an integer into a vector of `u8`.
pub fn int_as_bytes(num: u64) -> SmallVec<[u8; 8]> {
	if num < 1000 {
		let mut buf = itoa::Buffer::new();
		SmallVec::<[u8; 8]>::from_slice(buf.format(num).as_bytes())
	}
	// Handle commas and whatnot for big numbers.
	else {
		let mut buf = num_format::Buffer::default();
		buf.write_formatted(&num, &num_format::Locale::en);
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
	fn t_chopped_len() {
		assert_eq!(chopped_len(b"Hello World", 15), 11);
		assert_eq!(chopped_len(b"Hello \x1b[1mWorld\x1b[0m", 15), 19);
		assert_eq!(chopped_len(b"Hello \x1b[1mWorld\x1b[0m", 7), 11);
	}

	#[test]
	fn t_human_elapsed() {
		_human_elapsed(0, "0 seconds");
		_human_elapsed(1, "1 second");
		_human_elapsed(50, "50 seconds");
		_human_elapsed(100, "1 minute and 40 seconds");
		_human_elapsed(2121, "35 minutes and 21 seconds");
		_human_elapsed(36015, "10 hours and 15 seconds");
		_human_elapsed(37732, "10 hours, 28 minutes, and 52 seconds");
		_human_elapsed(37740, "10 hours and 29 minutes");
		_human_elapsed(37740, "10 hours and 29 minutes");
		_human_elapsed(428390, "1+ days");
	}

	fn _human_elapsed(num: u32, expected: &str) {
		assert_eq!(
			human_elapsed(num).as_ref(),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}

	#[test]
	fn t_int_as_bytes() {
		assert_eq!(&int_as_bytes(1)[..], &b"1"[..]);
		assert_eq!(&int_as_bytes(10)[..], &b"10"[..]);
		assert_eq!(&int_as_bytes(1000)[..], &b"1,000"[..]);
		assert_eq!(&int_as_bytes(1000000)[..], &b"1,000,000"[..]);
	}

	#[test]
	fn t_secs_chunks() {
		assert_eq!(secs_chunks(1), [0, 0, 1]);
		assert_eq!(secs_chunks(30), [0, 0, 30]);
		assert_eq!(secs_chunks(90), [0, 1, 30]);
		assert_eq!(secs_chunks(3600), [1, 0, 0]);
	}
}
