/*!
# FYI Msg: Buffer
*/

use std::{
	fmt,
	hash::{
		Hash,
		Hasher,
	},
	ops::{
		Deref,
		Range,
	},
	ptr,
};

macro_rules! define_buffer {
	($name:ident, $size:literal, $num:expr) => {
		#[derive(Debug, Clone, Default)]
		#[doc = "Message Buffer with `"]
		#[doc = $num]
		#[doc = "` parts."]
		pub struct $name {
			buf: Vec<u8>,
			toc: [usize; $size],
		}

		impl Deref for $name {
			type Target = [u8];
			#[inline]
			fn deref(&self) -> &Self::Target { &self.buf }
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				f.write_str(std::str::from_utf8(self).map_err(|_| fmt::Error::default())?)
			}
		}

		impl Eq for $name {}

		impl Hash for $name {
			#[inline]
			fn hash<H: Hasher>(&self, state: &mut H) { self.buf.hash(state); }
		}

		impl PartialEq for $name {
			#[inline]
			fn eq(&self, other: &Self) -> bool { self.buf == other.buf }
		}

		impl PartialEq<[u8]> for $name {
			#[inline]
			fn eq(&self, other: &[u8]) -> bool { self.buf == other }
		}

		impl PartialEq<Vec<u8>> for $name {
			#[inline]
			fn eq(&self, other: &Vec<u8>) -> bool { self.buf == *other }
		}

		/// ## Instantiation.
		impl $name {
			#[must_use]
			#[inline]
			/// # Instantiate From Raw Parts.
			///
			/// This directly sets the struct's fields, exactly like
			/// constructing it manually would, but since the fields are
			/// private, this is how that's done.
			///
			/// The `toc` part indexes should alternate between an inclusive
			/// starting index, and exclusive ending index. For example, if the
			/// first part is 2 bytes and the second part is 5 bytes, the array
			/// would begin like `[0, 2, 2, 7, …]`.
			///
			/// Parts do not need to be connected to each other, but must be in
			/// order sequentially. For example, `[0, 2, 10, 15, …]` is fine,
			/// but `[10, 15, 0, 2, …]` will panic.
			///
			/// ## Safety
			///
			/// No validation is performed; the data must make sense or undefined
			/// things will happen down the road.
			///
			/// The table of contents must be properly aligned and ordered.
			pub fn from_raw_parts(buf: Vec<u8>, toc: [usize; $size]) -> Self {
				Self { buf, toc }
			}
		}

		/// ## Casting.
		impl $name {
			#[must_use]
			#[inline]
			/// # As Bytes.
			///
			/// Return as a byte slice.
			pub fn as_bytes(&self) -> &[u8] { self }

			#[must_use]
			#[inline]
			/// # As Pointer.
			///
			/// This method returns a read-only pointer to the underlying buffer.
			pub fn as_ptr(&self) -> *const u8 { self.buf.as_ptr() }

			#[must_use]
			#[inline]
			/// # As Mut Pointer.
			///
			/// This method returns a mutable pointer to the underlying buffer.
			///
			/// ## Safety
			///
			/// Any changes written to the pointer must not affect the table of
			/// contents or undefined things will happen!
			pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 { self.buf.as_mut_ptr() }

			#[must_use]
			/// # As Str.
			///
			/// This method returns the underlying vector as a string slice.
			///
			/// ## Panic
			///
			/// This method will panic if the contents are not valid UTF-8.
			#[inline]
			pub fn as_str(&self) -> &str {
				std::str::from_utf8(self).unwrap()
			}

			#[must_use]
			#[inline]
			/// # Into String.
			///
			/// Consume and return the underlying vector as a `String`.
			///
			/// ## Panic
			///
			/// This method will panic if the contents are not valid UTF-8.
			pub fn into_string(self) -> String {
				String::from_utf8(self.buf).unwrap()
			}

			#[allow(clippy::missing_const_for_fn)] // This doesn't work.
			#[must_use]
			#[inline]
			/// # Into Vec.
			///
			/// Consume and return the underlying vector.
			pub fn into_vec(self) -> Vec<u8> { self.buf }
		}

		/// ## Whole Buffer Play.
		impl $name {
			#[must_use]
			#[inline]
			/// # Total Buffer Length.
			///
			/// Return the length of the entire buffer (rather than a single part).
			pub fn total_len(&self) -> usize { self.buf.len() }

			#[must_use]
			#[inline]
			/// # TOC Size.
			pub const fn size() -> usize { $size }

			/// # Clear Buffer.
			///
			/// This will empty the buffer and reset the TOC.
			pub fn clear(&mut self) {
				self.buf.clear();
				self.toc.iter_mut().for_each(|x| *x = 0);
			}
		}

		/// ## Individual Parts.
		impl $name {
			#[must_use]
			/// # Part Length.
			pub const fn len(&self, idx: usize) -> usize {
				self.end(idx) - self.start(idx)
			}

			#[cfg(feature = "fitted")]
			#[must_use]
			/// # Fit Width.
			///
			/// This returns the length of the slice that fits a given width.
			///
			/// ## Safety
			///
			/// The string must be valid UTF-8 or undefined things will happen.
			pub unsafe fn fitted(&self, idx: usize, width: usize) -> usize {
				use unicode_width::UnicodeWidthChar;

				let len: usize = self.len(idx);
				if len > width {
					let mut total_len: usize = 0;
					let mut total_width: usize = 0;

					// For our purposes, basic ANSI markup (of the kind we use)
					// is considered 0-width;
					let mut in_ansi: bool = false;

					// Convert to a string slice so we can iterate over
					// individual chars.
					for c in std::str::from_utf8_unchecked(self.get(idx)).chars() {
						// Find the "length" of this char.
						let ch_len: usize = c.len_utf8();
						total_len += ch_len;

						// If we're in the middle of an ANSI sequence nothing
						// counts, but we need to watch for the end marker so
						// we can start paying attention again.
						if in_ansi {
							// We're only interested in A/K/m signals.
							if c == 'A' || c == 'K' || c == 'm' { in_ansi = false; }
							continue;
						}
						// Are we entering an ANSI sequence?
						else if c == '\x1b' {
							in_ansi = true;
							continue;
						}

						// The width matters!
						let ch_width: usize = UnicodeWidthChar::width(c).unwrap_or_default();
						total_width += ch_width;

						// Widths can creep up unevenly. If we've gone over, we
						// need to revert a step and exit.
						if total_width > width {
							return total_len - ch_len;
						}
					}
				}

				len
			}

			#[cfg(feature = "fitted")]
			#[must_use]
			/// # Part Width.
			///
			/// Return the "display width" of the part.
			///
			/// ## Safety
			///
			/// The string must be valid UTF-8 or undefined things will happen.
			pub unsafe fn width(&self, idx: usize) -> usize {
				use unicode_width::UnicodeWidthChar;

				if self.len(idx) == 0 { 0 }
				else {
					let mut width: usize = 0;
					let mut in_ansi: bool = false;
					for c in std::str::from_utf8_unchecked(self.get(idx)).chars() {
						if in_ansi {
							// We're only interested in A/K/m signals.
							if c == 'A' || c == 'K' || c == 'm' { in_ansi = false; }
						}
						// Are we entering an ANSI sequence?
						else if c == '\x1b' { in_ansi = true; }
						else {
							width += UnicodeWidthChar::width(c).unwrap_or_default();
						}
					}

					width
				}
			}

			#[must_use]
			/// # Part Start.
			pub const fn start(&self, idx: usize) -> usize {
				self.toc[idx << 1]
			}

			#[must_use]
			/// # Part End.
			pub const fn end(&self, idx: usize) -> usize {
				self.toc[(idx << 1) + 1]
			}

			#[must_use]
			/// # Part Range.
			pub const fn range(&self, idx: usize) -> Range<usize> {
				self.start(idx)..self.end(idx)
			}

			#[must_use]
			/// # Get Part.
			pub fn get(&self, idx: usize) -> &[u8] {
				&self.buf[self.range(idx)]
			}

			#[must_use]
			/// # Get Mutable Part.
			pub fn get_mut(&mut self, idx: usize) -> &mut [u8] {
				let rng = self.range(idx);
				&mut self.buf[rng]
			}

			/// # Extend Part.
			pub fn extend(&mut self, idx: usize, buf: &[u8]) {
				let len = buf.len();
				if len != 0 {
					let end = self.end(idx);

					// End of buffer trick.
					if end == self.buf.len() {
						self.buf.extend_from_slice(buf);
						self.increase(idx, len);
					}
					else {
						self.resize_grow(idx, len);
						unsafe {
							std::ptr::copy_nonoverlapping(
								buf.as_ptr(),
								self.buf.as_mut_ptr().add(end),
								len
							);
						}
					}
				}
			}

			/// # Replace Part.
			pub fn replace(&mut self, idx: usize, buf: &[u8]) {
				// Get the lengths.
				let old_len = self.len(idx);
				let new_len = buf.len();

				// Expand it.
				if old_len < new_len {
					self.resize_grow(idx, new_len - old_len);
				}
				// Shrink it.
				else if new_len < old_len {
					self.resize_shrink(idx, old_len - new_len);
				}

				// Write it!
				if 0 != new_len {
					unsafe {
						std::ptr::copy_nonoverlapping(
							buf.as_ptr(),
							self.buf.as_mut_ptr().add(self.start(idx)),
							new_len
						);
					}
				}
			}

			/// # Truncate Part.
			pub fn truncate(&mut self, idx: usize, len: usize) {
				let old_len = self.len(idx);
				if old_len > len {
					self.resize_shrink(idx, old_len - len);
				}
			}
		}

		/// ## Misc.
		impl $name {
			/// # Grow.
			fn resize_grow(&mut self, idx: usize, adj: usize) {
				let end: usize = self.end(idx);
				let len: usize = self.buf.len();

				self.buf.resize(len + adj, 0);

				// We need to shift things over.
				if end < len {
					unsafe {
						ptr::copy(
							self.buf.as_ptr().add(end),
							self.buf.as_mut_ptr().add(end + adj),
							len - end
						);
					}
				}

				self.increase(idx, adj);
			}

			/// # Shrink.
			fn resize_shrink(&mut self, idx: usize, adj: usize) {
				assert!(self.len(idx) >= adj);
				let end = self.end(idx);

				// End-of-buffer shortcut.
				if end == self.buf.len() {
					self.buf.truncate(end - adj);
				}
				// Middle incision.
				else {
					self.buf.drain(end - adj..end);
				}

				self.decrease(idx, adj);
			}

			/// # Decrease Indexing From.
			///
			/// This decreases the length of a partition, nudging all
			/// subsequent partitions to the left.
			///
			/// ## Panics
			///
			/// This method will panic if the adjustent is larger than any of
			/// the affected parts.
			fn decrease(&mut self, idx: usize, adj: usize) {
				self.toc.iter_mut().skip((idx << 1) + 1).for_each(|x| *x -= adj);
			}

			/// # Increase Indexing From.
			///
			/// This increases the length of a partition, nudging all
			/// subsequent partitions to the right.
			fn increase(&mut self, idx: usize, adj: usize) {
				self.toc.iter_mut().skip((idx << 1) + 1).for_each(|x| *x += adj);
			}
		}
	};
}

