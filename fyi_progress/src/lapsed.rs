/*!
# FYI Progress: Lapsed.
*/

use std::borrow::Cow;
use fyi_msg::{
	Timestamp,
	utility,
};



#[must_use]
/// Elapsed Short.
pub fn compact(num: u32) -> Cow<'static, [u8]> {
	static ZERO: &[u8] = b"00:00:00";

	if 0 == num {
		Cow::Borrowed(ZERO)
	}
	// Under a day means 3 pairs.
	else if num < 86400 {
		Cow::Owned({
			let c = secs_chunks(num);
			let mut buf: Vec<u8> = ZERO.to_vec();
			utility::slice_swap(&mut buf[0..2], Timestamp::time_format_dd(c[0]));
			utility::slice_swap(&mut buf[3..5], Timestamp::time_format_dd(c[1]));
			utility::slice_swap(&mut buf[6..8], Timestamp::time_format_dd(c[2]));
			buf
		})
	}
	else {
		Cow::Borrowed(b"23:59:59")
	}
}

#[must_use]
/// Elapsed.
pub fn full(num: u32) -> Cow<'static, [u8]> {
	static ONE: [&[u8]; 3] = [b" hour", b" minute", b" second"];
	static MANY: [&[u8]; 3] = [b" hours", b" minutes", b" seconds"];
	static AND: &[u8] = b", and ";

	if 1 == num {
		Cow::Borrowed(b"1 second")
	}
	// Just seconds.
	else if num < 60 {
		Cow::Owned({
			let mut buf: Vec<u8> = Vec::with_capacity(MANY[2].len() + 2);
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
/// Elapsed Chunks
///
/// Return a fixed array containing the number of days, hours,
/// minutes, and seconds.
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



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_full() {
		_full(0, "0 seconds");
		_full(1, "1 second");
		_full(50, "50 seconds");
		_full(100, "1 minute and 40 seconds");
		_full(2121, "35 minutes and 21 seconds");
		_full(36015, "10 hours and 15 seconds");
		_full(37732, "10 hours, 28 minutes, and 52 seconds");
		_full(37740, "10 hours and 29 minutes");
		_full(37740, "10 hours and 29 minutes");
		_full(428390, "1+ days");
	}

	fn _full(num: u32, expected: &str) {
		assert_eq!(
			full(num).as_ref(),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}

	#[test]
	fn t_compact() {
		_compact(0, "00:00:00");
		_compact(1, "00:00:01");
		_compact(50, "00:00:50");
		_compact(100, "00:01:40");
		_compact(2121, "00:35:21");
		_compact(37732, "10:28:52");
		_compact(428390, "23:59:59");
	}

	fn _compact(num: u32, expected: &str) {
		assert_eq!(
			compact(num).as_ref(),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}
}
