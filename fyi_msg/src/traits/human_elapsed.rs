/*!
# FYI Msg Traits: `HumanTime`

This is a very simple trait for converting seconds (`u64`) into human-readable
UTF-8, either as a `Cow<str>` or `Cow<[u8]>` depending on the implementation.

This trait exposes two methods:
* `::human_elapsed()`, which returns an oxford-joined string like "1 minute and 2 seconds" or "3 days, 10 minutes, and 15 seconds".
* `::human_elapsed_short()`, which returns a compact clock-like format, "00:00:00". Days are included only if there are any.

## Example:

```no_run
use fyi_msg::traits::HumanElapsed;

let words = <[u8]>::human_elapsed(100);
assert_eq!(words, b"1 minute and 40 seconds");

let short = <[u8]>::human_elapsed_short(100);
assert_eq!(words, b"00:01:40");
```
*/

use crate::traits::DoubleTime;
use std::borrow::Cow;



/// Human-Readable Elapsed.
pub trait HumanElapsed<'elapsed> {
	/// Output target.
	type Target;

	/// Human-Readable Elapsed Time
	///
	/// This turns seconds into a human list like 1 hour and 2 seconds.
	fn human_elapsed(num: u64) -> Self::Target;

	/// Elapsed Time (Compact)
	///
	/// This turns seconds into a 00:00:00-style display. Days are included
	/// only if positive.
	fn human_elapsed_short(num: u64) -> Self::Target;
}

impl<'elapsed> HumanElapsed<'elapsed> for [u8] {
	/// Output target.
	type Target = Cow<'elapsed, [u8]>;

	/// Human-Readable Elapsed Time
	fn human_elapsed (num: u64) -> Self::Target {
		lazy_static::lazy_static! {
			static ref ELAPSED_ONE: [&'static [u8]; 4] = [
				b" day",
				b" hour",
				b" minute",
				b" second",
			];
			static ref ELAPSED_MANY: [&'static [u8]; 4] = [
				b" days",
				b" hours",
				b" minutes",
				b" seconds",
			];
		}

		if 0 == num {
			Cow::Borrowed(b"0 seconds")
		}
		else if 1 == num {
			Cow::Borrowed(b"1 second")
		}
		else if num < 60 {
			Cow::Owned({
				let mut buf: Vec<u8> = Vec::with_capacity(ELAPSED_MANY[3].len() + 2);
				itoa::write(&mut buf, num).unwrap();
				buf.extend_from_slice(ELAPSED_MANY[3]);
				buf
			})
		}
		else {
			let c = chunks(num);
			let len: usize = c.iter().filter(|&n| *n != 0).count();
			assert!(len > 0);

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
					1 => buf.extend_from_slice(ELAPSED_ONE[i]),
					_ => buf.extend_from_slice(ELAPSED_MANY[i]),
				}

				i += 1;
				j += 1;

				if j == len {
					break;
				}
				else if len - j == 1 {
					if len > 2 {
						buf.extend_from_slice(b", and ");
					}
					else {
						buf.extend_from_slice(b" and ");
					}
				}
				else {
					buf.extend_from_slice(b", ");
				}
			}

			Cow::Owned(buf)
		}
	}

	/// Elapsed Time (Compact)
	fn human_elapsed_short(num: u64) -> Self::Target {
		if 0 == num {
			Cow::Borrowed(b"00:00:00")
		}
		// Under a minute.
		else if num < 60 {
			Cow::Owned([
				b"00:00:",
				Self::double_digit_time(num),
			].concat())
		}
		// Under an hour.
		else if num < 3600 {
			let c = chunks(num);
			Cow::Owned([
				b"00:",
				Self::double_digit_time(c[2]),
				b":",
				Self::double_digit_time(c[3]),
			].concat())
		}
		// Under a day.
		else if num < 86400 {
			let c = chunks(num);
			Cow::Owned([
				Self::double_digit_time(c[1]),
				b":",
				Self::double_digit_time(c[2]),
				b":",
				Self::double_digit_time(c[3]),
			].concat())
		}
		// Under 60 days.
		else if num < 5_184_000 {
			let c = chunks(num);
			Cow::Owned([
				Self::double_digit_time(c[0]),
				b":",
				Self::double_digit_time(c[1]),
				b":",
				Self::double_digit_time(c[2]),
				b":",
				Self::double_digit_time(c[3]),
			].concat())
		}
		// Above 60 days, we need to write the days directly to the buffer.
		else {
			use std::io::Write;
			let c = chunks(num);

			let mut out: Vec<u8> = Vec::with_capacity(11);
			itoa::write(&mut out, c[0]).unwrap();
			write!(
				out,
				":{}:{}:{}",
				str::double_digit_time(c[1]),
				str::double_digit_time(c[2]),
				str::double_digit_time(c[3]),
			).expect("Bad time.");
			Cow::Owned(out)
		}
	}
}

