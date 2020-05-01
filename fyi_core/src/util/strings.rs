/*!
# FYI Core: Strings
*/

use num_traits::cast::AsPrimitive;
use std::{
	borrow::Cow,
	ffi::{
		OsStr,
		OsString,
	},
};



#[inline]
/// From `OsStr(ing)`.
pub fn from_os_string<S> (text: S) -> String
where S: Into<OsString> {
	text.into().to_str().unwrap_or("").to_string()
}

#[inline]
/// To `OsString`.
pub fn to_os_string<S> (text: S) -> OsString
where S: AsRef<str> {
	OsStr::new(text.as_ref()).to_os_string()
}

/// Make whitespace.
///
/// Generate a string consisting of X spaces.
pub fn whitespace<N> (count: N) -> Cow<'static, str>
where N: AsPrimitive<usize> {
	lazy_static::lazy_static! {
		// Precompute 100 spaces; it is cheaper to shrink than to grow.
		static ref WHITE: Cow<'static, str> = Cow::Owned("                                                                                                    ".to_string());
	}

	let count: usize = count.as_();
	if 0 == count {
		"".into()
	}
	else if count <= 100 {
		WHITE[0..count].into()
	}
	else {
		unsafe { String::from_utf8_unchecked(vec![b' '; count]).into() }
	}
}

/// Make whitespace.
///
/// Generate a string consisting of X spaces.
pub fn whitespace_bytes<N> (count: N) -> Cow<'static, [u8]>
where N: AsPrimitive<usize> {
	lazy_static::lazy_static! {
		// Precompute 100 spaces; it is cheaper to shrink than to grow.
		static ref WHITE: Cow<'static, [u8]> = Cow::Owned(vec![b' '; 100]);
	}

	let count: usize = count.as_();
	if 0 == count {
		vec![].into()
	}
	else if count <= 100 {
		WHITE[0..count].into()
	}
	else {
		vec![b' '; count].into()
	}
}

#[must_use]
/// Zero-padded Time Digits As Bytes
///
/// This converts a u32 — i.e. what `Chrono` returns — into a utf8 byte
/// representation. This method is probably not very useful outside
/// the `Msg` struct's timestamp printing.
pub fn zero_padded_time_bytes(num: u32) -> &'static [u8] {
	match num {
		0 => b"00",
		1 => b"01",
		2 => b"02",
		3 => b"03",
		4 => b"04",
		5 => b"05",
		6 => b"06",
		7 => b"07",
		8 => b"08",
		9 => b"09",
		10 => b"10",
		11 => b"11",
		12 => b"12",
		13 => b"13",
		14 => b"14",
		15 => b"15",
		16 => b"16",
		17 => b"17",
		18 => b"18",
		19 => b"19",
		20 => b"20",
		21 => b"21",
		22 => b"22",
		23 => b"23",
		24 => b"24",
		25 => b"25",
		26 => b"26",
		27 => b"27",
		28 => b"28",
		29 => b"29",
		30 => b"30",
		31 => b"31",
		32 => b"32",
		33 => b"33",
		34 => b"34",
		35 => b"35",
		36 => b"36",
		37 => b"37",
		38 => b"38",
		39 => b"39",
		40 => b"40",
		41 => b"41",
		42 => b"42",
		43 => b"43",
		44 => b"44",
		45 => b"45",
		46 => b"46",
		47 => b"47",
		48 => b"48",
		49 => b"49",
		50 => b"50",
		51 => b"51",
		52 => b"52",
		53 => b"53",
		54 => b"54",
		55 => b"55",
		56 => b"56",
		57 => b"57",
		58 => b"58",
		59 => b"59",
		_ => b"",
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn os_string() {
		let test_str: &OsStr = OsStr::new("Hello World!");
		let test_string: OsString = test_str.to_os_string();

		assert_eq!(super::from_os_string(&test_str), "Hello World!".to_string());
		assert_eq!(super::to_os_string("Hello World!"), test_string);
	}

	#[test]
	fn whitespace() {
		for i in (1..111).into_iter() {
			let tmp: Cow<str> = super::whitespace(i);
			assert_eq!(tmp.len(), i);
			assert!(tmp.trim().is_empty());
		}
	}
}
