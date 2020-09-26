/*!
# FYI Msg: Buffer
*/

use std::{
	fmt,
	hash::{
		Hash,
		Hasher,
	},
	ops::Deref,
	ops::DerefMut,
	ptr,
};



#[derive(Debug, Clone, Default)]
/// # Message Buffer.
///
/// "Buffer" isn't the right word. This is more of a contiguous, partitioned
/// byte string.
///
/// The contiguity(?) allows for fast slice borrows (for e.g. printing), while
/// the partitioning makes it easy to update select portions of the buffer in-
/// place.
///
/// The partitioning may be arbitrary, and does not need to have full coverage
/// or be contiguous with itself. That said, all part boundaries must be
/// sequential, non-overlapping, and within range.
///
/// Up to `16` parts are supported, defined in pairs of inclusive start and
/// exclusive end indexes, like `[0, 5, 5, 10, ...]`. "Unused" ranges should
/// simply be set to the maximum used end value. If our previous 2-part example
/// were all there was, 28 `10`s would follow to fill out the table.
///
/// One final important note about partitions: the indexes are stored
/// internally as `u16` values, so none of their ranges may exceed [`u16::MAX`]
/// or they will overflow. (Unpartitioned space can, however, run all the way
/// through [`usize::MAX`].)
///
/// ## Safety
///
/// This struct is built for performance and largely requires security/sanity
/// be handled by the implementing library. As such, most of its methods are
/// marked "unsafe".
///
/// It is not designed for use outside these crates and is subject to change in
/// breaking ways.
pub struct MsgBuffer {
	buf: Vec<u8>,
	toc: [u16; 32],
}

impl Deref for MsgBuffer {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.buf }
}

impl DerefMut for MsgBuffer {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.buf }
}

impl fmt::Display for MsgBuffer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(std::str::from_utf8(self).map_err(|_| fmt::Error::default())?)
	}
}

impl Eq for MsgBuffer {}

impl Hash for MsgBuffer {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) { self.buf.hash(state); }
}

impl PartialEq for MsgBuffer {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.buf == other.buf }
}

impl PartialEq<[u8]> for MsgBuffer {
	#[inline]
	fn eq(&self, other: &[u8]) -> bool { self.buf == other }
}

impl PartialEq<Vec<u8>> for MsgBuffer {
	#[inline]
	fn eq(&self, other: &Vec<u8>) -> bool { self.buf == *other }
}

/// ## Instantiation.
///
/// This section provides methods for generating new `MsgBuffer` objects.
impl MsgBuffer {
	#[must_use]
	#[inline]
	/// # Instantiate From Raw Parts.
	///
	/// This directly sets the struct's fields, exactly like constructing it
	/// manually would, but since the fields are private, this is how that's
	/// done.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::MsgBuffer;
	///
	/// let buf = unsafe { MsgBuffer::from_raw_parts(vec![1, 2, 3], [0_u16; 32]) };
	/// ```
	///
	/// ## Safety
	///
	/// No validation is performed; the data must make sense or undefined
	/// things will happen.
	///
	/// The total buffer length is limited to [`u16::MAX`] — or more
	/// specifically any portion defined by a partition may not land outside
	/// the `u16` range.
	///
	/// The table of contents must be properly aligned and ordered. Any
	/// "unused" parts must begin and end at the maximum position of the "used"
	/// parts, e.g. `[0, 5, 5, 5, 5, 5, 5, 5...]`.
	pub unsafe fn from_raw_parts(buf: Vec<u8>, toc: [u16; 32]) -> Self {
		Self { buf, toc }
	}
}

/// ## Casting.
///
/// This section provides methods for converting `MsgBuffer` instances into
/// other types.
///
/// Note: this struct can also be dereferenced to `&[u8]`.
impl MsgBuffer {
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
	#[inline]
	/// # As Str.
	///
	/// Return the buffer content as a string slice.
	///
	/// ## Safety
	///
	/// The string's UTF-8 is not validated for sanity!
	pub unsafe fn as_str(&self) -> &str { std::str::from_utf8_unchecked(self) }

	#[must_use]
	#[inline]
	/// # As Bytes.
	///
	/// Return as a slice.
	pub fn as_bytes(&self) -> &[u8] { self }

	#[allow(clippy::missing_const_for_fn)] // This doesn't work.
	#[must_use]
	#[inline]
	/// # Into Vec.
	///
	/// Consume and return the underlying vector.
	pub fn into_vec(self) -> Vec<u8> { self.buf }
}

/// ## Operations.
///
/// This section provides methods for working with `MsgBuffer` instances.
impl MsgBuffer {
	#[must_use]
	#[inline]
	/// # Total Buffer Length.
	///
	/// Return the length of the entire buffer (rather than a single part).
	pub fn total_len(&self) -> usize { self.buf.len() }

