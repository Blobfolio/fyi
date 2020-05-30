/*!
# FYI Message: Message Buffer

`MsgBuf` is a light wrapper containing a 1024-capacity `bytes::BytesMut` and a
partition table to logically slices within the buffer.
*/

use bytes::BytesMut;
use smallvec::SmallVec;
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
	parts: SmallVec<[(usize, usize); 16]>,
}

impl Borrow<[u8]> for MsgBuf {
	#[inline]
	fn borrow(&self) -> &[u8] {
		self.buf.deref()
	}
}

impl Default for MsgBuf {
	/// Default.
	fn default() -> Self {
		Self {
			buf: BytesMut::with_capacity(1024),
			parts: SmallVec::<[(usize, usize); 16]>::new(),
		}
	}
}

impl Deref for MsgBuf {
	type Target = [u8];

	/// Deref.
	fn deref(&self) -> &Self::Target {
		self.buf.deref()
	}
}

impl fmt::Display for MsgBuf {
	#[inline]
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(unsafe { std::str::from_utf8_unchecked(self.deref()) })
	}
}

impl MsgBuf {
	// ------------------------------------------------------------------------
	// Static Methods
	// ------------------------------------------------------------------------

	#[must_use]
	/// New Buffer With Partitioning.
	///
	/// Like `from()`, except you can supply a partitioning scheme to apply to
	/// the stream.
	pub fn new(buf: &[u8], parts: &[(usize, usize)]) -> Self {
		let mut out = Self::default();

		if ! buf.is_empty() {
			out.buf.extend_from_slice(buf);
		}
		unsafe { out.partition(parts); }

		out
	}

	#[must_use]
	/// New Buffer From Slice.
	///
	/// Create a new buffer from the slice with a single partition.
	///
	/// For an unpartitioned, empty buffer, use `MsgBuf::default()`.
	pub fn from(buf: &[u8]) -> Self {
		Self {
			buf: BytesMut::from(buf),
			parts: SmallVec::from_slice(&[(0, buf.len())]),
		}
	}

	#[must_use]
	/// New Buffer From Slices.
	///
	/// Build a new buffer from an array of slices, assigning partitioning
	/// around each part.
	///
	/// Panics if `bufs` is empty. (The parts within `bufs` can, however, be
	/// empty).
	pub fn from_many(bufs: &[&[u8]]) -> Self {
		assert!(! bufs.is_empty());

		let mut out = Self::default();
		let mut start: usize = 0;
		bufs.iter().for_each(|b| {
			let end: usize = start + b.len();
			if end != start {
				out.buf.extend_from_slice(b);
			}
			out.parts.push((start, end));
			start = end;
		});

		out
	}

	#[must_use]
	/// Empty With Parts
	///
	/// This creates an empty buffer, but with X number of empty parts that can
	/// later be written to.
	pub fn with_parts(num: usize) -> Self {
		Self {
			buf: BytesMut::with_capacity(1024),
			parts: SmallVec::from_elem((0, 0), usize::max(1, num)),
		}
	}



	// ------------------------------------------------------------------------
	// Working on the Total
	// ------------------------------------------------------------------------

	/// Clear.
	///
	/// Clear both the buffer and partitioning table. This restores the
	/// instance to the equivalent of `default()`, but does not re-allocate.
	pub fn clear(&mut self) {
		self.buf.truncate(0);
		self.parts.clear();
	}

	#[allow(clippy::comparison_chain)]
	/// Replace the Buffer and Repartition.
	///
	/// Replace the buffer and partitioning table on the instance to e.g. open
	/// it up to a second life and higher purpose.
	///
	/// If the new buffer is empty, a single zero-length partition will be
	/// created. To fully reset the instance, use `clear()` instead.
	pub fn replace(&mut self, buf: &[u8], parts: &[(usize, usize)]) {
		// If the new part is empty, clear the whole buffer.
		if buf.is_empty() {
			self.buf.truncate(0);
		}
		// If the old part was empty, push the whole thing.
		else if self.buf.is_empty() {
			self.buf.extend_from_slice(buf);
		}
		// We have to do a little more figuring.
		else {
			let old_len: usize = self.buf.len();
			let new_len: usize = buf.len();

			// The new buffer is bigger.
			if new_len > old_len {
				self.buf.resize(new_len, 0);
			}
			// The new buffer is smaller.
			else if old_len > new_len {
				self.buf.truncate(new_len);
			}

			// Now that everything's the same size, copy!
			self.buf.copy_from_slice(buf);
		}

		// Handle the partitioning and we're done!
		self.repartition(parts);
	}



	// ------------------------------------------------------------------------
	// Working With Lines
	// ------------------------------------------------------------------------

