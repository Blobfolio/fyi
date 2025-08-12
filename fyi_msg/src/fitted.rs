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
/// Slice (and maybe rejigger) a string to make it fit within a given display
/// width.
///
/// Unlike [`length_width`], this method works line-by-line,
/// evaluating/truncating each individually to keep them within `width` columns.
///
/// ANSI formatting sequences and line breaks are always preserved; a new
/// string will be allocated to stitch everything back together when necessary.
///
/// **This requires the `fitted` crate feature.**
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
/// This method does not "parse" ANSI sequences, but will recognize and
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
///
/// // If there are multiple sequences in the cut, they'll be added back in
/// // one big lump at the end, minus their original content.
/// let s = "\x1b[1mHello\x1b[0m \x1b[91mWorld\x1b[0m";
/// assert_eq!(
///     fit_to_width(s, 5),
///     "\x1b[1mHello\x1b[0m\x1b[91m\x1b[0m",
/// );
/// ```
pub fn fit_to_width(src: &str, width: usize) -> Cow<'_, str> {
	/// # Trailing Line Break?
	const fn terminator(src: &str) -> &str {
		match src.as_bytes() {
			[.., b'\r', b'\n'] => "\r\n",
			[.., b'\n'] => "\n",
			_ => "",
		}
	}

	// Easy aborts.
	if src.is_empty() || width == 0 { return Cow::Borrowed(""); }
	if src.len() <= width { return Cow::Borrowed(src); }

	// First pass: run through the lines unless/until we find one that doesn't
	// fit the specified width.
	let mut split = 0;
	let mut lines = src.split_inclusive('\n');
	for line in lines.by_ref() {
		let keep = length_width(line, width);

		// Everything fits!
		if keep == line.len() { split += keep; }
		// Second pass: build a new string with only the bits that fit.
		else {
			macro_rules! split_push {
				($line:ident, $keep:ident, $out:ident) => (
					// This should never fail.
					let Some((keep, kill)) = $line.split_at_checked($keep) else {
						return Cow::Borrowed("");
					};

					// Keep the fitable, "lost" ANSI, and trailing breaks.
					$out.push_str(keep);
					$out.extend(OnlyAnsi::new(kill));
					$out.push_str(terminator($line));
				);
			}

			// Start with what we know to fit.
			let mut out = String::with_capacity(src.len());
			out.push_str(&src[..split]);
			split_push!(line, keep, out);

			// Do the same for the remaining lines.
			for line in lines {
				let keep = length_width(line, width);
				if keep == line.len() { out.push_str(line); }
				else { split_push!(line, keep, out); }
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
/// *width*, such that `slice[..len]` will be a valid substring likely to fit.
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
/// let idx = fyi_msg::length_width(full, 5);
/// assert_eq!(
///     &full[..idx],
///     "\x1b[2mBjörk\x1b[0m",
/// );
///
/// // Be careful with strings containing ANSI sequences or line breaks,
/// // though; they can be lost depending on where the max width falls.
/// let idx = fyi_msg::length_width(full, 4);
/// assert_eq!(
///     &full[..idx],
///     "\x1b[2mBjör", // No reset!
/// );
///
/// // Were you to send the above to the printer, this would happen:
/// println!("{}", &full[..idx]); // Looks fine.
/// println!("Something else!");  // This is dim too!
///
/// // To avoid this, use `fit_to_width` instead.
/// let fitted = fyi_msg::fit_to_width(&full, 4);
/// assert_eq!(
///     fitted,
///     "\x1b[2mBjör\x1b[0m",     // There we are!
/// );
/// println!("{fitted}");         // Looks the same as before.
/// println!("Something else!");  // Now this looks fine too.
/// ```
pub fn length_width(src: &str, max_width: usize) -> usize {
	let mut width = 0;
	let mut iter = NoAnsi::new(src);
	while let Some(v) = iter.next() {
		width += UnicodeWidthChar::width(v).unwrap_or(0);
		// This one won't fit; rewind!
		if max_width < width {
			return iter.byte_pos() - v.len_utf8();
		}
	}

	// All good!
	src.len()
}

#[must_use]
/// # Width.
///
/// Find the (total) "display width" of a string slice.
///
/// Like anything having to do with width vs length, this should be considered
/// at best an approximation. The [`unicode_width`](https://crates.io/crates/unicode-width) crate
/// is used under-the-hood; refer to their documentation for more details on
/// how terrible Unicode is. Haha.
///
/// This method is ANSI-aware and will automatically skip past any
/// [CSI](https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences) or [OSC](https://en.wikipedia.org/wiki/ANSI_escape_code#OSC_(Operating_System_Command)_sequences)
/// sequences it encounters.
///
/// **This requires the `fitted` crate feature.**
///
/// ## Examples
///
/// ```
/// use fyi_msg::width;
///
/// // The problem with length is inclusivity:
/// assert_eq!("\x1b[1mBjörk\x1b[0m".len(), 14);
///
/// // Even without the ANSI, the length is still too high:
/// assert_eq!("Björk".len(), 6);
///
/// // Width is just what it looks like:
/// assert_eq!(width("\x1b[1mBjörk\x1b[0m"), 5);
/// assert_eq!(
///     width("Björk"), // Unicode.
///     width("Bjork"), // ASCII.
/// );
///
/// // Be careful with multi-line content; widths are _total_.
/// assert_eq!(width("Hello World"),  11);
/// assert_eq!(width("Hello\nWorld"), 10); // Line breaks have no width.
/// ```
pub fn width(src: &str) -> usize {
	NoAnsi::new(src).fold(0, |acc, v|
		UnicodeWidthChar::width(v).map_or(acc, |w| acc + w)
	)
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_fit_to_width() {
		for &(slice, stop, expected) in &[
			("Hello\nWorld", 4, "Hell\nWorl"),
			("\x1b[1mHello\x1b[0m\n\x1b[2mWorld\x1b[0m", 4, "\x1b[1mHell\x1b[0m\n\x1b[2mWorl\x1b[0m"),
			("\n\nHello\n\n", 4, "\n\nHell\n\n"),
			("\n\nHello\n\n", 5, "\n\nHello\n\n"),
			("\n\nHello\n\n", 500, "\n\nHello\n\n"),
		] {
			assert_eq!(
				fit_to_width(slice, stop),
				expected,
			);
		}
	}

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
				length_width(slice, stop),
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
				width(slice), expected,
				"Invalid width for {slice:?}."
			);
		}
	}
}