define_buffer!(MsgBuffer2, 4, "2");
define_buffer!(MsgBuffer3, 6, "3");
define_buffer!(MsgBuffer4, 8, "4");
define_buffer!(MsgBuffer5, 10, "5");
define_buffer!(MsgBuffer6, 12, "6");
define_buffer!(MsgBuffer7, 14, "7");
define_buffer!(MsgBuffer8, 16, "8");
define_buffer!(MsgBuffer9, 18, "9");
define_buffer!(MsgBuffer10, 20, "10");



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn replace() {
		let mut buf = MsgBuffer3::from_raw_parts(
			vec![0, 0, 1, 1, 0, 0],
			[
				2, 4,
				4, 6,
				6, 6,
			]
		);

		// Bigger.
		buf.replace(0, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2, 0, 0]);
		assert_eq!(buf.start(0)..buf.end(0), 2..5);
		assert_eq!(buf.len(0), 3);
		assert_eq!(buf.start(1)..buf.end(1), 5..7);
		assert_eq!(buf.len(1), 2);

		// Same Size.
		buf.replace(0, &[3, 3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3, 3, 0, 0]);
		assert_eq!(buf.start(0)..buf.end(0), 2..5);
		assert_eq!(buf.start(1)..buf.end(1), 5..7);

		// Smaller.
		buf.replace(0, &[1]);
		assert_eq!(buf, vec![0, 0, 1, 0, 0]);
		assert_eq!(buf.start(0)..buf.end(0), 2..3);
		assert_eq!(buf.start(1)..buf.end(1), 3..5);

		// Empty.
		buf.replace(0, &[]);
		assert_eq!(buf, vec![0, 0, 0, 0]);
		assert_eq!(buf.start(0)..buf.end(0), 2..2);
		assert_eq!(buf.len(0), 0);
		assert_eq!(buf.start(1)..buf.end(1), 2..4);
		assert_eq!(buf.len(1), 2);

		// Bigger (End).
		buf.replace(1, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2]);
		assert_eq!(buf.start(0)..buf.end(0), 2..2);
		assert_eq!(buf.start(1)..buf.end(1), 2..5);

		// Smaller (End).
		buf.replace(1, &[3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3]);
		assert_eq!(buf.start(0)..buf.end(0), 2..2);
		assert_eq!(buf.start(1)..buf.end(1), 2..4);
	}

	#[test]
	fn truncate() {
		let mut buf = MsgBuffer3::from_raw_parts(
			vec![0, 0, 1, 1, 0, 0],
			[
				2, 4,
				4, 6,
				6, 6,
			]
		);

		// Empty.
		buf.truncate(0, 0);
		assert_eq!(buf, vec![0, 0, 0, 0]);
		assert_eq!(buf.start(0)..buf.end(0), 2..2);
		assert_eq!(buf.start(1)..buf.end(1), 2..4);
	}
}