	#[must_use]
	/// Line count.
	///
	/// This returns `n + 1` where `n` is the number of `\n` characters stored
	/// in the buffer.
	///
	/// This crate doesn't do carriage returns! Haha.
	pub fn count_lines(&self) -> usize {
		if self.buf.is_empty() { 0 }
		else { bytecount::count(&self.buf, b'\n') + 1 }
	}



	// ------------------------------------------------------------------------
	// Partitioning
	// ------------------------------------------------------------------------

	/// Add (Empty) Partition
	///
	/// Add an empty partition to the end of the table.
	pub fn add_partition(&mut self) {
		let from: usize = self.buf.len();
		self.parts.push((from, from));
	}

	#[must_use]
	/// Partitions count.
	///
	/// Return the number of partitions. Empty buffers can return values other
	/// than zero if they've been partitioned, however an unpartitioned buffer
	/// will always return `0`.
	pub fn count_partitions(&self) -> usize {
		self.parts.len()
	}

	/// Flatten Partitions.
	///
	/// Replace the partitioning table with a single `0..len()` partition.
	///
	/// If the buffer is unpartitioned, a partition will be added.
	pub fn flatten_partitions(&mut self) {
		if self.is_partitioned() {
			// Reduce to one part if we're over that.
			if self.parts.len() > 1 {
				self.parts.truncate(1);
			}

			// Replace the part.
			self.parts[0].0 = 0;
			self.parts[0].1 = self.buf.len();
		}
		else {
			self.parts.push((0, 0));
		}
	}

	#[must_use]
	/// Get Paritition
	///
	/// Return the start and end positions of the partition.
	///
	/// Panics if `idx` is out of bounds.
	pub fn get_partition(&self, idx: usize) -> (usize, usize) {
		self.parts[idx]
	}

	#[must_use]
	/// Get Paritition
	///
	/// Return the start position of `idx1` and the end position of `idx2`.
	///
	/// Panics if `idx1` or `idx2` are out of bounds or out of order.
	pub fn get_partitions(&self, idx1: usize, idx2: usize) -> (usize, usize) {
		(self.parts[idx1].0, self.parts[idx2].1)
	}

	/// Insert (Empty) Partition
	///
	/// Insert an empty partition at the specified index.
	///
	/// Panics if `idx` is out of bounds (and non-zero).
	pub fn insert_partition(&mut self, idx: usize) {
		if self.parts.is_empty() {
			self.parts.push((0, 0));
		}
		else {
			self.parts.insert_from_slice(idx, &[(self.parts[idx].0, self.parts[idx].0)]);
		}
	}

	#[must_use]
	/// Is Partitioned?
	///
	/// This is only `true` after calling `default()` or `clear()`, otherwise
	/// even empty buffers will generally have a zero-length partition.
	pub fn is_partitioned(&self) -> bool {
		! self.parts.is_empty()
	}

	/// Repartition
	///
	/// Replace the current partition table with the the provided ranges.
	///
	/// Ranges must run contiguously from `0..len()`. If the provided ranges
	/// have gaps, additional parts will be inserted and sized to fit. Ranges
	/// must not overlap unless they are zero-length and begin where the
	/// previous range ends.
	///
	/// This method panics if any value is out of bounds or out of order.
	pub fn repartition(&mut self, parts: &[(usize, usize)]) {
		// Remove the old table.
		self.parts.clear();
		// Push the new one.
		unsafe { self.partition(parts); }
	}



	// ------------------------------------------------------------------------
	// Parts
	// ------------------------------------------------------------------------

	/// Add Part.
	///
	/// Extend the buffer with a slice, creating a new partition around it. The
	/// resulting partition's index is returned.
	///
	/// If the buffer is empty, an empty partition is added.
	pub fn add_part(&mut self, buf: &[u8]) -> usize {
		let start: usize = self.buf.len();
		let end: usize = start + buf.len();
		if start < end {
			self.buf.extend_from_slice(buf);
		}
		self.parts.push((start, end));
		self.parts.len() - 1
	}

	/// Add Parts.
	///
	/// This is an optimized version of `add_part()` for adding multiple parts
	/// in one go. The behavior is identical to calling `add_part()` separately
	/// for each part, so see that method's documentation for more details.
	///
	/// The last inserted partition index is returned.
	///
	/// Panics if `bufs` is empty (though it may contain empty values).
	pub fn add_parts(&mut self, bufs: &[&[u8]]) -> usize {
		let mut start: usize = self.buf.len();
		bufs.iter().for_each(|b| {
			let end: usize = start + b.len();
			if end != start {
				self.buf.extend_from_slice(b);
			}
			self.parts.push((start, end));
			start = end;
		});

		self.parts.len() - 1
	}

