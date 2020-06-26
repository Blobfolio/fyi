/*!
# FYI Message: Message Buffer

`MsgBuf` is a light wrapper containing a 1024-capacity `bytes::BytesMut` and a
partition table to logically slices within the buffer.

This is intended for use within the crate only; it does not implement safety
checks on its own.
*/

use bytes::BytesMut;
use crate::{
	Partitions,
	traits::{
		PrintyPlease,
		EPrintyPlease,
	},
};
use std::{
	borrow::Borrow,
	fmt,
	ops::{
		AddAssign,
		Deref,
		Index,
		IndexMut,
	},
};



#[derive(Debug, Clone, Hash, PartialEq)]
/// Message Buffer.
///
/// This handles the actual data.
pub struct MsgBuf {
	buf: BytesMut,
	parts: Partitions,
}

impl AddAssign<&[u8]> for MsgBuf {
	fn add_assign(&mut self, other: &[u8]) {
		let len: usize = other.len();
		self.parts += len;
		if 0 != len {
			self.buf.extend_from_slice(other);
		}
	}
}

impl Borrow<[u8]> for MsgBuf {
	#[inline]
	fn borrow(&self) -> &[u8] {
		&*self.buf
	}
}

impl Default for MsgBuf {
	#[inline]
	fn default() -> Self {
		Self {
			buf: BytesMut::with_capacity(1024),
			parts: Partitions::default(),
		}
	}
}

impl Deref for MsgBuf {
	type Target = [u8];

	#[inline]
	fn deref(&self) -> &Self::Target {
		&*self.buf
	}
}

impl fmt::Display for MsgBuf {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(unsafe { std::str::from_utf8_unchecked(&*self.buf) })
	}
}

impl<'a> From<&'a [u8]> for MsgBuf {
	#[inline]
	fn from(buf: &'a [u8]) -> Self {
		Self {
			buf: BytesMut::from(buf),
			parts: Partitions::one(buf.len()),
		}
	}
}

/// Handle all the stupid slice sizes since this doesn't coerce. Haha.
macro_rules! from_many {
	($size:literal) => {
		impl<'a> From<&'a [&'a [u8]; $size]> for MsgBuf {
			fn from(bufs: &'a [&'a [u8]; $size]) -> Self {
				let mut out = Self::default();
				bufs.iter().for_each(|b| { out += b; });
				out
			}
		}
	};
}

/// Optimized From Empty.
impl<'a> From<&'a [&'a [u8]; 0]> for MsgBuf {
	#[inline]
	fn from(_bufs: &'a [&'a [u8]; 0]) -> Self {
		Self::default()
	}
}

/// Optimized From One.
impl<'a> From<&'a [&'a [u8]; 1]> for MsgBuf {
	#[inline]
	fn from(bufs: &'a [&'a [u8]; 1]) -> Self {
		Self {
			buf: BytesMut::from(bufs[0]),
			parts: Partitions::one(bufs[0].len()),
		}
	}
}

from_many!(2);
from_many!(3);
from_many!(4);
from_many!(5);
from_many!(6);
from_many!(7);
from_many!(8);
from_many!(9);
from_many!(10);
from_many!(11);
from_many!(12);
from_many!(13);
from_many!(14);
from_many!(15);

impl Index<usize> for MsgBuf {
	type Output = [u8];

	#[inline]
	fn index(&self, idx: usize) -> &Self::Output {
		&self.buf[self.parts.part(idx)]
	}
}

impl IndexMut<usize> for MsgBuf {
	#[inline]
	fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
		&mut self.buf[self.parts.part(idx)]
	}
}

impl MsgBuf {
	// ------------------------------------------------------------------------
	// Instantiation
	// ------------------------------------------------------------------------

