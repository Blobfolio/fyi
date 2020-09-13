/*!
# FYI Msg: Table of Contents (SIMD)

The functionality is identical to that of the non-SIMD-optimized version; it is
just faster for x86-64 machines supporting SSE/AVX/etc.

**Note:** This is not intended for external use and is subject to change.
*/

use packed_simd::u16x32;
use std::ptr;



const IDX_LEN: usize = 16;



#[derive(Debug, Copy, Clone, Default)]
/// `Toc` stores arbitrary index ranges (`start..end`), providing a means of
/// logically partitioning the byte streams used by [Msg](crate::Msg) and `Witching`.
///
/// A total of **16** partitions are supported. They must be in relative order
/// with each other, but do not have to be contiguous.
///
/// It is worth noting that `Toc` is agnostic in regards to how many partitions
/// are actually in use by the implementing library. To prevent overflows from
/// subtraction operations, it is important to pad any unused entries using the
/// highest index from the active parts, like `0, 1, 2, 3, 3, 3, 3, 3...`.
pub struct Toc(u16x32);

impl Toc {
	#[allow(clippy::too_many_arguments)]
	#[must_use]
	/// # New Instance.
	///
	/// Create a new `Toc` instance defining each partition as a pair of `start..end`
	/// values. As mentioned in the struct's docs, if fewer than 16 partitions
	/// are actually used, the remaining values should be set to the maximum
	/// used index to avoid subtraction overflows.
	///
	/// # Safety
	///
	/// The ordering of partition indexes is not verified (as that would be
	/// tedious). If an implementing library submits data out of order,
	/// undefined things could happen!
	pub const fn new(
		a0: u16,
		a1: u16,
		a2: u16,
		a3: u16,
		a4: u16,
		a5: u16,
		a6: u16,
		a7: u16,
		a8: u16,
		a9: u16,
		a10: u16,
		a11: u16,
		a12: u16,
		a13: u16,
		a14: u16,
		a15: u16,
		a16: u16,
		a17: u16,
		a18: u16,
		a19: u16,
		a20: u16,
		a21: u16,
		a22: u16,
		a23: u16,
		a24: u16,
		a25: u16,
		a26: u16,
		a27: u16,
		a28: u16,
		a29: u16,
		a30: u16,
		a31: u16,
	) -> Self {
		Self(u16x32::new(
			a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
			a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
			a20, a21, a22, a23, a24, a25, a26, a27, a28, a29,
			a30, a31,
		))
	}

	#[must_use]
	#[inline]
	/// # Part Start.
	///
	/// Get the (inclusive) starting index of the part number `idx`.
	///
	/// ## Panics
	///
	/// This method might panic if `idx` is out of range.
	pub fn start(&self, idx: usize) -> usize {
		assert!(idx < IDX_LEN);
		unsafe { self.start_unchecked(idx) }
	}

