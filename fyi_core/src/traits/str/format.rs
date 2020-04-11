use std::borrow::Cow;

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
		if false == self.as_ref().is_empty() {
			const STATE_NORMAL: u8 = 1;
			const STATE_ESCAPE: u8 = 2;
			const STATE_CSI: u8 = 3;

			let mut state = STATE_NORMAL;
			Cow::Owned(
				String::from_utf8(
					self.as_ref().as_bytes()
						.iter()
						.filter_map(|x|
							match state {
								STATE_NORMAL => if *x == 0x1B {
										state = STATE_ESCAPE;
										None
									} else {
										Some(*x)
									},
								STATE_ESCAPE => if *x == 0x5B {
										state = STATE_CSI;
										None
									} else {
										state = STATE_NORMAL;
										None
									},
								STATE_CSI => if *x >= 0x40 && *x < 0x80 {
										state = STATE_NORMAL;
										None
									} else {
										None
									}
								_ => None,
							}
						)
						.collect::<Vec<u8>>()
				).expect("Failed to unwrap string.")
			)
		}
		else {
			Cow::Borrowed("")
		}
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
