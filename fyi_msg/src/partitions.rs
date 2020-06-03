/*!
# FYI Message: Message Buffer Partitions

This is the partition scheme for the message buffer.
*/

use std::{
	ops::{
		AddAssign,
		Index,
		Range,
	},
	ptr,
};



#[derive(Debug, Clone, Copy, Default, Hash, PartialEq)]
/// Partitions
///
/// This is a very simple partitioning table, each index — up to 15 —
/// representing an Exclude(end). The first "end" is always zero.
pub struct Partitions {
	inner: [usize; 16],
	used: usize,
}

impl AddAssign<usize> for Partitions {
	fn add_assign(&mut self, other: usize) {
		assert!(
			self.used < Self::MAX_USED,
			"Partitions are full."
		);
		self.used += 1;
		unsafe {
			ptr::copy_nonoverlapping(
				&(other + self.inner[self.used - 1]),
				&mut self.inner[self.used],
				1
			);
		}
	}
}

impl Index<usize> for Partitions {
	type Output = usize;

	fn index(&self, idx: usize) -> &Self::Output {
		assert!(idx <= self.used);
		&self.inner[idx]
	}
}



impl Partitions {
	/// The first index (0) is reserved as, well, `0`. That leaves fifteen
	/// slots for people.
	pub const MAX_USED: usize = 15;



	// ------------------------------------------------------------------------
	// Instantiation
	// ------------------------------------------------------------------------

	#[must_use]
	/// New
	///
	/// Create a new instance using pre-calculated part lengths.
	///
	/// If you pass an empty parts partition, this is equivalent to calling
	/// `default()`, which you should do instead.
	///
	/// Panics if there are more than `15` slices.
	pub fn new(parts: &[usize]) -> Self {
		let mut out = Self::default();

		unsafe {
			let mut last: usize = 0;
			parts.iter().for_each(|p| {
				last += p;
				out.used += 1;
				ptr::copy_nonoverlapping(
					&last,
					&mut out.inner[out.used],
					1
				);
			});
		}

		out
	}

	#[must_use]
	/// New (Bounded)
	///
	/// Create a new instance using pre-calculated part lengths, ensuring the
	/// end reaches `max`. If the provided parts fall short, an additional part
	/// will be created to fill the gap.
	///
	/// If you pass an empty parts partition, this is equivalent to calling
	/// `one()`, which you should do instead.
	///
	/// This method panics if there are more than `15` slices, or if any of
	/// the parts are out of range.
	pub fn new_bounded(parts: &[usize], max: usize) -> Self {
		if parts.is_empty() {
			Self::one(max)
		}
		else {
			let mut out = Self::new(parts);

			// If the last part falls short of `max`, add one more.
			if out.inner[out.used] < max {
				out.used += 1;
				unsafe {
					ptr::copy_nonoverlapping(
						&max,
						&mut out.inner[out.used],
						1
					);
				}
			}
			else {
				assert_eq!(
					out.inner[out.used],
					max,
					"The partition boundaries ends at {:?} instead of {:?}",
					out.inner[out.used],
					max
				);
			}

			out
		}
	}

	#[must_use]
	/// Single
	///
	/// Create a single partition with the specified length.
	pub const fn one(len: usize) -> Self {
		Self {
			inner: [0, len, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			used: 1,
		}
	}

	#[must_use]
	/// Splat
	///
	/// Create `num` empty partitions, where `num` is between 1 and 16.
	pub fn splat(num: usize) -> Self {
		let num = usize::max(1, num);
		unsafe { Self::splat_unchecked(usize::min(Self::MAX_USED, num)) }
	}

	#[must_use]
	/// Splat (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub const unsafe fn splat_unchecked(num: usize) -> Self {
		Self {
			inner: [0; 16],
			used: num,
		}
	}



	// ------------------------------------------------------------------------
	// Working on the Whole
	// ------------------------------------------------------------------------

	/// Clear
	pub fn clear(&mut self) {
		self.inner[1..].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
		self.used = 0;
	}