	#[must_use]
	#[inline]
	/// # Part Start (Unchecked).
	///
	/// Get the (inclusive) starting index of the part number `idx`.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	pub unsafe fn start_unchecked(&self, idx: usize) -> usize {
		self.0.extract_unchecked(idx * 2) as usize
	}

	#[must_use]
	#[inline]
	/// # Part End.
	///
	/// Get the (exclusive) terminating index of the part number `idx`.
	///
	/// ## Panics
	///
	/// This method might panic if `idx` is out of range.
	pub fn end(&self, idx: usize) -> usize {
		assert!(idx < IDX_LEN);
		unsafe { self.end_unchecked(idx) }
	}

	#[must_use]
	#[inline]
	/// # Part End (Unchecked).
	///
	/// Get the (exclusive) terminating index of the part number `idx`.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	pub unsafe fn end_unchecked(&self, idx: usize) -> usize {
		self.0.extract_unchecked(idx * 2 + 1) as usize
	}

	#[must_use]
	#[inline]
	/// # Part Length.
	///
	/// Return the total length of a given part, equivalent to `end - start`.
	///
	/// ## Panics
	///
	/// This method might panic if `idx` is out of range.
	pub fn len(&self, idx: usize) -> usize {
		assert!(idx < IDX_LEN);
		unsafe { self.len_unchecked(idx) }
	}

	#[must_use]
	#[inline]
	/// # Part Length (Unchecked).
	///
	/// Return the total length of a given part, equivalent to `end - start`.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	pub unsafe fn len_unchecked(&self, idx: usize) -> usize {
		(self.0.extract_unchecked(idx * 2 + 1) - self.0.extract_unchecked(idx * 2)) as usize
	}

	#[must_use]
	#[inline]
	/// # Part Is Empty?
	///
	/// This returns `true` if the part has no length, or `false` if it does.
	///
	/// ## Panics
	///
	/// This method might panic if `idx` is out of range.
	pub fn is_empty(&self, idx: usize) -> bool {
		assert!(idx < IDX_LEN);
		unsafe { self.is_empty_unchecked(idx) }
	}

	#[must_use]
	#[inline]
	/// # Part Is Empty (Unchecked)?
	///
	/// This returns `true` if the part has no length, or `false` if it does.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	pub unsafe fn is_empty_unchecked(&self, idx: usize) -> bool {
		self.0.extract_unchecked(idx * 2) == self.0.extract_unchecked(idx * 2 + 1)
	}

	#[inline]
	/// # Decrease Part Length.
	///
	/// This decreases the length of a part by `adj`, and shifts any subsequent
	/// part boundaries that many places to the left.
	///
	/// ## Panics
	///
	/// This method will panic if the adjustment is greater than the length of
	/// the part, and might panic if the `idx` is out of range.
	pub fn decrease(&mut self, idx: usize, adj: u16) {
		self.0 -= pad_adj(idx, adj);
	}

	#[inline]
	/// # Increase Part Length.
	///
	/// This increases the length of a part by `adj`, and shifts any subsequent
	/// part boundaries that many places to the right.
	///
	/// ## Panics
	///
	/// This method might panic if `idx` is out of range.
	pub fn increase(&mut self, idx: usize, adj: u16) {
		self.0 += pad_adj(idx, adj);
	}

	/// # Replace Vec Range.
	///
	/// This method performs an in-place replacement to the section of a buffer
	/// corresponding to the partition. If the replacement value is of a
	/// different length than the original, the partitions will be realigned
	/// accordingly.
	///
	/// ## Panics
	///
	/// This method might panic if `idx` is out of range.
	pub fn replace(&mut self, src: &mut Vec<u8>, idx: usize, buf: &[u8]) {
		assert!(idx < IDX_LEN);
		unsafe { self.replace_unchecked(src, idx, buf) }
	}

	#[allow(clippy::comparison_chain)]
	/// # Replace Vec Range.
	///
	/// This method performs an in-place replacement to the section of a buffer
	/// corresponding to the partition. If the replacement value is of a
	/// different length than the original, the partitions will be realigned
	/// accordingly.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	pub unsafe fn replace_unchecked(&mut self, src: &mut Vec<u8>, idx: usize, buf: &[u8]) {
		let (old_len, new_len) = (self.len_unchecked(idx), buf.len());

		// We need to expand the part.
		if old_len < new_len { self.resize_grow(src, idx, new_len - old_len); }
		// We need to shrink the part.
		else if new_len < old_len { self.resize_shrink(src, idx, old_len - new_len); }

		if 0 != new_len {
			std::ptr::copy_nonoverlapping(
				buf.as_ptr(),
				src.as_mut_ptr().add(self.start_unchecked(idx)),
				new_len
			);
		}
	}

	/// # Shrink Vec Range.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	unsafe fn resize_shrink(&mut self, src: &mut Vec<u8>, idx: usize, adj: usize) {
		let end: usize = self.end_unchecked(idx);

		// End-of-buffer shortcut.
		if end == src.len() {
			src.truncate(end - adj);
		}
		// Middle incision.
		else {
			src.drain(end - adj..end);
		}

		self.decrease(idx, adj as u16);
	}

	/// # Grow Vec Range.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range. Additionally,
	/// data in the vector may be left uninitialized and will need to be
	/// written to before being used!
	unsafe fn resize_grow(&mut self, src: &mut Vec<u8>, idx: usize, adj: usize) {
		let end: usize = self.end_unchecked(idx);
		let len: usize = src.len();

		src.reserve(adj);

		// We need to shift things over.
		if end < len {
			ptr::copy(
				src.as_ptr().add(end),
				src.as_mut_ptr().add(end + adj),
				len - end
			);
		}

		src.set_len(len + adj);
		self.increase(idx, adj as u16);
	}

	/// # Zero Part.
	///
	/// Truncate a part to zero-length.
	///
	/// ## Panics
	///
	/// This method might panic if `idx` is out of range.
	pub fn zero(&mut self, src: &mut Vec<u8>, idx: usize) {
		assert!(idx < IDX_LEN);
		unsafe { self.zero_unchecked(src, idx) }
	}

	/// # Zero Part (Unchecked).
	///
	/// Truncate a part to zero-length.
	///
	/// ## Safety
	///
	/// Undefined things will happen if `idx` is out of range.
	pub unsafe fn zero_unchecked(&mut self, src: &mut Vec<u8>, idx: usize) {
		src.drain(self.start_unchecked(idx)..self.end_unchecked(idx));
		self.decrease(idx, self.len_unchecked(idx) as u16);
	}
}

