/*!
# FYI Message: Message Buffer Partitioning

This is a simple trait to give us some short-hand methods for dealing with
partitions. It has no application beyond this crate.
*/



/// What an individual partition can do!
///
/// None of these methods check for or enforce ordering sanity, etc. That is
/// left for the implementors.
pub trait Partition {
	/// Check the partition for logical issues.
	fn partition_check(&self) -> bool;

	/// Set both start and end to `num`.
	fn partition_slop(&mut self, num: usize);

	/// Length.
	fn partition_len(&self) -> usize;

	/// Is Empty.
	fn partition_is_empty(&self) -> bool;

	/// Shift values to the left.
	fn partition_nudge_left(&mut self, adj: usize) -> usize;

	/// Shift values to the right.
	fn partition_nudge_right(&mut self, adj: usize) -> usize;
}

/// This goes on a (usize, usize) tuple.
impl Partition for (usize, usize) {
	/// Check the partition for logical issues.
	fn partition_check(&self) -> bool {
		self.0 <= self.1
	}

	/// Set both start and end to `num`.
	fn partition_slop(&mut self, num: usize) {
		self.0 = num;
		self.1 = num;
	}

	/// Length.
	fn partition_len(&self) -> usize {
		self.1 - self.0
	}

	/// Is Empty.
	fn partition_is_empty(&self) -> bool {
		self.1 == self.0
	}

	/// Shift values to the left.
	fn partition_nudge_left(&mut self, adj: usize) -> usize {
		self.0 -= adj;
		self.1 -= adj;
		self.1
	}

	/// Shift values to the right.
	fn partition_nudge_right(&mut self, adj: usize) -> usize {
		self.0 += adj;
		self.1 += adj;
		self.1
	}
}