impl<'elapsed> HumanElapsed<'elapsed> for str {
	/// Output target.
	type Target = Cow<'elapsed, str>;

	/// Human-Readable Elapsed Time
	fn human_elapsed (num: u64) -> Self::Target {
		// Vecs handle this annoying logic better.
		Cow::Owned(unsafe {
			String::from_utf8_unchecked(<[u8]>::human_elapsed(num).to_owned().to_vec())
		})
	}

	/// Elapsed Time (Compact)
	fn human_elapsed_short(num: u64) -> Self::Target {
		if 0 == num {
			Cow::Borrowed("00:00:00")
		}
		// Under a minute.
		else if num < 60 {
			Cow::Owned([
				"00:00:",
				Self::double_digit_time(num),
			].concat())
		}
		// Under an hour.
		else if num < 3600 {
			let c = chunks(num);
			Cow::Owned([
				"00:",
				Self::double_digit_time(c[2]),
				":",
				Self::double_digit_time(c[3]),
			].concat())
		}
		// Under a day.
		else if num < 86400 {
			let c = chunks(num);
			Cow::Owned([
				Self::double_digit_time(c[1]),
				":",
				Self::double_digit_time(c[2]),
				":",
				Self::double_digit_time(c[3]),
			].concat())
		}
		// Under 60 days.
		else if num < 5_184_000 {
			let c = chunks(num);
			Cow::Owned([
				Self::double_digit_time(c[0]),
				":",
				Self::double_digit_time(c[1]),
				":",
				Self::double_digit_time(c[2]),
				":",
				Self::double_digit_time(c[3]),
			].concat())
		}
		// Above 60 days, we need to write the days directly to the buffer.
		else {
			use std::fmt::Write;
			let c = chunks(num);

			let mut out: String = String::with_capacity(11);
			itoa::fmt(&mut out, c[0]).unwrap();
			write!(
				out,
				":{}:{}:{}",
				str::double_digit_time(c[1]),
				str::double_digit_time(c[2]),
				str::double_digit_time(c[3]),
			).expect("Bad time.");
			Cow::Owned(out)
		}
	}
}

#[must_use]
/// Elapsed Chunks
///
/// Return a fixed array containing the number of days, hours,
/// minutes, and seconds.
pub fn chunks(num: u64) -> [u64; 4] {
	let mut out: [u64; 4] = [0, 0, 0, num];

	// Days.
	if out[3] >= 86400 {
		out[0] = num_integer::div_floor(out[3], 86400);
		out[3] -= out[0] * 86400;
	}

	// Hours.
	if out[3] >= 3600 {
		out[1] = num_integer::div_floor(out[3], 3600);
		out[3] -= out[1] * 3600;
	}

	// Minutes.
	if out[3] >= 60 {
		out[2] = num_integer::div_floor(out[3], 60);
		out[3] -= out[2] * 60;
	}

	out
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn human_elapsed() {
		_human_elapsed(0, "0 seconds");
		_human_elapsed(1, "1 second");
		_human_elapsed(50, "50 seconds");
		_human_elapsed(100, "1 minute and 40 seconds");
		_human_elapsed(2121, "35 minutes and 21 seconds");
		_human_elapsed(37732, "10 hours, 28 minutes, and 52 seconds");
		_human_elapsed(428390, "4 days, 22 hours, 59 minutes, and 50 seconds");
		_human_elapsed(5847294, "67 days, 16 hours, 14 minutes, and 54 seconds");
	}

	fn _human_elapsed(num: u64, expected: &str) {
		assert_eq!(
			<[u8]>::human_elapsed(num).as_ref(),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected.as_bytes()
		);

		assert_eq!(
			str::human_elapsed(num).as_ref(),
			expected,
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}

	#[test]
	fn human_elapsed_short() {
		_human_elapsed_short(0, "00:00:00");
		_human_elapsed_short(1, "00:00:01");
		_human_elapsed_short(50, "00:00:50");
		_human_elapsed_short(100, "00:01:40");
		_human_elapsed_short(2121, "00:35:21");
		_human_elapsed_short(37732, "10:28:52");
		_human_elapsed_short(428390, "04:22:59:50");
		_human_elapsed_short(5847294, "67:16:14:54");
	}

	fn _human_elapsed_short(num: u64, expected: &str) {
		assert_eq!(
			<[u8]>::human_elapsed_short(num).as_ref(),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected.as_bytes()
		);

		assert_eq!(
			str::human_elapsed_short(num).as_ref(),
			expected,
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}
}