	#[allow(clippy::comparison_chain)] // We're only matching two arms.
	/// # Replace Part.
	///
	/// This method replaces a given part of the buffer with `buf`, which can
	/// be of any size. If the new content is a different length than the
	/// original, the table of contents will be adjusted accordingly.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::MsgBuffer;
	///
	/// unsafe {
	///     let mut buf = MsgBuffer::from_raw_parts(Vec::new(), [0_u16; 32]);
	///
	///     assert_eq!(buf.deref(), b"");
	///     assert_eq!(buf.len_unchecked(0), 0);
	///
	///     buf.replace_unchecked(0, b"Hello World");
	///
	///     assert_eq!(buf.deref(), b"Hello World");
	///     assert_eq!(buf.len_unchecked(0), 11);
	///     assert_eq!(buf.start_unchecked(0), 0);
	///     assert_eq!(buf.end_unchecked(0), 11);
	/// }
	/// ```
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range, or any of the
	/// partitions exceed [`u16::MAX`].
	pub unsafe fn replace_unchecked(&mut self, idx: usize, buf: &[u8]) {
		// Get the lengths.
		let (old_len, new_len) = (self.len_unchecked(idx) as usize, buf.len());

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
			std::ptr::copy_nonoverlapping(
				buf.as_ptr(),
				self.buf.as_mut_ptr().add(self.start_unchecked(idx) as usize),
				new_len
			);
		}
	}

	/// # Resize: Grow.
	///
	/// This extends the buffer in the appropriate place and adjusts the table
	/// of contents accordingly.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range. Additionally,
	/// data in the vector may be left uninitialized and will need to be
	/// written to before being used!
	unsafe fn resize_grow(&mut self, idx: usize, adj: usize) {
		let end: usize = self.end_unchecked(idx) as usize;
		let len: usize = self.buf.len();

		self.buf.reserve(adj);

		// We need to shift things over.
		if end < len {
			ptr::copy(
				self.buf.as_ptr().add(end),
				self.buf.as_mut_ptr().add(end + adj),
				len - end
			);
		}

		self.buf.set_len(len + adj);
		self.increase(idx, adj as u16);
	}

	/// # Resize: Shrink.
	///
	/// This shrinks the buffer in the appropriate place and adjusts the table
	/// of contents accordingly.
	///
	/// ## Panics
	///
	/// This method may panic if the adjustment is larger than the length of
	/// the affected parts (i.e. causing their positions to overflow).
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	unsafe fn resize_shrink(&mut self, idx: usize, adj: usize) {
		let end: usize = self.end_unchecked(idx) as usize;

		// End-of-buffer shortcut.
		if end == self.buf.len() {
			self.buf.truncate(end - adj);
		}
		// Middle incision.
		else {
			self.buf.drain(end - adj..end);
		}

		self.decrease(idx, adj as u16);
	}

	/// # Zero Part (Unchecked).
	///
	/// This method truncates a part to zero-length, shifting all subsequent
	/// parts to the left as necessary.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	pub unsafe fn zero_unchecked(&mut self, idx: usize) {
		self.buf.drain(self.start_unchecked(idx) as usize..self.end_unchecked(idx) as usize);
		self.decrease(idx, self.len_unchecked(idx));
	}
}

/// ## Part Details.
///
/// These methods deal with individual parts, things like their lengths,
/// ranges, etc.
impl MsgBuffer {
	#[must_use]
	#[inline]
	/// # Part Is Empty (Unchecked).
	///
	/// This returns `true` if a given part is empty.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of bounds.
	pub const unsafe fn is_empty_unchecked(&self, idx: usize) -> bool {
		self.start_unchecked(idx) == self.end_unchecked(idx)
	}

	#[must_use]
	#[inline]
	/// # Part Length (Unchecked).
	///
	/// This returns the length of a given part, equivalent to `end-start`.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of bounds.
	pub const unsafe fn len_unchecked(&self, idx: usize) -> u16 {
		self.end_unchecked(idx) - self.start_unchecked(idx)
	}

	#[must_use]
	#[inline]
	/// # Part Start (Unchecked).
	///
	/// This returns the inclusive buffer starting index  for a given part.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of bounds.
	pub const unsafe fn start_unchecked(&self, idx: usize) -> u16 { self.toc[idx * 2] }

	#[must_use]
	#[inline]
	/// # Part End (Unchecked).
	///
	/// This returns the exclusive buffer endind index for a given part.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of bounds.
	pub const unsafe fn end_unchecked(&self, idx: usize) -> u16 {
		self.toc[(idx << 1) + 1]
	}

