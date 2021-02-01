/*!
# FYI Num: "Nice" Integer
*/

use crate::DOUBLE;

use std::{
	fmt,
	ops::Deref,
	ptr,
};



/// # Total Buffer Size.
const SIZE: usize = 26;

/// # Starting Index For Percentage Decimal.
const IDX_PERCENT_DECIMAL: usize = SIZE - 3;



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// `NiceInt` provides a quick way to convert an integer — any unsigned value
/// up to `u64::MAX` — into a formatted byte string for e.g. printing. Commas
/// are added for every thousand.
///
/// That's it!
///
/// ## Examples
///
/// ```no_run
/// use fyi_num::NiceInt;
/// assert_eq!(
///     NiceInt::from(33231).as_str(),
///     "33,231"
/// );
pub struct NiceInt {
	inner: [u8; SIZE],
	from: usize,
}

impl Deref for NiceInt {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.inner[self.from..] }
}

impl Default for NiceInt {
	#[inline]
	fn default() -> Self {
		Self {
			inner: [0, 0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0,],
			from: SIZE,
		}
	}
}

impl fmt::Display for NiceInt {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<u8> for NiceInt {
	fn from(num: u8) -> Self {
		unsafe {
			let mut out = Self::default();
			let ptr = out.inner.as_mut_ptr();

			if num >= 100 {
				out.from -= 3;
				write_u8_3(ptr.add(out.from), usize::from(num));
			}
			else if num >= 10 {
				out.from -= 2;
				write_u8_2(ptr.add(out.from), usize::from(num));
			}
			else {
				out.from -= 1;
				write_u8_1(ptr.add(out.from), usize::from(num));
			}

			out
		}
	}
}

impl From<u16> for NiceInt {
	fn from(mut num: u16) -> Self {
		unsafe {
			let mut out = Self::default();
			let ptr = out.inner.as_mut_ptr();

			// For `u16` this can only trigger once.
			if num >= 1000 {
				let (div, rem) = num_integer::div_mod_floor(num, 1000);
				write_u8_3(ptr.add(out.from - 3), usize::from(rem));
				num = div;
				out.from -= 4;
			}

			if num >= 100 {
				out.from -= 3;
				write_u8_3(ptr.add(out.from), usize::from(num));
			}
			else if num >= 10 {
				out.from -= 2;
				write_u8_2(ptr.add(out.from), usize::from(num));
			}
			else {
				out.from -= 1;
				write_u8_1(ptr.add(out.from), usize::from(num));
			}

			out
		}
	}
}

impl From<u32> for NiceInt {
	fn from(num: u32) -> Self {
		// Skip all the index casts.
		Self::from(num as usize)
	}
}

impl From<u64> for NiceInt {
	#[cfg(target_pointer_width = "64")]
	fn from(num: u64) -> Self {
		// Skip all the index casts.
		Self::from(num as usize)
	}

	#[cfg(not(target_pointer_width = "64"))]
	fn from(mut num: u64) -> Self {
		unsafe {
			let mut out = Self::default();
			let ptr = out.inner.as_mut_ptr();

			while num >= 1000 {
				let (div, rem) = num_integer::div_mod_floor(num, 1000);
				write_u8_3(ptr.add(out.from - 3), usize::from(rem));
				num = div;
				out.from -= 4;
			}

			if num >= 100 {
				out.from -= 3;
				write_u8_3(ptr.add(out.from), num as usize);
			}
			else if num >= 10 {
				out.from -= 2;
				write_u8_2(ptr.add(out.from), num as usize);
			}
			else {
				out.from -= 1;
				write_u8_1(ptr.add(out.from), num as usize);
			}

			out
		}
	}
}

impl From<usize> for NiceInt {
	fn from(mut num: usize) -> Self {
		unsafe {
			let mut out = Self::default();
			let ptr = out.inner.as_mut_ptr();

			while num >= 1000 {
				let (div, rem) = num_integer::div_mod_floor(num, 1000);
				write_u8_3(ptr.add(out.from - 3), rem);
				num = div;
				out.from -= 4;
			}

			if num >= 100 {
				out.from -= 3;
				write_u8_3(ptr.add(out.from), num);
			}
			else if num >= 10 {
				out.from -= 2;
				write_u8_2(ptr.add(out.from), num);
			}
			else {
				out.from -= 1;
				write_u8_1(ptr.add(out.from), num);
			}

			out
		}
	}
}



/// ## Miscellaneous.
///
/// This section contains a few random odds and ends.
impl NiceInt {
	#[must_use]
	#[inline]
	/// # Is Empty.
	///
	/// Returns true if the struct is uninitialized.
	///
	/// Note: a value of "0" would not be empty.
	pub const fn is_empty(&self) -> bool { self.from == SIZE }

	#[must_use]
	#[inline]
	/// # Is Zero.
	///
	/// Returns true if the value is equivalent to "0".
	pub const fn is_zero(&self) -> bool { self.len() == 1 && self.inner[SIZE - 1] == 48 }

