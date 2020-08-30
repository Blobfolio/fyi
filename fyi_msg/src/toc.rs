/*!
# FYI Msg: Table of Contents

`Toc` stores arbitrary index ranges (`start..end`), providing a means of
logically partitioning the byte streams used by `Msg` and `Witching`.

It is not intended for use outside the FYI libraries.
*/

use crate::utility;
use std::ops::Range;



#[derive(Debug, Copy, Clone, Default)]
/// Table of Contents.
pub struct Toc([u16; 32]);

impl Toc {
	#[allow(clippy::too_many_arguments)]
	#[must_use]
	/// New.
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
		Self([
			a0, a1, a2, a3, a4, a5, a6, a7, a8, a9,
			a10, a11, a12, a13, a14, a15, a16, a17, a18, a19,
			a20, a21, a22, a23, a24, a25, a26, a27, a28, a29,
			a30, a31,
		])
	}

	#[must_use]
	/// Part Start.
	pub const fn start(&self, idx: usize) -> usize {
		self.0[idx * 2] as usize
	}

	#[must_use]
	/// Part End.
	pub const fn end(&self, idx: usize) -> usize {
		self.0[idx * 2 + 1] as usize
	}

	#[must_use]
	/// Part Length.
	pub const fn len(&self, idx: usize) -> usize {
		self.0[idx * 2 + 1] as usize - self.0[idx * 2] as usize
	}

	#[must_use]
	/// Part Is Empty.
	pub const fn is_empty(&self, idx: usize) -> bool {
		self.0[idx * 2] == self.0[idx * 2 + 1]
	}

	#[must_use]
	/// Part Range.
	pub const fn range(&self, idx: usize) -> Range<usize> {
		self.0[idx * 2] as usize .. self.0[idx * 2 + 1] as usize
	}

	/// Decrease Part.
	pub fn decrease(&mut self, idx: usize, adj: u16) {
		self.0.iter_mut().skip(idx * 2 + 1).for_each(|x| *x -= adj);
	}

	/// Increase Part.
	pub fn increase(&mut self, idx: usize, adj: u16) {
		self.0.iter_mut().skip(idx * 2 + 1).for_each(|x| *x += adj);
	}

	/// Replace Vec Range.
	pub fn replace(&mut self, src: &mut Vec<u8>, idx: usize, buf: &[u8]) {
		self.resize(src, idx, buf.len());
		if ! buf.is_empty() {
			src[self.range(idx)].copy_from_slice(buf);
		}
	}

	#[allow(clippy::comparison_chain)] // We only need two arms.
	/// Resize Vec Range.
	pub fn resize(&mut self, src: &mut Vec<u8>, idx: usize, len: usize) {
		let old_len: usize = self.len(idx);

		// Shrink it.
		if old_len > len {
			let adj: usize = old_len - len;
			let end: usize = self.end(idx);

			// End-of-buffer shortcut.
			if end == src.len() {
				let last: u16 = end as u16 - adj as u16;
				src.truncate(last as usize);
				self.0.iter_mut().skip(idx * 2 + 1).for_each(|x| *x = last);
			}
			// Middle incision.
			else {
				src.drain(end - adj..end);
				self.decrease(idx, adj as u16);
			}
		}
		// Grow it!
		else if len > old_len {
			let adj: usize = len - old_len;
			utility::vec_resize_at(src, self.end(idx), adj);
			self.increase(idx, adj as u16);
		}
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
		assert_eq!(toc.range(0), 2..5);
		assert_eq!(toc.range(1), 5..7);

		// Same Size.
		toc.replace(&mut buf, 0, &[3, 3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3, 3, 0, 0]);
		assert_eq!(toc.range(0), 2..5);
		assert_eq!(toc.range(1), 5..7);

		// Smaller.
		toc.replace(&mut buf, 0, &[1]);
		assert_eq!(buf, vec![0, 0, 1, 0, 0]);
		assert_eq!(toc.range(0), 2..3);
		assert_eq!(toc.range(1), 3..5);

		// Empty.
		toc.replace(&mut buf, 0, &[]);
		assert_eq!(buf, vec![0, 0, 0, 0]);
		assert_eq!(toc.range(0), 2..2);
		assert_eq!(toc.range(1), 2..4);

		// Bigger (End).
		toc.replace(&mut buf, 1, &[2, 2, 2]);
		assert_eq!(buf, vec![0, 0, 2, 2, 2]);
		assert_eq!(toc.range(0), 2..2);
		assert_eq!(toc.range(1), 2..5);

		// Smaller (End).
		toc.replace(&mut buf, 1, &[3, 3]);
		assert_eq!(buf, vec![0, 0, 3, 3]);
		assert_eq!(toc.range(0), 2..2);
		assert_eq!(toc.range(1), 2..4);
	}
}
