/*!
# FYI Message: Message Buffer

`MsgBuf` is a light wrapper containing a 1024-capacity `bytes::BytesMut` and a
partition table to logically slices within the buffer.
*/

use bytes::BytesMut;
use crate::Partitions;
use std::{
	borrow::Borrow,
	fmt,
	ops::Deref,
};



#[derive(Debug, Clone, Hash, PartialEq)]
/// Message Buffer.
///
/// This handles the actual data.
pub struct MsgBuf {
	buf: BytesMut,
	parts: Partitions,
}

impl Borrow<[u8]> for MsgBuf {
	#[inline]
	fn borrow(&self) -> &[u8] {
		&*self.buf
	}
}

impl Default for MsgBuf {
	/// Default.
	fn default() -> Self {
		Self {
			buf: BytesMut::with_capacity(1024),
			parts: Partitions::default(),
		}
	}
}

impl Deref for MsgBuf {
	type Target = [u8];

	/// Deref.
	fn deref(&self) -> &Self::Target {
		&*self.buf
	}
}

impl fmt::Display for MsgBuf {
	#[inline]
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(unsafe { std::str::from_utf8_unchecked(&*self.buf) })
	}
}

impl MsgBuf {
	// ------------------------------------------------------------------------
	// Instantiation
	// ------------------------------------------------------------------------

	#[must_use]
	/// New
	///
	/// Create a new `MsgBuf` from a single buffer with pre-calculated
	/// partition sizes. If the chunk total is lower than the buffer's length,
	/// an additional chunk will be created to fill the gap.
	///
	/// Panics if more than 15 parts are constructed or the chunks add up to
	/// more than the buffer's length.
	pub fn new(buf: &[u8], parts: &[usize]) -> Self {
		assert!(
			Partitions::MAX_USED >= parts.len(),
			"MsgBufs may not have more than {} parts.",
			Partitions::MAX_USED
		);
		Self {
			buf: BytesMut::from(buf),
			parts: Partitions::new_bounded(parts, buf.len()),
		}
	}

	#[must_use]
	/// From
	///
	/// Create a new `MsgBuf` from a single buffer with a single partition.
	pub fn from(buf: &[u8]) -> Self {
		Self {
			buf: BytesMut::from(buf),
			parts: Partitions::one(buf.len()),
		}
	}

	#[must_use]
	/// From Many
	///
	/// Create a new `MsgBuf` from multiple buffers, storing each in its own
	/// partitioning table.
	///
	/// Panics if more than `15` parts are needed.
	pub fn from_many(bufs: &[&[u8]]) -> Self {
		assert!(
			Partitions::MAX_USED >= bufs.len(),
			"MsgBufs may not have more than {} parts.",
			Partitions::MAX_USED
		);
		unsafe { Self::from_many_unchecked(bufs) }
	}

	#[must_use]
	/// From Many (Unchecked)
	///
	/// Same as `from_many()` but without the length assertion.
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn from_many_unchecked(bufs: &[&[u8]]) -> Self {
		let mut out = Self::default();

		bufs.iter().for_each(|b| {
			out.buf.extend_from_slice(b);
			out.parts.add_part_unchecked(b.len());
		});

		out
	}

	#[must_use]
	/// Splat
	///
	/// Create a new `MsgBuf` with `num` empty parts.
	///
	/// Panics if there are more than `15` parts.
	pub fn splat(num: usize) -> Self {
		Self {
			buf: BytesMut::with_capacity(1024),
			parts: Partitions::splat(num),
		}
	}



	// ------------------------------------------------------------------------
	// Working on the Whole
	// ------------------------------------------------------------------------

	/// Clear
	///
	/// Clear the buffer and partitioning table, restoring the instance to the
	/// equivalent of `default()` (save for any allocations that already
	/// happened).
	///
	/// See also `zero()`, which leaves behind one zero-length partition.
	pub fn clear(&mut self) {
		self.buf.clear();
		self.parts.clear();
	}

	/// Flatten
	///
	/// Keep the buffer, but drop to a single, spanning partition.
	pub fn flatten(&mut self) {
		self.parts.flatten();
	}

	#[must_use]
	/// Buffer Is Empty.
	pub fn is_empty(&self) -> bool {
		self.buf.is_empty()
	}

