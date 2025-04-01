/*!
# FYI Msg: ANSI Helpers
*/

use std::{
	fmt,
	hash,
	ops::Range,
};



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
	/// use fyi_msg::ansi::AnsiColor;
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
	/// use fyi_msg::ansi::AnsiColor;
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
	/// use fyi_msg::ansi::AnsiColor;
	///
	/// // Only "Blue" prints, well, blue.
	/// println!(
	///     "{}Blue{} is the warmest color.",
	///     AnsiColor::Blue,
	///     AnsiColor::RESET,
	/// );
	/// ```
	pub const RESET: &'static str = "\x1b[0m";

	/// # Prefix Reset.
	///
	/// Same as [`AnsiColor::RESET`], but with a colon before and space after.
	pub(crate) const RESET_PREFIX: &'static str = ":\x1b[0m ";
}



#[derive(Debug, Clone)]
/// # ANSI-Stripping Iterator.
///
/// This iterator strips ANSI [CSI](https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences) and [OSC](https://en.wikipedia.org/wiki/ANSI_escape_code#OSC_(Operating_System_Command)_sequences) sequences
/// from existing `char`/`u8` iterators.
///
/// Note that other types of (less common) ANSI sequences and miscellaneous
/// control characters may remain.
pub struct NoAnsi<U: Copy + fmt::Debug, I: Iterator<Item=U>> {
	/// # Iterator.
	iter: I,

	/// # ANSI Match State.
	///
	/// This keeps track of whether and where we are inside an ANSI sequence.
	state: NoAnsiState<U>,

	/// # Byte Position.
	///
	/// The current position inside the original source.
	pos: usize,
}

impl<U: Copy + fmt::Debug, I: Iterator<Item=U>> NoAnsi<U, I> {
	/// # Bytes Traversed.
	///
	/// Return the number of bytes traversed by the iterator (so far),
	/// including any ANSI sequences that have been ignored.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::ansi::NoAnsi;
	///
	/// let raw = "\x1b[1mHello\x1b[0m";
	/// let mut iter = NoAnsi::<char, _>::new(raw.chars());
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
	pub const fn byte_pos(&self) -> usize { self.pos }
}

impl<I: Iterator<Item=char>> NoAnsi<char, I> {
	#[inline]
	/// # New.
	///
	/// Create a new instance from an existing `Iterator<Item=char>`.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::ansi::NoAnsi;
	///
	/// let fmt: &str = "\x1b[1mHello\x1b[0m \x1b[38;5;199mWorld!\x1b[m";
	/// assert_eq!(
	///     NoAnsi::<char, _>::new(fmt.chars()).collect::<String>(),
	///     "Hello World!",
	/// );
	/// ```
	pub const fn new(iter: I) -> Self {
		Self {
			iter,
			state: NoAnsiState::None,
			pos: 0,
		}
	}
}

impl<I: Iterator<Item=char>> Iterator for NoAnsi<char, I> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		// One character at a time.
		while let Some(mut next) = self.state.take().or_else(|| self.iter.next()) {
			// Increment the position counter before we get too far afield.
			self.pos += next.len_utf8();

			// What it means varies by state.
			match self.state {
				// We aren't in any particular state.
				NoAnsiState::None =>
					// We might be entering a sequence.
					if next == NoAnsiState::<char>::ESCAPE {
						match self.iter.next() {
							Some('[') => {
								self.state = NoAnsiState::Csi;
								self.pos += 1;
							},
							Some(']') => {
								self.state = NoAnsiState::Osc;
								self.pos += 1;
							},
							Some(next) => {
								self.state = NoAnsiState::NeverMind(next);
								return Some(NoAnsiState::<char>::ESCAPE);
							},
							None => return Some(NoAnsiState::<char>::ESCAPE),
						}
					}
					else { return Some(next); },

				// We're in a CSI sequence, ignoring until escaped.
				NoAnsiState::Csi => if matches!(next, '\x40'..='\x7E') {
					self.state = NoAnsiState::None;
				},

				// We're in an OSC sequence, ignoring until escaped.
				NoAnsiState::Osc => loop {
					match next {
						NoAnsiState::<char>::OE | NoAnsiState::<char>::BELL => {
							self.state = NoAnsiState::None;
							break;
						},
						NoAnsiState::<char>::ESCAPE => {
							next = self.iter.next()?;
							self.pos += next.len_utf8();
							if next == '\\' {
								self.state = NoAnsiState::None;
								break;
							}
						},
						_ => break,
					}
				},

				// Unreachable.
				NoAnsiState::NeverMind(_) => {},
			}
		}

		// We're done!
		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let (_, max) = self.iter.size_hint();
		(0, max)
	}
}

