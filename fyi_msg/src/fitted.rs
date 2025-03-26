/*!
# FYI Msg - Fitted Widths

This optional module contains methods for counting the display width of byte strings, and/or figuring out the closest char index to chop on to make something fit.
*/

use crate::ansi::{
	NoAnsi,
	OnlyAnsi,
};
use std::borrow::Cow;
use unicode_width::UnicodeWidthChar;



#[must_use]
/// # Fit to Width.
///
/// Return a version of the string cropped to the target display `width`.
///
/// This method works line-by-line, truncating any that are too long. Line
/// breaks are preserved (even if the line gets chopped), except in cases where
/// a line has to be completely dropped.
///
/// Allocations are only made if alteration is required, otherwise the original
/// is passed back through.
///
/// ## Examples
///
/// ```
/// use fyi_msg::fit_to_width;
///
/// // A single line string that fits as-is.
/// assert_eq!(
///     fit_to_width("Hello World", 11),
///     "Hello World",
/// );
///
/// // A single line with trailing break that doesn't _quite_ fit. Note the
/// // "d" is dropped, but not the trailing "\n".
/// assert_eq!(
///     fit_to_width("Hello World\n", 10),
///     "Hello Worl\n",
/// );
///
/// // Same as above, but with a stupid "\r\n" break, which is also supported.
/// assert_eq!(
///     fit_to_width("Hello World\r\n", 10),
///     "Hello Worl\r\n",
/// );
///
/// // Multi-line trim!
/// assert_eq!(
///     fit_to_width("\
///         Apples\n\
///         Bananas\n\
///         Carrots\n\
///     ", 6),
///     "\
///     Apples\n\
///     Banana\n\
///     Carrot\n\
///     ",
/// );
/// ```
///
/// This method does not "parse" Ansi sequences, but will recognize and
/// preserve them (even in chopped regions) to prevent any accidental
/// display weirdness.
///
/// ```
/// use fyi_msg::fit_to_width;
///
/// let s = "\x1b[1mHello World\x1b[0m";
/// assert_eq!(
///     fit_to_width(s, 5),
///     "\x1b[1mHello\x1b[0m", // The reset was saved!
/// );
/// ```
pub fn fit_to_width(src: &str, width: usize) -> Cow<str> {
	/// # Trailing Line Break?
	fn terminator(src: &str) -> &str {
		if src.ends_with("\r\n") { "\r\n" }
		else if src.ends_with('\n') { "\n" }
		else { "" }
	}

	// Easy aborts.
	if src.is_empty() || width == 0 { return Cow::Borrowed(""); }
	if src.len() <= width { return Cow::Borrowed(src); }

	let mut lines = src.split_inclusive('\n');

	// First pass: run through the lines unless/until we find one that doesn't
	// fit the specified width.
	let mut split = 0;
	for line in lines.by_ref() {
		let keep = length_width(line.as_bytes(), width);

		// Everything fits!
		if keep == line.len() { split += keep; }
		// Second pass: build a new string with only the bits that fit.
		else {
			// Start with the good line(s).
			let mut out = String::with_capacity(src.len());
			out.push_str(&src[..split]);

			// This should never fail.
			let Some((keep, kill)) = line.split_at_checked(keep) else {
				return Cow::Borrowed("");
			};

			// Keep the keepable, "lost" ANSI formatting, and/or trailing
			// line breaks.
			out.push_str(keep);
			for seq in OnlyAnsi::new(kill) { out.push_str(seq); }
			out.push_str(terminator(line));

			// The rest of the lines.
			for line in lines {
				let keep = length_width(line.as_bytes(), width);
				if keep == line.len() { out.push_str(line); }
				else {
					// This should never fail.
					let Some((keep, kill)) = line.split_at_checked(keep) else {
						return Cow::Borrowed("");
					};
					out.push_str(keep);
					for seq in OnlyAnsi::new(kill) { out.push_str(seq); }
					out.push_str(terminator(line));
				}
			}

			return Cow::Owned(out);
		}
	}

	// Everything fit!
	Cow::Borrowed(src)
}

