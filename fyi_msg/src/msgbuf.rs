/*!
# FYI Msg: Buffered Message
*/

#![allow(missing_debug_implementations)]

use std::{
	fmt,
	hash::{
		Hash,
		Hasher,
	},
	io::{
		self,
		Write,
	},
	iter::ExactSizeIterator,
	ops::{
		self,
		AddAssign,
		Deref,
		DerefMut,
		Index,
		IndexMut,
		Range,
	},
	ptr,
};



/// Idx holding size information.
const P_IDX_COUNT: usize = 15;

/// Min idx for user partition.
const P_IDX_UMIN: usize = 1;

/// Max idx for user partition.
const P_IDX_UMAX: usize = 14;


/// Traits for Vec<u8>.
///
/// It is simply easier to handle some bits directly!
pub trait MsgBufVec {
	/// Grow the Buffer at `idx` by `adj`.
	fn buf_grow(&mut self, idx: usize, adj: usize);
	/// Insert Bytes at Position `idx`.
	fn buf_insert(&mut self, idx: usize, buf: &[u8]);
	/// Replace a Range of Bytes.
	fn buf_replace(&mut self, range: Range<usize>, buf: &[u8]);
	/// Remove a Range from the Buffer.
	fn buf_snip(&mut self, range: Range<usize>);
}

impl MsgBufVec for Vec<u8> {
	/// Grow the Buffer at `idx` by `adj`.
	///
	/// This functions like `resize()`, but from the middle. New entries are
	/// initialized with `0`.
	///
	/// Panics if `idx` is greater than the length (but it can *be* the length,
	/// in which case the bytes are just tacked onto the end).
	fn buf_grow(&mut self, idx: usize, adj: usize) {
		assert!(idx <= self.len());

		// Are we just adding to the end?
		let old_len: usize = self.len();
		if idx == old_len {
			self.resize(old_len + adj, 0);
		}
		// Shift bits over to make room.
		else {
			// Make sure we have the room.
			self.reserve(adj);

			unsafe {
				ptr::copy(
					self.as_ptr().add(idx),
					self.as_mut_ptr().add(idx + adj),
					old_len - idx
				);

				self.set_len(old_len + adj);
			}
		}
	}

	#[allow(clippy::comparison_chain)] // We're only matching 2 of 3 cases.
	/// Insert Bytes at Position `idx`.
	///
	/// This functions like `extend_from_slice()`, but from the middle.
	///
	/// Panics if `idx` is out of range.
	fn buf_insert(&mut self, idx: usize, other: &[u8]) {
		assert!(idx < self.len());

		// Is there anything to even add?
		let other_len: usize = other.len();
		if 1 == other_len {
			self.insert(idx, other[0]);
		}
		else if 1 < other_len {
			self.buf_grow(idx, other_len);

			unsafe {
				// Copy the new bits in.
				ptr::copy_nonoverlapping(
					other.as_ptr(),
					self.as_mut_ptr().add(idx),
					other_len
				);
			}
		}
	}

	#[allow(clippy::comparison_chain)] // We're only matching 2 of 3 cases.
	#[allow(clippy::len_zero)] // is_empty() is unstable.
	/// Replace a Range of Bytes.
	///
	/// Perform a ranged replace, copying `other` into that position. The range
	/// can be bigger or smaller than `other`; elements will be moved left or
	/// right as needed.
	///
	/// Panics if the range falls outside `0..self.len()`.
	fn buf_replace(&mut self, range: Range<usize>, other: &[u8]) {
		assert!(range.end <= self.len());

		// Note: unsure if this needs a temporary variable, but hopefully the
		// compiler will remove it if not.
		let range_len: usize = range.len();

		// Requires expansion?
		if range_len < other.len() {
			self.buf_grow(range.end, other.len() - range_len);
		}
		// Requires a snip?
		else if other.len() < range_len {
			self.buf_snip(range.start + other.len()..range.end);
			if other.is_empty() {
				return;
			}
		}

		unsafe {
			// Copy the new bits in.
			ptr::copy_nonoverlapping(
				other.as_ptr(),
				self.as_mut_ptr().add(range.start),
				other.len()
			);
		}
	}

