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



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fyi_chars_len() {
		assert_eq!("Hello World".fyi_chars_len(), 11);
		assert_eq!("Björk Guðmundsdóttir".fyi_chars_len(), 20);
		assert_eq!("\x1B[1mBjörk\x1B[1m Guðmundsdóttir".fyi_chars_len(), 28);
	}

	#[test]
	fn fyi_lines_len() {
		assert_eq!("".fyi_lines_len(), 0);
		assert_eq!("Hello World".fyi_lines_len(), 1);
		assert_eq!("Hello\nWorld".fyi_lines_len(), 2);
		assert_eq!("Hello\nWorld\n".fyi_lines_len(), 3);
	}

	#[test]
	fn fyi_shorten() {
		assert_eq!("Hello World".fyi_shorten(0), Cow::Borrowed(""));
		assert_eq!("Hello World".fyi_shorten(1), Cow::Borrowed("…"));
		assert_eq!("Hello World".fyi_shorten(2), Cow::Borrowed("H…"));
		assert_eq!("Hello World".fyi_shorten(6), Cow::Borrowed("Hello…"));
		assert_eq!("Hello World".fyi_shorten(7), Cow::Borrowed("Hello …"));
		assert_eq!("Hello World".fyi_shorten(11), Cow::Borrowed("Hello World"));
		assert_eq!("Hello World".fyi_shorten(100), Cow::Borrowed("Hello World"));

		assert_eq!("Björk Guðmundsdóttir".fyi_shorten(0), Cow::Borrowed(""));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten(1), Cow::Borrowed("…"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten(2), Cow::Borrowed("B…"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten(6), Cow::Borrowed("Björk…"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten(7), Cow::Borrowed("Björk …"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten(10), Cow::Borrowed("Björk Guð…"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten(20), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten(100), Cow::Borrowed("Björk Guðmundsdóttir"));
	}

	#[test]
	fn fyi_shorten_reverse() {
		assert_eq!("Hello World".fyi_shorten_reverse(0), Cow::Borrowed(""));
		assert_eq!("Hello World".fyi_shorten_reverse(1), Cow::Borrowed("…"));
		assert_eq!("Hello World".fyi_shorten_reverse(2), Cow::Borrowed("…d"));
		assert_eq!("Hello World".fyi_shorten_reverse(6), Cow::Borrowed("…World"));
		assert_eq!("Hello World".fyi_shorten_reverse(7), Cow::Borrowed("… World"));
		assert_eq!("Hello World".fyi_shorten_reverse(11), Cow::Borrowed("Hello World"));
		assert_eq!("Hello World".fyi_shorten_reverse(100), Cow::Borrowed("Hello World"));

		assert_eq!("Björk Guðmundsdóttir".fyi_shorten_reverse(0), Cow::Borrowed(""));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten_reverse(1), Cow::Borrowed("…"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten_reverse(2), Cow::Borrowed("…r"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten_reverse(6), Cow::Borrowed("…óttir"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten_reverse(7), Cow::Borrowed("…dóttir"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten_reverse(10), Cow::Borrowed("…ndsdóttir"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten_reverse(20), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!("Björk Guðmundsdóttir".fyi_shorten_reverse(100), Cow::Borrowed("Björk Guðmundsdóttir"));
	}

	#[test]
	fn fyi_strip_ansi() {
		assert_eq!("Hello World".fyi_strip_ansi(), Cow::Borrowed("Hello World"));
		assert_eq!("Hello \x1B[1mWorld\x1B[0m".fyi_strip_ansi(), Cow::Borrowed("Hello World"));

		assert_eq!("Björk Guðmundsdóttir".fyi_strip_ansi(), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!("\x1B[1mBjörk\x1B[1m Guðmundsdóttir".fyi_strip_ansi(), Cow::Borrowed("Björk Guðmundsdóttir"));
	}

	#[test]
	fn fyi_width() {
		assert_eq!("Hello World".fyi_width(), 11);
		assert_eq!("Björk Guðmundsdóttir".fyi_width(), 20);
		assert_eq!("\x1B[1mBjörk\x1B[1m Guðmundsdóttir".fyi_width(), 20);
	}
}
