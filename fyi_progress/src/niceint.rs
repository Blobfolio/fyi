/*!
# FYI Progress: "Nice" Integer

This is a quick way to convert an integer into a formatted byte string. That's
it!
*/

use std::{
	fmt,
	ops::Deref,
};



#[derive(Debug, Clone, Copy, Default, Hash, PartialEq)]
/// Nice Integer
///
/// This is a very simple partitioning table, each index — up to 15 —
/// representing an Exclude(end). The first "end" is always zero.
pub struct NiceInt {
	inner: [u8; 15],
	len: usize,
}



impl Deref for NiceInt {
	type Target = [u8];

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.inner[..self.len]
	}
}

impl fmt::Display for NiceInt {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(unsafe { std::str::from_utf8_unchecked(&*self) })
	}
}

impl From<u8> for NiceInt {
	#[inline]
	fn from(num: u8) -> Self {
		Self::from_small(u64::from(num))
	}
}

impl From<u16> for NiceInt {
	#[inline]
	fn from(num: u16) -> Self {
		if num < 1000 { Self::from_small(u64::from(num)) }
		else { Self::from_big(u64::from(num)) }
	}
}

impl From<u32> for NiceInt {
	#[inline]
	fn from(num: u32) -> Self {
		if num < 1000 { Self::from_small(u64::from(num)) }
		else { Self::from_big(u64::from(num)) }
	}
}

impl From<u64> for NiceInt {
	#[inline]
	fn from(num: u64) -> Self {
		if num < 1000 { Self::from_small(num) }
		else { Self::from_big(num) }
	}
}

impl From<u128> for NiceInt {
	#[inline]
	fn from(num: u128) -> Self {
		if num < 1000 { Self::from_small(num as u64) }
		else { Self::from_big(u64::min(999_999_999_999, num as u64)) }
	}
}

impl From<usize> for NiceInt {
	#[inline]
	fn from(num: usize) -> Self {
		if num < 1000 { Self::from_small(num as u64) }
		else { Self::from_big(num as u64) }
	}
}



impl NiceInt {
	/// From Small
	fn from_small(num: u64) -> Self {
		let mut out = Self::default();
		out.len = itoa::write(&mut out.inner[..], num).unwrap_or(0);
		out
	}

	/// From Big
	fn from_big(num: u64) -> Self {
		use num_format::WriteFormatted;
		let mut out = Self::default();
		out.len = (&mut out.inner[..]).write_formatted(&num, &num_format::Locale::en).unwrap_or(0);
		out
	}
}



#[test]
fn t_nice_int() {
	assert_eq!(&*NiceInt::from(1_u64), &b"1"[..]);
	assert_eq!(&*NiceInt::from(10_u64), &b"10"[..]);
	assert_eq!(&*NiceInt::from(1_000_u64), &b"1,000"[..]);
	assert_eq!(&*NiceInt::from(1_000_000_u64), &b"1,000,000"[..]);
	assert_eq!(&*NiceInt::from(6_884_372_993_u64), &b"6,884,372,993"[..]);
}