	/// Remove a Range from the Buffer.
	///
	/// This removes bytes within the range, shifting anything after left to
	/// fill the gaps.
	///
	/// Panics if the range falls outside `0..self.len()`.
	fn buf_snip(&mut self, range: Range<usize>) {
		assert!(range.end <= self.len());

		// Coming from the end?
		if range.end == self.len() {
			self.truncate(range.start);
		}
		// Shift bytes over, then shrink.
		else {
			let old_len: usize = self.len();
			unsafe {
				ptr::copy(
					self.as_ptr().add(range.end),
					self.as_mut_ptr().add(range.start),
					old_len - range.end
				);

				self.set_len(old_len - range.len());
			}
		}
	}
}



#[derive(Debug, Clone)]
/// Message Buffer!
pub struct MsgBuf {
	/// The full string buffer.
	buf: Vec<u8>,

	/// The partition table.
	///
	/// The first `[0]` is always zero, marking the range start. Slices `[1]`
	/// through `[14]` mark the range ends for each used part. The last slice
	/// `[15]` keeps track of the number of parts actually in use.
	p: [usize; 16],
}

impl AddAssign<&[u8]> for MsgBuf {
	fn add_assign(&mut self, other: &[u8]) {
		self.add(other);
	}
}

impl Default for MsgBuf {
	fn default() -> Self {
		Self {
			buf: Vec::with_capacity(1024),
			p: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
		}
	}
}

impl Deref for MsgBuf {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.buf[..]
	}
}

impl DerefMut for MsgBuf {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.buf[..]
	}
}

impl fmt::Display for MsgBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl Eq for MsgBuf {}