	#[must_use]
	/// Buffer length.
	pub fn len(&self) -> usize {
		self.buf.len()
	}

	/// Zero
	///
	/// Clear the buffer, but keep one empty partition.
	pub fn zero(&mut self) {
		self.buf.clear();
		self.parts.zero();
	}



	// ------------------------------------------------------------------------
	// Fetching Parts
	// ------------------------------------------------------------------------

	#[must_use]
	/// Number of Parts.
	pub const fn parts_len(&self) -> usize {
		self.parts.len()
	}

	#[must_use]
	/// Part.
	///
	/// Return the portion of the buffer corresponding to the part.
	///
	/// Panics if `idx` is out of range.
	pub fn part(&self, idx: usize) -> &[u8] {
		&self.buf[self.parts.part(idx)]
	}

	#[must_use]
	/// Part (Unchecked).
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn part_unchecked(&self, idx: usize) -> &[u8] {
		&self.buf[self.parts.part_unchecked(idx)]
	}

	#[must_use]
	/// Part Mut.
	///
	/// Mutably return the portion of the buffer corresponding to the part.
	///
	/// Panics if `idx` is out of range.
	pub fn part_mut(&mut self, idx: usize) -> &mut [u8] {
		&mut self.buf[self.parts.part(idx)]
	}

	#[must_use]
	/// Part Mut (Unchecked).
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn part_mut_unchecked(&mut self, idx: usize) -> &mut [u8] {
		&mut self.buf[self.parts.part_unchecked(idx)]
	}

	#[must_use]
	/// Spread.
	///
	/// Return the contiguous spread of two parts.
	///
	/// Panics if `idx1` or `idx2` are out of range.
	pub fn spread(&self, idx1: usize, idx2: usize) -> &[u8] {
		&self.buf[self.parts.spread(idx1, idx2)]
	}

	#[must_use]
	/// Spread Unchecked.
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn spread_unchecked(&self, idx1: usize, idx2: usize) -> &[u8] {
		&self.buf[self.parts.spread_unchecked(idx1, idx2)]
	}

	#[must_use]
	/// Spread Mut.
	///
	/// Return the contiguous spread of two parts.
	///
	/// Panics if `idx1` or `idx2` are out of range.
	pub fn spread_mut(&mut self, idx1: usize, idx2: usize) -> &mut [u8] {
		&mut self.buf[self.parts.spread(idx1, idx2)]
	}

	#[must_use]
	/// Spread Mut Unchecked.
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn spread_mut_unchecked(&mut self, idx1: usize, idx2: usize) -> &mut [u8] {
		&mut self.buf[self.parts.spread_unchecked(idx1, idx2)]
	}

	#[must_use]
	/// Is Part Empty
	///
	/// Panics if `idx` is out of range.
	pub fn part_is_empty(&self, idx: usize) -> bool {
		self.parts.part_is_empty(idx)
	}