	#[inline]
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
		Self {
			buf: BytesMut::from(buf),
			parts: Partitions::new_bounded(parts, buf.len()),
		}
	}

	#[inline]
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

	#[inline]
	/// Flatten
	///
	/// Keep the buffer, but drop to a single, spanning partition.
	pub fn flatten(&mut self) {
		self.parts.flatten();
	}

	#[inline]
	#[must_use]
	/// Buffer Is Empty.
	///
	/// Technically, we're testing the parts rather than the buffer since that
	/// lets us make this a `const fn`.
	pub const fn is_empty(&self) -> bool {
		0 == self.parts.max()
	}

	#[inline]
	#[must_use]
	/// Buffer length.
	///
	/// Same as with `is_empty()`, we're technically checking the parts rather
	/// than the buffer since that lets us make this a `const fn`.
	pub const fn len(&self) -> usize {
		self.parts.max()
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

	#[inline]
	#[must_use]
	/// Number of Parts.
	pub const fn parts_len(&self) -> usize {
		self.parts.len()
	}

	#[inline]
	#[must_use]
	/// Spread.
	///
	/// Return the contiguous spread of two parts.
	///
	/// Panics if `idx1` or `idx2` are out of range.
	pub fn spread(&self, idx1: usize, idx2: usize) -> &[u8] {
		&self.buf[self.parts.spread(idx1, idx2)]
	}

	#[inline]
	#[must_use]
	/// Spread Mut.
	///
	/// Return the contiguous spread of two parts.
	///
	/// Panics if `idx1` or `idx2` are out of range.
	pub fn spread_mut(&mut self, idx1: usize, idx2: usize) -> &mut [u8] {
		&mut self.buf[self.parts.spread(idx1, idx2)]
	}

	#[inline]
	#[must_use]
	/// Is Part Empty
	///
	/// Panics if `idx` is out of range.
	pub const fn part_is_empty(&self, idx: usize) -> bool {
		self.parts.part_is_empty(idx)
	}

	#[inline]
	#[must_use]
	/// Get Part Length
	///
	/// Panics if `idx` is out of range.
	pub const fn part_len(&self, idx: usize) -> usize {
		self.parts.part_len(idx)
	}



	// ------------------------------------------------------------------------
	// Adding/Removing Parts
	// ------------------------------------------------------------------------

	/// Insert Part
	///
	/// Panics if `idx` is out of bounds or the table is full.
	pub fn insert_part(&mut self, idx: usize, buf: &[u8]) {
		let len: usize = buf.len();
		if 0 != len {
			// Grow before the start of the position.
			let start: usize = self.parts[idx - 1];
			let adj: usize = buf.len();
			self.grow_buffer_at(start, adj);

			// Copy the new data to that spot.
			let end: usize = start + adj;
			self.buf[start..end].copy_from_slice(buf);
		}

		// Realign the partitions.
		self.parts.insert_part(idx, len);
	}

	/// Remove Part
	///
	/// Panics if `idx` is out of bounds.
	pub fn remove_part(&mut self, idx: usize) {
		let len: usize = self.parts.part_len(idx);
		// Shrink the buffer.
		if 0 != len {
			self.shrink_buffer_at(self.parts[idx], len);
		}

		// Realign the partitions.
		self.parts.remove_part(idx);
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
			// Shrink the buffer.
			self.shrink_buffer_at(self.parts[idx], len);

			// Realign the partitions.
			self.parts.shrink_part(idx, len);
		}
	}

	#[allow(clippy::comparison_chain)] // False positive: match len.cmp()
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

		let old_len: usize = self.parts.part_len(idx);

		// Grow the part to size.
		if new_len > old_len {
			let adj: usize = new_len - old_len;
			self.grow_buffer_at(self.parts[idx], adj);
			self.parts.grow_part(idx, adj);
		}
		// Shrink the part to size.
		else if old_len > new_len {
			let adj: usize = old_len - new_len;
			self.shrink_buffer_at(self.parts[idx], adj);
			self.parts.shrink_part(idx, adj);
		}

		// Sizes match, now we can copy!
		self.buf[self.parts.part(idx)].copy_from_slice(buf);
	}



	// ------------------------------------------------------------------------
	// Internal Helpers
	// ------------------------------------------------------------------------

	/// Grow Buffer At/By
	///
	/// Insert `adj` zeroes into the buffer at `pos - 1`.
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



impl PrintyPlease for MsgBuf {
	/// Print to STDOUT.
	fn fyi_print(&self) {
		use std::io::Write;
		std::io::stdout().write_all(&self.buf).unwrap();
	}

	/// Print to STDOUT with trailing line.
	fn fyi_println(&self) {
		use std::io::Write;
		std::io::stdout().write_all(&self.buf.iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
	}

	/// Locked/flushed print to STDOUT.
	fn fyi_print_flush(&self) {
		use std::io::Write;
		let writer = std::io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.flush().unwrap();
	}

	/// Locked/Flushed print to STDOUT with trailing line.
	fn fyi_println_flush(&self) {
		use std::io::Write;
		let writer = std::io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.buf.iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
		handle.flush().unwrap();
	}
}

impl EPrintyPlease for MsgBuf {
	/// Print to STDERR.
	fn fyi_eprint(&self) {
		use std::io::Write;
		std::io::stderr().write_all(&self.buf).unwrap();
	}

	/// Print to STDERR with trailing line.
	fn fyi_eprintln(&self) {
		use std::io::Write;
		std::io::stderr().write_all(&self.buf.iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
	}

	/// Locked/flushed print to STDERR.
	fn fyi_eprint_flush(&self) {
		use std::io::Write;
		let writer = std::io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.flush().unwrap();
	}

	/// Locked/Flushed print to STDERR with trailing line.
	fn fyi_eprintln_flush(&self) {
		use std::io::Write;
		let writer = std::io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.buf.iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
		handle.flush().unwrap();
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
		assert_eq!(&buf[1], SM1);

		// New filled, spanning part.
		buf = MsgBuf::new(SM1, &[SM1.len()]);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), SM1.len());
		assert_eq!(&buf[1], SM1);

