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
pub fn inflect<'a, N, S> (num: N, singular: S, plural: S) -> Cow<'static, str>
where
	N: ToPrimitive + One + PartialEq,
	S: Into<Cow<'a, str>> {
	Cow::Owned(match num.is_one() {
		true => ["1 ", &singular.into()].concat(),
		false => format!(
			"{} {}",
			numbers::human_int(num),
			&plural.into(),
		),
	})
}

/// Oxford Join
///
/// Join a `Vec<String>` with correct comma usage and placement. If
/// there is one item, that item is returned. If there are two, they
/// are joined with the operator. Three or more entries will use
/// the Oxford Comma.
pub fn oxford_join<'a, S> (list: &[String], glue: S) -> Cow<'static, str>
where S: Into<Cow<'a, str>> {
	match list.len() {
		0 => Cow::Borrowed(""),
		1 => Cow::Owned(list[0].to_string()),
		2 => Cow::Owned(list.join(&[" ", glue.into().trim(), " "].concat())),
		x => {
			let len = x - 1;
			Cow::Owned([
				&list[0..len].join(", "),
				", ",
				glue.into().trim(),
				" ",
				&list[len]
			].concat())
		}
	}
}

/// Make whitespace.
///
/// Generate a string consisting of X spaces.
pub fn whitespace<N> (count: N) -> Cow<'static, str>
where N: ToPrimitive {
	lazy_static::lazy_static! {
		// Precompute 100 spaces; it is cheaper to shrink than to grow.
		static ref WHITE: Cow<'static, str> = Cow::Owned("                                                                                                    ".to_string());
	}

	let count = count.to_usize().unwrap_or(0);
	if 0 == count {
		Cow::Borrowed("")
	}
	else if count <= 100 {
		Cow::Borrowed(&WHITE[0..count])
	}
	else {
		Cow::Owned(String::from_utf8(vec![b' '; count]).unwrap())
	}
}