	#[must_use]
	/// Is Part Empty (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub const unsafe fn part_is_empty_unchecked(&self, idx: usize) -> bool {
		self.parts.part_is_empty_unchecked(idx)
	}

	#[must_use]
	/// Get Part Length
	///
	/// Panics if `idx` is out of range.
	pub fn part_len(&self, idx: usize) -> usize {
		self.parts.part_len(idx)
	}

	#[must_use]
	/// Get Part Length (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub const unsafe fn part_len_unchecked(&self, idx: usize) -> usize {
		self.parts.part_len_unchecked(idx)
	}



	// ------------------------------------------------------------------------
	// Adding/Removing Parts
	// ------------------------------------------------------------------------

	/// Add Part
	///
	/// Panics if the partition table is full.
	pub fn add_part(&mut self, buf: &[u8]) {
		let len: usize = buf.len();
		if 0 != len {
			self.buf.extend_from_slice(buf);
		}
		self.parts.add_part(len);
	}

	/// Insert Part
	///
	/// Panics if `idx` is out of bounds or the table is full.
	pub fn insert_part(&mut self, idx: usize, buf: &[u8]) {
		let len: usize = buf.len();
		if 0 != len {
			// Grow before the start of the position.
			let start: usize = self.parts.part_start(idx);
			let adj: usize = buf.len();
			self.grow_buffer_at(start, adj);

			// Copy the new data to that spot.
			let end: usize = start + adj;
			self.buf[start..end].copy_from_slice(buf);
		}

		// Realign the partitions.
		self.parts.insert_part(idx, buf.len());
	}

	/// Remove Part
	///
	/// Panics if `idx` is out of bounds.
	pub fn remove_part(&mut self, idx: usize) {
		let len: usize = self.parts.part_len(idx);
		if 0 != len {
			unsafe {
				// Shrink the buffer.
				let end: usize = self.parts.part_end_unchecked(idx);
				self.shrink_buffer_at(end, len);
			}
		}

		// Realign the partitions.
		unsafe { self.parts.remove_part_unchecked(idx); }
	}



	// ------------------------------------------------------------------------
	// Changing Parts
	// ------------------------------------------------------------------------

	/// Clear Part
	///
	/// Panics if `idx` is out of bounds.
	pub fn clear_part(&mut self, idx: usize) {
		let len: usize = self.parts.part_len(idx);
		if 0 != len {
			unsafe {
				// Shrink the buffer.
				let end: usize = self.parts.part_end_unchecked(idx);
				self.shrink_buffer_at(end, len);

				// Realign the partitions.
				self.parts.shrink_part_unchecked(idx, len);
			}
		}
	}

	/// Clear Part (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn clear_part_unchecked(&mut self, idx: usize) {
		let len: usize = self.parts.part_len_unchecked(idx);
		if 0 != len {
			// Shrink the buffer.
			let end: usize = self.parts.part_end_unchecked(idx);
			self.shrink_buffer_at(end, len);

			// Realign the partitions.
			self.parts.shrink_part_unchecked(idx, len);
		}
	}

	#[allow(clippy::comparison_chain)]
	/// Replace Part
	///
	/// Panics if `idx` is out of bounds.
	pub fn replace_part(&mut self, idx: usize, buf: &[u8]) {
		// Check the new size first; we might just need to clear the buffer.
		let new_len: usize = buf.len();
		if 0 == new_len {
			self.clear_part(idx);
			return;
		}

		unsafe {
			let old_len: usize = self.parts.part_len(idx);

			// Grow the part to size.
			if new_len > old_len {
				let adj: usize = new_len - old_len;
				self.grow_buffer_at(self.parts.part_end_unchecked(idx), adj);
				self.parts.grow_part_unchecked(idx, adj);
			}
			// Shrink the part to size.
			else if old_len > new_len {
				let adj: usize = old_len - new_len;
				self.shrink_buffer_at(self.parts.part_end_unchecked(idx), adj);
				self.parts.shrink_part_unchecked(idx, adj);
			}

			// Sizes match, now we can copy!
			self.buf[self.parts.part_unchecked(idx)].copy_from_slice(buf);
		}
	}

	#[allow(clippy::comparison_chain)]
	/// Replace Part (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn replace_part_unchecked(&mut self, idx: usize, buf: &[u8]) {
		// Check the new size first; we might just need to clear the buffer.
		let new_len: usize = buf.len();
		if 0 == new_len {
			self.clear_part_unchecked(idx);
			return;
		}

		let old_len: usize = self.parts.part_len_unchecked(idx);

		// Grow the part to size.
		if new_len > old_len {
			let adj: usize = new_len - old_len;
			self.grow_buffer_at(self.parts.part_end_unchecked(idx), adj);
			self.parts.grow_part_unchecked(idx, adj);
		}
		// Shrink the part to size.
		else if old_len > new_len {
			let adj: usize = old_len - new_len;
			self.shrink_buffer_at(self.parts.part_end_unchecked(idx), adj);
			self.parts.shrink_part_unchecked(idx, adj);
		}

		// Sizes match, now we can copy!
		self.buf[self.parts.part_unchecked(idx)].copy_from_slice(buf);
	}



	// ------------------------------------------------------------------------
	// Internal Helpers
	// ------------------------------------------------------------------------

	/// Grow Buffer At/By
	///
	/// Insert `adj` zeroes into the buffer at `pos - 1`.
	///
	/// # Safety
	///
	/// It is up to the parent method to adjust the partitions accordingly.
	/// At the very least, the method will panic if the position is out of
	/// range.
	fn grow_buffer_at(&mut self, pos: usize, adj: usize) {
		let len: usize = self.buf.len();

		// If it is at the end, we can just work straight on the buffer.
		if pos == len {
			self.buf.resize(pos + adj, 0);
		}
		// Otherwise we need split, fiddle, and glue it up.
		else {
			let b = self.buf.split_off(pos);
			self.buf.resize(pos + adj, 0);
			self.buf.unsplit(b);
		}
	}

	/// Shrink Buffer At/By
	///
	/// Remove the previous `adj` bytes preceding position `pos`.
	///
	/// # Safety
	///
	/// It is up to the parent method to adjust the partitions accordingly.
	/// At the very least, the method will panic if the position is out of
	/// range.
	fn shrink_buffer_at(&mut self, pos: usize, adj: usize) {
		let len: usize = self.buf.len();

		// If it is at the end, we can just work straight on the buffer.
		if pos == len {
			self.buf.truncate(pos - adj);
		}
		// Otherwise we need split, fiddle, and glue it up.
		else {
			let b = self.buf.split_off(pos);
			self.buf.truncate(pos - adj);
			self.buf.unsplit(b);
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	// Some strings of different lengths.
	const SM1: &[u8] = b"cat";
	const SM2: &[u8] = b"dog";
	const MD1: &[u8] = b"pencil";
	const MD2: &[u8] = b"yellow";
	const LG1: &[u8] = b"dinosaurs";
	const LG2: &[u8] = b"arcosaurs";

	#[test]
	fn t_new() {
		// New empty.
		let mut buf = MsgBuf::new(&[], &[]);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), 0);

		// New filled, no parts.
		buf = MsgBuf::new(SM1, &[]);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), SM1.len());
		assert_eq!(buf.part(1), SM1);

		// New filled, spanning part.
		buf = MsgBuf::new(SM1, &[SM1.len()]);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), SM1.len());
		assert_eq!(buf.part(1), SM1);

		// New filled, 2 parts.
		buf = MsgBuf::new(LG1, &[4, 5]);
		assert_eq!(buf.len(), LG1.len());
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(buf.part(1), &b"dino"[..]);
		assert_eq!(buf.part(2), &b"saurs"[..]);

		// New filled, 2 parts (implied).
		buf = MsgBuf::new(LG1, &[4]);
		assert_eq!(buf.len(), LG1.len());
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(buf.part(1), &b"dino"[..]);
		assert_eq!(buf.part(2), &b"saurs"[..]);
	}

	#[test]
	fn t_from() {
		// From empty.
		let mut buf = MsgBuf::from(&[]);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), 0);

		// From filled.
		buf = MsgBuf::from(SM1);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), SM1.len());
		assert_eq!(buf.part(1), SM1);

		// From Many empty.
		buf = MsgBuf::from_many(&[]);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);

		// From many one.
		buf = MsgBuf::from_many(&[SM1]);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part(1), SM1);

		buf = MsgBuf::from_many(&[SM1, MD1, LG1]);
		assert_eq!(buf.len(), 18);
		assert_eq!(buf.parts_len(), 3);
		assert_eq!(buf.part(1), SM1);
		assert_eq!(buf.part(2), MD1);
		assert_eq!(buf.part(3), LG1);
	}

	#[test]
	fn t_clear_splat_zero() {
		// Clearing empty.
		let mut buf = MsgBuf::default();
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);
		buf.clear();
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);

		// Clear filled.
		buf = MsgBuf::from(SM1);
		buf.clear();
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);

		// Zero empty.
		buf = MsgBuf::default();
		buf.zero();
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), 0);

		// Zero full.
		buf = MsgBuf::from(SM1);
		buf.zero();
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), 0);

		// Check a splat.
		buf = MsgBuf::splat(3);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 3);
		assert_eq!(buf.part_len(1), 0);
		assert_eq!(buf.part_len(2), 0);
		assert_eq!(buf.part_len(3), 0);
	}

	#[test]
	fn t_add_remove_part() {
		let mut buf = MsgBuf::default();

		// Add empty.
		buf.add_part(&[]);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), 0);

		// Remove it.
		buf.remove_part(1);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);

		// Add something.
		buf.add_part(SM1);
		buf.add_part(MD1);
		buf.add_part(LG1);
		assert_eq!(buf.len(), 18);
		assert_eq!(buf.parts_len(), 3);
		assert_eq!(buf.part(1), SM1);
		assert_eq!(buf.part(2), MD1);
		assert_eq!(buf.part(3), LG1);

		// Try removing from each index.
		buf = MsgBuf::from_many(&[SM1, MD1, LG1]);
		buf.remove_part(1);
		assert_eq!(buf.len(), 15);
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(buf.part(1), MD1);
		assert_eq!(buf.part(2), LG1);

		buf = MsgBuf::from_many(&[SM1, MD1, LG1]);
		buf.remove_part(2);
		assert_eq!(buf.len(), 12);
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(buf.part(1), SM1);
		assert_eq!(buf.part(2), LG1);

		buf = MsgBuf::from_many(&[SM1, MD1, LG1]);
		buf.remove_part(3);
		assert_eq!(buf.len(), 9);
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(buf.part(1), SM1);
		assert_eq!(buf.part(2), MD1);

		// Now try to remove all parts, from the left.
		buf = MsgBuf::from_many(&[SM1, MD1, LG1]);
		buf.remove_part(1);
		buf.remove_part(1);
		buf.remove_part(1);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);

		// And again from the right.
		buf = MsgBuf::from_many(&[SM1, MD1, LG1]);
		buf.remove_part(3);
		buf.remove_part(2);
		buf.remove_part(1);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);
	}

	#[test]
	fn t_insert_part() {
		// Test insertion into a spread buffer.
		for b in [&[], SM1].iter() {
			let mut buf = MsgBuf::from(SM1);
			buf.insert_part(1, b);
			assert_eq!(buf.len(), SM1.len() + b.len());
			assert_eq!(buf.parts_len(), 2);
			assert_eq!(buf.part(1), *b);
			assert_eq!(buf.part(2), SM1);
		}

		// Test insertion into a multi-part buffer at each index.
		for i in 1..4 {
			for b in [&[], SM1].iter() {
				let mut buf = MsgBuf::from_many(&[SM2, MD2, LG2]);
				buf.insert_part(i, b);
				assert_eq!(buf.len(), 18 + b.len());
				assert_eq!(buf.parts_len(), 4);
				assert_eq!(buf.part(i), *b);
			}
		}
	}

	#[test]
	// Can cover `clear()` as well.
	fn t_replace_part() {
		// Test replacement when there's just one part.
		for b in [SM1, MD1, LG1].iter() {
			let mut buf = MsgBuf::from(MD2);
			buf.replace_part(1, b);
			assert_eq!(buf.len(), b.len());
			assert_eq!(buf.parts_len(), 1);
			assert_eq!(&buf.part(1), b);
		}

		// Test replacement at each index.
		for i in 1..4 {
			for b in [SM1, MD1, LG1].iter() {
				let mut buf = MsgBuf::from_many(&[SM2, MD2, LG2]);
				buf.replace_part(i, b);
				assert_eq!(buf.parts_len(), 3);
				assert_eq!(&buf.part(i), b);
			}
		}

		// And real quick test an empty replacement.
		let mut buf = MsgBuf::from_many(&[SM2, MD2, LG2]);
		buf.replace_part(1, &[]);
		assert_eq!(buf.parts_len(), 3);
		assert_eq!(buf.len(), 15);
	}

	#[test]
	fn t_spread() {
		let buf = MsgBuf::from_many(&[SM2, MD2, LG2]);

		assert_eq!(buf.spread(1, 2), b"dogyellow");
		assert_eq!(buf.spread(2, 3), b"yellowarcosaurs");
		assert_eq!(buf.spread(1, 3), b"dogyellowarcosaurs");
	}

	#[test]
	fn t_deref() {
		const SM2: &[u8] = b"dog";
		const MD2: &[u8] = b"yellow";
		const LG2: &[u8] = b"arcosaurs";
		let buf = MsgBuf::from_many(&[SM2, MD2, LG2]);

		assert_eq!(buf.deref(), b"dogyellowarcosaurs");
	}
}