	/// Flatten
	///
	/// Reduce to a single part spanning `0..end`.
	pub fn flatten(&mut self) {
		if 0 == self.used {
			self.used = 1;
		}
		else if 1 < self.used {
			// Copy the last value to the first user index.
			unsafe {
				ptr::copy_nonoverlapping(
					&self.inner[self.used],
					&mut self.inner[1],
					1
				);
			}

			// Zero out everything else.
			self.inner[2..].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
			self.used = 1;
		}
	}

	#[must_use]
	/// Is Empty.
	pub const fn is_empty(&self) -> bool {
		0 == self.used
	}

	#[must_use]
	/// Number of Partitions.
	pub const fn len(&self) -> usize {
		self.used
	}

	#[must_use]
	/// Maximum Value.
	pub const fn max(&self) -> usize {
		self.inner[self.used]
	}

	/// Zero
	///
	/// This combines `clear()` and `flatten()` so that you end up with a
	/// single zero-length part.
	pub fn zero(&mut self) {
		self.inner[1..].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
		self.used = 1;
	}



	// ------------------------------------------------------------------------
	// Fetching Parts
	// ------------------------------------------------------------------------

	#[must_use]
	/// Get Part
	///
	/// Panics if `idx` is out of range.
	pub fn part(&self, idx: usize) -> Range<usize> {
		assert!(
			0 < idx && idx <= self.used,
			"Index {:?} is out of range.",
			idx
		);
		unsafe { self.part_unchecked(idx) }
	}

