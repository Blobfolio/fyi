/*!
# FYI Progress: Utility

This mod contains miscellaneous utility functions for the crate.
*/

use std::borrow::Cow;
use smallvec::SmallVec;
use unicode_width::UnicodeWidthChar;



const LBL_AND: &[u8] = &[32, 97, 110, 100, 32];
const LBL_HOUR: &[u8] = &[32, 104, 111, 117, 114];
const LBL_HOURS: &[u8] = &[32, 104, 111, 117, 114, 115];
const LBL_MINUTE: &[u8] = &[32, 109, 105, 110, 117, 116, 101];
const LBL_MINUTES: &[u8] = &[32, 109, 105, 110, 117, 116, 101, 115];
const LBL_SECOND: &[u8] = &[32, 115, 101, 99, 111, 110, 100];
const LBL_SECONDS: &[u8] = &[32, 115, 101, 99, 111, 110, 100, 115];

// Helper: Write inflected time unit to buffer.
macro_rules! write_time {
	($buf:ident, $val:expr, $one:expr, $many:expr) => {
		itoa::write(&mut $buf, $val).unwrap();
		if $val == 1 { $buf.extend_from_slice($one); }
		else { $buf.extend_from_slice($many); }
	};
}

// Helper: Generate method for producing a single unit of time's time, e.g.
// "2 minutes".
macro_rules! one_time {
	($fn:ident, $one1:expr, $many1:expr) => {
		/// Gloop One Time Together.
		fn $fn(val1: u32) -> Cow<'static, [u8]> {
			let mut buf: Vec<u8> = Vec::with_capacity(10);
			write_time!(buf, val1, $one1, $many1);
			Cow::Owned(buf)
		}
	};
}

// Helper: Generate method for producing two units of times' times, e.g.
// "2 minutes and 30 seconds".
macro_rules! time_and_time {
	($fn:ident, $one1:expr, $many1:expr, $one2:expr, $many2:expr) => {
		/// Gloop Two Times Together.
		fn $fn(val1: u32, val2: u32) -> Cow<'static, [u8]> {
			let mut buf: Vec<u8> = Vec::with_capacity(25);

			write_time!(buf, val1, $one1, $many1);
			buf.extend_from_slice(LBL_AND);
			write_time!(buf, val2, $one2, $many2);

			Cow::Owned(buf)
		}
	};
}



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

one_time!(human_h, LBL_HOUR, LBL_HOURS);
one_time!(human_m, LBL_MINUTE, LBL_MINUTES);
one_time!(human_s, LBL_SECOND, LBL_SECONDS);
time_and_time!(human_hm, LBL_HOUR, LBL_HOURS, LBL_MINUTE, LBL_MINUTES);
time_and_time!(human_hs, LBL_HOUR, LBL_HOURS, LBL_SECOND, LBL_SECONDS);
time_and_time!(human_ms, LBL_MINUTE, LBL_MINUTES, LBL_SECOND, LBL_SECONDS);

/// Gloop All Times Together.
fn human_hms(val1: u32, val2: u32, val3: u32) -> Cow<'static, [u8]> {
	let mut buf: Vec<u8> = Vec::with_capacity(36);

	write_time!(buf, val1, &[32, 104, 111, 117, 114, 44, 32], &[32, 104, 111, 117, 114, 115, 44, 32]);
	write_time!(buf, val2, &[32, 109, 105, 110, 117, 116, 101, 44, 32, 97, 110, 100, 32], &[32, 109, 105, 110, 117, 116, 101, 115, 44, 32, 97, 110, 100, 32]);
	write_time!(buf, val3, LBL_SECOND, LBL_SECONDS);

	Cow::Owned(buf)
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
	if 1 == num {
		Cow::Borrowed(b"1 second")
	}
	// Just seconds.
	else if num < 60 {
		human_s(num)
	}
	// Just minutes and/or seconds.
	else if num < 3600 {
		let m: u32 = num_integer::div_floor(num, 60);
		let s: u32 = num - m * 60;

		if s > 0 { human_ms(m, s) }
		else { human_m(m) }
	}
	// Let's build it.
	else if num < 86400 {
		let c = secs_chunks(num);

		match (c[1] == 0, c[2] == 0) {
			// All Three.
			(false, false) => human_hms(c[0], c[1], c[2]),
			// Hour, Minute.
			(false, true) => human_hm(c[0], c[1]),
			// Hour, Second.
			(true, false) => human_hs(c[0], c[2]),
			// Only Hours.
			(true, true) => human_h(c[0]),
		}
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

		_human_elapsed(60, "1 minute");
		_human_elapsed(61, "1 minute and 1 second");
		_human_elapsed(100, "1 minute and 40 seconds");
		_human_elapsed(2101, "35 minutes and 1 second");
		_human_elapsed(2121, "35 minutes and 21 seconds");

		_human_elapsed(3600, "1 hour");
		_human_elapsed(3601, "1 hour and 1 second");
		_human_elapsed(3602, "1 hour and 2 seconds");
		_human_elapsed(3660, "1 hour and 1 minute");
		_human_elapsed(3661, "1 hour, 1 minute, and 1 second");
		_human_elapsed(3662, "1 hour, 1 minute, and 2 seconds");
		_human_elapsed(3720, "1 hour and 2 minutes");
		_human_elapsed(3721, "1 hour, 2 minutes, and 1 second");
		_human_elapsed(3723, "1 hour, 2 minutes, and 3 seconds");
		_human_elapsed(36001, "10 hours and 1 second");
		_human_elapsed(36015, "10 hours and 15 seconds");
		_human_elapsed(36060, "10 hours and 1 minute");
		_human_elapsed(37732, "10 hours, 28 minutes, and 52 seconds");
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
