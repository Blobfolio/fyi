/*!
# FYI Msg: ANSI Helpers
*/

use fyi_ansi::csi;
use std::{
	fmt,
	hash,
	ops::Range,
};

#[cfg(any(feature = "fitted", feature = "progress"))]
use std::str::Chars;



/// # Escape Character.
const ANSI_ESCAPE: char = '\x1b';

/// # Bell Character.
const ANSI_BELL: char = '\x07';

/// # OE Character.
const ANSI_OE: char = '\u{0153}';



include!(concat!(env!("OUT_DIR"), "/ansi-color.rs"));



impl fmt::Display for AnsiColor {
	#[inline]
	/// # Print ANSI Sequence.
	///
	/// Print the full ANSI sequence for the selected color, e.g.
	/// `"\x1b[38;5;199m"`.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::AnsiColor;
	///
	/// assert_eq!(
	///     AnsiColor::LightMagenta.to_string(),
	///     "\x1b[95m",
	/// );
	///
	/// // Note the same thing can be obtained without allocation:
	/// assert_eq!(
	///     AnsiColor::LightMagenta.as_str(),
	///     "\x1b[95m",
	/// );
	/// ```
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		<str as fmt::Display>::fmt(self.as_str(), f)
	}
}

impl From<u8> for AnsiColor {
	#[inline]
	/// # From `u8`.
	///
	/// Note this can also be achieved using the const [`AnsiColor::from_u8`]
	/// method.
	///
	/// ```
	/// use fyi_msg::AnsiColor;
	///
	/// // All the colors of the termbow!
	/// for i in u8::MIN..=u8::MAX {
	///     assert_eq!(
	///         AnsiColor::from(i),
	///         AnsiColor::from_u8(i),
	///     );
	/// }
	/// ```
	fn from(num: u8) -> Self { Self::from_u8(num) }
}

impl hash::Hash for AnsiColor {
	#[inline]
	fn hash<H: hash::Hasher>(&self, state: &mut H) { state.write_u8(*self as u8); }
}

impl PartialEq<u8> for AnsiColor {
	#[inline]
	fn eq(&self, other: &u8) -> bool { *self as u8 == *other }
}

impl PartialEq<AnsiColor> for u8 {
	#[inline]
	fn eq(&self, other: &AnsiColor) -> bool { *self == *other as Self }
}

impl AnsiColor {
	/// # ANSI Reset Sequence.
	///
	/// This tiny sequence resets all style/color attributes back to the
	/// terminal defaults.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::AnsiColor;
	///
	/// // Only "Blue" prints, well, blue.
	/// println!(
	///     "{}Blue{} is the warmest color.",
	///     AnsiColor::Blue,
	///     AnsiColor::RESET,
	/// );
	/// ```
	pub const RESET: &'static str = csi!();

	/// # Prefix Reset.
	///
	/// Same as [`AnsiColor::RESET`], but with a colon before and space after.
	pub(crate) const RESET_PREFIX: &'static str = concat!(":", csi!(), " ");
}



#[cfg(any(feature = "fitted", feature = "progress"))]
#[derive(Debug, Clone)]
/// # ANSI-Stripping Iterator.
///
/// This iterator strips ANSI [CSI](https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences) and [OSC](https://en.wikipedia.org/wiki/ANSI_escape_code#OSC_(Operating_System_Command)_sequences) sequences
/// from a string slice.
///
/// Note that other types of (less common) ANSI sequences and miscellaneous
/// control characters may remain.
pub(crate) struct NoAnsi<'a> {
	/// # (Remaining) Source.
	iter: Chars<'a>,

	/// # Buffer Character.
	buf: Option<char>,

	/// # Byte Position.
	///
	/// The current position inside the original source.
	pos: usize,
}

#[cfg(any(feature = "fitted", feature = "progress"))]
impl<'a> NoAnsi<'a> {
	#[inline]
	#[must_use]
	/// # New.
	///
	/// Create a new instance from a string slice.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use fyi_msg::ansi::NoAnsi;
	///
	/// let fmt: &str = "\x1b[1mHello\x1b[0m \x1b[38;5;199mWorld!\x1b[m";
	/// assert_eq!(
	///     NoAnsi::new(fmt).collect::<String>(),
	///     "Hello World!",
	/// );
	/// ```
	pub(crate) fn new(src: &'a str) -> Self {
		Self {
			iter: src.chars(),
			buf: None,
			pos: 0,
		}
	}

