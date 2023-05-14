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
};



/// # Buffer x2.
pub const BUFFER2: usize = 4;

/// # Buffer x3.
pub const BUFFER3: usize = 6;

/// # Buffer x4.
pub const BUFFER4: usize = 8;

/// # Buffer x5.
pub const BUFFER5: usize = 10;

/// # Buffer x6.
pub const BUFFER6: usize = 12;

/// # Buffer x7.
pub const BUFFER7: usize = 14;

/// # Buffer x8.
pub const BUFFER8: usize = 16;

/// # Buffer x9.
pub const BUFFER9: usize = 18;

/// # Buffer x10.
pub const BUFFER10: usize = 20;



/// # Maximum size.
const BUFFER_MAX_LEN: usize = 4_294_967_295;
const BUFFER_OVERFLOW: &str = "Buffer lengths may not exceed u32::MAX.";
const BUFFER_UNDERFLOW: &str = "Adjustment is larger than the part.";



#[derive(Debug, Clone)]
/// # Message Buffer.
pub struct MsgBuffer<const N: usize> {
	buf: Vec<u8>,
	toc: [u32; N]
}

impl<const N: usize> Default for MsgBuffer<N> {
	#[inline]
	fn default() -> Self {
		Self {
			buf: Vec::new(),
			toc: [0_u32; N],
		}
	}
}

impl<const N: usize> AsRef<[u8]> for MsgBuffer<N> {
	#[inline]
	fn as_ref(&self) -> &[u8] { &self.buf }
}

impl<const N: usize> Deref for MsgBuffer<N> {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.buf }
}

impl<const N: usize> fmt::Display for MsgBuffer<N> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(std::str::from_utf8(self).map_err(|_| fmt::Error::default())?)
	}
}

impl<const N: usize> Eq for MsgBuffer<N> {}

impl<const N: usize> Hash for MsgBuffer<N> {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) { self.buf.hash(state); }
}

impl<const N: usize> PartialEq for MsgBuffer<N> {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.buf == other.buf }
}

impl<const N: usize> PartialEq<[u8]> for MsgBuffer<N> {
	#[inline]
	fn eq(&self, other: &[u8]) -> bool { self.buf == other }
}

impl<const N: usize> PartialEq<Vec<u8>> for MsgBuffer<N> {
	#[inline]
	fn eq(&self, other: &Vec<u8>) -> bool { self.buf == *other }
}

/// ## Instantiation.
impl<const N: usize> MsgBuffer<N> {
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
	/// ## Panics
	///
	/// The total buffer length may not exceed `u32::MAX` It will panic if
	/// trying to go larger.
	///
	/// This will also panic if trying to create a buffer on a 16-bit platform
	/// as that architecture is not supported.
	///
	/// ## Safety
	///
	/// No validation is performed; the data must make sense or undefined
	/// things will happen down the road.
	///
	/// The table of contents must be properly aligned and ordered.
	pub fn from_raw_parts(buf: Vec<u8>, toc: [u32; N]) -> Self {
		#[cfg(target_pointer_width = "16")]
		panic!("Message buffers do not support 16-bit platforms.");

		assert!(buf.len() <= BUFFER_MAX_LEN, "{}", BUFFER_OVERFLOW);
		Self { buf, toc }
	}
}

/// ## Casting.
impl<const N: usize> MsgBuffer<N> {
	#[must_use]
	#[inline]
	/// # As Bytes.
	///
	/// Return as a byte slice.
	pub fn as_bytes(&self) -> &[u8] { &self.buf }

	#[must_use]
	#[inline]
	/// # As Str.
	///
	/// This method returns the underlying vector as a string slice.
	///
	/// ## Panics
	///
	/// This method will panic if the contents are not valid UTF-8.
	pub fn as_str(&self) -> &str { std::str::from_utf8(&self.buf).unwrap() }

	#[allow(unsafe_code)]
	#[must_use]
	#[inline]
	/// # As Str (Unchecked).
	///
	/// This method returns the underlying vector as a string slice.
	///
	/// ## Safety
	///
	/// The string must be valid UTF-8 or undefined things will happen.
	pub unsafe fn as_str_unchecked(&self) -> &str {
		std::str::from_utf8_unchecked(&self.buf)
	}

	#[must_use]
	#[inline]
	/// # Into String.
	///
	/// Consume and return the underlying vector as a `String`.
	///
	/// ## Panics
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
impl<const N: usize> MsgBuffer<N> {
	#[must_use]
	#[inline]
	#[allow(clippy::cast_possible_truncation)] // We've previously asserted it fits.
	/// # Total Buffer Length.
	///
	/// Return the length of the entire buffer (rather than a single part).
	pub fn total_len(&self) -> u32 { self.buf.len() as u32 }

	/// # Clear Buffer.
	///
	/// This will empty the buffer and reset the TOC.
	pub fn clear(&mut self) {
		self.buf.clear();
		self.zero_parts();
	}
}

#[allow(clippy::len_without_is_empty)] // We don't need it.
/// ## Individual Parts.
impl<const N: usize> MsgBuffer<N> {
	#[must_use]
	/// # Part Length as `u32`.
	pub const fn len(&self, idx: usize) -> u32 { self.end(idx) - self.start(idx) }

