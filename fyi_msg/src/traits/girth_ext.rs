/*!
# FYI Msg Traits: `GirthExt`

The `GirthExt` trait brings additional length-related helpers to UTF-8 string
types:
* `count_chars()` Returns the total number of characters.
* `count_lines()` Returns the number of lines (`\n`-separated) the value occupies.
* `count_width()` Returns an approximate "column width" for the string, i.e. the printable bits.

## Example:

```no_run
use fyi_msg::traits::GirthExt;

assert_eq!("Björk".len(), 7);
assert_eq!("Björk".count_chars(), 5);
assert_eq!("Björk".count_width(), 5);
assert_eq!("Björk".count_lines(), 1);

assert_eq!("\x1B[1mBjörk".len(), 11);
assert_eq!("\x1B[1mBjörk".count_chars(), 9);
assert_eq!("\x1B[1mBjörk".count_width(), 5);
assert_eq!("\x1B[1mBjörk".count_lines(), 1);
```
*/

use unicode_width::UnicodeWidthChar;



/// Extra Length Helpers.
pub trait GirthExt {
	/// Number of characters.
	///
	/// This is largely the same as doing `str.chars().count()`, but faster.
	fn count_chars(&self) -> usize;

	/// Number of lines.
	///
	/// This returns `"\n".count() + 1` for strings with length, or `0` for
	/// empty strings.
	fn count_lines(&self) -> usize;

	/// Display Width.
	///
	/// This method returns an *approximate* display/column width for a string.
	/// Like every other Rust program, it uses the `unicode-width` to determine
	/// the printable width of each character, but it also avoids certain ANSI
	/// sequences (namely the color stylings FYI uses).
	///
	/// Like with `unicode-width`, it will incorrectly measure certain
	/// combinative characters, and because this is optimized for speed around
	/// our own use cases, it will also incorrectly measure ANSI formatting
	/// we don't account for.
	///
	/// For a slightly more accurate (and slower) reading for ANSI-containing
	/// strings, run `val.strip_ansi()` first, then calculate the width.
	///
	/// If you aren't using any "fat" characters, combine `val.strip_ansi()`
	/// (if there's ANSI) with `val.count_chars()` for a simple char count,
	/// which is more or less equivalent to display width in such cases.
	fn count_width(&self) -> usize;
}

/// The main implementation is for Stringish things.
impl GirthExt for [u8] {
	#[inline]
	/// Number of characters.
	fn count_chars(&self) -> usize {
		if self.is_empty() { 0 }
		else {
			bytecount::num_chars(self)
		}
	}

	#[inline]
	/// Number of lines.
	fn count_lines(&self) -> usize {
		if self.is_empty() { 0 }
		else {
			bytecount::count(self, b'\n') + 1
		}
	}

	/// Display Width.
	fn count_width(&self) -> usize {
		if self.is_empty() { 0 }
		else {
			let mut in_ansi: bool = false;
			unsafe { std::str::from_utf8_unchecked(self) }.chars()
				.fold(0, |width, c| {
					if in_ansi {
						if c == 'A' || c == 'K' || c == 'm' {
							in_ansi = false;
						}
						width
					}
					else if c == '\x1b' {
						in_ansi = true;
						width
					}
					else {
						width + UnicodeWidthChar::width(c).unwrap_or(0)
					}
				})
		}
	}
}

/// The main implementation is for Stringish things.
impl GirthExt for str {
	#[inline]
	/// Number of characters.
	fn count_chars(&self) -> usize {
		if self.is_empty() { 0 }
		else {
			bytecount::num_chars(self.as_bytes())
		}
	}

	#[inline]
	/// Number of lines.
	fn count_lines(&self) -> usize {
		if self.is_empty() { 0 }
		else {
			bytecount::count(self.as_bytes(), b'\n') + 1
		}
	}

	/// Display Width.
	fn count_width(&self) -> usize {
		if self.is_empty() { 0 }
		else {
			let mut in_ansi: bool = false;
			self.chars()
				.fold(0, |width, c| {
					if in_ansi {
						if c == 'A' || c == 'K' || c == 'm' {
							in_ansi = false;
						}
						width
					}
					else if c == '\x1b' {
						in_ansi = true;
						width
					}
					else {
						width + UnicodeWidthChar::width(c).unwrap_or(0)
					}
				})
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn count_chars() {
		_count_chars("", 0);
		_count_chars("Hello", 5);
		_count_chars("\x1B[1mHello", 9);
		_count_chars("Björk", 5);
	}

	fn _count_chars(text: &str, expected: usize) {
		assert_eq!(
			text.count_chars(),
			expected,
			"{:?} should be have {} chars",
			text,
			expected
		);

		assert_eq!(
			text.as_bytes().count_chars(),
			expected,
			"{:?} should be have {} chars",
			text.as_bytes(),
			expected
		);
	}

	#[test]
	fn count_lines() {
		_count_lines("", 0);
		_count_lines("Hello", 1);
		_count_lines("Hello\nWorld", 2);
		_count_lines("Hello\nWorld\n", 3);
	}

	fn _count_lines(text: &str, expected: usize) {
		assert_eq!(
			text.count_lines(),
			expected,
			"{:?} should be have {} lines",
			text,
			expected
		);

		assert_eq!(
			text.as_bytes().count_lines(),
			expected,
			"{:?} should be have {} lines",
			text.as_bytes(),
			expected
		);
	}

	#[test]
	fn count_width() {
		_count_width("", 0);
		_count_width("Hello", 5);
		_count_width("\x1B[1mHello", 5);
		_count_width("Björk", 5);
		_count_width("B\u{1b}[1;38;5;199mjö\x1b[0mrk", 5);
	}

	fn _count_width(text: &str, expected: usize) {
		assert_eq!(
			text.count_width(),
			expected,
			"{:?} should be have a width of {}",
			text,
			expected
		);

		assert_eq!(
			text.as_bytes().count_width(),
			expected,
			"{:?} should be have a width of {}",
			text.as_bytes(),
			expected
		);
	}
}