	#[inline]
	#[must_use]
	/// # Bytes Traversed.
	///
	/// Return the number of bytes traversed by the iterator (so far),
	/// including any ANSI sequences that have been ignored.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use fyi_msg::ansi::NoAnsi;
	///
	/// let raw = "\x1b[1mHello\x1b[0m";
	/// let mut iter = NoAnsi::new(raw);
	///
	/// // The leading ANSI sequence is ignored; the first thing we get back
	/// // from the iterator is the H.
	/// assert_eq!(iter.next(), Some('H'));
	///
	/// // The byte position counts everything, though, so can be used to
	/// // figure out where we are relative to the original slice.
	/// assert_eq!(
	///     &raw[..iter.byte_pos()],
	///     "\x1b[1mH",
	/// );
	/// ```
	pub(crate) const fn byte_pos(&self) -> usize { self.pos }

	#[cold]
	/// # Drain OSC Sequence.
	///
	/// Advance the wrapped iterator until an ANSI OSC break is found.
	fn drain_osc(&mut self) {
		while let Some(c) = self.buf.take().or_else(|| self.iter.next()) {
			self.pos += c.len_utf8();
			match c {
				ANSI_OE | ANSI_BELL => { break; },
				ANSI_ESCAPE => {
					// This is only actually the end if
					// followed by a backslash.
					match self.iter.next() {
						Some('\\') => {
							self.pos += 1;
							break;
						},
						// Save for next time.
						Some(c) => { self.buf.replace(c); },
						_ => {},
					}
				},
				_ => {},
			}
		}
	}
}

#[cfg(any(feature = "fitted", feature = "progress"))]
impl Iterator for NoAnsi<'_> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(next) = self.buf.take().or_else(|| self.iter.next()) {
			if ANSI_ESCAPE == next {
				match self.iter.next() {
					// New CSI sequence.
					Some('[') => {
						// Count the opening bytes, then drain (and count) the
						// remainder of the sequence.
						self.pos += 2;
						for c in self.iter.by_ref() {
							self.pos += c.len_utf8();
							if matches!(c, '\x40'..='\x7E') { break; }
						}
						continue;
					},
					// New OSC sequence.
					Some(']') => {
						self.pos += 2;
						self.drain_osc();
						continue;
					},
					// Save for next time.
					Some(c) => { self.buf.replace(c); },
					None => {},
				}
			}

			self.pos += next.len_utf8();
			return Some(next);
		}

		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let (_, max) = self.iter.size_hint();
		(0, max)
	}
}

#[cfg(any(feature = "fitted", feature = "progress"))]
impl std::iter::FusedIterator for NoAnsi<'_> {}



#[cfg(feature = "fitted")]
/// # Only ANSI Iterator.
///
/// This iterator yields the ANSI sequences within a string, but nothing else.
/// It is effectively the opposite of the [`NoAnsi`] iterator.
///
/// Internally, we use this to find and preserve sequences that would otherwise
/// be lost when chopping a string to its display width.
pub(crate) struct OnlyAnsi<'a>(&'a str);

#[cfg(feature = "fitted")]
impl<'a> OnlyAnsi<'a> {
	#[inline]
	#[must_use]
	/// # New Instance.
	///
	/// Initialize an iterator for the given string slice.
	pub(crate) const fn new(src: &'a str) -> Self { Self(src) }
}

#[cfg(feature = "fitted")]
impl<'a> Iterator for OnlyAnsi<'a> {
	type Item = &'a str;

	fn next(&mut self) -> Option<Self::Item> {
		let rng = next_ansi(self.0)?;
		let end = rng.end;
		let next = self.0.get(rng);               // Shouldn't ever fail.
		self.0 = self.0.get(end..).unwrap_or(""); // Shouldn't ever fail.
		next
	}
}

#[cfg(feature = "fitted")]
impl std::iter::FusedIterator for OnlyAnsi<'_> {}



/// # Next ANSI Sequence.
///
/// Find and return the range corresponding to the first/next ANSI sequence
/// within the string, or `None`.
pub(crate) fn next_ansi(src: &str) -> Option<Range<usize>> {
	// Find the start of the range.
	let len = src.len();
	let start = src.as_bytes()
		.windows(2)
		.position(|pair| matches!(pair, b"\x1b[" | b"\x1b]"))?;

	// Now let's look for the end.
	let mut end = None;
	let mut chars = src.get(start..)?.char_indices().skip(1);
	match chars.next() {
		// CSI sequence.
		Some((1, '[')) => {
			end = chars.find_map(|(k, c)| matches!(c, '\x40'..='\x7E').then_some(k + c.len_utf8()));
		},
		// OSC sequence.
		Some((1, ']')) => {
			// These unfortunately have one- and two-byte stops. Haha.
			let mut buf = None;
			while let Some((k, c)) = buf.take().or_else(|| chars.next()) {
				match c {
					ANSI_OE | ANSI_BELL => {
						end = Some(k + c.len_utf8());
						break;
					},
					ANSI_ESCAPE => match chars.next() {
						Some((k, '\\')) => {
							end = Some(k + c.len_utf8());
							break;
						},
						Some(nope) => { buf.replace(nope); },
						None => {},
					},
					_ => {},
				}
			}
		},
		// Unreachable.
		_ => {},
	}

	Some(start..start + end.unwrap_or(len))
}



#[cfg(test)]
mod test {
	use super::*;

