use bytes::BytesMut;
use regex::Regex;
use std::borrow::{
	Borrow,
	Cow,
};



lazy_static::lazy_static! {
	// Regex is expensive. Do this just once.
	static ref RE_ANSI_BYTES: regex::bytes::Regex =
		regex::bytes::Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
			.unwrap();

	// And again for a string version.
	static ref RE_ANSI_STR: Regex =
		Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
			.unwrap();
}



/// Miscellaneous String Formatting Helpers.
pub trait AnsiBitsy<'ab> {
	/// Return type for stripped.
	type AnsiReturn;

	/// Number of characters.
	fn chars_len(&self) -> usize;

	/// Number of lines.
	///
	/// This only considers "\n". Fuck carriages. Haha.
	fn lines_len(&self) -> usize;

	/// Strip ANSI.
	fn strip_ansi(&'ab self) -> Self::AnsiReturn;

	/// Display Width.
	fn width<'w> (&'w self) -> usize;
}



/// The main implementation is for Stringish things.
impl<'ab, T> AnsiBitsy<'ab> for T
where T: Borrow<str> {
	/// Return type for stripped.
	type AnsiReturn = Cow<'ab, str>;

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
	fn strip_ansi(&'ab self) -> Self::AnsiReturn {
		RE_ANSI_STR.replace_all(self.borrow(), "")
	}

	#[inline]
	/// Display Width.
	fn width<'w> (&'w self) -> usize {
		bytecount::num_chars(&RE_ANSI_BYTES.replace_all(
			self.borrow().as_bytes(),
			regex::bytes::NoExpand(b"")
		))
	}
}



/// Wish that &[u8] could be extended, but Rust doesn't allow it.
impl<'ab> AnsiBitsy<'ab> for [u8] {
	/// Return type for stripped.
	type AnsiReturn = Cow<'ab, [u8]>;

	#[inline]
	/// Number of characters.
	fn chars_len(&self) -> usize {
		match self.is_empty() {
			true => 0,
			false => bytecount::num_chars(self),
		}
	}

	#[inline]
	/// Number of lines.
	///
	/// This only considers "\n". Fuck carriages. Haha.
	fn lines_len(&self) -> usize {
		match self.is_empty() {
			true => 0,
			false => bytecount::count(self, b'\n') + 1,
		}
	}

	/// Strip ANSI.
	fn strip_ansi(&'ab self) -> Self::AnsiReturn {
		RE_ANSI_BYTES.replace_all(self, regex::bytes::NoExpand(b""))
	}

	#[inline]
	/// Display Width.
	fn width<'w> (&'w self) -> usize {
		bytecount::num_chars(&RE_ANSI_BYTES.replace_all(
			self,
			regex::bytes::NoExpand(b"")
		))
	}
}



/// We want to operate on BytesMut to avoid allocation, but unless we
/// express the impl as a dyn-AsRef Rust won't allow it. Haha.
impl<'ab> AnsiBitsy<'ab> for dyn AsRef<BytesMut> {
	/// Return type for stripped.
	type AnsiReturn = Cow<'ab, [u8]>;

	#[inline]
	/// Number of characters.
	fn chars_len(&self) -> usize {
		let tmp = self.as_ref();
		match tmp.is_empty() {
			true => 0,
			false => bytecount::num_chars(tmp),
		}
	}

	#[inline]
	/// Number of lines.
	///
	/// This only considers "\n". Fuck carriages. Haha.
	fn lines_len(&self) -> usize {
		let tmp = self.as_ref();
		match tmp.is_empty() {
			true => 0,
			false => bytecount::count(tmp, b'\n') + 1,
		}
	}

	/// Strip ANSI.
	fn strip_ansi(&'ab self) -> Self::AnsiReturn {
		RE_ANSI_BYTES.replace_all(
			self.as_ref().borrow(),
			regex::bytes::NoExpand(b"")
		)
	}

	#[inline]
	/// Display Width.
	fn width<'w> (&'w self) -> usize {
		let tmp = self.as_ref();
		let len = tmp.len();
		bytecount::num_chars(&RE_ANSI_BYTES.replace_all(
			&tmp[0..len],
			regex::bytes::NoExpand(b"")
		))
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use bytes::BufMut;

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

		// Try a buffer now.
		let mut b: BytesMut = BytesMut::with_capacity(128);
		b.put("\x1B[1mBjörk\x1B[1m Guðmundsdóttir".as_bytes());
		assert_eq!(b.strip_ansi(), Cow::Borrowed("Björk Guðmundsdóttir".as_bytes()));
	}

	#[test]
	fn width() {
		assert_eq!("Hello World".width(), 11);
		assert_eq!("Björk Guðmundsdóttir".width(), 20);
		assert_eq!("\x1B[1mBjörk\x1B[1m Guðmundsdóttir".width(), 20);

		// And the width.
		let mut b: BytesMut = BytesMut::with_capacity(128);
		b.put("\x1B[1mBjörk\x1B[1m Guðmundsdóttir".as_bytes());
		assert_eq!(b.width(), 20);
	}
}
