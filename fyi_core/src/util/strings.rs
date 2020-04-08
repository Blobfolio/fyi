/*!
# FYI Core: Strings
*/

use crate::util::numbers;
use std::{
	borrow::Cow,
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
	N: ToPrimitive + One + PartialEq,
	S: Into<Cow<'a, str>> {
	match num.is_one() {
		true => ["1 ", &singular.into()].concat(),
		false => format!(
			"{} {}",
			numbers::human_int(num),
			&plural.into(),
		),
	}
}

/// Indentation in Spaces.
///
/// Return a string consisting of 4 spaces for each requested tab.
pub fn indentation<N> (indent: N) -> Cow<'static, str>
where N: ToPrimitive {
	whitespace(indent.to_usize().unwrap_or(0) * 4)
}

/// Oxford Join
///
/// Join a `Vec<String>` with correct comma usage and placement. If
/// there is one item, that item is returned. If there are two, they
/// are joined with the operator. Three or more entries will use
/// the Oxford Comma.
pub fn oxford_join<'a, S> (mut list: Vec<String>, glue: S) -> Cow<'static, str>
where S: Into<Cow<'a, str>> {
	match list.len() {
		0 => Cow::Borrowed(""),
		1 => Cow::Owned(list[0].to_string()),
		2 => Cow::Owned(list.join(&[" ", glue.into().trim(), " "].concat())),
		_ => {
			let last = list.pop().unwrap();
			Cow::Owned([
				&list.join(", "),
				", ",
				glue.into().trim(),
				" ",
				&last
			].concat())
		}
	}
}

/// Make whitespace.
///
/// Generate a string consisting of X spaces.
pub fn whitespace<N> (count: N) -> Cow<'static, str>
where N: ToPrimitive {
	match count.to_usize().unwrap_or(0) {
		0 => Cow::Borrowed(""),
		x => match String::from_utf8(vec![b' '; x]) {
			Ok(y) => Cow::Owned(y),
			_ => Cow::Borrowed(""),
		},
	}
}
