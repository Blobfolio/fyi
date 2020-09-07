/*!
# FYI Witcher: "Nice" Integer
*/

use std::{
	fmt,
	mem::{
		self,
		MaybeUninit,
	},
	ops::Deref,
};



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// `NiceInt` provides a quick way to convert an integer — any unsigned value
/// under a trillion — into a formatted byte string for e.g. printing.
///
/// That's it!
///
/// For values under `1000`, the [`itoa`](https://crates.io/crates/itoa) crate is used;
/// for values requiring punctuation — i.e. US thousands separators — [`num_format`](https://crates.io/crates/num_format)
/// is used instead.
///
/// Both are much faster than relying on `to_string()` or the like.
///
/// ## Examples
///
/// ```no_run
/// use fyi_witcher::NiceInt;
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
	fn deref(&self) -> &Self::Target { self.as_bytes() }
}

impl Default for NiceInt {
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
	/// # From `u8`
	///
	/// `u8`s are small enough we can just brute-force the answer with a small
	/// conditional.
	fn from(mut num: u8) -> Self {
		unsafe {
			let mut buf = [MaybeUninit::<u8>::uninit(); 15];
			let dst = buf.as_mut_ptr() as *mut u8;

			let len: usize =
				if num >= 100 {
					if num >= 200 {
						dst.write(50_u8);
						num -= 200;
					}
					else {
						dst.write(49_u8);
						num -= 100;
					}

					if num >= 10 {
						dst.add(1).write(num / 10 + 48);
						dst.add(2).write(num % 10 + 48);
					}
					else {
						dst.add(1).write(48_u8);
						dst.add(2).write(num + 48);
					}

					3
				}
				else if num >= 10 {
					dst.write(num / 10 + 48);
					dst.add(1).write(num % 10 + 48);
					2
				}
				else {
					dst.write(num + 48);
					1
				};

			Self {
				inner: mem::transmute::<_, [u8; 15]>(buf),
				len
			}
		}
	}
}

impl From<u16> for NiceInt {
	fn from(num: u16) -> Self {
		unsafe { from_int(num) }
	}
}

impl From<u32> for NiceInt {
	fn from(num: u32) -> Self {
		unsafe { from_int(num) }
	}
}

impl From<u64> for NiceInt {
	fn from(num: u64) -> Self {
		unsafe { from_int(999_999_999_999.min(num)) }
	}
}

impl From<usize> for NiceInt {
	fn from(num: usize) -> Self {
		unsafe { from_int(999_999_999_999.min(num)) }
	}
}

impl From<u128> for NiceInt {
	fn from(num: u128) -> Self {
		unsafe { from_int(999_999_999_999.min(num)) }
	}
}

impl NiceInt {
	#[must_use]
	/// # As Bytes.
	///
	/// Return the value as a byte string.
	pub fn as_bytes(&self) -> &[u8] { &self.inner[..self.len] }

	#[must_use]
	/// # As Str.
	///
	/// Return the value as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
	}
}

/// # From Num.
///
/// Everything other than `u8` works the same way.
///
/// ## Safety
///
/// This is only used privately so all starting conditions are sane and safe.
unsafe fn from_int<N>(num: N) -> NiceInt
where N: itoap::Integer {
	let mut buf = [MaybeUninit::<u8>::uninit(); 15];
	let dst = buf.as_mut_ptr() as *mut u8;

	// Write the number.
	let mut len: usize = itoap::write_to_ptr(dst, num);
	// Add the commas.
	len += insert_commas(dst, len);

	NiceInt {
		inner: mem::transmute::<_, [u8; 15]>(buf),
		len
	}
}

/// # Insert Commas.
///
/// This inserts comma separators into an ASCII-fied number byte string,
/// turning values like "1000" into "1,000".
///
/// Because our `NiceInt` behaviors are capped at `999,999,999,999`, we can
/// handle this semi-manually.
///
/// The number of extra bytes allocated for commas, if any, are returned.
///
/// ## Safety
///
/// This is only used privately so all starting conditions are sane and safe.
unsafe fn insert_commas(src: *mut u8, len: usize) -> usize {
	use std::ptr;

	// We need 3 commas.
	if len > 9 {
		ptr::copy(src.add(len - 9), src.add(len - 8), 9);
		src.add(len - 9).write(b',');

		ptr::copy(src.add(len - 5), src.add(len - 4), 6);
		src.add(len - 5).write(b',');

		ptr::copy(src.add(len - 1), src.add(len), 3);
		src.add(len - 1).write(b',');

		3
	}
	else if len > 6 {
		ptr::copy(src.add(len - 6), src.add(len - 5), 6);
		src.add(len - 6).write(b',');

		ptr::copy(src.add(len - 2), src.add(len - 1), 3);
		src.add(len - 2).write(b',');

		2
	}
	else if len > 3 {
		ptr::copy(src.add(len - 3), src.add(len - 2), 3);
		src.add(len - 3).write(b',');

		1
	}
	else { 0 }
}



#[test]
fn t_nice_int() {
	use num_format::{ToFormattedString, Locale};

	for i in [
		1_u64,
		5_u64,
		9_u64,
		10_u64,
		98_u64,
		99_u64,
		100_u64,
		101_u64,
		678_u64,
		999_u64,
		1000_u64,
		1001_u64,
		9999_u64,
		10000_u64,
		10001_u64,
		99999_u64,
		100000_u64,
		100001_u64,
		999999_u64,
		1000000_u64,
		1000001_u64,
		9999999_u64,
		10000000_u64,
		10000001_u64,
		99999999_u64,
		100000000_u64,
		100000001_u64,
		999999999_u64,
		1000000000_u64,
		1000000001_u64,
		9999999999_u64,
		10000000000_u64,
		10000000001_u64,
		99999999999_u64,
		100000000000_u64,
		100000000001_u64,
	].iter() {
		assert_eq!(&*NiceInt::from(*i), i.to_formatted_string(&Locale::en).as_bytes(), "{:?}", *i);
	}
}
