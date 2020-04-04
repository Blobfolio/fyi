/*!
# FYI Core: Strings
*/

use std::{
	borrow::Cow,
	fmt::Display,
	ffi::{
		OsStr,
		OsString,
	},
};
use num_traits::{
	cast::ToPrimitive,
	identities::One,
};



/// From OsStr(ing).
pub fn from_os_string<S> (text: S) -> String
where S: Into<OsString> {
	text.into().to_str().unwrap_or("").to_string()
}

/// To OsString.
pub fn to_os_string<'a, S> (text: S) -> OsString
where S: Into<Cow<'a, str>> {
	OsStr::new(text.into().as_ref()).to_os_string()
}

/// Inflect.
///
/// Return a string like "NUMBER LABEL" where the label is
/// appropriately singular or plural given the value.
pub fn inflect<'a, N, S> (num: N, singular: S, plural: S) -> String
where
	N: One + Display + PartialEq,
	S: Into<Cow<'a, str>> {
	match num.is_one() {
		true => ["1", &singular.into()].concat(),
		false => [num.to_string().as_ref(), " ", &plural.into()].concat(),
	}
}

/// Indentation in Spaces.
///
/// Return a string consisting of 4 spaces for each requested tab.
pub fn indentation<N> (indent: N) -> String
where N: ToPrimitive {
	whitespace(indent.to_usize().unwrap_or(0) * 4)
}

/// Line Count.
pub fn lines<'a, S> (text: S) -> usize
where S: Into<Cow<'a, str>> {
	let text: String = text.into().trim().to_string();
	match text.is_empty() {
		true => 0,
		false => bytecount::count(text.as_bytes(), b'\n') + 1,
	}
}

/// Oxford Join
///
/// Join a `Vec<String>` with correct comma usage and placement. If
/// there is one item, that item is returned. If there are two, they
/// are joined with the operator. Three or more entries will use
/// the Oxford Comma.
pub fn oxford_join<'a, S> (mut list: Vec<String>, glue: S) -> String
where S: Into<Cow<'a, str>> {
	match list.len() {
		0 => String::new(),
		1 => list[0].to_string(),
		2 => list.join(&[" ", glue.into().trim(), " "].concat()),
		_ => {
			let last = list.pop().unwrap();
			[
				&list.join(", "),
				", ",
				glue.into().trim(),
				" ",
				&last
			].concat()
		}
	}
}

/// Find Padding Needed.
fn pad_diff<'a, S, N> (text: S, len: N) -> usize
where
	S: Into<Cow<'a, str>>,
	N: ToPrimitive
{
	let text_len: usize = text.into().len();
	let len = len.to_usize().unwrap_or(0);
	match text_len >= len {
		true => 0,
		false => len - text_len,
	}
}

/// Pad String On Left.
pub fn pad_left<S>(text: S, pad_len: usize, pad_fill: u8) -> String
where S: Into<String> {
	let text = text.into();
	match pad_diff(&text, pad_len) {
		0 => text,
		x => [
			String::from_utf8(vec![pad_fill; x]).unwrap_or(String::new()),
			text,
		].concat(),
	}
}

/// Pad String On Right.
pub fn pad_right<S>(text: S, pad_len: usize, pad_fill: u8) -> String
where S: Into<String> {
	let text = text.into();
	match pad_diff(&text, pad_len) {
		0 => text,
		x => [
			text,
			String::from_utf8(vec![pad_fill; x]).unwrap_or(String::new()),
		].concat(),
	}
}

/// Shorten String From Left (Keeping Right).
pub fn shorten_left<S, N>(text: S, len: N) -> String
where
	S: Into<String>,
	N: ToPrimitive
{
	match len.to_usize().unwrap_or(0) {
		0 => String::new(),
		1 => "…".to_string(),
		x => {
			// Pull text details.
			let text = text.into();
			let text_len: usize = text.len();

			// Shorten away!
			match text_len <= x {
				true => text,
				false => [
					"…",
					text.chars()
						.skip(text_len - x + 1)
						.collect::<String>()
						.trim(),
				].concat(),
			}
		}
	}
}

/// Shorten String From Right (Keeping Left).
pub fn shorten_right<S, N>(text: S, len: N) -> String
where
	S: Into<String>,
	N: ToPrimitive
{
	match len.to_usize().unwrap_or(0) {
		0 => String::new(),
		1 => "…".to_string(),
		x => {
			// Pull text details.
			let text = text.into();
			let text_len: usize = text.len();

			// Shorten away!
			match text_len <= x {
				true => text,
				false => [
					text.chars()
						.take(x - 1)
						.collect::<String>()
						.trim(),
					"…",
				].concat(),
			}
		}
	}
}

/// Stripped Length.
///
/// Return the length of a string without counting any ANSI codes, etc.
pub fn stripped_len<'a, S> (text: S) -> usize
where S: Into<Cow<'a, str>> {
	strip_styles(text).len()
}

/// Strip Styles
///
/// Remove ANSI codes, etc., from a string.
pub fn strip_styles<'a, S> (text: S) -> String
where S: Into<Cow<'a, str>> {
	match strip_ansi_escapes::strip(text.into().as_bytes()) {
		Ok(x) => String::from_utf8(x).unwrap_or(String::new()),
		_ => String::new(),
	}
}

/// Make whitespace.
///
/// Generate a string consisting of X spaces.
pub fn whitespace<N> (count: N) -> String
where N: ToPrimitive {
	match count.to_usize().unwrap_or(0) {
		0 => String::new(),
		x => String::from_utf8(vec![b' '; x]).unwrap_or(String::new())
	}
}
