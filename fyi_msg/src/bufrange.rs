/*!
# FYI Msg: `BufRange`

Storing partitioned data in a single buffer — like a Vec<u8> — offers many
performance and accessibility advantages over storing each component
separately, but makes it more difficult to manage the individual components of
said buffer.

`BufRange` is simply a `start..end` range identifying a logical "partition"
within a continuous buffer. Paired with `replace_buf_range()`, this allows for
fairly mindless manipulation of the buffer part-by-part.

At the moment, `BufRange`s are stored in fixed-length arrays and passed as
slices, but it would be a lot less janky if we could implement a `BufRangeSet`
wrapper. We'll hop on that once Rust's generics support improves.
*/

use std::{
	cmp::Ordering,
	ops::Range,
	ptr,
};



#[derive(Debug, Clone, Copy, Default, Hash, PartialEq)]
/// Progress Buffer Range.
///
/// This is essentially a copyable `Range<usize>`, used to store the
/// (inclusive) start and (exclusive) end points of malleable buffer parts like
/// the title and elapsed times.
pub struct BufRange {
	/// The start index, inclusive.
	start: usize,
	/// The end index, exclusive.
	end: usize,
}

impl From<Range<usize>> for BufRange {
	fn from(src: Range<usize>) -> Self { Self::new(src.start, src.end) }
}

impl From<(usize, usize)> for BufRange {
	fn from(src: (usize, usize)) -> Self { Self::new(src.0, src.1) }
}

impl BufRange {
	#[must_use]
	/// New.
	///
	/// Create a new range from `start` to `end`.
	///
	/// Note: this method is `const` and therefore cannot explicitly assert,
	/// however `start` must be less than or equal to `end`. The struct is
	/// private, so this is more a Note-to-Self than anything.
	pub fn new(start: usize, end: usize) -> Self {
		if start <= end {
			Self { start, end }
		}
		else {
			Self {
				start: end,
				end: start,
			}
		}
	}

	#[must_use]
	/// Is Empty.
	///
	/// Returns true if the range is empty.
	pub const fn is_empty(&self) -> bool {
		self.end == self.start
	}

	#[must_use]
	/// Length.
	///
	/// Returns the length of the range.
	pub const fn len(&self) -> usize {
		self.end - self.start
	}

	#[must_use]
	/// Range.
	pub const fn as_range(&self) -> Range<usize> { self.start..self.end }

	#[must_use]
	/// Get the starting value.
	pub const fn start(&self) -> usize { self.start }

	#[must_use]
	/// Get the ending value.
	pub const fn end(&self) -> usize { self.start }

	/// Grow Set At.
	pub fn grow_set_at(set: &mut [Self], idx: usize, adj: usize) {
		set[idx].end += adj;
		set.iter_mut()
			.skip(idx + 1)
			.for_each(|x| x.increment(adj));
	}

	/// Grow Set At.
	pub fn shrink_set_at(set: &mut [Self], idx: usize, adj: usize) {
		set[idx].end -= adj;
		set.iter_mut()
			.skip(idx + 1)
			.for_each(|x| x.decrement(adj));
	}

	/// Decrement.
	///
	/// Decrease both `start` and `end` by `adj`.
	fn decrement(&mut self, adj: usize) {
		self.start -= adj;
		self.end -= adj;
	}

	/// Increment.
	///
	/// Increase both `start` and `end` by `adj`.
	fn increment(&mut self, adj: usize) {
		self.start += adj;
		self.end += adj;
	}
}



/// Replace Buffer Range.
///
/// This is basically a ranged-replace for `Vec<u8>` that also adjusts the
/// `BufRange`s. The replacement chunk can be any size or empty.
pub fn replace_buf_range(
	src: &mut Vec<u8>,
	toc: &mut [BufRange],
	idx: usize,
	buf: &[u8]
) {
	if src[toc[idx].as_range()].ne(buf) {
		resize_buf_range(src, toc, idx, buf.len());
		if ! buf.is_empty() {
			src[toc[idx].as_range()].copy_from_slice(buf);
		}
	}
}

