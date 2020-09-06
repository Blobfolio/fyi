/*!
# FYI Witcher: "Nice" Integer
*/

use std::{
	fmt,
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
	fn from(num: u8) -> Self {
		Self::from_small(num)
	}
}

impl From<u16> for NiceInt {
	fn from(num: u16) -> Self {
		if num < 1000 { Self::from_small(num) }
		else { Self::from_big(&num) }
	}
}

impl From<u32> for NiceInt {
	fn from(num: u32) -> Self {
		if num < 1000 { Self::from_small(num) }
		else { Self::from_big(&num) }
	}
}

impl From<u64> for NiceInt {
	fn from(num: u64) -> Self {
		if num < 1000 { Self::from_small(num) }
		else { Self::from_big(&999_999_999_999.min(num)) }
	}
}

impl From<u128> for NiceInt {
	fn from(num: u128) -> Self {
		if num < 1000 { Self::from_small(num) }
		else { Self::from_big(&999_999_999_999.min(num)) }
	}
}

impl From<usize> for NiceInt {
	fn from(num: usize) -> Self {
		if num < 1000 { Self::from_small(num) }
		else { Self::from_big(&999_999_999_999.min(num)) }
	}
}



impl NiceInt {
	/// # From Small
	///
	/// For numbers less than `1000`, we can skip the overhead of figuring out
	/// punctuation and just leverage `itoa`.
	fn from_small<N>(num: N) -> Self
	where N: itoa::Integer {
		let mut out = Self::default();
		out.len = itoa::write(&mut out.inner[..], num).unwrap_or_default();
		out
	}

	/// # From Big
	///
	/// For numbers greater or equal to `1000`, commas come into play,
	/// requiring the use of the relatively heavier `num_format` crate.
	fn from_big<N>(num: &N) -> Self
	where N: num_format::ToFormattedString {
		use num_format::WriteFormatted;
		let mut out = Self::default();
		out.len = (&mut out.inner[..]).write_formatted(num, &num_format::Locale::en).unwrap_or_default();
		out
	}

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



#[test]
fn t_nice_int() {
	use num_format::{ToFormattedString, Locale};

	for i in [
		1_u64,
		10_u64,
		99_u64,
		100_u64,
		101_u64,
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
		assert_eq!(&*NiceInt::from(*i), i.to_formatted_string(&Locale::en).as_bytes());
	}
}
