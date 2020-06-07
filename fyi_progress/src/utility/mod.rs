/*!
# FYI Progress: Utility

This mod contains miscellaneous utility functions for the crate.
*/

use unicode_width::UnicodeWidthChar;
mod time;

/// Re-exports.
pub use time::{
	human_elapsed,
	secs_chunks,
};



#[must_use]
/// Byte Index At Width
///
/// Go char-by-char through a string, counting the widths of printable
/// characters, and return the byte length of the substring that fits.
pub fn chopped_len(buf: &[u8], width: usize) -> usize {
	let len: usize = buf.len();
	// Width can't exceed length, so we can quit early if it is short.
	if len <= width { len }
	else {
		let mut cur_len: usize = 0;
		let mut cur_width: usize = 0;
		let mut in_ansi: bool = false;

		for c in unsafe { std::str::from_utf8_unchecked(buf) }.chars() {
			let char_len: usize = c.len_utf8();

			// If we're in the middle of an ANSI command, normal printability
			// does not apply. But if we hit an exit, we should note that for
			// the next loop.
			if in_ansi {
				if c == 'A' || c == 'K' || c == 'm' {
					in_ansi = false;
				}

				cur_len += char_len;
				continue;
			}
			// Enter an ANSI command.
			else if c == '\x1b' {
				in_ansi = true;
				cur_len += char_len;
				continue;
			}

			// For everything else, trust the `UnicodeWidth` crate to come up
			// with a decent size estimation.
			let char_width: usize = UnicodeWidthChar::width(c).unwrap_or(0);
			cur_len += char_len;
			cur_width += char_width;

			// If we went over budget, back up a step and exit.
			if cur_width > width {
				cur_len -= char_len;
				break;
			}
		}

		cur_len
	}
}

#[must_use]
/// Term Width
///
/// This is a simple wrapper around `term_size::dimensions()` to provide
/// the current terminal column width. We don't have any use for height,
/// so that property is ignored.
pub fn term_width() -> usize {
	// Reserve one space at the end "just in case".
	if let Some((w, _)) = term_size::dimensions() { w.saturating_sub(1) }
	else { 0 }
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_chopped_len() {
		assert_eq!(chopped_len(b"Hello World", 15), 11);
		assert_eq!(chopped_len(b"Hello \x1b[1mWorld\x1b[0m", 15), 19);
		assert_eq!(chopped_len(b"Hello \x1b[1mWorld\x1b[0m", 7), 11);
	}
}
