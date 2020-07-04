/*!
# FYI Msg: Message Buffer

The `MsgBuf` struct combines a `Vec<u8>` with a small, fixed, 14-part
"partitioning" table. This allows the complete String to be stored in a single
memory space, while allowing fairly straightforward logical manipulation of
its inner slices.

This storage model is a sort of compromise between being able to quickly and
repeatedly access the whole of the thing, while also having reasonable logical
access to view or change individual pieces of it. (For use cases where the
parts matter more than the whole, just use `HashMap` or similar!)

## Examples

The `Msg` struct is really just a wrapper around `MsgBuf` with slots designated
for the various ANSI sequences and message components like prefix and text. A
simpler version could bypass the need for a custom struct altogether.

```no_run
// We want three separate parts: a prefix, a colon/space, and some text.
// Let's start with something that creates: "Error: This Broke!"
let mut msg = MsgBuf::from(&[
    &b"Error"[..],   // msg[1]
    &b": "[..],      // msg[2]
    &b"This Broke!", // msg[3]
]);

// You can get the whole thing as a string if you wanted:
let txt = msg.as_str();

// You could change a part:
msg.replace(3, "This caught fire!");

// You can print it without macros:
msg.println();

// You can get a range of parts:
assert_eq!(msg.spread(1..=2), b"Error: ");
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
	str::FromStr,
};



/// Idx holding size information.
const P_IDX_COUNT: usize = 15;

/// Min idx for user partition.
const P_IDX_UMIN: usize = 1;

/// Max idx for user partition.
const P_IDX_UMAX: usize = 14;


/// Methods for Vec<u8>.
///
/// The `Vec` struct actually has most of what we need, but life is easier with
/// ways to grow or shrink from the middle, as well as insert and/or replace
/// entries (plural).
///
/// Adding this as a trait helps keep it out of the way.
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
	///
	/// # Examples
	///
	/// ```no_run
	/// let mut v = vec![0, 1, 2, 3, 4, 5];
	/// v.buf_grow(1, 3);
	/// assert_eq!(v, vec![0, 0, 0, 0, 1, 2, 3, 4, 5]);
	/// ```
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
	///
	/// # Examples
	///
	/// ```no_run
	/// let mut v = vec![0, 1, 2, 3, 4, 5];
	/// v.buf_insert(1, &[6, 7, 8]);
	/// assert_eq!(v, vec![0, 6, 7, 8, 1, 2, 3, 4, 5]);
	/// ```
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
	///
	/// # Examples
	///
	/// ```no_run
	/// let mut v = vec![0, 1, 2, 3, 4, 5];
	/// v.buf_replace(1..3, &[6, 7, 8]);
	/// assert_eq!(v, vec![0, 6, 7, 8, 3, 4, 5]);
	/// ```
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
	///
	/// # Examples
	///
	/// ```no_run
	/// let mut v = vec![0, 1, 2, 3, 4, 5];
	/// v.buf_snip(1..3);
	/// assert_eq!(v, vec![0, 3, 4, 5]);
	/// ```
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
	/// The exclusive "end" points of each part are stored in sequence, such
	/// that a range could be constructed via `idx-1..idx`.
	///
	/// The first entry `[0]` is always `0`, removing the need for a bounds
	/// check on `idx-1`.
	///
	/// The last entry `[15]` holds the number of active partitions rather
	/// than an end point. This moots the need for an additional variable at
	/// the expense of the total number of slots available for users (14).
	p: [usize; 16],
}

impl AddAssign<&[u8]> for MsgBuf {
	fn add_assign(&mut self, other: &[u8]) {
		self.add(other);
	}
}

impl Default for MsgBuf {
	/// Default.
	///
	/// The default buffer is empty, but initialized with a capacity of `1024`
	/// for the eventual message.
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
	/// From Str.
	///
	/// Create a buffer from the string with a single partition covering all of
	/// it.
	fn from(raw: &str) -> Self {
		Self {
			p: [0, raw.len(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
			buf: raw.as_bytes().to_vec(),
		}
	}
}

impl From<String> for MsgBuf {
	/// From Str.
	///
	/// Create a buffer from the string with a single partition covering all of
	/// it.
	fn from(raw: String) -> Self {
		<Self as From<&str>>::from(&raw)
	}
}

impl From<&[u8]> for MsgBuf {
	/// From Bytes.
	///
	/// Create a buffer from a byte string with a single partition covering all
	/// of it.
	fn from(raw: &[u8]) -> Self {
		Self {
			p: [0, raw.len(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
			buf: raw.to_vec(),
		}
	}
}

impl From<Vec<u8>> for MsgBuf {
	/// From Vec.
	///
	/// Create a buffer from a `Vec<u8>` with a single partition covering all
	/// of it.
	fn from(raw: Vec<u8>) -> Self {
		Self {
			p: [0, raw.len(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
			buf: raw,
		}
	}
}

impl From<&[&[u8]]> for MsgBuf {
	/// From Slice of Slices.
	///
	/// Create a buffer from multiple byte strings, creating a separate part
	/// for each.
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
	/// From Vec of Vecs.
	///
	/// Create a buffer from multiple `Vec<u8>`s, creating a separate part for
	/// each.
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

impl FromStr for MsgBuf {
	type Err = std::num::ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from(s))
	}
}

impl Hash for MsgBuf {
	/// Hash.
	///
	/// The `Hash` trait is manually implemented to ensure that only the used
	/// portions of the partition table — which don't get rezeroed on clear,
	/// etc. — factor into the comparison.
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
	/// Self Compare.
	///
	/// For self comparisons, make sure the buffer and partitions match up. For
	/// all other implementations, only the buffer contents matter.
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

impl PartialEq<str> for MsgBuf {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl PartialEq<&str> for MsgBuf {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
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
	/// cover the buffer (i.e. end with `buf.len()`) in cases where it falls
	/// short.
	///
	/// Panics if there are more parts than allowed.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// assert_eq!(msg.get(1), b"Hello");
	/// assert_eq!(msg.get(2), b" ");
	/// assert_eq!(msg.get(3), b"World");
	/// ```
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
	/// Not to be confused with `clear()`, which clears a single part.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// assert_eq!(*msg, b"Hello World");
	///
	/// msg.reset();
	/// assert_eq!(*msg, b"");
	/// ```
	pub fn reset(&mut self) {
		self.buf.clear();
		unsafe { self.p.as_mut_ptr().add(P_IDX_COUNT).write(0); }
	}

	#[must_use]
	/// Splat
	///
	/// Create a new `MsgBuf` with `num` empty parts.
	///
	/// As with all `MsgBuf` incarnations, there may only be up to 14
	/// individual parts; any excess will be ignored.
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
	/// Buffer Length.
	pub const fn len(&self) -> usize {
		self.p[self.p[P_IDX_COUNT]]
	}

	#[must_use]
	/// Buffer Is Empty.
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
	///
	/// This method assumes `idx` is in range; meaningless and possibly panic-
	/// worthy results can be returned if you screw this up.
	///
	/// When in doubt, use `get()` instead and determine a range from that.
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
	///
	/// This method assumes `idx` is in range; meaningless and possibly panic-
	/// worthy results can be returned if you screw this up.
	///
	/// When in doubt, use `get()` instead and determine a length from that.
	pub const unsafe fn p_len(&self, idx: usize) -> usize {
		self.p[idx] - self.p[idx - 1]
	}

	#[must_use]
	/// Partition Is Empty.
	///
	/// # Safety
	///
	/// This method assumes `idx` is in range; meaningless and possibly panic-
	/// worthy results can be returned if you screw this up.
	///
	/// When in doubt, use `get()` instead and determine emptiness from that.
	pub const unsafe fn p_is_empty(&self, idx: usize) -> bool {
		self.p[idx] - self.p[idx - 1] == 0
	}

	#[must_use]
	/// Return Used Parts
	///
	/// This returns only the used slice of the partitioning table.
	///
	/// Note: This is used internally for determining hash equivalence.
	pub fn p_used(&self) -> &[usize] {
		&self.p[0..=self.p[P_IDX_COUNT]]
	}

	#[must_use]
	/// Partition Is Used?
	///
	/// Answer whether or not a partition has been initialized. This serves as
	/// an assertion test for a lot of this struct's methods.
	///
	/// A partition is in "use" if it is at least "1" and less than or equal to
	/// the total count (since user indexes begin at "1", there's no +1 offset
	/// for the count length; it is itself an index).
	pub fn p_is_used(&self, idx: usize) -> bool {
		P_IDX_UMIN <= idx && idx <= self.p[P_IDX_COUNT]
	}



	// ------------------------------------------------------------------------
	// Access
	// ------------------------------------------------------------------------

	#[must_use]
	/// As Slice.
	///
	/// Return an `&[u8]` slice of the entire buffer.
	pub fn as_bytes(&self) -> &[u8] {
		&self.buf[..]
	}

	#[must_use]
	/// As Slice.
	///
	/// Return a mutable `&[u8]` slice of the entire buffer. If making changes
	/// to the data, be sure you do so within the confines of the partitions
	/// you've established or methods like `replace()` and `insert()` and `get()`
	/// will be confused.
	pub fn as_bytes_mut(&mut self) -> &mut [u8] {
		&mut self.buf[..]
	}

	#[must_use]
	/// As String.
	///
	/// Return the buffer as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.buf[..]) }
	}

	#[must_use]
	/// To Vec.
	///
	/// Return an owned clone of the inner buffer as a Vec.
	pub fn to_vec(&self) -> Vec<u8> {
		self.buf.clone()
	}

	#[must_use]
	/// Get Partition.
	///
	/// Return a slice of the buffer corresponding to the given part.
	///
	/// Remember, partitions start at `1`, not `0`.
	///
	/// Panics if `idx` is out of range.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// assert_eq!(msg[1], msg.get(1));
	/// assert_eq!(msg[1], b"Hello");
	/// ```
	pub fn get(&self, idx: usize) -> &[u8] {
		assert!(self.p_is_used(idx));

		&self.buf[unsafe { self.p(idx) }]
	}

	#[must_use]
	/// Get Mutable Partition.
	///
	/// Return a mutable slice of the buffer corresponding to the given part.
	///
	/// Remember, partitions start at `1`, not `0`.
	///
	/// Panics if `idx` is out of range.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// let mut p1 = msg.get_mut(1);
	/// p1[0] = b'J';
	///
	/// assert_eq!(*msg, b"Jello World");
	/// ```
	pub fn get_mut(&mut self, idx: usize) -> &mut [u8] {
		assert!(self.p_is_used(idx));

		let prg = unsafe { self.p(idx) };
		&mut self.buf[prg]
	}

	#[must_use]
	/// Partition Spread.
	///
	/// Return a slice of the buffer spanning multiple parts, from the
	/// beginning of `idx1` to the ending of `idx2`.
	///
	/// Remember, partitions start at `1`, not `0`.
	///
	/// Panics if the indexes are out of range or out of order.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// assert_eq!(msg.spread(1, 2), b"Hello ");
	/// ```
	pub fn spread(&self, idx1: usize, idx2: usize) -> &[u8] {
		assert!(P_IDX_UMIN <= idx1 && idx1 < idx2 && idx2 <= self.count());

		&self.buf[self.p[idx1 - 1]..self.p[idx2]]
	}



	// ------------------------------------------------------------------------
	// Adding
	// ------------------------------------------------------------------------

	/// Add Part (to End).
	///
	/// This is a `push()` method, basically, extending the buffer and adding a
	/// new partition to cover it.
	///
	/// Panics if the maximum number of parts has already been reached.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// msg += &b" "[..];  // Add with an operator.
	/// msg.add(b"Hello"); // Add procedurally.
	///
	/// assert_eq!(msg.count(), 5);
	/// assert_eq!(*msg, b"Hello World Hello");
	/// ```
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
	/// Insert a part at `idx`, shifting the old `idx` and everything after it
	/// to the right to make room.
	///
	/// Panics if the maximum number of parts has already been reached or `idx`
	/// is out of range.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// msg.insert(2, b" ");
	/// msg.insert(3, b"Sweet");
	///
	/// assert_eq!(msg.count(), 5);
	/// assert_eq!(*msg, b"Hello Sweet World");
	/// ```
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
	/// Like `remove()`, except the part is preserved, albeit without length.
	///
	/// Panics if `idx` is out of range.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// msg.clear(2);
	/// assert_eq!(*msg, b"HelloWorld");
	/// ```
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
	/// This adds data onto the existing part at `idx`, growing the partition
	/// length accordingly.
	///
	/// Panics if `idx` is out of range.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// msg.extend(3, b"lets");
	/// assert_eq!(*msg, b"Hello Worldlets");
	/// ```
	pub fn extend(&mut self, idx: usize, buf: &[u8]) {
		assert!(self.p_is_used(idx));

		if ! buf.is_empty() {
			self.buf.buf_insert(self.p[idx], buf);
			unsafe { self.p_grow(idx, buf.len()); }
		}
	}

	/// Flatten Parts.
	///
	/// This method reduces the partition table to a single, all-spanning
	/// entry. (The buffer remains unchanged.)
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
	/// Remove a part from the buffer and the partitioning table.
	///
	/// Panics if `idx` is out of range.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// msg.remove(2);
	/// msg.remove(2);
	///
	/// assert_eq!(msg.count(), 1);
	/// assert_eq!(*msg, b"Hello");
	/// ```
	pub fn remove(&mut self, mut idx: usize) {
		assert!(self.p_is_used(idx));

		let len: usize = unsafe { self.p_len(idx) };
		let count: usize = self.count();

		// If this is the last index, we can cheat a bit.
		if idx == count {
			if len > 0 {
				self.buf.truncate(self.len() - len);
			}
			unsafe { *self.p.as_mut_ptr().add(P_IDX_COUNT) -= 1; }
		}
		// If the partition is empty, we can just shift everything down.
		else if 0 == len {
			let ptr = self.p.as_mut_ptr();
			unsafe {
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
	/// Replace a part with new data. The new data can be any size; it needn't
	/// match the length of the original.
	///
	/// Panics if `idx` is out of range.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// msg.replace(1, b"Good-bye");
	/// msg.replace(3, b"Dolly");
	/// assert_eq!(*msg, b"Good-bye Dolly");
	/// ```
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
	/// Reduce the length of part `idx` to `len`, similar to the same method
	/// on `Vec`, except it can work from the middle. This does nothing if the
	/// length is already at or lower than `len`.
	///
	/// Panics if `idx` is out of range.
	///
	/// # Examples
	///
	/// ```
	/// use fyi_msg::MsgBuf;
	///
	/// let mut msg = MsgBuf::new(b"Hello World", &[5, 1, 5]);
	/// msg.truncate(1, 4);
	/// assert_eq!(*msg, b"Hell World"); // Sounds nice!
	/// ```
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

	/// Print to `STDOUT`.
	///
	/// This is equivalent to manually writing the bytes to a locked
	/// `io::stdout()` and flushing the handle.
	pub fn print(&self) {
		let writer = std::io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.flush().unwrap();
	}

	/// Print to `STDOUT` (w/ line break)
	///
	/// Same as `print()`, except a trailing line break `10_u8` is appended,
	/// like using the `println!()` macro, but faster.
	pub fn println(&self) {
		let writer = std::io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.write_all(&[10]).unwrap();
		handle.flush().unwrap();
	}

	/// Print to `STDERR`.
	///
	/// This is equivalent to manually writing the bytes to a locked
	/// `io::stderr()` and flushing the handle.
	pub fn eprint(&self) {
		let writer = std::io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.flush().unwrap();
	}

	/// Print to `STDERR` (w/ line break)
	///
	/// Same as `eprint()`, except a trailing line break `10_u8` is appended,
	/// like using the `eprintln!()` macro, but faster.
	pub fn eprintln(&self) {
		let writer = std::io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.buf).unwrap();
		handle.write_all(&[10]).unwrap();
		handle.flush().unwrap();
	}

	/// Print to `io::sink()`.
	///
	/// This is equivalent to manually writing the bytes to `io::sink()`,
	/// namely useful for benchmarking purposes.
	pub fn sink(&self) {
		let mut handle = io::sink();
		handle.write_all(&self.buf).unwrap();
		handle.flush().unwrap();
	}

	/// Print to `io::sink()` (w/ line break)
	///
	/// Same as `sink()`, except a trailing line break `10_u8` is appended.
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
	/// The length of `idx` is increased by `adj`, then all subsequent entries
	/// are shifted up accordingly (while maintaining their original lengths).
	///
	/// # Safety
	///
	/// This is only used internally; `idx` will have already been verified
	/// before reaching this point.
	unsafe fn p_grow(&mut self, mut idx: usize, adj: usize) {
		let ptr = self.p.as_mut_ptr();
		let count: usize = *ptr.add(P_IDX_COUNT);
		while idx <= count {
			*ptr.add(idx) += adj;
			idx += 1;
		}
	}

	/// Shrink & Shift Partitions
	///
	/// The length of `idx` is decreased by `adj`, then all subsequent entries
	/// are shifted down accordingly (while maintaining their original
	/// lengths).
	///
	/// Panics if `adj` is greater than `self.p[idx]`.
	///
	/// # Safety
	///
	/// This is only used internally; `idx` will have already been verified
	/// before reaching this point.
	unsafe fn p_shrink(&mut self, mut idx: usize, adj: usize) {
		assert!(self.p[idx] >= adj);

		let ptr = self.p.as_mut_ptr();
		let count: usize = *ptr.add(P_IDX_COUNT);
		while idx <= count {
			*ptr.add(idx) -= adj;
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


