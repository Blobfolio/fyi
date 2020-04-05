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

/// Find End Byte of First X Chars.
fn char_len_n(data: &[u8], stop: usize) -> usize {
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

/// String helpers!
pub trait FYIStringFormat {
	/// Find Byte Index X Chars From Start.
	fn fyi_chars_len_start(&self, num: usize) -> usize;

	/// Find Byte Index X Chars From End.
	fn fyi_chars_len_end(&self, num: usize) -> usize;

	/// Number of Chars in String.
	fn fyi_chars_len(&self) -> usize;

	/// Number of Chars in String.
	fn fyi_lines_len(&self) -> usize;

	/// Truncate to X Chars.
	fn fyi_shorten(&self, keep: usize) -> Cow<'_, str>;

	/// Remove First X Chars.
	fn fyi_shorten_reverse(&self, keep: usize) -> Cow<'_, str>;

	/// Stretch a String Filling End With X.
	fn fyi_stretch(&self, num: usize, filler: u8) -> Cow<'_, str>;

	/// Stretch a String Filling Start With X.
	fn fyi_stretch_reverse(&self, num: usize, filler: u8) -> Cow<'_, str>;

	/// Strip Formatting.
	fn fyi_strip_ansi(&self) -> Cow<'_, str>;

	/// String "width".
	fn fyi_width(&self) -> usize;
}

impl <T> FYIStringFormat for T
where T: AsRef<str> {
	/// Find Byte Index X Chars From Start.
	fn fyi_chars_len_start(&self, num: usize) -> usize {
		if num == 0 {
			return 0;
		}

		let me = self.as_ref();
		let len = me.len();
		if len == 0 {
			return 0;
		}
		else if num >= len {
			return len;
		}

		char_len_n(me.as_bytes(), num)
	}

	/// Find Byte Index X Chars From End.
	fn fyi_chars_len_end(&self, num: usize) -> usize {
		if num == 0 {
			return 0;
		}

		let me = self.as_ref();
		let len = me.len();
		if len == 0 {
			return 0;
		}
		else if num >= len {
			return len;
		}

		char_len_n(me.as_bytes(), me.fyi_chars_len() - num)
	}

	/// Number of Chars in String.
	fn fyi_lines_len(&self) -> usize {
		let me = self.as_ref();
		match me.is_empty() {
			true => 0,
			false => bytecount::count(me.as_bytes(), b'\n') + 1,
		}
	}

	/// Number of Chars in String.
	fn fyi_chars_len(&self) -> usize {
		let me = self.as_ref();
		match me.is_empty() {
			true => 0,
			false => bytecount::num_chars(me.as_bytes()),
		}
	}

	/// Truncate to X Chars.
	fn fyi_shorten(&self, keep: usize) -> Cow<'_, str> {
		let me = self.as_ref();
		let size = me.fyi_chars_len();
		if keep >= size {
			Cow::Borrowed(&me)
		}
		else if 1 == keep {
			Cow::Borrowed("…")
		}
		else if 0 == keep {
			Cow::Borrowed("")
		}
		else {
			let len = me.len();
			let end = me.fyi_chars_len_start(keep - 1);
			if end != len {
				if let Some(x) = me.get(0..end) {
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
				Cow::Borrowed(&me)
			}
		}
	}

	/// Remove First X Chars.
	fn fyi_shorten_reverse(&self, keep: usize) -> Cow<'_, str> {
		let me = self.as_ref();
		let size = me.fyi_chars_len();
		if keep >= size {
			Cow::Borrowed(&me)
		}
		else if 1 == keep {
			Cow::Borrowed("…")
		}
		else if 0 == keep {
			Cow::Borrowed("")
		}
		else {
			let len = me.len();
			let end = me.fyi_chars_len_end(keep - 1);
			if end != len {
				if let Some(x) = me.get(end..) {
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
				Cow::Borrowed(&me)
			}
		}
	}

	/// Stretch a String Filling End With X.
	fn fyi_stretch(&self, num: usize, filler: u8) -> Cow<'_, str> {
		let me = self.as_ref();
		let size = me.fyi_chars_len();
		if num <= size {
			Cow::Borrowed(&me)
		}
		else {
			let len = num - size;
			if let Ok(x) = String::from_utf8(vec![filler; len]) {
				Cow::Owned([
					&me,
					x.as_str(),
				].concat())
			}
			else {
				Cow::Borrowed(&me)
			}
		}
	}

	/// Stretch a String Filling Start With X.
	fn fyi_stretch_reverse(&self, num: usize, filler: u8) -> Cow<'_, str> {
		let me = self.as_ref();
		let size = me.fyi_chars_len();
		if num <= size {
			Cow::Borrowed(&me)
		}
		else {
			let len = num - size;
			if let Ok(x) = String::from_utf8(vec![filler; len]) {
				Cow::Owned([
					x.as_str(),
					&me,
				].concat())
			}
			else {
				Cow::Borrowed(&me)
			}
		}
	}

	/// Strip ANSI.
	fn fyi_strip_ansi(&self) -> Cow<'_, str> {
		let me = self.as_ref();
		if false == me.is_empty() {
			if let Ok(x) = strip_ansi_escapes::strip(me.as_bytes()) {
				if let Ok(y) = String::from_utf8(x) {
					if y == *me {
						return Cow::Borrowed(&me);
					}
					else {
						return Cow::Owned(y);
					}
				}
			}
		}

		Cow::Borrowed("")
	}

	/// String "width".
	fn fyi_width(&self) -> usize {
		bytecount::num_chars(self.fyi_strip_ansi().as_bytes())
	}
}

