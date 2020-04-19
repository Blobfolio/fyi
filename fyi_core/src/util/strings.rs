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
		true => {
			let singular = singular.into();
			let mut out: String = String::with_capacity(singular.len() + 2);
			out.push_str("1 ");
			out.push_str(&singular);
			out
		},
		false => {
			let num = numbers::human_int(num);
			let plural = plural.into();
			let mut out: String = String::with_capacity(num.len() + plural.len() + 1);
			out.push_str(&num);
			out.push(' ');
			out.push_str(&plural);
			out
		},
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
		2 => Cow::Owned(list.join(&{
			let glue = glue.into();
			let mut out: String = String::with_capacity(glue.len() + 2);
			out.push(' ');
			out.push_str(&glue);
			out.push(' ');
			out
		})),
		x => {
			let len = x - 1;
			Cow::Owned({
				let first = &list[0..len].join(", ");
				let glue = glue.into();
				let mut out: String = String::with_capacity(first.len() + glue.len() + 3 + list[len].len());
				out.push_str(&first);
				out.push_str(", ");
				out.push_str(&glue);
				out.push(' ');
				out.push_str(&list[len]);
				out
			})
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
	fn inflect() {
		assert_eq!(super::inflect(0, "book", "books"), Cow::Borrowed("0 books"));
		assert_eq!(super::inflect(1, "book", "books"), Cow::Borrowed("1 book"));
		assert_eq!(super::inflect(2, "book", "books"), Cow::Borrowed("2 books"));
		assert_eq!(super::inflect(5000, "book", "books"), Cow::Borrowed("5,000 books"));
	}

	#[test]
	fn oxford_join() {
		let data: [String; 5] = [
			"apples".to_string(),
			"bananas".to_string(),
			"carrots".to_string(),
			"dates".to_string(),
			"eggplants".to_string(),
		];
		let expected_and: [&str; 6] = [
			"",
			"apples",
			"apples and bananas",
			"apples, bananas, and carrots",
			"apples, bananas, carrots, and dates",
			"apples, bananas, carrots, dates, and eggplants",
		];
		let expected_or: [&str; 6] = [
			"",
			"apples",
			"apples or bananas",
			"apples, bananas, or carrots",
			"apples, bananas, carrots, or dates",
			"apples, bananas, carrots, dates, or eggplants",
		];

		for i in (0..6).into_iter() {
			assert_eq!(
				&super::oxford_join(&data[0..i], "and"),
				expected_and[i]
			);
			assert_eq!(
				&super::oxford_join(&data[0..i], "or"),
				expected_or[i]
			);
		}
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
