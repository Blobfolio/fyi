use crate::{
	ELLIPSIS,
	traits::AnsiBitsy
};
use std::borrow::{
	Borrow, Cow,
};



/// Shorten Strings (For Display).
pub trait Shorty {
	/// Shorten, chopping right.
	fn shorten(&self, keep: usize) -> Cow<'_, str>;

	/// Shorten, chopping left.
	fn shorten_reverse(&self, keep: usize) -> Cow<'_, str>;

	/// Find Byte Index X Chars From Start.
	fn _chars_len_start(&self, num: usize) -> usize;

	/// Find Byte Index X Chars From End.
	fn _chars_len_end(&self, num: usize) -> usize;
}



impl<T> Shorty for T
where T: Borrow<str> {
	/// Shorten, chopping right.
	fn shorten(&self, keep: usize) -> Cow<'_, str> {
		let size: usize = self.chars_len();
		if keep >= size {
			self.borrow().into()
		}
		else if 1 == keep {
			ELLIPSIS.into()
		}
		else if 0 == keep {
			"".into()
		}
		else {
			let text = self.borrow();
			let len: usize = text.len();
			if len <= keep {
				return text.into();
			}

			let end: usize = if len == size {
				keep - 1
			}
			else {
				text._chars_len_start(keep - 1)
			};

			if end == len {
				text.into()
			}
			else if let Some(x) = text.get(0..end) {
				let mut out: String = String::with_capacity(end + 3);
				out.push_str(x);
				out.push_str(ELLIPSIS);
				out.into()
			}
			else {
				ELLIPSIS.into()
			}
		}
	}

	/// Shorten, chopping left.
	fn shorten_reverse(&self, keep: usize) -> Cow<'_, str> {
		let size: usize = self.chars_len();
		if keep >= size {
			self.borrow().into()
		}
		else if 1 == keep {
			ELLIPSIS.into()
		}
		else if 0 == keep {
			"".into()
		}
		else {
			let text = self.borrow();
			let len = text.len();
			if len <= keep {
				return text.into();
			}

			let end: usize = if len == size {
				len - keep + 1
			}
			else {
				text._chars_len_end(keep - 1)
			};

			if end == len {
				text.into()
			}
			else if let Some(x) = text.get(end..) {
				let mut out: String = String::with_capacity(end + 3);
				out.push_str(ELLIPSIS);
				out.push_str(x);
				out.into()
			}
			else {
				ELLIPSIS.into()
			}
		}
	}

	/// Find Byte Index X Chars From Start.
	fn _chars_len_start(&self, num: usize) -> usize {
		if num == 0 {
			return 0;
		}

		let text = self.borrow();
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
	fn _chars_len_end(&self, num: usize) -> usize {
		if num == 0 {
			return 0;
		}

		let text = self.borrow();
		let len = text.len();
		if len == 0 {
			return 0;
		}
		else if num >= len {
			return len;
		}

		_char_len_n(text.as_bytes(), text.chars_len() - num)
	}
}



/// Find End Byte of First X Chars.
///
/// This is used internally for shortening.
fn _char_len_n(data: &[u8], stop: usize) -> usize {
	let mut chars = 0;

	for (k, &v) in data.iter().enumerate() {
		if (&v >> 6) != 0b10_u8 {
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
	fn shorten() {
		assert_eq!("Hello World".shorten(0), Cow::Borrowed(""));
		assert_eq!("Hello World".shorten(1), Cow::Borrowed(ELLIPSIS));
		assert_eq!("Hello World".shorten(2), Cow::Borrowed("H…"));
		assert_eq!("Hello World".shorten(6), Cow::Borrowed("Hello…"));
		assert_eq!("Hello World".shorten(7), Cow::Borrowed("Hello …"));
		assert_eq!("Hello World".shorten(11), Cow::Borrowed("Hello World"));
		assert_eq!("Hello World".shorten(100), Cow::Borrowed("Hello World"));

		assert_eq!("Björk Guðmundsdóttir".shorten(0), Cow::Borrowed(""));
		assert_eq!("Björk Guðmundsdóttir".shorten(1), Cow::Borrowed(ELLIPSIS));
		assert_eq!("Björk Guðmundsdóttir".shorten(2), Cow::Borrowed("B…"));
		assert_eq!("Björk Guðmundsdóttir".shorten(6), Cow::Borrowed("Björk…"));
		assert_eq!("Björk Guðmundsdóttir".shorten(7), Cow::Borrowed("Björk …"));
		assert_eq!("Björk Guðmundsdóttir".shorten(10), Cow::Borrowed("Björk Guð…"));
		assert_eq!("Björk Guðmundsdóttir".shorten(20), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!("Björk Guðmundsdóttir".shorten(100), Cow::Borrowed("Björk Guðmundsdóttir"));
	}

	#[test]
	fn shorten_reverse() {
		assert_eq!("Hello World".shorten_reverse(0), Cow::Borrowed(""));
		assert_eq!("Hello World".shorten_reverse(1), Cow::Borrowed(ELLIPSIS));
		assert_eq!("Hello World".shorten_reverse(2), Cow::Borrowed("…d"));
		assert_eq!("Hello World".shorten_reverse(6), Cow::Borrowed("…World"));
		assert_eq!("Hello World".shorten_reverse(7), Cow::Borrowed("… World"));
		assert_eq!("Hello World".shorten_reverse(11), Cow::Borrowed("Hello World"));
		assert_eq!("Hello World".shorten_reverse(100), Cow::Borrowed("Hello World"));

		assert_eq!("Björk Guðmundsdóttir".shorten_reverse(0), Cow::Borrowed(""));
		assert_eq!("Björk Guðmundsdóttir".shorten_reverse(1), Cow::Borrowed(ELLIPSIS));
		assert_eq!("Björk Guðmundsdóttir".shorten_reverse(2), Cow::Borrowed("…r"));
		assert_eq!("Björk Guðmundsdóttir".shorten_reverse(6), Cow::Borrowed("…óttir"));
		assert_eq!("Björk Guðmundsdóttir".shorten_reverse(7), Cow::Borrowed("…dóttir"));
		assert_eq!("Björk Guðmundsdóttir".shorten_reverse(10), Cow::Borrowed("…ndsdóttir"));
		assert_eq!("Björk Guðmundsdóttir".shorten_reverse(20), Cow::Borrowed("Björk Guðmundsdóttir"));
		assert_eq!("Björk Guðmundsdóttir".shorten_reverse(100), Cow::Borrowed("Björk Guðmundsdóttir"));
	}
}