impl<I: Iterator<Item=u8>> NoAnsi<u8, I> {
	#[inline]
	/// # New.
	///
	/// Create a new instance from an existing `Iterator<Item=u8>`.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::ansi::NoAnsi;
	///
	/// let fmt: &[u8] = b"\x1b[1mHello\x1b[0m \x1b[38;5;199mWorld!\x1b[m";
	/// assert_eq!(
	///     NoAnsi::<u8, _>::new(fmt.iter().copied()).collect::<Vec<u8>>(),
	///     b"Hello World!",
	/// );
	/// ```
	pub const fn new(iter: I) -> Self {
		Self {
			iter,
			state: NoAnsiState::None,
			pos: 0,
		}
	}
}

impl<I: Iterator<Item=u8>> Iterator for NoAnsi<u8, I> {
	type Item = u8;

	fn next(&mut self) -> Option<Self::Item> {
		// One byte at a time.
		while let Some(mut next) = self.state.take().or_else(|| self.iter.next()) {
			// Increment the position counter before we get too far afield.
			self.pos += 1;

			// What it means varies by state.
			match self.state {
				// We aren't in any particular state.
				NoAnsiState::None =>
					// We might be entering a sequence.
					if next == NoAnsiState::<u8>::ESCAPE {
						match self.iter.next() {
							Some(b'[') => {
								self.state = NoAnsiState::Csi;
								self.pos += 1;
							},
							Some(b']') => {
								self.state = NoAnsiState::Osc;
								self.pos += 1;
							},
							Some(next) => {
								self.state = NoAnsiState::NeverMind(next);
								return Some(NoAnsiState::<u8>::ESCAPE);
							},
							None => return Some(NoAnsiState::<u8>::ESCAPE),
						}
					}
					else { return Some(next); },

				// We're in a CSI sequence, ignoring until escaped.
				NoAnsiState::Csi => if matches!(next, b'\x40'..=b'\x7E') {
					self.state = NoAnsiState::None;
				},

				// We're in an OSC sequence, ignoring until escaped.
				NoAnsiState::Osc => loop {
					match next {
						NoAnsiState::<u8>::BELL => {
							self.state = NoAnsiState::None;
							break;
						},
						NoAnsiState::<u8>::OE_1 => {
							next = self.iter.next()?;
							self.pos += 1;
							if next == NoAnsiState::<u8>::OE_2 {
								self.state = NoAnsiState::None;
								break;
							}
						},
						NoAnsiState::<u8>::ESCAPE => {
							next = self.iter.next()?;
							self.pos += 1;
							if next == b'\\' {
								self.state = NoAnsiState::None;
								break;
							}
						},
						_ => break,
					}
				},

				// Unreachable.
				NoAnsiState::NeverMind(_) => {},
			}
		}

		// We're done!
		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let (_, max) = self.iter.size_hint();
		(0, max)
	}
}



#[derive(Debug, Clone, Copy)]
/// # Parsing State.
///
/// This short list is all we need to guide our output.
pub(crate) enum NoAnsiState<U: Copy + fmt::Debug> {
	/// # No particular state.
	None,

	/// # Inside a CSI sequence.
	Csi,

	/// # Inside an OSC sequence.
	Osc,

	/// # Lookahead buffer for false escapes.
	NeverMind(U),
}

impl<U: Copy + fmt::Debug> NoAnsiState<U> {
	/// # Take Value.
	///
	/// If `Self::NeverMind`, reset `self` to `Self::None` and return the
	/// buffered value.
	pub(crate) const fn take(&mut self) -> Option<U> {
		if let Self::NeverMind(c) = *self {
			*self = Self::None;
			Some(c)
		}
		else { None }
	}
}