/*impl FYIStringFormat for String {
	/// Find Byte Index X Chars From Start.
	fn fyi_chars_len_start(&self, num: usize) -> usize {
		let num = num.to_usize().unwrap_or(0);
		if num == 0 {
			return 0;
		}

		let len = self.len();
		if len == 0 {
			return 0;
		}
		else if num >= len {
			return len;
		}

		char_len_n(self.as_bytes(), num)
	}

	/// Find Byte Index X Chars From End.
	fn fyi_chars_len_end(&self, num: usize) -> usize {
		let num = num.to_usize().unwrap_or(0);
		if num == 0 {
			return 0;
		}

		let len = self.len();
		if len == 0 {
			return 0;
		}
		else if num >= len {
			return len;
		}

		char_len_n(self.as_bytes(), self.fyi_chars_len() - num)
	}

	/// Number of Chars in String.
	fn fyi_lines_len(&self) -> usize {
		match self.is_empty() {
			true => 0,
			false => bytecount::count(self.as_bytes(), b'\n') + 1,
		}
	}

	/// Number of Chars in String.
	fn fyi_chars_len(&self) -> usize {
		match self.is_empty() {
			true => 0,
			false => bytecount::num_chars(self.as_bytes()),
		}
	}

	/// Truncate to X Chars.
	fn fyi_shorten(&self, keep: usize) -> Cow<'_, str> {
		let size = self.fyi_chars_len();
		if keep >= size {
			Cow::Borrowed(&self)
		}
		else if 1 == keep {
			Cow::Borrowed("…")
		}
		else if 0 == keep {
			Cow::Borrowed("")
		}
		else {
			let len = self.len();
			let end = self.fyi_chars_len_start(keep - 1);
			if end != len {
				if let Some(x) = self.get(0..end) {
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
				Cow::Borrowed(&self)
			}
		}
	}

	/// Remove First X Chars.
	fn fyi_shorten_reverse(&self, keep: usize) -> Cow<'_, str> {
		let size = self.fyi_chars_len();
		if keep >= size {
			Cow::Borrowed(&self)
		}
		else if 1 == keep {
			Cow::Borrowed("…")
		}
		else if 0 == keep {
			Cow::Borrowed("")
		}
		else {
			let len = self.len();
			let end = self.fyi_chars_len_end(keep - 1);
			if end != len {
				if let Some(x) = self.get(end..) {
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
				Cow::Borrowed(&self)
			}
		}
	}

	/// Stretch a String Filling End With X.
	fn fyi_stretch(&self, num: usize, filler: u8) -> Cow<'_, str> {
		let size = self.fyi_chars_len();
		if num <= size {
			Cow::Borrowed(&self)
		}
		else {
			let len = num - size;
			if let Ok(x) = String::from_utf8(vec![filler; len]) {
				Cow::Owned([
					&self,
					x.as_str(),
				].concat())
			}
			else {
				Cow::Borrowed(&self)
			}
		}
	}

	/// Stretch a String Filling Start With X.
	fn fyi_stretch_reverse(&self, num: usize, filler: u8) -> Cow<'_, str> {
		let size = self.fyi_chars_len();
		if num <= size {
			Cow::Borrowed(&self)
		}
		else {
			let len = num - size;
			if let Ok(x) = String::from_utf8(vec![filler; len]) {
				Cow::Owned([
					x.as_str(),
					&self,
				].concat())
			}
			else {
				Cow::Borrowed(&self)
			}
		}
	}

	/// Strip ANSI.
	fn fyi_strip_ansi(&self) -> Cow<'_, str> {
		if false == self.is_empty() {
			if let Ok(x) = strip_ansi_escapes::strip(self.as_bytes()) {
				if let Ok(y) = String::from_utf8(x) {
					if y == *self {
						return Cow::Borrowed(&self);
					}
					else {
						return Cow::Owned(y);
					}
				}
			}
		}

		Cow::Borrowed("")
	}

	/// String "width".
	fn fyi_width(&self) -> usize {
		bytecount::num_chars(self.fyi_strip_ansi().as_bytes())
	}
}*/