	#[must_use]
	#[inline]
	/// # Length.
	///
	/// Return the byte length of the value.
	pub const fn len(&self) -> usize { SIZE - self.from }
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
/// much not integers, at least after the decimal, but close enough.
impl NiceInt {
	#[must_use]
	/// # Percent `f64`.
	///
	/// The value will be cast between `0..=1`, multiplied by `100`, and
	/// recorded with two additional decimal places. The end result will look
	/// something like `78.03%`.
	///
	/// ## Safety
	///
	/// It's fine; this method just uses a lot of pointer writes.
	pub unsafe fn percent_f64(mut num: f64) -> Self {
		// Shortcut for overflowing values.
		if num <= 0.0 {
			return Self {
				inner: *b"0000000000000000000000.00%",
				from: SIZE - 5,
			};
		}
		else if 1.0 <= num {
			return Self {
				inner: *b"0000000000000000000100.00%",
				from: SIZE - 7,
			};
		}

		// Start with the bits we know.
		let mut out = Self {
			inner: *b"0000000000000000000000.00%",
			from: SIZE - 4,
		};
		let ptr = out.inner.as_mut_ptr();

		// Write the integer parts.
		num *= 100.0;
		let base = f64::floor(num);

		if base >= 10.0 {
			out.from -= 2;
			write_u8_2(ptr.add(out.from), base as usize);
		}
		else {
			out.from -= 1;
			write_u8_1(ptr.add(out.from), num as usize);
		}

		// Write the rest.
		write_u8_2(
			ptr.add(IDX_PERCENT_DECIMAL),
			f64::floor((num - base) * 100.0) as usize
		);

		out
	}
}



/// # Write `u8` x 3
///
/// ## Safety
///
/// The destination pointer must have at least 3 bytes free or undefined
/// things may happen!
unsafe fn write_u8_3(buf: *mut u8, num: usize) {
	let (div, rem) = num_integer::div_mod_floor(num, 100);
	let ptr = DOUBLE.as_ptr();
	ptr::copy_nonoverlapping(ptr.add((div << 1) + 1), buf, 1);
	ptr::copy_nonoverlapping(ptr.add(rem << 1), buf.add(1), 2);
}

/// # Write `u8` x 2
///
/// ## Safety
///
/// The destination pointer must have at least 2 bytes free or undefined
/// things may happen!
unsafe fn write_u8_2(buf: *mut u8, num: usize) {
	ptr::copy_nonoverlapping(DOUBLE.as_ptr().add(num << 1), buf, 2);
}

/// # Write `u8` x 1
///
/// ## Safety
///
/// The destination pointer must have at least 1 byte free or undefined
/// things may happen!
unsafe fn write_u8_1(buf: *mut u8, num: usize) {
	ptr::copy_nonoverlapping(DOUBLE.as_ptr().add((num << 1) + 1), buf, 1);
}



#[cfg(test)]
mod tests {
	use super::*;
	use num_format::{ToFormattedString, Locale};
	use rand::Rng;

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
		let mut rng = rand::thread_rng();

		for _ in 0..1_000_000 {
			let num: u32 = rng.gen();
			assert_eq!(
				NiceInt::from(num).as_str(),
				num.to_formatted_string(&Locale::en),
			);
		}

		assert_eq!(
			NiceInt::from(u32::MAX).as_str(),
			u32::MAX.to_formatted_string(&Locale::en),
		);
	}

	#[test]
	fn t_nice_int_u64() {
		let mut rng = rand::thread_rng();

		for _ in 0..1_000_000 {
			let num: u64 = rng.gen();
			assert_eq!(
				NiceInt::from(num).as_str(),
				num.to_formatted_string(&Locale::en),
			);
		}

		assert_eq!(
			NiceInt::from(u64::MAX).as_str(),
			u64::MAX.to_formatted_string(&Locale::en),
		);
	}

	#[test]
	fn t_other() {
		let mut tmp = NiceInt::from(10_usize);
		assert_eq!(tmp.is_empty(), false);
		assert_eq!(tmp.len(), 2);
		assert_eq!(tmp.is_zero(), false);

		tmp = NiceInt::from(10_000_usize);
		assert_eq!(tmp.is_empty(), false);
		assert_eq!(tmp.len(), 6);
		assert_eq!(tmp.is_zero(), false);

		tmp = NiceInt::default();
		assert_eq!(tmp.is_empty(), true);
		assert_eq!(tmp.len(), 0);
		assert_eq!(tmp.is_zero(), false);

		tmp = NiceInt::from(0_usize);
		assert_eq!(tmp.is_empty(), false);
		assert_eq!(tmp.len(), 1);
		assert_eq!(tmp.is_zero(), true);
	}

	#[test]
	fn t_percent_f64() {
		unsafe {
			assert_eq!(NiceInt::percent_f64(-30.0_f64).as_str(), "0.00%");
			assert_eq!(NiceInt::percent_f64(0.0_f64).as_str(), "0.00%");
			assert_eq!(NiceInt::percent_f64(0.5656_f64).as_str(), "56.56%");
			assert_eq!(NiceInt::percent_f64(0.2_f64).as_str(), "20.00%");
			assert_eq!(NiceInt::percent_f64(0.18999_f64).as_str(), "18.99%");
			assert_eq!(NiceInt::percent_f64(1.0_f64).as_str(), "100.00%");
			assert_eq!(NiceInt::percent_f64(1.1_f64).as_str(), "100.00%");
		}
	}
}