	/// Clear Part.
	///
	/// Remove the part from the buffer and shrink the partition to zero.
	///
	/// Panics if `idx` is out of bounds.
	pub fn clear_part(&mut self, idx: usize) {
		if ! self.part_is_empty(idx) {
			self.resize_clear_part(idx);
		}
	}

	/// Extend Part.
	///
	/// Add data to an existing part, expanding the partition as needed.
	///
	/// Panics if `idx` is out of bounds.
	pub fn extend_part(&mut self, idx: usize, buf: &[u8]) {
		if ! buf.is_empty() {
			// The last part is special.
			if idx == self.parts.len() - 1 {
				self.buf.extend_from_slice(buf);
				self.parts[idx].1 += buf.len();
			}
			else {
				let adj: usize = buf.len();

				// Split at the part's end.
				let b = self.buf.split_off(self.parts[idx].1);

				// Append the new bit.
				self.buf.extend_from_slice(buf);

				// Glue it back together.
				self.buf.unsplit(b);

				// Shift the indexes.
				self.parts[idx].1 += adj;
				self.shift_partitions_right(idx + 1, adj);
			}
		}
	}

	#[must_use]
	/// Get Part.
	///
	/// Return the buffer slice corresponding to the partition.
	///
	/// Panics if `idx` is out of bounds.
	pub fn get_part(&self, idx: usize) -> &[u8] {
		&self.buf[self.parts[idx].0..self.parts[idx].1]
	}

	#[must_use]
	/// Get Part (Mut).
	///
	/// Return the buffer slice corresponding to the partition.
	///
	/// Panics if `idx` is out of bounds.
	pub fn get_part_mut(&mut self, idx: usize) -> &mut [u8] {
		&mut self.buf[self.parts[idx].0..self.parts[idx].1]
	}

	#[must_use]
	/// Get Part.
	///
	/// If you want all parts, use `parts()` instead. This returns the slice
	/// of the buffer beginning with part `idx1` and ending with part `idx2`.
	///
	/// Panics if `idx1` or `idx2` are out of bounds or out of order.
	pub fn get_parts(&self, idx1: usize, idx2: usize) -> &[u8] {
		&self.buf[self.parts[idx1].0..self.parts[idx2].1]
	}

	#[must_use]
	/// Get Part.
	///
	/// If you want all parts, use `parts()` instead. This returns the slice
	/// of the buffer beginning with part `idx1` and ending with part `idx2`.
	///
	/// Panics if `idx1` or `idx2` are out of bounds or out of order.
	pub fn get_parts_mut(&mut self, idx1: usize, idx2: usize) -> &mut [u8] {
		&mut self.buf[self.parts[idx1].0..self.parts[idx2].1]
	}

	#[must_use]
	/// Get Part Length.
	///
	/// Return the byte length of the partition.
	///
	/// Panics if `idx` is out of bounds.
	pub fn get_part_len(&self, idx: usize) -> usize {
		self.parts[idx].1 - self.parts[idx].0
	}

	#[must_use]
	/// Get Range.
	///
	/// Return an arbitrary buffer slice. Equivalent to `buf[start..end]`.
	///
	/// Panics if `start` or `end` are out of bounds.
	pub fn get_slice(&self, start: usize, end: usize) -> &[u8] {
		&self.buf[start..end]
	}

	#[must_use]
	/// Get Range.
	///
	/// Return an arbitrary buffer slice. Equivalent to `buf[start..end]`.
	///
	/// Panics if `start` or `end` are out of bounds.
	pub fn get_slice_mut(&mut self, start: usize, end: usize) -> &[u8] {
		&mut self.buf[start..end]
	}

	/// Insert Part.
	///
	/// Insert a part into the partition table (and the data into the
	/// corresponding part of the buffer), shifting all subsequent partitions
	/// to the right.
	///
	/// To insert a part into an *unpartitioned* table, you must use
	/// `add_part()`.
	///
	/// If you just wish to insert a partition (no data), use
	/// `insert_partition()` instead.
	///
	/// Panics if `idx` is out of bounds.
	pub fn insert_part(&mut self, idx: usize, buf: &[u8]) {
		self.insert_partition(idx);

		// Anything to fill?
		if ! buf.is_empty() {
			self.resize_grow_part(idx, buf.len());
			self.get_part_mut(idx).copy_from_slice(buf);
		}
	}

	#[must_use]
	/// Part Is Empty?
	///
	/// Panics if `idx` is out of bounds.
	pub fn part_is_empty(&self, idx: usize) -> bool {
		self.parts[idx].0 == self.parts[idx].1
	}