impl From<&str> for MsgBuf {
	fn from(raw: &str) -> Self {
		Self {
			p: [0, raw.len(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
			buf: raw.as_bytes().to_vec(),
		}
	}
}

impl From<String> for MsgBuf {
	fn from(raw: String) -> Self {
		<Self as From<&str>>::from(&raw)
	}
}

impl From<&[u8]> for MsgBuf {
	fn from(raw: &[u8]) -> Self {
		Self {
			p: [0, raw.len(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
			buf: raw.to_vec(),
		}
	}
}

impl From<Vec<u8>> for MsgBuf {
	fn from(raw: Vec<u8>) -> Self {
		Self {
			p: [0, raw.len(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
			buf: raw,
		}
	}
}

impl From<&[&[u8]]> for MsgBuf {
	fn from(raw: &[&[u8]]) -> Self {
		let mut out = Self::default();
		let len: usize = raw.len().min(P_IDX_UMAX);
		let ptr = out.p.as_mut_ptr();

		unsafe {
			// Set the count.
			ptr.add(P_IDX_COUNT).write(len);

			let mut idx: usize = 0;
			while idx < len {
				let len2: usize = raw[idx].len();
				if 0 == len2 {
					ptr.add(idx + 1).copy_from_nonoverlapping(ptr.add(idx), 1);
				}
				else {
					out.buf.extend_from_slice(raw[idx]);
					ptr.add(idx + 1).write(*ptr.add(idx) + len2);
				}

				idx += 1;
			}
		}

		out
	}
}

impl From<&[&[u8]; 0]> for MsgBuf {
	fn from(_: &[&[u8]; 0]) -> Self {
		Self {
			buf: Vec::with_capacity(1024),
			p: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
		}
	}
}

impl From<&[&[u8]; 1]> for MsgBuf {
	fn from(raw: &[&[u8]; 1]) -> Self {
		Self {
			p: [0, raw[0].len(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
			buf: raw[0].to_vec(),
		}
	}
}

macro_rules! from_u8s {
	($size:literal) => {
		impl From<&[&[u8]; $size]> for MsgBuf {
			fn from(raw: &[&[u8]; $size]) -> Self {
				let mut out = Self::default();
				let ptr = out.p.as_mut_ptr();

				unsafe {
					// Set the count.
					ptr.add(P_IDX_COUNT).write($size);

					let mut idx: usize = 0;
					while idx < $size {
						let len2: usize = raw[idx].len();
						if 0 == len2 {
							ptr.add(idx + 1).copy_from_nonoverlapping(ptr.add(idx), 1);
						}
						else {
							out.buf.extend_from_slice(raw[idx]);
							ptr.add(idx + 1).write(*ptr.add(idx) + len2);
						}

						idx += 1;
					}
				}

				out
			}
		}
	};
}

from_u8s!(2);
from_u8s!(3);
from_u8s!(4);
from_u8s!(5);
from_u8s!(6);
from_u8s!(7);
from_u8s!(8);
from_u8s!(9);
from_u8s!(10);
from_u8s!(11);
from_u8s!(12);
from_u8s!(13);
from_u8s!(14);

impl From<Vec<Vec<u8>>> for MsgBuf {
	fn from(raw: Vec<Vec<u8>>) -> Self {
		let mut out = Self::default();
		let len: usize = raw.len().min(P_IDX_UMAX);
		let ptr = out.p.as_mut_ptr();

		unsafe {
			// Set the count.
			ptr.add(P_IDX_COUNT).write(len);

			let mut idx: usize = 0;
			while idx < len {
				let len2: usize = raw[idx].len();
				if 0 == len2 {
					ptr.add(idx + 1).copy_from_nonoverlapping(ptr.add(idx), 1);
				}
				else {
					out.buf.extend_from_slice(&raw[idx]);
					ptr.add(idx + 1).write(*ptr.add(idx) + len2);
				}

				idx += 1;
			}
		}

		out
	}
}

impl Hash for MsgBuf {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.buf.hash(state);
		self.p_used().hash(state);
	}
}

impl Index<usize> for MsgBuf {
	type Output = [u8];

	fn index(&self, idx: usize) -> &Self::Output {
		self.get(idx)
	}
}

impl IndexMut<usize> for MsgBuf {
	fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
		self.get_mut(idx)
	}
}

impl PartialEq for MsgBuf {
	fn eq(&self, other: &Self) -> bool {
		self.buf == other.buf && self.p_used() == other.p_used()
	}
}

impl PartialEq<[u8]> for MsgBuf {
	fn eq(&self, other: &[u8]) -> bool {
		self.buf == other
	}
}

impl PartialEq<&[u8]> for MsgBuf {
	fn eq(&self, other: &&[u8]) -> bool {
		self.buf == *other
	}
}

impl PartialEq<Vec<u8>> for MsgBuf {
	fn eq(&self, other: &Vec<u8>) -> bool {
		&self.buf == other
	}
}

impl MsgBuf {
	// ------------------------------------------------------------------------
	// Shiva
	// ------------------------------------------------------------------------

	#[must_use]
	/// New
	///
	/// Create a new `MsgBuf` from a single buffer with pre-determined
	/// partition sizes. The last partition will automatically be adjusted to
	/// cover the buffer in cases where it falls short.
	///
	/// Panics if there are more parts than allowed.
	pub fn new(buf: &[u8], parts: &[usize]) -> Self {
		assert!(parts.len() <= P_IDX_UMAX);

		// Don't waste time on small shit.
		if parts.len() <= 1 {
			return Self::from(buf);
		}

		Self {
			buf: Vec::from(buf),
			p: {
				let mut p: [usize; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, parts.len()];
				let max: usize = buf.len();
				let last: usize = 0;
				let mut idx: usize = 0;

				while idx < parts.len() {
					if last == max {
						p[idx + 1] = max;
					}
					else {
						p[idx + 1] = usize::min(max, last + parts[idx]);
					}

					idx += 1;
				}

				if last != max {
					p[parts.len()] = max;
				}

				p
			}
		}
	}

	/// Reset the Whole Thing.
	///
	/// This will clear the buffer and remove all parts, without reallocating.
	pub fn reset(&mut self) {
		self.buf.clear();
		unsafe { self.p.as_mut_ptr().add(P_IDX_COUNT).write(0); }
	}

	#[must_use]
	/// Splat
	///
	/// Create a new `MsgBuf` with `num` empty parts.
	pub fn splat(num: usize) -> Self {
		Self {
			buf: Vec::with_capacity(1024),
			p: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, num.min(P_IDX_UMAX)],
		}
	}

	// ------------------------------------------------------------------------
	// State
	// ------------------------------------------------------------------------

	#[must_use]
	/// Len.
	pub const fn len(&self) -> usize {
		self.p[self.p[P_IDX_COUNT]]
	}

	#[must_use]
	/// Is Empty.
	pub const fn is_empty(&self) -> bool {
		self.p[self.p[P_IDX_COUNT]] == 0
	}

	#[must_use]
	/// Number of Partitions.
	pub const fn count(&self) -> usize {
		self.p[P_IDX_COUNT]
	}

	#[must_use]
	/// Part.
	///
	/// # Safety
	/// `idx` must be previously validated to be in range.
	pub const unsafe fn p(&self, idx: usize) -> ops::Range<usize> {
		ops::Range{
			start: self.p[idx - 1],
			end: self.p[idx],
		}
	}

	#[must_use]
	/// Partition Length.
	///
	/// # Safety
	/// `idx` must be previously validated to be in range.
	pub const unsafe fn p_len(&self, idx: usize) -> usize {
		self.p[idx] - self.p[idx - 1]
	}

	#[must_use]
	/// Partition Is Empty.
	///
	/// # Safety
	/// `idx` must be previously validated to be in range.
	pub const unsafe fn p_is_empty(&self, idx: usize) -> bool {
		self.p[idx] - self.p[idx - 1] == 0
	}

	#[must_use]
	/// Return Used Parts
	///
	/// This is mostly used for hashing purposes.
	pub fn p_used(&self) -> &[usize] {
		&self.p[0..=self.p[P_IDX_COUNT]]
	}

	#[must_use]
	/// Partition Is Used?
	pub fn p_is_used(&self, idx: usize) -> bool {
		P_IDX_UMIN <= idx && idx <= self.p[P_IDX_COUNT]
	}



	// ------------------------------------------------------------------------
	// Access
	// ------------------------------------------------------------------------

	#[must_use]
	/// As Slice.
	pub fn as_bytes(&self) -> &[u8] {
		&self.buf[..]
	}

	#[must_use]
	/// As Slice.
	pub fn as_bytes_mut(&mut self) -> &mut [u8] {
		&mut self.buf[..]
	}

	#[must_use]
	/// As String.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.buf[..]) }
	}

	#[must_use]
	/// To Vec.
	pub fn to_vec(&self) -> Vec<u8> {
		self.buf.clone()
	}

	#[must_use]
	/// Partition.
	pub fn get(&self, idx: usize) -> &[u8] {
		assert!(self.p_is_used(idx));

		&self.buf[unsafe { self.p(idx) }]
	}

	#[must_use]
	/// Partition Mut.
	pub fn get_mut(&mut self, idx: usize) -> &mut [u8] {
		assert!(self.p_is_used(idx));

		let prg = unsafe { self.p(idx) };
		&mut self.buf[prg]
	}

	#[must_use]
	/// Partition Spread.
	pub fn spread(&self, idx1: usize, idx2: usize) -> &[u8] {
		assert!(P_IDX_UMIN <= idx1 && idx1 < idx2 && idx2 <= self.count());

		&self.buf[self.p[idx1 - 1]..self.p[idx2]]
	}



	// ------------------------------------------------------------------------
	// Adding
	// ------------------------------------------------------------------------

	/// Add Part (to End).
	///
	/// Panics if the maximum number of parts has already been reached.
	pub fn add(&mut self, buf: &[u8]) {
		assert!(self.count() < P_IDX_UMAX);

		// Just increase the part count.
		if buf.is_empty() {
			let ptr = self.p.as_mut_ptr();
			unsafe {
				let count: usize = *ptr.add(P_IDX_COUNT);
				// Copy the previous point since this has no length.
				ptr.add(count + 1).copy_from_nonoverlapping(ptr.add(count), 1);
				// Update the count count.
				*ptr.add(P_IDX_COUNT) += 1;
			}
		}
		else {
			self.buf.extend_from_slice(buf);

			let ptr = self.p.as_mut_ptr();
			unsafe {
				let count: usize = *ptr.add(P_IDX_COUNT);
				// Copy the previous point since this has no length.
				ptr.add(count + 1).write(*ptr.add(count) + buf.len());
				// Update the count count.
				*ptr.add(P_IDX_COUNT) += 1;
			}
		}
	}

	/// Insert Part At `idx`
	///
	/// Insert a part and shove everything previously at or after `idx` to the
	/// right.
	///
	/// Panics if the maximum number of parts has already been reached or `idx`
	/// is out of range.
	pub fn insert(&mut self, idx: usize, buf: &[u8]) {
		assert!(self.count() < P_IDX_UMAX && self.p_is_used(idx));

		self.buf.buf_insert(self.p[idx - 1], buf);

		// Adjusting the part table takes a bit more.
		let len: usize = buf.len();
		let mut tail_idx: usize = self.count() + 1;
		let ptr = self.p.as_mut_ptr();
		unsafe {
			*ptr.add(P_IDX_COUNT) += 1;
			// Empty parts can be straight copies.
			if len == 0 {
				while tail_idx >= idx {
					ptr.add(tail_idx).copy_from_nonoverlapping(ptr.add(tail_idx - 1), 1);
					tail_idx -= 1;
				}
			}
			// Otherwise we need to write some math.
			else {
				while tail_idx >= idx {
					ptr.add(tail_idx).write(*ptr.add(tail_idx - 1) + len);
					tail_idx -= 1;
				}
			}
		}
	}



	// ------------------------------------------------------------------------
	// Editing
	// ------------------------------------------------------------------------

	/// Clear Part.
	///
	/// Like `remove()`, except the part is kept at zero-length.
	///
	/// Panics if `idx` is out of range.
	pub fn clear(&mut self, idx: usize) {
		assert!(self.p_is_used(idx));

		let len: usize = unsafe { self.p_len(idx) };
		if 0 != len {
			self.buf.buf_snip(unsafe { self.p(idx) });
			unsafe { self.p_shrink(idx, len); }
		}
	}

	/// Extend Part.
	///
	/// This adds data onto part `idx`.
	///
	/// Panics if `idx` is out of range.
	pub fn extend(&mut self, idx: usize, buf: &[u8]) {
		assert!(self.p_is_used(idx));

		if ! buf.is_empty() {
			self.buf.buf_insert(self.p[idx], buf);
			unsafe { self.p_grow(idx, buf.len()); }
		}
	}

	/// Flatten Parts.
	pub fn flatten(&mut self) {
		let ptr = self.p.as_mut_ptr();
		unsafe {
			if 1 < *ptr.add(P_IDX_COUNT) {
				ptr.add(1).copy_from_nonoverlapping(ptr.add(P_IDX_COUNT), 1);
				ptr.add(P_IDX_COUNT).write(1);
			}
		}
	}

	/// Remove Part.
	///
	/// Panics if `idx` is out of range.
	pub fn remove(&mut self, mut idx: usize) {
		assert!(self.p_is_used(idx));

		let len: usize = unsafe { self.p_len(idx) };

		// If the partition is empty, we can just shift everything down.
		if 0 == len {
			let ptr = self.p.as_mut_ptr();
			unsafe {
				let count: usize = *ptr.add(P_IDX_COUNT);
				while idx < count {
					ptr.add(idx).copy_from_nonoverlapping(ptr.add(idx + 1), 1);
					idx += 1;
				}

				*ptr.add(P_IDX_COUNT) -= 1;
			}
		}
		// Otherwise we need to cut it from the buffer and rewrite all other
		// entries.
		else {
			self.buf.buf_snip(unsafe { self.p(idx) });

			let ptr = self.p.as_mut_ptr();
			unsafe {
				let count: usize = *ptr.add(P_IDX_COUNT);
				while idx < count {
					ptr.add(idx).write(*ptr.add(idx + 1) - len);
					idx += 1;
				}

				*ptr.add(P_IDX_COUNT) -= 1;
			}
		}
	}

	#[allow(clippy::comparison_chain)] // We're only matching 2 of 3 cases.
	/// Replace Part.
	///
	/// Replace a part with new data, which can be any size.
	///
	/// Panics if `idx` is out of range.
	pub fn replace(&mut self, idx: usize, buf: &[u8]) {
		assert!(self.p_is_used(idx));

		// If the buffer is empty, just run `clear()` instead.
		let new_len: usize = buf.len();
		if 0 == new_len {
			return self.clear(idx);
		}

		// Update the buffer.
		self.buf.buf_replace(unsafe { self.p(idx) }, buf);

		// We have to adjust the partition table if the length changed.
		let old_len: usize = unsafe { self.p_len(idx) };
		if old_len < new_len {
			unsafe { self.p_grow(idx, new_len - old_len); }
		}
		else if new_len < old_len {
			unsafe { self.p_shrink(idx, old_len - new_len); }
		}
	}

	/// Truncate.
	///
	/// Reduce the length of part `idx` to `len`. This does nothing if the
	/// length is already at or lower than `len`.
	///
	/// Panics if `idx` is out of range.
	pub fn truncate(&mut self, idx: usize, len: usize) {
		assert!(self.p_is_used(idx));

		let end: usize = self.p[idx - 1] + len;
		if end < self.p[idx] {
			self.buf.buf_snip(self.p[idx - 1]..end);
			unsafe { self.p_shrink(idx, self.p[idx] - end); }
		}
	}



	// ------------------------------------------------------------------------
	// Printing
	// ------------------------------------------------------------------------

	/// Stdout Print.
	pub fn print(&self) {
		let writer = std::io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.flush().unwrap();
	}

	/// Stdout Print w/ Line.
	pub fn println(&self) {
		let writer = std::io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.write_all(&[10]).unwrap();
		handle.flush().unwrap();
	}

	/// Stderr Print.
	pub fn eprint(&self) {
		let writer = std::io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.flush().unwrap();
	}

	/// Stderr Print w/ Line.
	pub fn eprintln(&self) {
		let writer = std::io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.write_all(&[10]).unwrap();
		handle.flush().unwrap();
	}

	/// Sink Print.
	pub fn sink(&self) {
		let mut handle = io::sink();
		handle.write_all(&self.buf).unwrap();
		handle.flush().unwrap();
	}

	/// Stderr Print w/ Line.
	pub fn sinkln(&self) {
		let mut handle = io::sink();
		handle.write_all(&self.buf).unwrap();
		handle.write_all(&[10]).unwrap();
		handle.flush().unwrap();
	}



	// ------------------------------------------------------------------------
	// Internal Helpers
	// ------------------------------------------------------------------------

	/// Grow & Shift Partitions
	///
	/// The "length" of `idx` is increased by `len`, then all subsequent
	/// entries are shifted up accordingly (while maintaining their original
	/// "lengths".)
	///
	/// # Safety
	/// The index must have already been verified to exist.
	unsafe fn p_grow(&mut self, mut idx: usize, len: usize) {
		let ptr = self.p.as_mut_ptr();
		let count: usize = *ptr.add(P_IDX_COUNT);
		while idx <= count {
			*ptr.add(idx) += len;
			idx += 1;
		}
	}

	/// Shrink & Shift Partitions
	///
	/// The "length" of `idx` is decreased by `len`, then all subsequent
	/// entries are shifted down accordingly (while maintaining their original
	/// "lengths".)
	///
	/// Panics if `len` is greater than `self.p[idx]`.
	///
	/// # Safety
	/// The index must have already been verified to exist.
	unsafe fn p_shrink(&mut self, mut idx: usize, len: usize) {
		assert!(self.p[idx] >= len);

		let ptr = self.p.as_mut_ptr();
		let count: usize = *ptr.add(P_IDX_COUNT);
		while idx <= count {
			*ptr.add(idx) -= len;
			idx += 1;
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
		assert_eq!(buf.count(), 1);
		unsafe { assert_eq!(buf.p_len(1), 0); }

		// New filled, no parts.
		buf = MsgBuf::new(SM1, &[]);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.count(), 1);
		unsafe { assert_eq!(buf.p_len(1), SM1.len()); }
		assert_eq!(buf.get(1), SM1);
		assert_eq!(&buf[1], SM1);

		// New filled, spanning part.
		buf = MsgBuf::new(SM1, &[SM1.len()]);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.count(), 1);
		unsafe { assert_eq!(buf.p_len(1), SM1.len()); }
		assert_eq!(buf.get(1), SM1);
		assert_eq!(&buf[1], SM1);

		// New filled, 2 parts.
		buf = MsgBuf::new(LG1, &[4, 5]);
		assert_eq!(buf.len(), LG1.len());
		assert_eq!(buf.count(), 2);
		assert_eq!(buf.get(1), &b"dino"[..]);
		assert_eq!(buf.get(2), &b"saurs"[..]);
		assert_eq!(&buf[1], &b"dino"[..]);
		assert_eq!(&buf[2], &b"saurs"[..]);

		// New filled, 2 parts (implied).
		buf = MsgBuf::new(LG1, &[4]);
		assert_eq!(buf.len(), LG1.len());
		assert_eq!(buf.count(), 1);
		assert_eq!(buf.get(1), &b"dinosaurs"[..]);
		assert_eq!(&buf[1], &b"dinosaurs"[..]);
	}

	#[test]
	fn t_from() {
		// From empty.
		let mut buf = <MsgBuf as From<&[u8]>>::from(&[]);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 1);
		unsafe { assert_eq!(buf.p_len(1), 0); }

		// From filled.
		buf = MsgBuf::from(SM1);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.count(), 1);
		unsafe { assert_eq!(buf.p_len(1), SM1.len()); }
		assert_eq!(&buf[1], SM1);

		// From Many empty.
		buf = MsgBuf::from(&[]);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 1);

		// From many one.
		buf = MsgBuf::from(&[SM1]);
		assert_eq!(buf.len(), SM1.len());
		assert_eq!(buf.count(), 1);
		assert_eq!(&buf[1], SM1);

		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		assert_eq!(buf.len(), 18);
		assert_eq!(buf.count(), 3);
		assert_eq!(&buf[1], SM1);
		assert_eq!(&buf[2], MD1);
		assert_eq!(&buf[3], LG1);
	}

