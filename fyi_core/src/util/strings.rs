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
/// From OsStr(ing).
pub fn from_os_string<S> (text: S) -> String
where S: Into<OsString> {
	text.into().to_str().unwrap_or("").to_string()
}

#[inline]
/// To OsString.
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