	#[must_use]
	/// Parts Iterator.
	///
	/// Loop through the `MsgBuf` part-by-part, including empty ones.
	///
	/// Returns `None` for unpartitioned buffers.
	pub const fn parts(&'_ self) -> MsgBufPartsIter<'_> {
		MsgBufPartsIter {
			buf: self,
			pos: 0,
		}
	}

	/// Remove Part
	///
	/// This is like `clear_part()` except the partition is also removed.
	///
	/// If this is the last and only part, a zero-length buffer will remain. To
	/// fully unpartition a buffer, use `clear()` instead.
	///
	/// Panics if `idx` is out of bounds.
	pub fn remove_part(&mut self, idx: usize) {
		if self.has_one_partition() {
			self.buf.truncate(0);
			self.parts[0].1 = 0;
		}
		else {
			if ! self.part_is_empty(idx) {
				self.resize_clear_part(idx);
			}
			self.parts.remove(idx);
		}
	}

	#[allow(clippy::comparison_chain)]
	/// Replace Part
	///
	/// Replace an existing part with the new data. The partition table bounds
	/// will be updated accordingly.
	///
	/// Panics if `idx` is out of bounds.
	pub fn replace_part(&mut self, idx: usize, buf: &[u8]) {
		let new_len: usize = buf.len();
		if new_len == 0 {
			self.resize_clear_part(idx);
			return;
		}

		let old_len: usize = self.get_part_len(idx);

		// The new part is bigger.
		if new_len > old_len {
			self.resize_grow_part(idx, new_len - old_len);
		}
		// The new part is smaller.
		else if old_len > new_len {
			self.resize_shrink_part(idx, old_len - new_len);
		}

		// Now that everything's the same size, copy!
		self.get_part_mut(idx).copy_from_slice(buf);
	}



	// ------------------------------------------------------------------------
	// Internal Helpers
	// ------------------------------------------------------------------------

	/// Is Only Part.
	fn has_one_partition(&self) -> bool {
		self.parts.len() == 1
	}

	/// Is Trailing Part.
	///
	/// Whether or not this part's chunk falls at the end of the buffer.
	///
	/// Panics if `idx` is out of bounds.
	fn is_trailing_part(&self, idx: usize) -> bool {
		self.parts[idx].1 == self.buf.len()
	}

	/// Partition.
	///
	/// Set up the partition table.
	///
	/// # Safety
	///
	/// This method will panic if the buffer is already partitioned or any of
	/// the new partitions are out of range or out of order.
	unsafe fn partition(&mut self, parts: &[(usize, usize)]) {
		// How much buffer we got?
		let max: usize = self.buf.len();

		// If the buffer is empty, we don't have to actually read `parts` to
		// partition; just enter however many `(0,0)` entries it takes. If
		// `parts` is empty, a single `(0,0)` will be created.
		if 0 == max {
			self.parts.extend_from_slice(&[(0, 0)].repeat(usize::max(1, parts.len())));
			return;
		}

		// Loop through the provided parts, filling in the gaps as needed.
		let mut last_idx: usize = 0;
		parts.iter().for_each(|p| {
			// The range must be in order of itself.
			// The range cannot go past the buffer boundaries.
			// The range cannot begin before the previous end.
			assert!(last_idx <= p.0 && p.0 <= p.1 && p.1 <= max);

			// We might need to inject one.
			if p.0 > last_idx {
				self.parts.push((last_idx, p.0));
			}

			last_idx = p.1;
			self.parts.push((p.0, p.1));
		});

		// If the last part falls short of `len()`, add one more.
		if last_idx < max {
			self.parts.push((last_idx, max));
		}
	}

	#[allow(clippy::suspicious_else_formatting)]
	/// Resize Part (Clear)
	///
	/// Clear the part and fix up the partitioning.
	///
	/// Panics if `idx` is out of bounds.
	fn resize_clear_part(&mut self, idx: usize) {
		// It is at the end.
		if self.is_trailing_part(idx) {
			self.buf.truncate(self.parts[idx].0);
			self.parts[idx].1 = self.parts[idx].0;

			// Push all remaining partitions to this one's start/end.
			self.shift_partitions_abs(idx + 1, self.parts[idx].1);
		}
		// Anywhere else we need to do some surgery.
		else {
			let adj = self.get_part_len(idx);

			// Split the buffer at the part's end.
			let b = self.buf.split_off(self.parts[idx].1);

			// Truncate the stub to the part's start.
			self.buf.truncate(self.parts[idx].0);

			// Glue it back together.
			self.buf.unsplit(b);

			// Update the parts.
			self.parts[idx].1 = self.parts[idx].0;

			// And shift remaining slices.
			self.shift_partitions_left(idx + 1, adj);
		}
	}

	#[allow(clippy::suspicious_else_formatting)]
	/// Resize Part (Grow)
	///
	/// Grow a part by the specified amount. This will fill the space with
	/// random data.
	///
	/// Panics if `idx` is out of bounds.
	fn resize_grow_part(&mut self, idx: usize, adj: usize) {
		// It is at the end.
		if self.is_trailing_part(idx) {
			self.parts[idx].1 += adj;
			self.buf.resize(self.parts[idx].1, 0);

			// And shift remaining slices.
			self.shift_partitions_abs(idx + 1, self.parts[idx].1);
		}
		// Anywhere else we need to do some surgery.
		else {
			// Split the buffer at the part's end.
			let b = self.buf.split_off(self.parts[idx].1);

			// Grow the stub to the adjusted length.
			self.parts[idx].1 += adj;
			self.buf.resize(self.parts[idx].1, 0);

			// Glue it back together.
			self.buf.unsplit(b);

			// And shift remaining slices.
			self.shift_partitions_right(idx + 1, adj);
		}
	}

	#[allow(clippy::suspicious_else_formatting)]
	/// Resize Part (Shrink)
	///
	/// Shrink a part by the specified amount.
	///
	/// Panics if `idx` is out of bounds.
	fn resize_shrink_part(&mut self, idx: usize, adj: usize) {
		// It is at the end.
		if self.is_trailing_part(idx) {
			self.parts[idx].1 -= adj;
			self.buf.truncate(self.parts[idx].1);

			// Push all remaining partitions to this one's start/end.
			self.shift_partitions_abs(idx + 1, self.parts[idx].1);
		}
		// Anywhere else we need to do some surgery.
		else {
			// Split the buffer at the part's end.
			let b = self.buf.split_off(self.parts[idx].1);

			// Truncate the stub to the adjusted length.
			self.parts[idx].1 -= adj;
			self.buf.truncate(self.parts[idx].1);

			// Glue it back together.
			self.buf.unsplit(b);

			// Shift all remaining parts left by the amount lost.
			self.shift_partitions_left(idx + 1, adj);
		}
	}

	/// Shift Parts Absolute.
	///
	/// This is an internal helper that shifts the indexes of all partitions
	/// beginning at `idx` to `num`.
	///
	/// This is used when a partition in the middle has shrunk or been removed.
	fn shift_partitions_abs(&mut self, mut idx: usize, num: usize) {
		let len: usize = self.parts.len();
		while idx < len {
			self.parts[idx].0 = num;
			self.parts[idx].1 = num;
			idx += 1;
		}
	}

	/// Shift Parts Left.
	///
	/// This is an internal helper that shifts the indexes of all partitions
	/// beginning at `idx` left by `num` amount.
	///
	/// This is used when a partition in the middle has shrunk or been removed.
	fn shift_partitions_left(&mut self, mut idx: usize, num: usize) {
		let len: usize = self.parts.len();
		while idx < len {
			self.parts[idx].0 -= num;
			self.parts[idx].1 -= num;
			idx += 1;
		}
	}

	/// Shift Parts Right.
	///
	/// This is an internal helper that shifts the indexes of all partitions
	/// beginning at `idx` right by `num` amount.
	///
	/// This is used when a partition in the middle has been added or expanded.
	fn shift_partitions_right(&mut self, mut idx: usize, num: usize) {
		let len: usize = self.parts.len();
		while idx < len {
			self.parts[idx].0 += num;
			self.parts[idx].1 += num;
			idx += 1;
		}
	}
}



#[derive(Debug, Clone)]
/// Parts Iterator
///
/// Loop through the `MsgBuf` part-by-part, including empty ones.
///
/// Returns `None` for unpartitioned buffers.
pub struct MsgBufPartsIter<'mb> {
	buf: &'mb MsgBuf,
	pos: usize,
}

