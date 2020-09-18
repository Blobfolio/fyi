/*!
# FYI Msg: "Nice" Integer
*/

use crate::utility;
use std::{
	fmt,
	mem::{
		self,
		MaybeUninit,
	},
	ops::Deref,
	ptr,
};



/// # Integer Ceiling.
///
/// We have no need for formatting truly titanic numbers. The `NiceInt`
/// routines are capped at `999_999_999_999`, i.e. support exists for anything
/// under a trillion.
const MAX_NICE_INT: u64 = 999_999_999_999;



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// `NiceInt` provides a quick way to convert an integer — any unsigned value
/// under a trillion — into a formatted byte string for e.g. printing. Commas
/// are added for every thousand.
///
/// That's it!
///
/// ## Examples
///
/// ```no_run
/// use fyi_msg::NiceInt;
/// assert_eq!(
///     NiceInt::from(33231).as_str(),
///     "33,231"
/// );
pub struct NiceInt {
	inner: [u8; 15],
	len: usize,
}

impl Deref for NiceInt {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.inner[..self.len] }
}

impl Default for NiceInt {
	#[inline]
	fn default() -> Self {
		Self {
			inner: [48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			len: 1,
		}
	}
}

impl fmt::Display for NiceInt {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<u8> for NiceInt {
	#[allow(clippy::integer_division)]
	/// # From `u8`.
	///
	/// We can just defer to [`utility::write_u8`](super::utility::write_u8) for this.
	fn from(num: u8) -> Self {
		unsafe {
			let mut buf = [MaybeUninit::<u8>::uninit(); 15];
			let len: usize = utility::write_u8(buf.as_mut_ptr() as *mut u8, num);
			Self {
				inner: mem::transmute::<_, [u8; 15]>(buf),
				len
			}
		}
	}
}

impl From<u16> for NiceInt {
	#[allow(clippy::integer_division)]
	fn from(num: u16) -> Self {
		// Smaller integers have more efficient conversions.
		if num <= 255 { return Self::from(num as u8); }

		unsafe {
			let mut buf = [MaybeUninit::<u8>::uninit(); 15];

			let len: usize =
				// Number is in `256..=999`.
				if num < 1000 {
					utility::write_u8_3(buf.as_mut_ptr() as *mut u8, num);
					3
				}
				// Number is in `1000..=65_535`.
				else {
					let mut dst = buf.as_mut_ptr() as *mut u8;

					// Number is in `1000..=9999`.
					if num < 10_000 {
						ptr::write(dst, (num / 1000) as u8 | utility::MASK_U8);
						dst = dst.add(1);
					}
					// Number is in `10_000..=65_535`.
					else {
						utility::write_u8_2(dst, (num / 1000) as u8);
						dst = dst.add(2);
					}

					dst = write_comma(dst);

					// Whatever's left.
					utility::write_u8_3(dst, num % 1000);
					dst.add(3) as usize - buf.as_ptr() as *const u8 as usize
				};

			Self {
				inner: mem::transmute::<_, [u8; 15]>(buf),
				len
			}
		}
	}
}

impl From<u32> for NiceInt {
	#[allow(clippy::integer_division)]
	fn from(mut num: u32) -> Self {
		// Smaller integers have more efficient conversions.
		if num <= 255 { return Self::from(num as u8); }
		else if num <= 65_535 { return Self::from(num as u16); }

		unsafe {
			let mut buf = [MaybeUninit::<u8>::uninit(); 15];

			let len: usize = {
				let mut dst = buf.as_mut_ptr() as *mut u8;

				// Number is in `65_536..=99_999`.
				if num < 100_000 {
					utility::write_u8_2(dst, (num / 1_000) as u8);
					dst = write_comma(dst.add(2));
				}
				// Number is in `100_000..=999_999`.
				else if num < 1_000_000 {
					utility::write_u8_3(dst, (num / 1_000) as u16);
					dst = write_comma(dst.add(3));
				}
				// Number is in `1_000_000..=999_999_999`.
				else if num < 1_000_000_000 {
					// Number is in `1_000_000..=9_999_999`.
					if num < 10_000_000 {
						ptr::write(dst, (num / 1_000_000) as u8 | utility::MASK_U8);
						dst = dst.add(1);
					}
					// Number is in `10_000_000..=99_999_999`.
					else if num < 100_000_000 {
						utility::write_u8_2(dst, (num / 1_000_000) as u8);
						dst = dst.add(2);
					}
					// Number is in `100_000_000..=999_999_999`.
					else {
						utility::write_u8_3(dst, (num / 1_000_000) as u16);
						dst = dst.add(3);
					}

					num %= 1_000_000;
					dst = write_comma(dst);

					// Thousands and comma.
					utility::write_u8_3(dst, (num / 1_000) as u16);
					dst = write_comma(dst.add(3));
				}
				// Number is in `1_000_000_000..=4_294_967_295`.
				else {
					// Billions and comma.
					ptr::write(dst, (num / 1_000_000_000) as u8 | utility::MASK_U8);
					num %= 1_000_000_000;
					dst = write_comma(dst.add(1));

					// Millions and comma.
					utility::write_u8_3(dst, (num / 1_000_000) as u16);
					num %= 1_000_000;
					dst = write_comma(dst.add(3));

					// Thousands and comma.
					utility::write_u8_3(dst, (num / 1_000) as u16);
					dst = write_comma(dst.add(3));
				}

				// Whatever's left.
				utility::write_u8_3(dst, (num % 1_000) as u16);

				// How much did we write?
				dst.add(3) as usize - buf.as_ptr() as *const u8 as usize
			};

			Self {
				inner: mem::transmute::<_, [u8; 15]>(buf),
				len
			}
		}
	}
}

impl From<u64> for NiceInt {
	#[allow(clippy::integer_division)]
	fn from(num: u64) -> Self {
		// Smaller integers have more efficient conversions.
		if num <= 255 { return Self::from(num as u8); }
		else if num <= 65_535 { return Self::from(num as u16); }
		else if num <= 4_294_967_295 { return Self::from(num as u32); }
		// `NiceInt` don't support values in the trillions.
		else if num >= MAX_NICE_INT {
			return Self {
				inner: *b"999,999,999,999",
				len: 15,
			};
		}

		unsafe {
			let mut buf = [MaybeUninit::<u8>::uninit(); 15];

			let len: usize = {
				let mut dst = buf.as_mut_ptr() as *mut u8;

				// Number is in `4_294_967_296..=9_999_999_999`.
				if num < 10_000_000_000 {
					ptr::write(dst, (num / 1_000_000_000) as u8 | utility::MASK_U8);
					dst = dst.add(1);
				}
				// Number is in `10_000_000_000..=99_999_999_999`.
				else if num < 100_000_000_000 {
					utility::write_u8_2(dst, (num / 1_000_000_000) as u8);
					dst = dst.add(2);
				}
				// Number is in `100_000_000_000..=999_999_999_998`.
				else {
					utility::write_u8_3(dst, (num / 1_000_000_000) as u16);
					dst = dst.add(3);
				}

				// With billions out of the way, we can switch to `u32`.
				let mut num: u32 = (num % 1_000_000_000) as u32;
				dst = write_comma(dst);

				// Millions and comma.
				utility::write_u8_3(dst, (num / 1_000_000) as u16);
				num %= 1_000_000;
				dst = write_comma(dst.add(3));

				// Thousands and comma.
				utility::write_u8_3(dst, (num / 1_000) as u16);
				dst = write_comma(dst.add(3));

				// Whatever's left.
				utility::write_u8_3(dst, (num % 1_000) as u16);

				// How much did we write?
				dst.add(3) as usize - buf.as_ptr() as *const u8 as usize
			};

			Self {
				inner: mem::transmute::<_, [u8; 15]>(buf),
				len,
			}
		}
	}
}

impl From<usize> for NiceInt {
	#[inline]
	fn from(num: usize) -> Self { Self::from(num as u64) }
}



/// ## Casting.
///
/// This section provides methods for converting `NiceInt` instances into
/// other types.
///
/// Note: this struct can also be dereferenced to `&[u8]`.
impl NiceInt {
	#[must_use]
	#[inline]
	/// # As Bytes.
	///
	/// Return the value as a byte string.
	pub fn as_bytes(&self) -> &[u8] { self }