/// # Pad Adjustment.
///
/// This method generates a suitable `u16x32` given an adjustment and index
/// such that adding it to the table increments or decrements only the cells
/// it is meant to.
///
/// As each part consists of a start and end, this mathematically works out to
/// zero-padding everything prior to `idx * 2 + 1`, and filling the rest with
/// the adjustment amount.
fn pad_adj(idx: usize, adj: u16) -> u16x32 {
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
		_ => panic!("Out of range!"),
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use criterion as _;

	#[test]
	fn t_replace() {
		let mut buf: Vec<u8> = vec![0, 0, 1, 1, 0, 0];
		let mut toc = Toc::new(
			2, 4,
			4, 6,
			6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
			6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
		);

		// Bigger.
		toc.replace(&mut buf, 0, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2, 0, 0]);
		assert_eq!(toc.start(0)..toc.end(0), 2..5);
		assert_eq!(toc.start(1)..toc.end(1), 5..7);

		// Same Size.
		toc.replace(&mut buf, 0, &[3, 3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3, 3, 0, 0]);
		assert_eq!(toc.start(0)..toc.end(0), 2..5);
		assert_eq!(toc.start(1)..toc.end(1), 5..7);

		// Smaller.
		toc.replace(&mut buf, 0, &[1]);
		assert_eq!(buf, vec![0, 0, 1, 0, 0]);
		assert_eq!(toc.start(0)..toc.end(0), 2..3);
		assert_eq!(toc.start(1)..toc.end(1), 3..5);

		// Empty.
		toc.replace(&mut buf, 0, &[]);
		assert_eq!(buf, vec![0, 0, 0, 0]);
		assert_eq!(toc.start(0)..toc.end(0), 2..2);
		assert_eq!(toc.start(1)..toc.end(1), 2..4);

		// Bigger (End).
		toc.replace(&mut buf, 1, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2]);
		assert_eq!(toc.start(0)..toc.end(0), 2..2);
		assert_eq!(toc.start(1)..toc.end(1), 2..5);

		// Smaller (End).
		toc.replace(&mut buf, 1, &[3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3]);
		assert_eq!(toc.start(0)..toc.end(0), 2..2);
		assert_eq!(toc.start(1)..toc.end(1), 2..4);
	}
}