	#[must_use]
	/// Get Part (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub const unsafe fn part_unchecked(&self, idx: usize) -> Range<usize> {
		Range { start: self.inner[idx - 1], end: self.inner[idx] }
	}

	#[must_use]
	/// Get Spanning Range Across Parts
	///
	/// Panics if `idx1` or `idx2` are out of range or out of order.
	pub fn spread(&self, idx1: usize, idx2: usize) -> Range<usize> {
		assert!(
			0 < idx1 && idx1 < idx2 && idx2 <= self.used,
			"Invalid spread range: {:?} to {:?}",
			idx1,
			idx2
		);
		unsafe { self.spread_unchecked(idx1, idx2) }
	}

	#[must_use]
	/// Get Spanning Range Across Parts (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub const unsafe fn spread_unchecked(&self, idx1: usize, idx2: usize) -> Range<usize> {
		Range {
			start: self.inner[idx1 - 1],
			end: self.inner[idx2],
		}
	}

	#[must_use]
	/// Get Part Length
	///
	/// Panics if `idx` is out of range.
	pub fn part_len(&self, idx: usize) -> usize {
		assert!(
			0 < idx && idx <= self.used,
			"Index {:?} is out of range.",
			idx
		);
		unsafe { self.part_len_unchecked(idx) }
	}

	#[must_use]
	/// Get Part Length (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub const unsafe fn part_len_unchecked(&self, idx: usize) -> usize {
		self.inner[idx] - self.inner[idx - 1]
	}

	#[must_use]
	/// Part is Empty
	///
	/// Panics if `idx` is out of range.
	pub fn part_is_empty(&self, idx: usize) -> bool {
		assert!(
			0 < idx && idx <= self.used,
			"Index {:?} is out of range.",
			idx
		);
		unsafe { self.part_is_empty_unchecked(idx) }
	}

	#[must_use]
	/// Part is Empty (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub const unsafe fn part_is_empty_unchecked(&self, idx: usize) -> bool {
		self.inner[idx] == self.inner[idx - 1]
	}



	// ------------------------------------------------------------------------
	// Adding Parts
	// ------------------------------------------------------------------------

	/// Insert Part
	///
	/// Panics if the maximum number of parts has been reached or `idx` is out
	/// of bounds.
	pub fn insert_part(&mut self, idx: usize, len: usize) {
		assert!(
			0 < idx && idx <= self.used && self.used < Self::MAX_USED,
			"Index {:?} is out of range, or the partitions are full.",
			idx
		);
		unsafe { self.insert_part_unchecked(idx, len) }
	}

	/// Insert Part (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn insert_part_unchecked(&mut self, idx: usize, len: usize) {
		if 0 == len { self.insert_empty_part_unchecked(idx); }
		else {
			// Shift and nudge the tail, working backwards.
			//
			// This job would normally be well-suited to a blanket `ptr::copy()` of
			// the entire tail one index up, but our array isn't big enough to
			// benefit from that. Copying items over individually within a reverse
			// loop is 2-25% faster.
			let mut tail_idx: usize = self.used;
			while tail_idx >= idx {
				ptr::copy_nonoverlapping(
					&(len + self.inner[tail_idx]),
					&mut self.inner[tail_idx + 1],
					1
				);
				tail_idx -= 1;
			}

			// And finally, adjust our "new" value.
			self.used += 1;
			ptr::copy_nonoverlapping(
				&(len + self.inner[idx - 1]),
				&mut self.inner[idx],
				1
			);
		}
	}

	/// Insert Empty Part (Unchecked)
	///
	/// This is the same as the main method, but skips the `len+` operations.
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	unsafe fn insert_empty_part_unchecked(&mut self, idx: usize) {
		let mut tail_idx: usize = self.used;
		while tail_idx >= idx {
			ptr::copy_nonoverlapping(
				&self.inner[tail_idx],
				&mut self.inner[tail_idx + 1],
				1
			);
			tail_idx -= 1;
		}

		// And finally, adjust our "new" value.
		self.used += 1;
		ptr::copy_nonoverlapping(
			&self.inner[idx - 1],
			&mut self.inner[idx],
			1
		);
	}

	/// Remove Part
	///
	/// Panics if `idx` is out of bounds.
	pub fn remove_part(&mut self, idx: usize) {
		assert!(
			0 < idx && idx <= self.used,
			"Index {:?} is out of range.",
			idx
		);
		unsafe { self.remove_part_unchecked(idx) }
	}

	/// Remove Part (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn remove_part_unchecked(&mut self, mut idx: usize) {
		// Shift and nudge the tail.
		let adj: usize = self.part_len_unchecked(idx);

		while idx < self.used {
			ptr::copy_nonoverlapping(
				&(self.inner[idx + 1] - adj),
				&mut self.inner[idx],
				1
			);
			idx += 1;
		}

		// Zero out the last part.
		ptr::copy_nonoverlapping(
			&self.inner[0],
			&mut self.inner[self.used],
			1
		);

		self.used -= 1;
	}



	// ------------------------------------------------------------------------
	// Changing Parts
	// ------------------------------------------------------------------------

	/// Grow Part
	///
	/// Panics if `idx` is out of range or `adj` is bigger than the part.
	pub fn grow_part(&mut self, idx: usize, adj: usize) {
		assert!(
			0 < idx && idx <= self.used,
			"Index {:?} is out of range.",
			idx
		);
		unsafe { self.grow_part_unchecked(idx, adj) }
	}

	/// Grow Part (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn grow_part_unchecked(&mut self, mut idx: usize, adj: usize) {
		while idx <= self.used {
			ptr::copy_nonoverlapping(
				&(adj + self.inner[idx]),
				&mut self.inner[idx],
				1
			);
			idx += 1;
		}
	}

	/// Shrink Part
	///
	/// Panics if `idx` is out of range or `adj` is bigger than the part.
	pub fn shrink_part(&mut self, idx: usize, adj: usize) {
		assert!(
			0 < idx && idx <= self.used && adj <= unsafe { self.part_len_unchecked(idx) },
			"Index {:?} is out of range or the part isn't big enough to reduce by {:?}.",
			idx,
			adj
		);
		unsafe { self.shrink_part_unchecked(idx, adj) }
	}

	/// Shrink Part (Unchecked)
	///
	/// # Safety
	///
	/// This method does not check index sanity.
	pub unsafe fn shrink_part_unchecked(&mut self, mut idx: usize, adj: usize) {
		while idx <= self.used {
			self.inner[idx] -= adj;
			idx += 1;
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_new() {
		// Empty should have one empty part.
		let empty = Partitions::new(&[]);
		assert_eq!(empty.len(), 0);
		assert_eq!(empty.max(), 0);

		// Check with one part.
		let one = Partitions::new(&[5]);
		assert_eq!(one.len(), 1);
		assert_eq!(one.max(), 5);

		// Check with many parts
		let many = Partitions::new(&[5, 4, 3, 2, 1]);
		assert_eq!(many.len(), 5);
		assert_eq!(many.max(), 15);
		for (k, v) in [5, 4, 3, 2, 1].iter().enumerate() {
			assert_eq!(many.part_len(k + 1), *v);
		}
	}

	#[test]
	fn t_new_bounded() {
		// Empty with zero max.
		let empty = Partitions::new_bounded(&[], 0);
		assert_eq!(empty.len(), 1);
		assert_eq!(empty.max(), 0);

		// Empty with non-zero max.
		let empty = Partitions::new_bounded(&[], 10);
		assert_eq!(empty.len(), 1);
		assert_eq!(empty.part_len(1), 10);
		assert_eq!(empty.max(), 10);

		// One part, at.
		let one = Partitions::new_bounded(&[5], 5);
		assert_eq!(one.len(), 1);
		assert_eq!(one.part_len(1), 5);
		assert_eq!(one.max(), 5);

		// One part, under.
		let one = Partitions::new_bounded(&[5], 15);
		assert_eq!(one.len(), 2);
		assert_eq!(one.max(), 15);
		assert_eq!(one.part_len(1), 5);
		assert_eq!(one.part_len(2), 10);

		// Many parts, at.
		let many = Partitions::new_bounded(&[5, 4, 3, 2, 1], 15);
		assert_eq!(many.len(), 5);
		assert_eq!(many.max(), 15);
		for (k, v) in [5, 4, 3, 2, 1].iter().enumerate() {
			assert_eq!(many.part_len(k + 1), *v);
		}

		// Many parts, under.
		let many = Partitions::new_bounded(&[5, 4, 3, 2, 1], 20);
		assert_eq!(many.len(), 6);
		assert_eq!(many.max(), 20);
		for (k, v) in [5, 4, 3, 2, 1, 5].iter().enumerate() {
			assert_eq!(many.part_len(k + 1), *v);
		}
	}

	#[test]
	fn t_one() {
		for v in [0, 1, 5].iter() {
			let one = Partitions::one(*v);
			assert_eq!(one.len(), 1);
			assert_eq!(one.max(), *v);
		}
	}

	#[test]
	fn t_splat() {
		for v in [0, 1, 5].iter() {
			let splat = Partitions::splat(*v);
			assert_eq!(splat.len(), usize::max(1, *v));
			assert_eq!(splat.max(), 0);
		}
	}

	#[test]
	fn t_part() {
		let many = Partitions::new(&[5, 4, 3, 2, 1]);
		assert_eq!(many.part(1), 0..5);
		assert_eq!(many.part(2), 5..9);
		assert_eq!(many.part(3), 9..12);
		assert_eq!(many.part(4), 12..14);
		assert_eq!(many.part(5), 14..15);
	}

	#[test]
	fn t_part_is_empty() {
		let many = Partitions::new(&[5, 0, 5, 0]);
		assert!(! many.part_is_empty(1));
		assert!(many.part_is_empty(2));
		assert!(! many.part_is_empty(3));
		assert!(many.part_is_empty(4));

		let many = Partitions::new(&[0, 5]);
		assert!(many.part_is_empty(1));
		assert!(! many.part_is_empty(2));
	}

	#[test]
	fn t_spread() {
		let many = Partitions::new(&[5, 4, 3, 2, 1]);
		assert_eq!(many.spread(1, 2), 0..9);
		assert_eq!(many.spread(1, 5), 0..15);
		assert_eq!(many.spread(2, 3), 5..12);
		assert_eq!(many.spread(3, 5), 9..15);
	}

	#[test]
	fn t_add_remove_part() {
		let mut empty = Partitions::default();
		assert_eq!(empty.len(), 0);
		assert_eq!(empty.max(), 0);

		// Add zero length.
		empty += 0;
		assert_eq!(empty.len(), 1);
		assert_eq!(empty.max(), 0);

		// Remove zero length.
		empty.remove_part(1);
		assert_eq!(empty.len(), 0);
		assert_eq!(empty.max(), 0);

		// Add lengthed.
		empty += 2;
		assert_eq!(empty.len(), 1);
		assert_eq!(empty.max(), 2);

		// Remove lengthed.
		empty.remove_part(1);
		assert_eq!(empty.len(), 0);
		assert_eq!(empty.max(), 0);

		// Add a few.
		empty += 1;
		empty += 2;
		empty += 3;
		assert_eq!(empty.len(), 3);
		assert_eq!(empty.max(), 6);
		assert_eq!(empty.part(1), 0..1);
		assert_eq!(empty.part(2), 1..3);
		assert_eq!(empty.part(3), 3..6);

		// Remove first twice.
		empty.remove_part(1);
		assert_eq!(empty.len(), 2);
		assert_eq!(empty.max(), 5);
		assert_eq!(empty.part(1), 0..2);
		assert_eq!(empty.part(2), 2..5);

		empty.remove_part(1);
		assert_eq!(empty.len(), 1);
		assert_eq!(empty.max(), 3);
		assert_eq!(empty.part(1), 0..3);

		// Remove second twice.
		empty = Partitions::new(&[1, 2, 3]);
		empty.remove_part(2);
		assert_eq!(empty.len(), 2);
		assert_eq!(empty.max(), 4);
		assert_eq!(empty.part(1), 0..1);
		assert_eq!(empty.part(2), 1..4);

		empty.remove_part(2);
		assert_eq!(empty.len(), 1);
		assert_eq!(empty.max(), 1);
		assert_eq!(empty.part(1), 0..1);
	}

	#[test]
	fn t_grow_shrink_part() {
		// Working on many.
		let mut parts = Partitions::new(&[1, 2, 3]);

		// Fake growth, fake shrink.
		let starts = [0, 1, 3];
		for (k, v) in [1, 2, 3].iter().enumerate() {
			// Grow by nothing.
			let end = starts[k] + *v;
			assert_eq!(parts.part(k + 1), starts[k]..end);
			parts.grow_part(k + 1, 0);
			assert_eq!(parts.part(k + 1), starts[k]..end);
			parts.shrink_part(k + 1, 0);
			assert_eq!(parts.part(k + 1), starts[k]..end);
		}

		// Growth x2.
		parts = Partitions::new(&[1, 2, 3]);
		let starts2 = [0, 3, 7];
		for (k, v) in [1, 2, 3].iter().enumerate() {
			// Grow by nothing.
			parts.grow_part(k + 1, 2);
			let end = starts2[k] + *v + 2;
			assert_eq!(parts.part(k + 1), starts2[k]..end, "{:?}", parts);
		}

		// Shrink x2.
		for i in 1..4 {
			parts.shrink_part(i, 2);
		}
		assert_eq!(parts.part(1), 0..1);
		assert_eq!(parts.part(2), 1..3);
		assert_eq!(parts.part(3), 3..6);

		// Make sure everything works fine on a single partition.
		parts = Partitions::one(2);
		assert_eq!(parts.part(1), 0..2);
		parts.grow_part(1, 0);
		assert_eq!(parts.part(1), 0..2);
		parts.grow_part(1, 2);
		assert_eq!(parts.part(1), 0..4);
		parts.shrink_part(1, 0);
		assert_eq!(parts.part(1), 0..4);
		parts.shrink_part(1, 3);
		assert_eq!(parts.part(1), 0..1);
	}

	#[test]
	fn t_insert_part() {
		let many = [1, 2, 3];

		for i in 1..4 {
			// Add empty to beginning, middle, end.
			let mut parts = Partitions::new(&many);
			parts.insert_part(i, 0);
			assert_eq!(parts.len(), 4);
			assert_eq!(parts.max(), 6, "{:?}\n{:?}", Partitions::new(&many), parts);
			assert_eq!(parts.part_len(i), 0);
			assert_eq!(parts.part_len(i + 1), many[i - 1]);
		}
	}
}