impl<'mb> Iterator for MsgBufPartsIter<'mb> {
	type Item = &'mb [u8];

	/// Next.
	fn next(&mut self) -> Option<Self::Item> {
		// We're still in range.
		if self.pos < self.buf.count_partitions() {
			self.pos += 1;
			Some(self.buf.get_part(self.pos - 1))
		}
		else { None }
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	// Assert something should be something via Display.
	macro_rules! ass {
		($label:expr, $found:expr, $expected:expr) => {
			assert_eq!(
				$found,
				$expected,
				"The {} should have been {} instead of {}.",
				$label,
				$expected,
				$found
			);
		};
	}

	// Assert something should be something via Debug.
	macro_rules! ass_dbg {
		($label:expr, $found:expr, $expected:expr) => {
			assert_eq!(
				$found,
				$expected,
				"The {} should have been {:?} instead of {:?}.",
				$label,
				$expected,
				$found
			);
		};
	}

	// We do so much [u8] it is handy to convert the failures to string before
	// debug-printing it.
	macro_rules! ass_u8 {
		($label:expr, $found:expr, $expected:expr) => {
			assert_eq!(
				$found,
				$expected,
				"The {} should have been {:?} instead of {:?}.",
				$label,
				unsafe { std::str::from_utf8_unchecked($expected) },
				unsafe { std::str::from_utf8_unchecked($found) }
			);
		};
	}

	const TEST1: &[u8] = &[32, 32, 32, 32];
	const TEST2: &[u8] = &[32, 10, 32, 10];
	const TEST3: &[u8] = &[10, 10, 10, 10, 10, 10];
	const TEST4: &[u8] = &[68, 68, 68];

	#[test]
	fn t_default() {
		// Test default.
		let buf = MsgBuf::default();
		ass_u8!("MsgBuf::default()", &buf[..], &[]);
		ass!("MsgBuf::default().len()", buf.len(), 0);
		ass!("MsgBuf::default().count_partitions()", buf.count_partitions(), 0);
	}

	#[test]
	fn t_from() {
		// Get one from some data.
		let mut buf = MsgBuf::from(TEST1);
		ass_u8!("MsgBuf::from(…)", &buf[..], TEST1);
		ass!("MsgBuf::from(…).len()", buf.len(), 4);
		ass!("MsgBuf::from(…).count_partitions()", buf.count_partitions(), 1);
		ass_u8!("MsgBuf::from(…)[..4]", &buf[0..4], TEST1);
		ass_u8!("MsgBuf::from(…).part(0)", buf.get_part(0), TEST1);
		ass_dbg!("MsgBuf::from(…).partition(0)", buf.get_partition(0), (0, 4));

		// Let's do a from_many now.
		buf = MsgBuf::from_many(&[TEST1, TEST2]);
		ass_u8!("from_many(…)", &buf[..], &[32, 32, 32, 32, 32, 10, 32, 10]);
		ass!("from_many(…).len()", buf.len(), 8);
		ass!("from_many(…).count_partitions()", buf.count_partitions(), 2);
		ass_u8!("from_many(…)[..4]", &buf[0..4], TEST1);
		ass_u8!("from_many(…).part(0)", buf.get_part(0), TEST1);
		ass_u8!("from_many(…)[4..]", &buf[4..], TEST2);
		ass_u8!("from_many(…).part(1)", buf.get_part(1), TEST2);
		ass_dbg!("from_many(…).partition(0)", buf.get_partition(0), (0, 4));
		ass_dbg!("from_many(…).partition(1)", buf.get_partition(1), (4, 8));
	}

	#[test]
	fn t_new() {
		// No partitions.
		let mut buf = MsgBuf::new(TEST1, &[]);
		ass_u8!("new(…)", &buf[..], TEST1);
		ass!("new(…).len", buf.len(), 4);
		ass!("new(…).count_partitions()", buf.count_partitions(), 1);
		ass_u8!("new(…)[0..4]", &buf[0..], TEST1);
		ass_u8!("new(…).part(0)", buf.get_part(0), TEST1);
		ass_dbg!("new(…).partition(0)", buf.get_partition(0), (0, 4));

		// One partition, left gap.
		buf = MsgBuf::new(TEST1, &[(2, 4)]);
		ass_u8!("new(…)", &buf[..], TEST1);
		ass!("new(…).len", buf.len(), 4);
		ass!("new(…).count_partitions()", buf.count_partitions(), 2);
		ass_u8!("new(…)[0..4]", &buf[0..], TEST1);
		ass_u8!("new(…).part(0)", buf.get_part(0), &TEST1[..2]);
		ass_u8!("new(…).part(1)", buf.get_part(1), &TEST1[2..]);
		ass_dbg!("new(…).partition(0)", buf.get_partition(0), (0, 2));
		ass_dbg!("new(…).partition(1)", buf.get_partition(1), (2, 4));

		// One partition, right gap.
		buf = MsgBuf::new(TEST1, &[(0, 2)]);
		ass_u8!("new(…)", &buf[..], TEST1);
		ass!("new(…).len", buf.len(), 4);
		ass!("new(…).count_partitions()", buf.count_partitions(), 2);
		ass_u8!("new(…)[0..4]", &buf[0..], TEST1);
		ass_u8!("new(…).part(0)", buf.get_part(0), &TEST1[..2]);
		ass_u8!("new(…).part(1)", buf.get_part(1), &TEST1[2..]);
		ass_dbg!("new(…).partition(0)", buf.get_partition(0), (0, 2));
		ass_dbg!("new(…).partition(1)", buf.get_partition(1), (2, 4));

		// One partition, left and right gap.
		buf = MsgBuf::new(TEST1, &[(1, 2)]);
		ass_u8!("new(…)", &buf[..], TEST1);
		ass!("new(…).len", buf.len(), 4);
		ass!("new(…).count_partitions()", buf.count_partitions(), 3);
		ass_u8!("new(…)[0..4]", &buf[0..], TEST1);
		ass_u8!("new(…).part(0)", buf.get_part(0), &TEST1[..1]);
		ass_u8!("new(…).part(1)", buf.get_part(1), &TEST1[1..2]);
		ass_u8!("new(…).part(2)", buf.get_part(2), &TEST1[2..]);
		ass_dbg!("new(…).partition(0)", buf.get_partition(0), (0, 1));
		ass_dbg!("new(…).partition(1)", buf.get_partition(1), (1, 2));
		ass_dbg!("new(…).partition(2)", buf.get_partition(2), (2, 4));

		// Two partitions, mid gap.
		buf = MsgBuf::new(TEST1, &[(0, 1),(2, 4)]);
		ass_u8!("new(…)", &buf[..], TEST1);
		ass!("new(…).len", buf.len(), 4);
		ass!("new(…).count_partitions()", buf.count_partitions(), 3);
		ass_u8!("new(…)[0..4]", &buf[0..], TEST1);
		ass_u8!("new(…).part(0)", buf.get_part(0), &TEST1[..1]);
		ass_u8!("new(…).part(1)", buf.get_part(1), &TEST1[1..2]);
		ass_u8!("new(…).part(2)", buf.get_part(2), &TEST1[2..]);
		ass_dbg!("new(…).partition(0)", buf.get_partition(0), (0, 1));
		ass_dbg!("new(…).partition(1)", buf.get_partition(1), (1, 2));
		ass_dbg!("new(…).partition(2)", buf.get_partition(2), (2, 4));
	}

	#[test]
	fn t_clear() {
		let mut buf = MsgBuf::new(TEST1, &[(1, 2)]);
		ass!("new(…).len", buf.len(), 4);
		ass!("new(…).count_partitions()", buf.count_partitions(), 3);
		buf.clear();
		ass!("new(…).len", buf.len(), 0);
		ass!("new(…).count_partitions()", buf.count_partitions(), 0);
	}

	#[test]
	fn t_replace() {
		let mut buf = MsgBuf::new(TEST1, &[(1, 2)]);
		ass!("new(…).len", buf.len(), 4);
		ass!("new(…).count_partitions()", buf.count_partitions(), 3);
		buf.replace(TEST2, &[]);
		ass_u8!("replaced buf", &buf[..], TEST2);
		ass!("replaced buf.len()", buf.len(), 4);
		ass!("replaced buf.count_partitions()", buf.count_partitions(), 1);
	}

	#[test]
	fn t_partitions() {
		let mut buf = MsgBuf::new(TEST1, &[(1, 2)]);
		ass!("buf.count_partitions()", buf.count_partitions(), 3);
		ass_u8!("buf.get_part(0)", buf.get_part(0), &TEST1[..1]);

		buf.flatten_partitions();
		ass!("flattened.count_partitions()", buf.count_partitions(), 1);
		ass_u8!("flattened.get_part(0)", buf.get_part(0), TEST1);

		// Insert left.
		buf.insert_partition(0);
		ass!("inserted.count_partitions()", buf.count_partitions(), 2);
		ass_u8!("inserted.get_part(0)", buf.get_part(0), &[]);
		ass_u8!("inserted.get_part(1)", buf.get_part(1), TEST1);

		// Insert Right.
		buf.add_partition();
		ass!("added.count_partitions()", buf.count_partitions(), 3);
		ass_u8!("added.get_part(0)", buf.get_part(0), &[]);
		ass_u8!("added.get_part(1)", buf.get_part(1), TEST1);
		ass_u8!("added.get_part(2)", buf.get_part(2), &[]);

		assert!(buf.is_partitioned());
		assert!(! MsgBuf::default().is_partitioned());

		// Repartition.
		buf.repartition(&[(0, 2)]);
		ass!("repartitioned.count_partitions()", buf.count_partitions(), 2);
		ass_u8!("repartitioned.get_part(0)", buf.get_part(0), &TEST1[..2]);
		ass_u8!("repartitioned.get_part(1)", buf.get_part(1), &TEST1[2..]);

		buf.clear();
		ass!("buf.len()", buf.len(), 0);
		ass!("repartitioned.count_partitions()", buf.count_partitions(), 0);
		buf.repartition(&[(0, 0), (0, 0), (0, 0)]);
		ass!("repartitioned.count_partitions()", buf.count_partitions(), 3);
	}

	#[test]
	fn t_parts() {
		let mut buf = MsgBuf::default();
		buf.add_part(TEST1);
		ass!("buf.len()", buf.len(), 4);
		ass!("buf.count_partitions()", buf.count_partitions(), 1);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);

		buf.add_part(TEST2);
		ass!("buf.len()", buf.len(), 8);
		ass!("buf.count_partitions()", buf.count_partitions(), 2);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);