	#[must_use]
	#[inline]
	/// # As Str.
	///
	/// Return the value as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
	}
}

/// ## Doubles.
///
/// This section provides methods for nicely rendering floats, which are very
/// much not integers, at last after the decimal, but close enough.
impl NiceInt {
	#[must_use]
	/// Percent `f64`.
	///
	/// The value will be cast between `0..=1`, multiplied by `100`, and
	/// recorded with two additional decimal places.
	///
	/// ## Safety
	///
	/// It's fine; it just uses a lot of pointer writes.
	pub unsafe fn percent_f64(mut num: f64) -> Self {
		if num < 0.0 {
			num = 0.0;
		}
		else if 1.0 < num {
			num = 1.0;
		}

		// Write the integer parts.
		num *= 100.0;
		let mut out = Self::from(f64::floor(num) as u8);

		// Write the rest.
		let dst = out.inner.as_mut_ptr().add(out.len);
		ptr::write(dst, b'.');
		utility::write_u8_2(dst.add(1), f64::floor((num - f64::floor(num)) * 100.0) as u8);
		ptr::write(dst.add(3), b'%');
		out.len += 4;

		out
	}
}



#[inline]
/// # Write Comma.
///
/// This simply writes a comma to the specified pointer and returns a new
/// pointer advanced by one.
unsafe fn write_comma(buf: *mut u8) -> *mut u8 {
	ptr::write(buf, b',');
	buf.add(1)
}



#[cfg(test)]
mod tests {
	use super::*;
	use num_format::{ToFormattedString, Locale};

	#[test]
	fn t_nice_int_u8() {
		for i in 0..=u8::MAX {
			assert_eq!(
				NiceInt::from(i).as_str(),
				format!("{}", i),
			);
		}
	}

	#[test]
	fn t_nice_int_u16() {
		for i in 0..=u16::MAX {
			assert_eq!(
				NiceInt::from(i).as_str(),
				i.to_formatted_string(&Locale::en),
			);
		}
	}

	#[test]
	fn t_nice_int_u32() {
		for i in (999_999..=u32::MAX).step_by(10_000) {
			assert_eq!(
				NiceInt::from(i).as_str(),
				i.to_formatted_string(&Locale::en),
			);
		}
	}

	#[test]
	fn t_nice_int_u64() {
		for i in (999_999_999..=999_999_999_999_u64).step_by(100_000_000) {
			assert_eq!(
				NiceInt::from(i).as_str(),
				i.to_formatted_string(&Locale::en),
			);
		}
	}

	#[test]
	fn t_percent_f64() {
		unsafe {
			assert_eq!(NiceInt::percent_f64(0.0_f64).as_str(), "0.00%");
			assert_eq!(NiceInt::percent_f64(0.5656_f64).as_str(), "56.56%");
			assert_eq!(NiceInt::percent_f64(0.2_f64).as_str(), "20.00%");
			assert_eq!(NiceInt::percent_f64(0.18999_f64).as_str(), "18.99%");
			assert_eq!(NiceInt::percent_f64(1.0_f64).as_str(), "100.00%");
		}
	}
}
