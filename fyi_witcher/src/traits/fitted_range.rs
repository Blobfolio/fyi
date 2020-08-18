/*!
# FYI Witcher: Traits: Fitted Range

This helps deal with the distinction between "length" and "width".
*/

use std::ops::Range;
use unicode_width::UnicodeWidthChar;



/// Return a `Range` representing the parts of the string that "fit" within a
/// given printable width, i.e. `0..len()` if it fits, or something shorter if
/// chopping is needed.
///
/// This is at best an approximation as the concept of "width" is mysterious
/// and unknowable. Haha. For our purposes, ANSI escape patterns are treated as
/// zero width, otherwise we use the approximations made by the `unicode_width`
/// trait, sharing in its successes and limitations.
pub trait FittedRange {
	/// Fitted Range
	///
	/// Return the range that fits.
	fn fitted_range(&self, width: usize) -> Range<usize>;
}

#[allow(clippy::use_self)] // False positive: UnicodeWidthChar::width()
impl FittedRange for str {
	/// Fitted Range
	///
	/// Return the range that fits.
	fn fitted_range(&self, width: usize) -> Range<usize> {
		// Width can't exceed length, so only count up widths if we need to.
		let len: usize = self.len();
		if len > width {
			// We need to run char through char, adding up widths and lengths
			// as we go.
			let mut total_len: usize = 0;
			let mut total_width: usize = 0;

			// For our purposes, we're considering a subset of basic ANSI
			// escape patterns as being zero-width. This is by no means
			// exhaustive and will provide invalid counts for things like
			// padding and cursor movements.
			let mut doing_ansi: bool = false;

			// The loop.
			for c in self.chars() {
				// The sum of char lengths should be equivalent to the Range's
				// end.
				let ch_len: usize = c.len_utf8();
				total_len += ch_len;

				// If we're in the middle of an ANSI sequence, nothing counts
				// toward width, but we need to watch for the end so we can
				// start paying attention again.
				if doing_ansi {
					// Really simple, look for an "A", "K", or "m" to signal
					// the end.
					if c == 'A' || c == 'K' || c == 'm' {
						doing_ansi = false;
					}
					continue;
				}
				// Or maybe we're just entering ANSIville? (Look for the escape
				// char).
				else if c == '\x1b' {
					doing_ansi = true;
					continue;
				}

				// The width matters!
				let ch_width: usize = UnicodeWidthChar::width(c).unwrap_or_default();
				total_width += ch_width;

				// Widths can creep up. If we went over, back up a step and
				// exit.
				if total_width > width {
					return Range { start: 0, end: total_len - ch_len };
				}
			}
		}

		// We didn't exit, ergo it fits!
		Range { start: 0, end: len }
	}
}

impl FittedRange for &[u8] {
	fn fitted_range(&self, width: usize) -> Range<usize> {
		unsafe { std::str::from_utf8_unchecked(self) }.fitted_range(width)
	}
}

impl FittedRange for Vec<u8> {
	fn fitted_range(&self, width: usize) -> Range<usize> {
		unsafe { std::str::from_utf8_unchecked(self) }.fitted_range(width)
	}
}



/// This trait allows for in-place chopping based on the results coughed up
/// from `FittedRange`. Only owned types can implement this, so at the moment
/// it is limited to byte vectors.
pub trait FittedRangeMut {
	/// Fit To Range
	fn fit_to_range(&mut self, width: usize);
}

impl FittedRangeMut for Vec<u8> {
	fn fit_to_range(&mut self, width: usize) {
		let rg = self.fitted_range(width);
		if rg.end < self.len() {
			self.truncate(rg.end);
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_chopped_len() {
		assert_eq!("Hello World".fitted_range(15), 0..11);
		assert_eq!("Hello \x1b[1mWorld\x1b[0m".fitted_range(15), 0..19);
		assert_eq!("Hello \x1b[1mWorld\x1b[0m".fitted_range(7), 0..11);
	}
}
