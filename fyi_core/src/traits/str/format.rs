use std::borrow::Cow;
use regex::Regex;



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
	fn fyi_chars_len(&self) -> usize {
		let me = self.as_ref();
		match me.is_empty() {
			true => 0,
			false => bytecount::num_chars(me.as_bytes()),
		}
	}

	/// Number of Chars in String.
	fn fyi_lines_len(&self) -> usize {
		let me = self.as_ref();
		match me.is_empty() {
			true => 0,
			false => bytecount::count(me.as_bytes(), b'\n') + 1,
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

	/// Strip ANSI.
	///
	/// This approach courtesy of "console"!
	fn fyi_strip_ansi(&self) -> Cow<'_, str> {
		lazy_static::lazy_static! {
			// Regex is expensive. Do this once.
			static ref STRIP_ANSI_RE: Regex =
				Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
					.unwrap();
		}

		STRIP_ANSI_RE.replace_all(self.as_ref(), "")
	}

	/// String "width".
	fn fyi_width(&self) -> usize {
		bytecount::num_chars(self.fyi_strip_ansi().as_bytes())
	}
}

/// Find End Byte of First X Chars.
///
/// This is used internally for shortening.
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
