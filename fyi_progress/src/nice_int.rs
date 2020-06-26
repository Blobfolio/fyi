/*!
# FYI Progress: "Nice" Integer

This is a quick way to convert an integer — any unsigned value under a trillion
— into a formatted byte string. That's it!
*/

use std::{
	fmt,
	ops::Deref,
};



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// Nice Integer
pub struct NiceInt {
	inner: [u8; 15],
	len: usize,
}



impl Deref for NiceInt {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.inner[..self.len]
	}
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
		f.write_str(unsafe { std::str::from_utf8_unchecked(&*self) })
	}
}

impl From<u8> for NiceInt {
	fn from(num: u8) -> Self {
		Self::from_small(u64::from(num))
	}
}

impl From<u16> for NiceInt {
	fn from(num: u16) -> Self {
		if num < 1000 { Self::from_small(u64::from(num)) }
		else { Self::from_big(u64::from(num)) }
	}
}

impl From<u32> for NiceInt {
	fn from(num: u32) -> Self {
		if num < 1000 { Self::from_small(u64::from(num)) }
		else { Self::from_big(u64::from(num)) }
	}
}

impl From<u64> for NiceInt {
	fn from(num: u64) -> Self {
		if num < 1000 { Self::from_small(num) }
		else { Self::from_big(u64::min(999_999_999_999, num)) }
	}
}

impl From<u128> for NiceInt {
	fn from(num: u128) -> Self {
		if num < 1000 { Self::from_small(num as u64) }
		else { Self::from_big(u64::min(999_999_999_999, num as u64)) }
	}
}

impl From<usize> for NiceInt {
	fn from(num: usize) -> Self {
		if num < 1000 { Self::from_small(num as u64) }
		else { Self::from_big(u64::min(999_999_999_999, num as u64)) }
	}
}



impl NiceInt {
	/// From Small
	fn from_small(num: u64) -> Self {
		let mut out = Self::default();
		out.len = itoa::write(&mut out.inner[..], num).unwrap_or_default();
		out
	}

	/// From Big
	fn from_big(num: u64) -> Self {
		use num_format::WriteFormatted;
		let mut out = Self::default();
		out.len = (&mut out.inner[..]).write_formatted(&num, &num_format::Locale::en).unwrap_or_default();
		out
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
