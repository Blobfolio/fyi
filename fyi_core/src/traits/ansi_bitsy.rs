use regex::Regex;
use std::borrow::{Borrow, Cow};



/// Miscellaneous String Formatting Helpers.
pub trait AnsiBitsy {
	/// Number of characters.
	fn chars_len(&self) -> usize;

	/// Number of lines.
	///
	/// This only considers "\n". Fuck carriages. Haha.
	fn lines_len(&self) -> usize;

	/// Strip ANSI.
	fn strip_ansi<'ab> (&'ab self) -> Cow<'ab, str>;

	/// Display Width.
	fn width<'w> (&'w self) -> usize;
}



impl<T> AnsiBitsy for T
where T: Borrow<str> {
	#[inline]
	/// Number of characters.
	fn chars_len(&self) -> usize {
		let tmp = self.borrow();
		match tmp.is_empty() {
			true => 0,
			false => bytecount::num_chars(tmp.as_bytes()),
		}
	}

	#[inline]
	/// Number of lines.
	///
	/// This only considers "\n". Fuck carriages. Haha.
	fn lines_len(&self) -> usize {
		let tmp = self.borrow();
		match tmp.is_empty() {
			true => 0,
			false => bytecount::count(tmp.as_bytes(), b'\n') + 1,
		}
	}

	/// Strip ANSI.
	fn strip_ansi<'ab> (&'ab self) -> Cow<'ab, str> {
		lazy_static::lazy_static! {
			// Regex is expensive. Do this just once.
			static ref STRIP_ANSI_RE: Regex =
				Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
					.unwrap();
		}

		STRIP_ANSI_RE.replace_all(self.borrow(), "")
	}

	#[inline]
	/// Display Width.
	fn width<'w> (&'w self) -> usize {
		bytecount::num_chars(self.strip_ansi().as_bytes())
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn chars_len() {
		assert_eq!("Hello World".chars_len(), 11);
		assert_eq!("Björk Guðmundsdóttir".chars_len(), 20);
		assert_eq!("\x1B[1mBjörk\x1B[0m Guðmundsdóttir".chars_len(), 28);
	}

	#[test]
	fn lines_len() {
		assert_eq!("".lines_len(), 0);
		assert_eq!("Hello World".lines_len(), 1);
		assert_eq!("Hello\nWorld".lines_len(), 2);
		assert_eq!("Hello\nWorld\n".lines_len(), 3);
	}

	#[test]
	fn strip_ansi() {
		assert_eq!("Hello World".strip_ansi(), Cow::Borrowed("Hello World"));
		assert_eq!("Hello \x1B[1mWorld\x1B[0m".strip_ansi(), Cow::Borrowed("Hello World"));

		assert_eq!("Björk Guðmundsdóttir".strip_ansi(), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!("\x1B[1mBjörk\x1B[1m Guðmundsdóttir".strip_ansi(), Cow::Borrowed("Björk Guðmundsdóttir"));
	}

	#[test]
	fn width() {
		assert_eq!("Hello World".width(), 11);
		assert_eq!("Björk Guðmundsdóttir".width(), 20);
		assert_eq!("\x1B[1mBjörk\x1B[1m Guðmundsdóttir".width(), 20);
	}
}