/// Resize Buffer Range.
///
/// This will resize a `BufRange` within a `Vec<u8>` to the specified length
/// and adjust the `BufRange`s accordingly.
///
/// When growing, no guarantees are placed on the particular data added to the
/// range. It might be zeroes or leftover bits from data that was copied right
/// to make room. (In other words, you'll want to perform a sensible write
/// operation resizing.)
pub fn resize_buf_range(
	src: &mut Vec<u8>,
	toc: &mut [BufRange],
	idx: usize,
	len: usize
) {
	match len.cmp(&toc[idx].len()) {
		// Expand it.
		Ordering::Greater => {
			let adj: usize = len - toc[idx].len();
			vec_resize_at(src, toc[idx].end, adj);
			BufRange::grow_set_at(toc, idx, adj);
		},
		// Shrink it.
		Ordering::Less => {
			let adj: usize = toc[idx].len() - len;
			if idx == src.len() { src.truncate(idx - adj); }
			else { src.drain(toc[idx].end - adj..toc[idx].end); }
			BufRange::shrink_set_at(toc, idx, adj);
		},
		Ordering::Equal => {},
	}
}

/// Grow `Vec<u8>` From Middle.
///
/// This is like `resize()` combined with `range_replace()`, except all it does
/// is efficiently expand the vector length from the middle out. No particular
/// data is written to the created space; it might contain values from the
/// previous occupants (now copied right), or zeroes.
///
/// If `idx` is out of range, this acts just like `resize()`, with new bytes
/// added to the end.
///
/// The main idea is after calling this, new data should be written to the
/// slice.
pub fn vec_resize_at(src: &mut Vec<u8>, idx: usize, adj: usize) {
	let old_len: usize = src.len();
	src.resize(old_len + adj, 0);

	// Copy everything from the split point to the right.
	if idx < old_len {
		let ptr = src.as_mut_ptr();
		unsafe {
			ptr::copy(
				ptr.add(idx),
				ptr.add(idx + adj),
				old_len - idx,
			)
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_replace_buf_range() {
		let mut buf: Vec<u8> = vec![0, 0, 1, 1, 0, 0];
		let mut toc: [BufRange; 2] = [
			BufRange::new(2, 4),
			BufRange::new(4, 6),
		];

		// Bigger.
		replace_buf_range(&mut buf, &mut toc, 0, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2, 0, 0]);
		assert_eq!(
			toc,
			[BufRange::new(2, 5), BufRange::new(5, 7)]
		);

		// Same Size.
		replace_buf_range(&mut buf, &mut toc, 0, &[3, 3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3, 3, 0, 0]);
		assert_eq!(
			toc,
			[BufRange::new(2, 5), BufRange::new(5, 7)]
		);

		// Smaller.
		replace_buf_range(&mut buf, &mut toc, 0, &[1]);
		assert_eq!(buf, vec![0, 0, 1, 0, 0]);
		assert_eq!(
			toc,
			[BufRange::new(2, 3), BufRange::new(3, 5)]
		);

		// Empty.
		replace_buf_range(&mut buf, &mut toc, 0, &[]);
		assert_eq!(buf, vec![0, 0, 0, 0]);
		assert_eq!(
			toc,
			[BufRange::new(2, 2), BufRange::new(2, 4)]
		);

		// Bigger (End).
		replace_buf_range(&mut buf, &mut toc, 1, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2]);
		assert_eq!(
			toc,
			[BufRange::new(2, 2), BufRange::new(2, 5)]
		);

		// Smaller (End).
		replace_buf_range(&mut buf, &mut toc, 1, &[3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3]);
		assert_eq!(
			toc,
			[BufRange::new(2, 2), BufRange::new(2, 4)]
		);
	}

	#[test]
	fn t_vec_resize_at() {
		let mut test: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
		vec_resize_at(&mut test, 4, 5);
		assert_eq!(
			test,
			vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 4, 5, 6, 7, 8, 9],
		);

		vec_resize_at(&mut test, 15, 5);
		assert_eq!(
			test,
			vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0],
		);
	}
}
