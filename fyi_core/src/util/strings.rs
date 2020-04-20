/*!
# FYI Core: Strings
*/

use crate::util::numbers;
use num_traits::{
	cast::ToPrimitive,
	identities::One,
};
use regex::Regex;
use std::{
	borrow::Cow,
	ffi::{
		OsStr,
		OsString,
	},
};



/// From OsStr(ing).
pub fn from_os_string<S> (text: S) -> String
where S: Into<OsString> {
	text.into().to_str().unwrap_or("").to_string()
}

/// To OsString.
pub fn to_os_string<S> (text: S) -> OsString
where S: AsRef<str> {
	OsStr::new(text.as_ref()).to_os_string()
}

/// Number of Chars in String.
pub fn chars_len<S> (text: S) -> usize
where S: AsRef<str> {
	let text = text.as_ref();
	match text.is_empty() {
		true => 0,
		false => bytecount::num_chars(text.as_bytes()),
	}
}

/// Number of Chars in String.
pub fn lines_len<S> (text: S) -> usize
where S: AsRef<str> {
	let text = text.as_ref();
	match text.is_empty() {
		true => 0,
		false => bytecount::count(text.as_bytes(), b'\n') + 1,
	}
}

/// Inflect.
///
/// Return a string like "NUMBER LABEL" where the label is
/// appropriately singular or plural given the value.
pub fn inflect<N, S> (num: N, singular: S, plural: S) -> Cow<'static, str>
where
	N: ToPrimitive + One + PartialEq,
	S: AsRef<str> {
	Cow::Owned(match num.is_one() {
		true => {
			let singular = singular.as_ref();
			let mut out: String = String::with_capacity(singular.len() + 2);
			out.push_str("1 ");
			out.push_str(&singular);
			out
		},
		false => {
			let num = numbers::human_int(num);
			let plural = plural.as_ref();
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
pub fn oxford_join<S> (list: &[String], glue: S) -> Cow<'static, str>
where S: AsRef<str> {
	match list.len() {
		0 => Cow::Borrowed(""),
		1 => Cow::Owned(list[0].to_string()),
		2 => Cow::Owned(list.join(&{
			let glue = glue.as_ref();
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
				let glue = glue.as_ref();
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

/// Truncate to X Chars.
pub fn shorten<'s> (text: &'s str, keep: usize) -> Cow<'s, str> {
	let size: usize = chars_len(&text);
	if keep >= size {
		Cow::Borrowed(&text)
	}
	else if 1 == keep {
		Cow::Borrowed("…")
	}
	else if 0 == keep {
		Cow::Borrowed("")
	}
	else {
		let len: usize = text.len();
		if len <= keep {
			return Cow::Borrowed(&text);
		}

		let end: usize = match len == size {
			true => keep - 1,
			false => _chars_len_start(&text, keep - 1),
		};

		if end != len {
			if let Some(x) = text.get(0..end) {
				Cow::Owned([
					x,
					"…"
				].concat())
			}
			else {
				Cow::Borrowed("…")
			}
		}
		else {
			Cow::Borrowed(&text)
		}
	}
}

/// Remove First X Chars.
pub fn shorten_reverse<'s> (text: &'s str, keep: usize) -> Cow<'s, str> {
	let size: usize = chars_len(&text);
	if keep >= size {
		Cow::Borrowed(&text)
	}
	else if 1 == keep {
		Cow::Borrowed("…")
	}
	else if 0 == keep {
		Cow::Borrowed("")
	}
	else {
		let len = text.len();
		if len <= keep {
			return Cow::Borrowed(&text);
		}

		let end: usize = match len == size {
			true => len - keep + 1,
			false => _chars_len_end(&text, keep - 1),
		};

		if end != len {
			if let Some(x) = text.get(end..) {
				Cow::Owned([
					"…",
					x,
				].concat())
			}
			else {
				Cow::Borrowed("…")
			}
		}
		else {
			Cow::Borrowed(&text)
		}
	}
}

/// Strip ANSI.
///
/// This approach courtesy of "console"!
pub fn strip_ansi<'sa> (text: &'sa str) -> Cow<'sa, str> {
	lazy_static::lazy_static! {
		// Regex is expensive. Do this once.
		static ref STRIP_ANSI_RE: Regex =
			Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
				.unwrap();
	}

	STRIP_ANSI_RE.replace_all(text, "")
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

/// String "width".
pub fn width<S> (text: S) -> usize
where S: AsRef<str> {
	bytecount::num_chars(strip_ansi(text.as_ref()).as_bytes())
}

/// Find End Byte of First X Chars.
///
/// This is used internally for shortening.
fn _char_len_n(data: &[u8], stop: usize) -> usize {
	let mut chars = 0;

	for (k, &v) in data.iter().enumerate() {
		if (&v >> 6) != 0b10u8 {
			chars += 1;
			if chars > stop {
				return k;
			}
		}
	}

	data.len()
}

/// Find Byte Index X Chars From Start.
fn _chars_len_start<S> (text: S, num: usize) -> usize
where S: AsRef<str> {
	if num == 0 {
		return 0;
	}

	let text = text.as_ref();
	let len = text.len();
	if len == 0 {
		return 0;
	}
	else if num >= len {
		return len;
	}

	_char_len_n(text.as_bytes(), num)
}

/// Find Byte Index X Chars From End.
fn _chars_len_end<S> (text: S, num: usize) -> usize
where S: AsRef<str> {
	if num == 0 {
		return 0;
	}

	let text = text.as_ref();
	let len = text.len();
	if len == 0 {
		return 0;
	}
	else if num >= len {
		return len;
	}

	_char_len_n(text.as_bytes(), chars_len(&text) - num)
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
	fn chars_len() {
		assert_eq!(super::chars_len("Hello World"), 11);
		assert_eq!(super::chars_len("Björk Guðmundsdóttir"), 20);
		assert_eq!(super::chars_len("\x1B[1mBjörk\x1B[0m Guðmundsdóttir"), 28);
	}

	#[test]
	fn lines_len() {
		assert_eq!(super::lines_len(""), 0);
		assert_eq!(super::lines_len("Hello World"), 1);
		assert_eq!(super::lines_len("Hello\nWorld"), 2);
		assert_eq!(super::lines_len("Hello\nWorld\n"), 3);
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
	fn shorten() {
		assert_eq!(super::shorten("Hello World", 0), Cow::Borrowed(""));
		assert_eq!(super::shorten("Hello World", 1), Cow::Borrowed("…"));
		assert_eq!(super::shorten("Hello World", 2), Cow::Borrowed("H…"));
		assert_eq!(super::shorten("Hello World", 6), Cow::Borrowed("Hello…"));
		assert_eq!(super::shorten("Hello World", 7), Cow::Borrowed("Hello …"));
		assert_eq!(super::shorten("Hello World", 11), Cow::Borrowed("Hello World"));
		assert_eq!(super::shorten("Hello World", 100), Cow::Borrowed("Hello World"));

		assert_eq!(super::shorten("Björk Guðmundsdóttir", 0), Cow::Borrowed(""));
		assert_eq!(super::shorten("Björk Guðmundsdóttir", 1), Cow::Borrowed("…"));
		assert_eq!(super::shorten("Björk Guðmundsdóttir", 2), Cow::Borrowed("B…"));
		assert_eq!(super::shorten("Björk Guðmundsdóttir", 6), Cow::Borrowed("Björk…"));
		assert_eq!(super::shorten("Björk Guðmundsdóttir", 7), Cow::Borrowed("Björk …"));
		assert_eq!(super::shorten("Björk Guðmundsdóttir", 10), Cow::Borrowed("Björk Guð…"));
		assert_eq!(super::shorten("Björk Guðmundsdóttir", 20), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!(super::shorten("Björk Guðmundsdóttir", 100), Cow::Borrowed("Björk Guðmundsdóttir"));
	}

	#[test]
	fn shorten_reverse() {
		assert_eq!(super::shorten_reverse("Hello World", 0), Cow::Borrowed(""));
		assert_eq!(super::shorten_reverse("Hello World", 1), Cow::Borrowed("…"));
		assert_eq!(super::shorten_reverse("Hello World", 2), Cow::Borrowed("…d"));
		assert_eq!(super::shorten_reverse("Hello World", 6), Cow::Borrowed("…World"));
		assert_eq!(super::shorten_reverse("Hello World", 7), Cow::Borrowed("… World"));
		assert_eq!(super::shorten_reverse("Hello World", 11), Cow::Borrowed("Hello World"));
		assert_eq!(super::shorten_reverse("Hello World", 100), Cow::Borrowed("Hello World"));

		assert_eq!(super::shorten_reverse("Björk Guðmundsdóttir", 0), Cow::Borrowed(""));
		assert_eq!(super::shorten_reverse("Björk Guðmundsdóttir", 1), Cow::Borrowed("…"));
		assert_eq!(super::shorten_reverse("Björk Guðmundsdóttir", 2), Cow::Borrowed("…r"));
		assert_eq!(super::shorten_reverse("Björk Guðmundsdóttir", 6), Cow::Borrowed("…óttir"));
		assert_eq!(super::shorten_reverse("Björk Guðmundsdóttir", 7), Cow::Borrowed("…dóttir"));
		assert_eq!(super::shorten_reverse("Björk Guðmundsdóttir", 10), Cow::Borrowed("…ndsdóttir"));
		assert_eq!(super::shorten_reverse("Björk Guðmundsdóttir", 20), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!(super::shorten_reverse("Björk Guðmundsdóttir", 100), Cow::Borrowed("Björk Guðmundsdóttir"));
	}

	#[test]
	fn fyi_strip_ansi() {
		assert_eq!(super::strip_ansi("Hello World"), Cow::Borrowed("Hello World"));
		assert_eq!(super::strip_ansi("Hello \x1B[1mWorld\x1B[0m"), Cow::Borrowed("Hello World"));

		assert_eq!(super::strip_ansi("Björk Guðmundsdóttir"), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!(super::strip_ansi("\x1B[1mBjörk\x1B[1m Guðmundsdóttir"), Cow::Borrowed("Björk Guðmundsdóttir"));
	}

	#[test]
	fn whitespace() {
		for i in (1..111).into_iter() {
			let tmp: Cow<str> = super::whitespace(i);
			assert_eq!(tmp.len(), i);
			assert!(tmp.trim().is_empty());
		}
	}

	#[test]
	fn width() {
		assert_eq!(super::width("Hello World"), 11);
		assert_eq!(super::width("Björk Guðmundsdóttir"), 20);
		assert_eq!(super::width("\x1B[1mBjörk\x1B[1m Guðmundsdóttir"), 20);
	}
}