impl NoAnsiState<char> {
	/// # Escape Character.
	pub(crate) const ESCAPE: char = '\x1b';

	/// # Bell Character.
	pub(crate) const BELL: char = '\x07';

	/// # OE Character.
	pub(crate) const OE: char = '\u{0153}';
}

impl NoAnsiState<u8> {
	/// # Escape Character.
	pub(crate) const ESCAPE: u8 = b'\x1b';

	/// # Bell Character.
	pub(crate) const BELL: u8 = b'\x07';

	/// # OE Character (part one).
	pub(crate) const OE_1: u8 = 197;

	/// # OE Character (part two).
	pub(crate) const OE_2: u8 = 147;
}



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
			let mut chars = chars.peekable();
			while let Some((k, c)) = chars.next() {
				match c {
					NoAnsiState::<char>::OE | NoAnsiState::<char>::BELL => {
						end = Some(k + c.len_utf8());
						break;
					},
					NoAnsiState::<char>::ESCAPE => if let Some((k, '\\')) = chars.peek() {
						end = Some(k + c.len_utf8());
						break;
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

	#[test]
	fn t_oe() {
		// Just check we split it right.
		let a = [
			NoAnsiState::<u8>::OE_1,
			NoAnsiState::<u8>::OE_2,
		];
		let b = std::str::from_utf8(a.as_slice()).expect("Invalid UTF-8");
		assert_eq!(
			b,
			NoAnsiState::<char>::OE.to_string(),
		);
	}

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
				NoAnsi::<char, _>::new(raw.chars()).collect::<String>(),
				expected,
			);

			assert_eq!(
				NoAnsi::<u8, _>::new(raw.bytes()).collect::<Vec<u8>>(),
				expected.as_bytes(),
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
			let stripped: String = NoAnsi::<char, _>::new(i.chars()).collect();
			assert_eq!(stripped, "One  Three", "Chars: {:?}", i.as_bytes());

			let stripped: Vec<u8> = NoAnsi::<u8, _>::new(i.bytes()).collect();
			assert_eq!(stripped, b"One  Three", "Bytes: {:?}", i.as_bytes());

			#[cfg(feature = "fitted")]
			assert_eq!(
				OnlyAnsi::new(i).collect::<String>(),
				i.strip_prefix("One ").and_then(|j| j.strip_suffix(" Three")).unwrap(),
				"Raw: {i:?}",
			);
		}

		// Additionally test byte stripping when the oe is split. This isn't
		// valid UTF-8, but the byte iterator doesn't necessarily require that.
		for i in [
			[b'A', NoAnsiState::<u8>::ESCAPE, b']', b'B', NoAnsiState::<u8>::OE_1, NoAnsiState::<u8>::ESCAPE, b'\\', b'C'].as_slice(),
			[b'A', NoAnsiState::<u8>::ESCAPE, b']', b'B', NoAnsiState::<u8>::OE_1, NoAnsiState::<u8>::ESCAPE, NoAnsiState::<u8>::ESCAPE, b'\\', b'C'].as_slice(),
			[b'A', NoAnsiState::<u8>::ESCAPE, b']', b'B', NoAnsiState::<u8>::OE_1, NoAnsiState::<u8>::BELL, b'C'].as_slice(),
			[b'A', NoAnsiState::<u8>::ESCAPE, b']', b'B', NoAnsiState::<u8>::OE_1, NoAnsiState::<u8>::OE_1, NoAnsiState::<u8>::OE_2, b'C'].as_slice(),
		] {
			let stripped: Vec<u8> = NoAnsi::<u8, _>::new(i.iter().copied()).collect();
			assert_eq!(stripped, b"AC", "Bytes: {i:?}");
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
			let mut iter = NoAnsi::<char, _>::new(s.chars());
			while iter.next().is_some() { }
			assert_eq!(s.len(), iter.byte_pos());

			// And byte bytes.
			let mut iter = NoAnsi::<u8, _>::new(s.bytes());
			while iter.next().is_some() { }
			assert_eq!(s.len(), iter.byte_pos());
		}
	}
}