		buf.add_parts(&[TEST1, TEST2]);
		ass!("buf.len()", buf.len(), 16);
		ass!("buf.count_partitions()", buf.count_partitions(), 4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST1);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST2);

		buf.clear_part(0);
		ass!("buf.len()", buf.len(), 12);
		ass!("buf.count_partitions()", buf.count_partitions(), 4);
		ass!("buf.get_part_len(0)", buf.get_part_len(0), 0);
		ass_u8!("buf.get_part(0)", buf.get_part(0), &[]);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST1);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST2);

		buf.clear_part(2);
		ass!("buf.len()", buf.len(), 8);
		ass!("buf.count_partitions()", buf.count_partitions(), 4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), &[]);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), &[]);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST2);

		buf.clear_part(3);
		ass!("buf.len()", buf.len(), 4);
		ass!("buf.count_partitions()", buf.count_partitions(), 4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), &[]);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), &[]);
		ass_u8!("buf.get_part(3)", buf.get_part(3), &[]);

		buf.extend_part(0, TEST1);
		ass!("buf.len()", buf.len(), 8);
		ass!("buf.count_partitions()", buf.count_partitions(), 4);
		ass!("buf.get_part_len(0)", buf.get_part_len(0), 4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), &[]);
		ass_u8!("buf.get_part(3)", buf.get_part(3), &[]);

		buf.extend_part(3, TEST1);
		ass!("buf.len()", buf.len(), 12);
		ass!("buf.count_partitions()", buf.count_partitions(), 4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), &[]);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST1);

		buf.extend_part(2, TEST1);
		ass!("buf.len()", buf.len(), 16);
		ass!("buf.count_partitions()", buf.count_partitions(), 4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST1);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST1);

		buf.insert_part(0, TEST3);
		ass!("buf.len()", buf.len(), 22);
		ass!("buf.count_partitions()", buf.count_partitions(), 5);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST3);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST1);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST2);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST1);
		ass_u8!("buf.get_part(4)", buf.get_part(4), TEST1);

		buf.insert_part(4, TEST3);
		ass!("buf.len()", buf.len(), 28);
		ass!("buf.count_partitions()", buf.count_partitions(), 6);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST3);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST1);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST2);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST1);
		ass_u8!("buf.get_part(4)", buf.get_part(4), TEST3);
		ass_u8!("buf.get_part(5)", buf.get_part(5), TEST1);

		ass!("buf.parts().count()", buf.parts().count(), 6);

		println!("{:?}", buf);
		buf.remove_part(0);
		println!("{:?}", buf);

		ass!("buf.len()", buf.len(), 22);
		ass!("buf.count_partitions()", buf.count_partitions(), 5);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST1);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST3);
		ass_u8!("buf.get_part(4)", buf.get_part(4), TEST1);

		buf.remove_part(3);
		ass!("buf.len()", buf.len(), 16);
		ass!("buf.count_partitions()", buf.count_partitions(), 4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST1);
		ass_u8!("buf.get_part(3)", buf.get_part(3), TEST1);

		buf.remove_part(3);
		ass!("buf.len()", buf.len(), 12);
		ass!("buf.count_partitions()", buf.count_partitions(), 3);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST1);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST1);

		ass!("buf.parts().count()", buf.parts().count(), 3);
		let mut parts = buf.parts();
		ass_dbg!("parts.next()", parts.next(), Some(TEST1));
		ass_dbg!("parts.next()", parts.next(), Some(TEST2));
		ass_dbg!("parts.next()", parts.next(), Some(TEST1));
		assert!(parts.next().is_none(), "parts.next() should be None.");

		// Replace same.
		buf = MsgBuf::from_many(&[TEST1, TEST1, TEST1]);
		buf.replace_part(0, TEST2);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST2);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST1);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST1);

		buf.replace_part(1, TEST2);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST2);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST1);

		buf.replace_part(2, TEST2);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST2);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST2);

		// Replace bigger.
		buf.replace_part(2, TEST3);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST2);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST2);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST3);

		buf.replace_part(1, TEST3);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST2);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST3);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST3);

		buf.replace_part(0, TEST3);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST3);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST3);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST3);

		// Replace smaller.
		buf.replace_part(0, TEST4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST4);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST3);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST3);

		// Replace smaller.
		buf.replace_part(2, TEST4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST4);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST3);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST4);

		// Replace smaller.
		buf.replace_part(1, TEST4);
		ass_u8!("buf.get_part(0)", buf.get_part(0), TEST4);
		ass_u8!("buf.get_part(1)", buf.get_part(1), TEST4);
		ass_u8!("buf.get_part(2)", buf.get_part(2), TEST4);
	}
}
