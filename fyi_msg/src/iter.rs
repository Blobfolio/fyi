/*!
# FYI Msg: Iterators.
*/

use std::fmt;



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

	/// # Ansi Match State.
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
	/// use fyi_msg::iter::NoAnsi;
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
	/// Create a new instance from an existing `Iterator<Item=Char>`.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::iter::NoAnsi;
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
		loop {
			// If we need to backtrack, empty and return the buffer.
			if let NoAnsiState::NeverMind(next) = self.state {
				self.state = NoAnsiState::None;
				self.pos += next.len_utf8();
				return Some(next);
			}

			// Check the next character!
			let mut next = self.iter.next()?;
			self.pos += next.len_utf8();
			match self.state {
				// We aren't in any particular state.
				NoAnsiState::None =>
					// We might be entering a sequence.
					if next == NoAnsiState::<char>::ESCAPE {
						match self.iter.next() {
							Some('[') => {
								self.state = NoAnsiState::Csi;
								self.pos += 1;
							}
							Some(']') => {
								self.state = NoAnsiState::Osc;
								self.pos += 1;
							}
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
	/// use fyi_msg::iter::NoAnsi;
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
		loop {
			// If we need to backtrack, empty and return the buffer.
			if let NoAnsiState::NeverMind(next) = self.state {
				self.state = NoAnsiState::None;
				self.pos += 1;
				return Some(next);
			}

			// Check the next character!
			let mut next = self.iter.next()?;
			self.pos += 1;
			match self.state {
				// We aren't in any particular state.
				NoAnsiState::None =>
					// We might be  entering a sequence.
					if next == NoAnsiState::<u8>::ESCAPE {
						match self.iter.next() {
							Some(b'[') => {
								self.state = NoAnsiState::Csi;
								self.pos += 1;
							}
							Some(b']') => {
								self.state = NoAnsiState::Osc;
								self.pos += 1;
							}
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
enum NoAnsiState<U: Copy + fmt::Debug> {
	/// # No particular state.
	None,

	/// # Inside a CSI sequence.
	Csi,

	/// # Inside an OSC sequence.
	Osc,

	/// # Lookahead buffer for false escapes.
	NeverMind(U),
}

impl NoAnsiState<char> {
	/// # Escape Character.
	const ESCAPE: char = '\x1b';

	/// # Bell Character.
	const BELL: char = '\x07';

	/// # OE Character.
	const OE: char = '\u{0153}';
}

impl NoAnsiState<u8> {
	/// # Escape Character.
	const ESCAPE: u8 = b'\x1b';

	/// # Bell Character.
	const BELL: u8 = b'\x07';

	/// # OE Character (part one).
	const OE_1: u8 = 197;

	/// # OE Character (part two).
	const OE_2: u8 = 147;
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
	fn t_noansi_osc() {
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