#[must_use]
/// # Length Width.
///
/// Return the maximum byte *length* for the slice that fits a given display
/// *width*, such that `slice[0..len]` will be a valid substring likely to fit.
///
/// This method accepts raw bytes for performance reasons, but is Unicode-safe;
/// the return value will always be a valid char boundary. In cases where the
/// input contains invalid UTF-8, only the leading ASCII bytes will be
/// considered/counted.
///
/// See the documentation for [`width`] for more information.
///
/// **This requires the `fitted` crate feature.**
///
/// ## Examples
///
/// ```
/// // Split to a display width of five.
/// let full: &str = "\x1b[2mBjörk\x1b[0m Guðmundsdóttir";
/// let idx = fyi_msg::length_width(full.as_bytes(), 5);
/// assert_eq!(
///     &full[..idx],
///     "\x1b[2mBjörk\x1b[0m",
/// );
/// ```
pub fn length_width(bytes: &[u8], stop: usize) -> usize {
	// Split on first non-ASCII character.
	let (a, b): (&[u8], &[u8]) = bytes.iter()
		.position(|b| ! b.is_ascii())
		.map_or((bytes, &[]), |pos| bytes.split_at(pos));

	// Short circuit.
	if a.len() <= stop && b.is_empty() { return bytes.len(); }

	// Iterate through the ASCII parts first, assuming length and width are
	// equivalent for non-control characters.
	let mut width = 0;
	let mut iter = NoAnsi::<u8, _>::new(a.iter().copied());
	while let Some(v) = iter.next() {
		if v != 0 && ! v.is_ascii_control() {
			if width == stop { return iter.byte_pos() - 1; }
			width += 1;
		}
	}

	// If we're still here, stringify the rest and keep going!
	if ! b.is_empty() {
		let Ok(b) = std::str::from_utf8(b) else { return a.len(); };
		let mut iter = NoAnsi::<char, _>::new(b.chars());
		while let Some(v) = iter.next() {
			width += UnicodeWidthChar::width(v).unwrap_or(0);
			// This one won't fit; rewind!
			if stop < width {
				return a.len() + iter.byte_pos() - v.len_utf8();
			}
		}
	}

	// The original length fits just fine.
	bytes.len()
}

#[must_use]
/// # Width.
///
/// Find the "display width" of a byte string.
///
/// This method accepts raw bytes for performance reasons, but is Unicode-safe.
/// In cases where the input contains invalid UTF-8, only the leading ASCII
/// bytes will be considered/counted.
///
/// Like anything having to do with width vs length, this should be considered
/// at best an approximation. For ASCII, every byte that is not a control
/// character or part of an ANSI [CSI](https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences) or [OSC](https://en.wikipedia.org/wiki/ANSI_escape_code#OSC_(Operating_System_Command)_sequences) sequence
/// is counted as having a length of 1. For Unicode, the [`unicode_width`](https://crates.io/crates/unicode-width) crate is used
/// to determine width.
///
/// Note: line breaks are ignored; the cumulative width of all lines is
/// returned. If you're trying to calculate *line* widths.
///
/// **This requires the `fitted` crate feature.**
///
/// ## Examples
///
/// ```
/// // Line breaks have no width:
/// assert_ne!(
///     fyi_msg::width(b"Hello World"),
///     fyi_msg::width(b"Hello\nWorld"),
/// );
/// ```
pub fn width(bytes: &[u8]) -> usize {
	// Short circuit.
	if bytes.is_empty() { return 0; }

	// Split on first non-ASCII character.
	let (a, b): (&[u8], &[u8]) = bytes.iter()
		.position(|b| ! b.is_ascii())
		.map_or((bytes, &[]), |pos| bytes.split_at(pos));

	// For the ASCII half, assume length and width are equivalent, except for
	// control characters.
	let mut width: usize = NoAnsi::<u8, _>::new(a.iter().copied())
		.fold(0, |acc, v|
			if v == 0 || v.is_ascii_control() { acc }
			else { acc + 1 }
		);

	// For the rest (if any), use the Unicode width estimate.
	if ! b.is_empty() {
		if let Ok(chars) = std::str::from_utf8(b) {
			width += NoAnsi::<char, _>::new(chars.chars())
				.fold(0, |acc, v|
					UnicodeWidthChar::width(v).map_or(acc, |w| acc + w)
				);
		}
	}

	width
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_length_width() {
		for &(slice, stop, expected) in &[
			("Hello", 5, 5),
			("Hello", 6, 5),
			("Hello", 4, 4),
			("Hello\nWorld", 10, 11),
			("Björk Guðmundsdóttir", 5, 6),
			("Björk Guðmundsdóttir", 3, 4),
			("Björk Guðmundsdóttir", 2, 2),
			("\x1b[2mBjörk\x1b[0m Guðmundsdóttir", 5, 14),
			("\x1b[2mHello\x1b[0m World", 5, 13),
		] {
			assert_eq!(
				length_width(slice.as_bytes(), stop),
				expected,
				"Invalid length/width for {slice:?} fit to {stop}."
			);
		}
	}

	#[test]
	fn t_width() {
		for &(slice, expected) in &[
			("Hello\nWorld", 10),
			("Hello World", 11),
			("\x1b[2mHello\x1b[0m World", 11),
			("Hello World…", 12),
			("Björk Guðmundsdóttir", 20),
			("\x1b[2mBjörk\x1b[0m Guðmundsdóttir", 20),
			("Björk \x1b[2mGuðmundsdóttir\x1b[0m", 20),
		] {
			assert_eq!(
				width(slice.as_bytes()), expected,
				"Invalid width for {slice:?}."
			);
		}
	}
}