	/// # Decrease Indexing From.
	///
	/// This decreases the length of a partition, nudging all subsequent
	/// partitions to the left.
	///
	/// ## Panics
	///
	/// This method will panic if the adjustent is larger than any of the
	/// affected parts.
	fn decrease(&mut self, mut idx: usize, adj: u16) {
		idx = (idx << 1) + 1;
		while idx < 32 {
			self.toc[idx] -= adj;
			idx += 1;
		}
	}

	/// # Increase Indexing From.
	///
	/// This increases the length of a partition, nudging all subsequent
	/// partitions to the right.
	fn increase(&mut self, mut idx: usize, adj: u16) {
		idx = (idx << 1) + 1;
		while idx < 32 {
			self.toc[idx] += adj;
			idx += 1;
		}
	}
}



/*
/// # Generate Adjustment Mask From Offset.
///
/// Offsets exclusively land on a partition's ending point, and affect all
/// subsequent entries (starting and ending). Thankfully that cuts the number
/// of variations in half, but that still leaves us with 16 chunks to fill...
fn adjustment_mask(idx: usize, adj: u16) -> u16x32 {
	match idx {
		 0 => u16x32::new(0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 1 => u16x32::new(0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 2 => u16x32::new(0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 3 => u16x32::new(0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 4 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 5 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 6 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 7 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 8 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		 9 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		10 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		11 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj, adj, adj),
		12 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj, adj, adj),
		13 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj, adj, adj),
		14 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj, adj, adj),
		15 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, adj),
		_ => panic!("Index out of range."),
	}
}
*/



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn replace() { unsafe {
		let mut buf = MsgBuffer::from_raw_parts(
			vec![0, 0, 1, 1, 0, 0],
			[
				2, 4,
				4, 6,
				6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
				6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6
			]
		);

		// Bigger.
		buf.replace_unchecked(0, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2, 0, 0]);
		assert_eq!(buf.start_unchecked(0)..buf.end_unchecked(0), 2..5);
		assert_eq!(buf.len_unchecked(0), 3);
		assert_eq!(buf.is_empty_unchecked(0), false);
		assert_eq!(buf.start_unchecked(1)..buf.end_unchecked(1), 5..7);
		assert_eq!(buf.len_unchecked(1), 2);
		assert_eq!(buf.is_empty_unchecked(1), false);

		// Same Size.
		buf.replace_unchecked(0, &[3, 3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3, 3, 0, 0]);
		assert_eq!(buf.start_unchecked(0)..buf.end_unchecked(0), 2..5);
		assert_eq!(buf.start_unchecked(1)..buf.end_unchecked(1), 5..7);

		// Smaller.
		buf.replace_unchecked(0, &[1]);
		assert_eq!(buf, vec![0, 0, 1, 0, 0]);
		assert_eq!(buf.start_unchecked(0)..buf.end_unchecked(0), 2..3);
		assert_eq!(buf.start_unchecked(1)..buf.end_unchecked(1), 3..5);

		// Empty.
		buf.replace_unchecked(0, &[]);
		assert_eq!(buf, vec![0, 0, 0, 0]);
		assert_eq!(buf.start_unchecked(0)..buf.end_unchecked(0), 2..2);
		assert_eq!(buf.len_unchecked(0), 0);
		assert_eq!(buf.is_empty_unchecked(0), true);
		assert_eq!(buf.start_unchecked(1)..buf.end_unchecked(1), 2..4);
		assert_eq!(buf.len_unchecked(1), 2);
		assert_eq!(buf.is_empty_unchecked(1), false);

		// Bigger (End).
		buf.replace_unchecked(1, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2]);
		assert_eq!(buf.start_unchecked(0)..buf.end_unchecked(0), 2..2);
		assert_eq!(buf.start_unchecked(1)..buf.end_unchecked(1), 2..5);

		// Smaller (End).
		buf.replace_unchecked(1, &[3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3]);
		assert_eq!(buf.start_unchecked(0)..buf.end_unchecked(0), 2..2);
		assert_eq!(buf.start_unchecked(1)..buf.end_unchecked(1), 2..4);
	}}

	#[test]
	fn zero() { unsafe {
		let mut buf = MsgBuffer::from_raw_parts(
			vec![0, 0, 1, 1, 0, 0],
			[
				2, 4,
				4, 6,
				6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
				6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6
			]
		);

		// Empty.
		buf.zero_unchecked(0);
		assert_eq!(buf, vec![0, 0, 0, 0]);
		assert_eq!(buf.start_unchecked(0)..buf.end_unchecked(0), 2..2);
		assert_eq!(buf.start_unchecked(1)..buf.end_unchecked(1), 2..4);
	}}
}