	#[test]
	fn t_reset_splat() {
		// Clearing empty.
		let mut buf = MsgBuf::default();
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 0);

		buf.reset();
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 0);

		// Clear filled.
		buf = MsgBuf::from(SM1);
		buf.reset();
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 0);

		// Check a splat.
		buf = MsgBuf::splat(3);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 3);
		unsafe {
			assert_eq!(buf.p_len(1), 0);
			assert_eq!(buf.p_len(2), 0);
			assert_eq!(buf.p_len(3), 0);
		}
	}

	#[test]
	fn t_add_remove() {
		let mut buf = MsgBuf::default();

		// Add empty.
		buf += &[];
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 1);
		unsafe { assert_eq!(buf.p_len(1), 0); }

		// Remove it.
		buf.remove(1);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 0);

		// Add something.
		buf += SM1;
		buf += MD1;
		buf += LG1;
		assert_eq!(buf.len(), 18);
		assert_eq!(buf.count(), 3);
		assert_eq!(&buf[1], SM1);
		assert_eq!(&buf[2], MD1);
		assert_eq!(&buf[3], LG1);

		// Try removing from each index.
		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove(1);
		assert_eq!(buf.len(), 15);
		assert_eq!(buf.count(), 2);
		assert_eq!(&buf[1], MD1);
		assert_eq!(&buf[2], LG1);

		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove(2);
		assert_eq!(buf.len(), 12);
		assert_eq!(buf.count(), 2);
		assert_eq!(&buf[1], SM1);
		assert_eq!(&buf[2], LG1);

		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove(3);
		assert_eq!(buf.len(), 9);
		assert_eq!(buf.count(), 2);
		assert_eq!(&buf[1], SM1);
		assert_eq!(&buf[2], MD1);

		// Now try to remove all parts, from the left.
		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove(1);
		buf.remove(1);
		buf.remove(1);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 0);

		// And again from the right.
		buf = MsgBuf::from(&[SM1, MD1, LG1]);
		buf.remove(3);
		buf.remove(2);
		buf.remove(1);
		assert_eq!(buf.len(), 0);
		assert_eq!(buf.count(), 0);
	}

	#[test]
	fn t_insert() {
		// Test insertion into a spread buffer.
		for b in [&[], SM1].iter() {
			let mut buf = MsgBuf::from(SM1);
			buf.insert(1, b);
			assert_eq!(buf.len(), SM1.len() + b.len());
			assert_eq!(buf.count(), 2);
			assert_eq!(&buf[1], *b);
			assert_eq!(&buf[2], SM1);
		}

		// Test insertion into a multi-part buffer at each index.
		for i in 1..4 {
			for b in [&[], SM1].iter() {
				let mut buf = MsgBuf::from(&[SM2, MD2, LG2]);
				buf.insert(i, b);
				assert_eq!(buf.len(), 18 + b.len());
				assert_eq!(buf.count(), 4);
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
			buf.replace(1, b);
			assert_eq!(buf.len(), b.len());
			assert_eq!(buf.count(), 1);
			assert_eq!(&buf[1], *b);
		}

		// Test replacement at each index.
		for i in 1..4 {
			for b in [SM1, MD1, LG1].iter() {
				let mut buf = MsgBuf::from(&[SM2, MD2, LG2]);
				buf.replace(i, b);
				assert_eq!(buf.count(), 3);
				assert_eq!(&buf[i], *b);
			}
		}

		// And real quick test an empty replacement.
		let mut buf = MsgBuf::from(&[SM2, MD2, LG2]);
		buf.replace(1, &[]);
		assert_eq!(buf.count(), 3);
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

	#[test]
	/// MsgBufVec: Test Insertion
	///
	/// This covers `buf_grow()` too.
	fn t_eq() {
		const SM2: &[u8] = b"dog";
		const MD2: &[u8] = b"yellow";
		const LG2: &[u8] = b"arcosaurs";

		let buf = MsgBuf::from(&[SM2, MD2, LG2]);
		let buf2 = MsgBuf::from(&[SM2, MD2, LG2]);
		assert_eq!(buf, buf2);
		assert_eq!(buf, buf2.deref());

		let buf2 = MsgBuf::from(&[SM2, MD2]);
		assert!(buf != buf2);
		assert!(buf != buf2.deref());
	}

	#[test]
	/// MsgBufVec: Test Insertion
	///
	/// This covers `buf_grow()` too.
	fn mbv_buf_insert() {
		// Add a middle name.
		let mut owned: Vec<u8> = Vec::from(&b"John Jingleheimer Schmidt"[..]);
		owned.buf_insert(4, &b" Jacob"[..]);
		assert_eq!(owned, &b"John Jacob Jingleheimer Schmidt"[..]);
	}

	#[test]
	/// MsgBufVec: Test Insertion
	///
	/// This covers `buf_grow()` and `buf_snip()` too.
	fn mbv_buf_replace() {
		// Replace Shorter (end).
		let mut owned: Vec<u8> = Vec::from(&b"John Jacob Jingleheimer Schmidt"[..]);
		owned.buf_replace(11..31, &b"Astor"[..]);
		assert_eq!(owned, &b"John Jacob Astor"[..]);

		// Replace Bigger (end).
		owned.buf_replace(11..16, &b"Jingleheimer Schmidt"[..]);
		assert_eq!(owned, &b"John Jacob Jingleheimer Schmidt"[..]);

		// Replace Shorter (middle).
		owned.buf_replace(5..10, &b"Dan"[..]);
		assert_eq!(owned, &b"John Dan Jingleheimer Schmidt"[..]);

		// Replace Bigger (middle).
		owned.buf_replace(5..8, &b"Jacob"[..]);
		assert_eq!(owned, &b"John Jacob Jingleheimer Schmidt"[..]);

		// Replace the whole thing.
		owned.buf_replace(0..31, &b"Something New!"[..]);
		assert_eq!(owned, &b"Something New!"[..]);

		// And replace with nothing.
		owned.buf_replace(9..14, &[]);
		assert_eq!(owned, &b"Something"[..]);

		// Totally nothing!
		owned.buf_replace(0..9, &[]);
		assert!(owned.is_empty());
	}
}