	#[must_use]
	/// # Part Start.
	pub const fn start(&self, idx: usize) -> u32 { self.toc[idx << 1] }

	#[must_use]
	/// # Part End.
	pub const fn end(&self, idx: usize) -> u32 { self.toc[(idx << 1) + 1] }

	#[must_use]
	/// # Part Range.
	pub const fn range(&self, idx: usize) -> Range<usize> {
		self.start(idx) as usize..self.end(idx) as usize
	}

	#[must_use]
	/// # Get Part.
	pub fn get(&self, idx: usize) -> &[u8] { &self.buf[self.range(idx)] }

	#[must_use]
	/// # Get Mutable Part.
	pub fn get_mut(&mut self, idx: usize) -> &mut [u8] {
		let rng = self.range(idx);
		&mut self.buf[rng]
	}

	#[allow(clippy::cast_possible_truncation)] // We've previously asserted it fits.
	/// # Extend Part.
	///
	/// ## Panics
	///
	/// The total buffer length may not exceed `u32::MAX` It will panic if
	/// trying to go larger.
	pub fn extend(&mut self, idx: usize, buf: &[u8]) {
		let len = u32::try_from(buf.len()).unwrap();
		if len != 0 {
			let end = self.end(idx);

			// End of buffer trick.
			if end == self.total_len() {
				assert!(buf.len() + self.buf.len() <= BUFFER_MAX_LEN, "{}", BUFFER_OVERFLOW);

				self.buf.extend_from_slice(buf);
				self.raise_parts_from(idx, len);
			}
			else {
				self.resize_grow(idx, len);
				let end = end as usize;
				self.buf[end..end + buf.len()].copy_from_slice(buf);
			}
		}
	}

	#[allow(clippy::comparison_chain)] // We're only matching 2/3.
	/// # Replace Part.
	///
	/// ## Panics
	///
	/// The total buffer length may not exceed `u32::MAX` It will panic if
	/// trying to go larger.
	pub fn replace(&mut self, idx: usize, buf: &[u8]) {
		// Get the lengths.
		let new_len = u32::try_from(buf.len()).unwrap();
		let old_len = self.len(idx);

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
			let from = self.start(idx) as usize;
			self.buf[from..from + buf.len()].copy_from_slice(buf);
		}
	}

	/// # Truncate Part.
	pub fn truncate(&mut self, idx: usize, len: u32) {
		let old_len = self.len(idx);
		if old_len > len {
			self.resize_shrink(idx, old_len - len);
		}
	}
}

/// ## Internal.
impl<const N: usize> MsgBuffer<N> {
	/// # Grow.
	///
	/// ## Panics
	///
	/// The total buffer length may not exceed `u32::MAX` It will panic if
	/// trying to go larger.
	///
	/// ## Safety
	///
	/// This method leaves data uninitialized! It is the responsibility
	/// of the callers to ensure data is written before any access is
	/// attempted. It is private, and the callers do do that, so it
	/// should be safe in practice. ;)
	fn resize_grow(&mut self, idx: usize, adj: u32) {
		// Quick short-circuit.
		if adj == 0 { return; }

		assert!(self.buf.len() + adj as usize <= BUFFER_MAX_LEN, "{}", BUFFER_OVERFLOW);

		let end: u32 = self.end(idx);
		let len: u32 = self.total_len();
		self.buf.resize((len + adj) as usize, b' ');

		// We need to shift things over.
		if end < len {
			self.buf.copy_within(end as usize..len as usize, (end + adj) as usize);
		}

		self.raise_parts_from(idx, adj);
	}

	/// # Shrink.
	fn resize_shrink(&mut self, idx: usize, adj: u32) {
		assert!(self.len(idx) >= adj, "{}", BUFFER_UNDERFLOW);

		let end = self.end(idx);

		// End-of-buffer shortcut.
		if end == self.total_len() {
			self.buf.truncate((end - adj) as usize);
		}
		// Middle incision.
		else {
			self.buf.drain((end - adj) as usize..end as usize);
		}

		self.lower_parts_from(idx, adj);
	}

	/// # Increment parts from.
	fn raise_parts_from(&mut self, idx: usize, adj: u32) {
		self.toc.iter_mut().skip((idx << 1) + 1).for_each(|x| *x += adj);
	}

	/// # Decrease parts from.
	fn lower_parts_from(&mut self, idx: usize, adj: u32) {
		assert!(self.len(idx) >= adj, "{}", BUFFER_UNDERFLOW);
		self.toc.iter_mut().skip((idx << 1) + 1).for_each(|x| *x -= adj);
	}

	/// # Zero out parts.
	fn zero_parts(&mut self) {
		self.toc.copy_from_slice(&[0_u32; N]);
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extend() {
		let mut buf = MsgBuffer::<BUFFER3>::from_raw_parts(
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
		let mut buf = MsgBuffer::<BUFFER3>::from_raw_parts(
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
		let mut buf = MsgBuffer::<BUFFER3>::from_raw_parts(
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