	#[cfg(any(feature = "fitted", feature = "progress"))]
	#[test]
	fn t_ansi_csi() {
		// This library does a lot with ANSI CSI sequences so the functionality
		// is well covered, but no harm adding a few more checks.
		const LIST: [&str; 4] = [
			"One \x1b[1mTwo\x1b[0m Three",
			"\x1b[38;5;199mBjörk\x1b[m \x1b[1mBjörk\x1b[0m \x1b[1;2;91mBjörk\x1b[0m",
			"\x1b\x1b[1mFakeout!\x1b[0m",
			"text/plain",
		];

		for (raw, expected) in LIST.iter().zip([
			"One Two Three",
			"Björk Björk Björk",
			"\x1bFakeout!",
			"text/plain",
		]) {
			assert_eq!(
				NoAnsi::new(raw).collect::<String>(),
				expected,
			);
		}

		#[cfg(feature = "fitted")]
		for (raw, expected) in LIST.iter().zip([
			"\x1b[1m\x1b[0m",
			"\x1b[38;5;199m\x1b[m\x1b[1m\x1b[0m\x1b[1;2;91m\x1b[0m",
			"\x1b[1m\x1b[0m",
			"",
		]) {
			assert_eq!(
				OnlyAnsi::new(raw).collect::<String>(),
				expected,
			);
		}
	}

	#[cfg(any(feature = "fitted", feature = "progress"))]
	#[test]
	fn t_ansi_osc() {
		// Verify two-byte endings are properly accounted for when mixed and
		// matched.
		for i in [
			"One \x1b]Two\x07 Three",
			"One \x1b]Twoœ Three",
			"One \x1b]Two\x1b\\ Three",
			"One \x1b]Two\x1b\x1b\\ Three",    // Double escape.
			"One \x1b]Two\x1bHi\x1b\\ Three",  // Fakeout.
			"One \x1b]Two\x1b\x07 Three",      // Escape followed by bell.
			"One \x1b]Two\x1bœ Three",         // Escape followed by oe.
			"One \x1b]Two\x1b\x1bœ Three",     // Escape Escape oe.
			"One \x1b]Two\x1b\x1b\x1bœ Three", // Escape Escape Escape oe.
		] {
			let stripped: String = NoAnsi::new(i).collect();
			assert_eq!(stripped, "One  Three", "Chars: {:?}", i.as_bytes());

			#[cfg(feature = "fitted")]
			assert_eq!(
				OnlyAnsi::new(i).collect::<String>(),
				i.strip_prefix("One ").and_then(|j| j.strip_suffix(" Three")).unwrap(),
				"Raw: {i:?}",
			);
		}
	}

	#[test]
	fn t_next_ansi() {
		// Most cases are covered elsewhere, but let's make sure multi-byte
		// positioning works out as expected.
		assert_eq!(
			next_ansi("Björk \x1b[2mGuðmundsdóttir\x1b[0m"),
			Some(7..11),
		);

		// And so do unclosed tags.
		assert_eq!(
			next_ansi("\x1b[38"),
			Some(0..4),
		);
	}

	#[cfg(any(feature = "fitted", feature = "progress"))]
	#[test]
	fn t_byte_pos() {
		for s in [
			"Björk \x1b[2mGuðmundsdóttir\x1b[0m",
			"One \x1b]Two\x1b\x1b\x1bœ Three",
			"One \x1b]Two\x1b\x1bœ Three",
			"One \x1b]Two\x1bHi\x1b\\ Three",
			"One \x1b]Twoœ Three",
		] {
			// Make sure string bytes are counted correctly.
			let mut iter = NoAnsi::new(s);
			while iter.next().is_some() { }
			assert_eq!(s.len(), iter.byte_pos());
		}

		// Let's test one more explicitly.
		let raw = "\x1b[1mHello\x1b[0m";
		let mut iter = NoAnsi::new(raw);
		assert_eq!(iter.next(), Some('H'));
		assert_eq!(&raw[..iter.byte_pos()], "\x1b[1mH");
	}
}