		// New filled, 2 parts.
		buf = MsgBuf::new(LG1, &[4, 5]);
		assert_eq!(buf.len(), LG1.len());
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(&buf[1], &b"dino"[..]);
		assert_eq!(&buf[2], &b"saurs"[..]);

		// New filled, 2 parts (implied).
		buf = MsgBuf::new(LG1, &[4]);
		assert_eq!(buf.len(), LG1.len());
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(&buf[1], &b"dino"[..]);
		assert_eq!(&buf[2], &b"saurs"[..]);
	}

	#[test]
	fn t_from() {
		// From empty.
		let mut buf = <MsgBuf as From<&[u8]>>::from(&[]);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), 0);

		// From filled.
		buf = MsgBuf::from(SM1);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), SM1.len());
		assert_eq!(&buf[1], SM1);

		// From Many empty.
		buf = MsgBuf::from(&[]);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);

		// From many one.
		buf = MsgBuf::from(&[SM1]);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(&buf[1], SM1);

		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		assert_eq!(buf.len(), 18);
		assert_eq!(buf.parts_len(), 3);
		assert_eq!(&buf[1], SM1);
		assert_eq!(&buf[2], MD1);
		assert_eq!(&buf[3], LG1);
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
		buf += &[];
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 1);
		assert_eq!(buf.part_len(1), 0);

		// Remove it.
		buf.remove_part(1);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);

		// Add something.
		buf += SM1;
		buf += MD1;
		buf += LG1;
		assert_eq!(buf.len(), 18);
		assert_eq!(buf.parts_len(), 3);
		assert_eq!(&buf[1], SM1);
		assert_eq!(&buf[2], MD1);
		assert_eq!(&buf[3], LG1);

		// Try removing from each index.
		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove_part(1);
		assert_eq!(buf.len(), 15);
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(&buf[1], MD1);
		assert_eq!(&buf[2], LG1);

		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove_part(2);
		assert_eq!(buf.len(), 12);
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(&buf[1], SM1);
		assert_eq!(&buf[2], LG1);

		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove_part(3);
		assert_eq!(buf.len(), 9);
		assert_eq!(buf.parts_len(), 2);
		assert_eq!(&buf[1], SM1);
		assert_eq!(&buf[2], MD1);

		// Now try to remove all parts, from the left.
		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove_part(1);
		buf.remove_part(1);
		buf.remove_part(1);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.parts_len(), 0);

		// And again from the right.
		buf = MsgBuf::from(&[SM1, MD1, LG1]);
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
			assert_eq!(&buf[1], *b);
			assert_eq!(&buf[2], SM1);
		}

		// Test insertion into a multi-part buffer at each index.
		for i in 1..4 {
			for b in [&[], SM1].iter() {
				let mut buf = MsgBuf::from(&[SM2, MD2, LG2]);
				buf.insert_part(i, b);
				assert_eq!(buf.len(), 18 + b.len());
				assert_eq!(buf.parts_len(), 4);
				assert_eq!(&buf[i], *b);
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
			assert_eq!(&buf[1], *b);
		}

		// Test replacement at each index.
		for i in 1..4 {
			for b in [SM1, MD1, LG1].iter() {
				let mut buf = MsgBuf::from(&[SM2, MD2, LG2]);
				buf.replace_part(i, b);
				assert_eq!(buf.parts_len(), 3);
				assert_eq!(&buf[i], *b);
			}
		}

		// And real quick test an empty replacement.
		let mut buf = MsgBuf::from(&[SM2, MD2, LG2]);
		buf.replace_part(1, &[]);
		assert_eq!(buf.parts_len(), 3);
		assert_eq!(buf.len(), 15);
	}

	#[test]
	fn t_spread() {
		let buf = MsgBuf::from(&[SM2, MD2, LG2]);

		assert_eq!(buf.spread(1, 2), b"dogyellow");
		assert_eq!(buf.spread(2, 3), b"yellowarcosaurs");
		assert_eq!(buf.spread(1, 3), b"dogyellowarcosaurs");
	}

	#[test]
	fn t_deref() {
		const SM2: &[u8] = b"dog";
		const MD2: &[u8] = b"yellow";
		const LG2: &[u8] = b"arcosaurs";
		let buf = MsgBuf::from(&[SM2, MD2, LG2]);

		assert_eq!(buf.deref(), b"dogyellowarcosaurs");
	}
}
