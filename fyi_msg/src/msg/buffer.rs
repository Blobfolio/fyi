/*!
# FYI Msg: Buffer

**Note:** This is not intended for external use and is subject to change.
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



/// # Default Message Buffer.
#[cfg(feature = "timestamps")] pub type DefaultMsgBuffer = [u32; 12];
#[cfg(not(feature = "timestamps"))] pub type DefaultMsgBuffer = [u32; 10];

/// # Table of Contents.
pub trait ToC: Copy + Sized + Default {
	/// # Length of part.
	fn part_len(&self, idx: usize) -> u32 {
		self.part_end(idx) - self.part_start(idx)
	}

	/// # Part is empty.
	fn part_is_empty(&self, idx: usize) -> bool {
		self.part_start(idx) == self.part_end(idx)
	}

	/// # Part range.
	fn part_rng(&self, idx: usize) -> Range<usize> {
		self.part_start(idx) as usize..self.part_end(idx) as usize
	}

	/// # Part start.
	fn part_start(&self, idx: usize) -> u32;

	/// # Part end.
	fn part_end(&self, idx: usize) -> u32;

	/// # Zero parts.
	fn zero_parts(&mut self);

	/// # Increment parts from.
	fn raise_parts_from(&mut self, idx: usize, adj: u32);

	/// # Decrease parts from.
	fn lower_parts_from(&mut self, idx: usize, adj: u32);
}

macro_rules! impl_toc {
	($Type:ident, $len:literal) => {
		/// # Sized Message Buffer.
		pub type $Type = [u32; $len];

		impl ToC for [u32; $len] {
			/// # Part start.
			fn part_start(&self, idx: usize) -> u32 { self[idx << 1] }

			/// # Part end.
			fn part_end(&self, idx: usize) -> u32 { self[(idx << 1) + 1] }

			/// # Zero parts.
			fn zero_parts(&mut self) {
				self.copy_from_slice(&[0_u32; $len]);
			}

			/// # Increment parts from.
			fn raise_parts_from(&mut self, idx: usize, adj: u32) {
				self.iter_mut().skip((idx << 1) + 1).for_each(|x| *x += adj);
			}

			/// # Decrease parts from.
			fn lower_parts_from(&mut self, idx: usize, adj: u32) {
				assert!(self.part_len(idx) >= adj);
				self.iter_mut().skip((idx << 1) + 1).for_each(|x| *x -= adj);
			}
		}
	};
}

impl_toc!(MsgBuffer2, 4);
impl_toc!(MsgBuffer3, 6);
impl_toc!(MsgBuffer4, 8);
impl_toc!(MsgBuffer5, 10);
impl_toc!(MsgBuffer6, 12);
impl_toc!(MsgBuffer7, 14);
impl_toc!(MsgBuffer8, 16);
impl_toc!(MsgBuffer9, 18);
impl_toc!(MsgBuffer10, 20);




#[derive(Debug, Default, Clone)]
/// # Message Buffer.
pub struct MsgBuffer<T: ToC = DefaultMsgBuffer> {
	buf: Vec<u8>,
	toc: T
}

impl<T: ToC> AsRef<[u8]> for MsgBuffer<T> {
	#[inline]
	fn as_ref(&self) -> &[u8] { &self.buf }
}

impl<T: ToC> Deref for MsgBuffer<T> {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.buf }
}

impl<T: ToC> fmt::Display for MsgBuffer<T> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(std::str::from_utf8(self).map_err(|_| fmt::Error::default())?)
	}
}

impl<T: ToC> Eq for MsgBuffer<T> {}

impl<T: ToC> Hash for MsgBuffer<T> {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) { self.buf.hash(state); }
}

impl<T: ToC> PartialEq for MsgBuffer<T> {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.buf == other.buf }
}

impl<T: ToC> PartialEq<[u8]> for MsgBuffer<T> {
	#[inline]
	fn eq(&self, other: &[u8]) -> bool { self.buf == other }
}

impl<T: ToC> PartialEq<Vec<u8>> for MsgBuffer<T> {
	#[inline]
	fn eq(&self, other: &Vec<u8>) -> bool { self.buf == *other }
}

/// ## Instantiation.
impl<T: ToC> MsgBuffer<T> {
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
	pub fn from_raw_parts(buf: Vec<u8>, toc: T) -> Self {
		Self { buf, toc }
	}
}

/// ## Casting.
impl<T: ToC> MsgBuffer<T> {
	#[must_use]
	#[inline]
	/// # As Bytes.
	///
	/// Return as a byte slice.
	pub fn as_bytes(&self) -> &[u8] { &self.buf }

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
	pub fn as_str(&self) -> &str { std::str::from_utf8(&self.buf).unwrap() }

	#[must_use]
	/// # As Str (Unchecked).
	///
	/// This method returns the underlying vector as a string slice.
	///
	/// ## Safety
	///
	/// The string must be valid UTF-8 or undefined things will happen.
	#[inline]
	pub unsafe fn as_str_unchecked(&self) -> &str {
		std::str::from_utf8_unchecked(&self.buf)
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
	pub fn into_string(self) -> String { String::from_utf8(self.buf).unwrap() }

	#[allow(clippy::missing_const_for_fn)] // Doesn't work.
	#[must_use]
	#[inline]
	/// # Into Vec.
	///
	/// Consume and return the underlying vector.
	pub fn into_vec(self) -> Vec<u8> { self.buf }
}

/// ## Whole Buffer Play.
impl<T: ToC> MsgBuffer<T> {
	#[must_use]
	#[inline]
	/// # Total Buffer Length.
	///
	/// Return the length of the entire buffer (rather than a single part).
	pub fn total_len(&self) -> usize { self.buf.len() }

	/// # Clear Buffer.
	///
	/// This will empty the buffer and reset the TOC.
	pub fn clear(&mut self) {
		self.buf.clear();
		self.toc.zero_parts();
	}
}

/// ## Individual Parts.
impl<T: ToC> MsgBuffer<T> {
	#[must_use]
	/// # Part Length.
	pub fn len(&self, idx: usize) -> usize { self.toc.part_len(idx) as usize }

	#[must_use]
	/// # Part Length as `u32`.
	pub fn len_u32(&self, idx: usize) -> u32 { self.toc.part_len(idx) }

	#[must_use]
	/// # Part Start.
	pub fn start(&self, idx: usize) -> usize {
		self.toc.part_start(idx) as usize
	}

	#[must_use]
	/// # Part End.
	pub fn end(&self, idx: usize) -> usize {
		self.toc.part_end(idx) as usize
	}

	#[must_use]
	/// # Part Range.
	pub fn range(&self, idx: usize) -> Range<usize> { self.toc.part_rng(idx) }

	#[must_use]
	/// # Get Part.
	pub fn get(&self, idx: usize) -> &[u8] { &self.buf[self.toc.part_rng(idx)] }

	#[must_use]
	/// # Get Mutable Part.
	pub fn get_mut(&mut self, idx: usize) -> &mut [u8] {
		let rng = self.toc.part_rng(idx);
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
				self.toc.raise_parts_from(idx, len as u32);
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

	#[allow(clippy::comparison_chain)] // We're only matching 2/3.
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

/// # Misc.
impl<T: ToC> MsgBuffer<T> {
	/// # Grow.
	///
	/// ## Safety
	///
	/// This method leaves data uninitialized! It is the responsibility
	/// of the callers to ensure data is written before any access is
	/// attempted. It is private, and the callers do do that, so it
	/// should be safe in practice. ;)
	fn resize_grow(&mut self, idx: usize, adj: usize) {
		// Quick short-circuit.
		if adj == 0 { return; }

		let end: usize = self.end(idx);
		let len: usize = self.buf.len();
		self.buf.reserve(adj);

		// We need to shift things over.
		if end < len {
			unsafe {
				ptr::copy(
					self.buf.as_ptr().add(end),
					self.buf.as_mut_ptr().add(end + adj),
					len - end
				);

				self.buf.set_len(len + adj);
			}
		}
		else {
			unsafe { self.buf.set_len(len + adj); }
		}

		self.toc.raise_parts_from(idx, adj as u32);
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

		self.toc.lower_parts_from(idx, adj as u32);
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extend() {
		let mut buf = MsgBuffer::<[u32; 6]>::from_raw_parts(
			vec![0, 0, 1, 1, 0, 0],
			[
				2, 4,
				4, 6,
				6, 6,
			]
		);

		buf.extend(0, &[3, 3, 3]);

		assert_eq!(buf, vec![0, 0, 1, 1, 3, 3, 3, 0, 0]);
		assert_eq!(buf.start(0)..buf.end(0), 2..7);
		assert_eq!(buf.len(0), 5);
		assert_eq!(buf.start(1)..buf.end(1), 7..9);
		assert_eq!(buf.len(1), 2);
		assert_eq!(buf.len(2), 0);

		buf.extend(2, &[4, 4]);
		assert_eq!(buf, vec![0, 0, 1, 1, 3, 3, 3, 0, 0, 4, 4]);
		assert_eq!(buf.len(2), 2);
		assert_eq!(buf.range(2), 9..11);
	}

	#[test]
	fn replace() {
		let mut buf = MsgBuffer::<[u32; 6]>::from_raw_parts(
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
		let mut buf = MsgBuffer::<[u32; 6]>::from_raw_parts(
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
