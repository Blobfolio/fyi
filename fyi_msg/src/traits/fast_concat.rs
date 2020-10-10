/*!
# FYI Msg Traits: `FastConcat`
*/

use crate::utility;
use std::ptr;



/// # Fast Concatenation.
///
/// This trait provides fast, specialized concatenation of slices of slices of
/// bytes. The result is comparable to `[].concat()`, just faster owing to the
/// fixed length.
pub trait FastConcat {
	/// # Combined Length.
	///
	/// Sum up all the slices.
	fn fast_concat_len(&self) -> usize;

	/// # Concatenate
	///
	/// Concatenate the slices into a single vector.
	fn fast_concat(&self) -> Vec<u8>;
}

impl FastConcat for [&[u8]; 2] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len()
	}

	fn fast_concat(&self) -> Vec<u8> {
		let (a, b) = (self[0].len(), self[1].len());
		let mut buf: Vec<u8> = Vec::with_capacity(a + b);

		unsafe {
			let dst = utility::write_advance(buf.as_mut_ptr(), self[0].as_ptr(), a);
			ptr::copy_nonoverlapping(self[1].as_ptr(), dst, b);

			buf.set_len(a + b);
		}

		buf
	}
}

impl FastConcat for [&[u8]; 3] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len()
	}

	fn fast_concat(&self) -> Vec<u8> {
		let len: usize = self.fast_concat_len();
		let mut buf: Vec<u8> = Vec::with_capacity(len);

		unsafe {
			let mut dst = buf.as_mut_ptr();

			dst = utility::write_advance(dst, self[0].as_ptr(), self[0].len());
			dst = utility::write_advance(dst, self[1].as_ptr(), self[1].len());
			ptr::copy_nonoverlapping(self[2].as_ptr(), dst, self[2].len());

			buf.set_len(len);
		}

		buf
	}
}

impl FastConcat for [&[u8]; 4] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len()
	}

	fn fast_concat(&self) -> Vec<u8> {
		let len: usize = self.fast_concat_len();
		let mut buf: Vec<u8> = Vec::with_capacity(len);

		unsafe {
			let mut dst = buf.as_mut_ptr();

			dst = utility::write_advance(dst, self[0].as_ptr(), self[0].len());
			dst = utility::write_advance(dst, self[1].as_ptr(), self[1].len());
			dst = utility::write_advance(dst, self[2].as_ptr(), self[2].len());
			ptr::copy_nonoverlapping(self[3].as_ptr(), dst, self[3].len());

			buf.set_len(len);
		}

		buf
	}
}

impl FastConcat for [&[u8]; 5] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len() + self[4].len()
	}

	fn fast_concat(&self) -> Vec<u8> {
		let len: usize = self.fast_concat_len();
		let mut buf: Vec<u8> = Vec::with_capacity(len);

		unsafe {
			let mut dst = buf.as_mut_ptr();

			dst = utility::write_advance(dst, self[0].as_ptr(), self[0].len());
			dst = utility::write_advance(dst, self[1].as_ptr(), self[1].len());
			dst = utility::write_advance(dst, self[2].as_ptr(), self[2].len());
			dst = utility::write_advance(dst, self[3].as_ptr(), self[3].len());
			ptr::copy_nonoverlapping(self[4].as_ptr(), dst, self[4].len());

			buf.set_len(len);
		}

		buf
	}
}

impl FastConcat for [&[u8]; 6] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len() + self[4].len() + self[5].len()
	}

	fn fast_concat(&self) -> Vec<u8> {
		let len: usize = self.fast_concat_len();
		let mut buf: Vec<u8> = Vec::with_capacity(len);

		unsafe {
			let mut dst = buf.as_mut_ptr();

			dst = utility::write_advance(dst, self[0].as_ptr(), self[0].len());
			dst = utility::write_advance(dst, self[1].as_ptr(), self[1].len());
			dst = utility::write_advance(dst, self[2].as_ptr(), self[2].len());
			dst = utility::write_advance(dst, self[3].as_ptr(), self[3].len());
			dst = utility::write_advance(dst, self[4].as_ptr(), self[4].len());
			ptr::copy_nonoverlapping(self[5].as_ptr(), dst, self[5].len());

			buf.set_len(len);
		}

		buf
	}
}

impl FastConcat for [&[u8]; 7] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len() + self[4].len() + self[5].len() + self[6].len()
	}

	fn fast_concat(&self) -> Vec<u8> {
		let len: usize = self.fast_concat_len();
		let mut buf: Vec<u8> = Vec::with_capacity(len);

		unsafe {
			let mut dst = buf.as_mut_ptr();

			dst = utility::write_advance(dst, self[0].as_ptr(), self[0].len());
			dst = utility::write_advance(dst, self[1].as_ptr(), self[1].len());
			dst = utility::write_advance(dst, self[2].as_ptr(), self[2].len());
			dst = utility::write_advance(dst, self[3].as_ptr(), self[3].len());
			dst = utility::write_advance(dst, self[4].as_ptr(), self[4].len());
			dst = utility::write_advance(dst, self[5].as_ptr(), self[5].len());
			ptr::copy_nonoverlapping(self[6].as_ptr(), dst, self[6].len());

			buf.set_len(len);
		}

		buf
	}
}

impl FastConcat for [&[u8]; 8] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len() + self[4].len() + self[5].len() + self[6].len() + self[7].len()
	}

	fn fast_concat(&self) -> Vec<u8> {
		let len: usize = self.fast_concat_len();
		let mut buf: Vec<u8> = Vec::with_capacity(len);

		unsafe {
			let mut dst = buf.as_mut_ptr();

			dst = utility::write_advance(dst, self[0].as_ptr(), self[0].len());
			dst = utility::write_advance(dst, self[1].as_ptr(), self[1].len());
			dst = utility::write_advance(dst, self[2].as_ptr(), self[2].len());
			dst = utility::write_advance(dst, self[3].as_ptr(), self[3].len());
			dst = utility::write_advance(dst, self[4].as_ptr(), self[4].len());
			dst = utility::write_advance(dst, self[5].as_ptr(), self[5].len());
			dst = utility::write_advance(dst, self[6].as_ptr(), self[6].len());
			ptr::copy_nonoverlapping(self[7].as_ptr(), dst, self[7].len());

			buf.set_len(len);
		}

		buf
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fast_concat() {
		assert_eq!(
			(
				[&b"one"[..], b"two"].fast_concat(),
				[&b"one"[..], b"two"].fast_concat_len()
			),
			(b"onetwo".to_vec(), 6_usize)
		);

		assert_eq!(
			(
				[&b"one"[..], b"two", b"three"].fast_concat(),
				[&b"one"[..], b"two", b"three"].fast_concat_len()
			),
			(b"onetwothree".to_vec(), 11_usize)
		);

		assert_eq!(
			(
				[&b"one"[..], b"two", b"three", b"four"].fast_concat(),
				[&b"one"[..], b"two", b"three", b"four"].fast_concat_len()
			),
			(b"onetwothreefour".to_vec(), 15_usize)
		);

		assert_eq!(
			(
				[&b"one"[..], b"two", b"three", b"four", b"five"].fast_concat(),
				[&b"one"[..], b"two", b"three", b"four", b"five"].fast_concat_len()
			),
			(b"onetwothreefourfive".to_vec(), 19_usize)
		);

		assert_eq!(
			(
				[&b"one"[..], b"two", b"three", b"four", b"five", b"six"].fast_concat(),
				[&b"one"[..], b"two", b"three", b"four", b"five", b"six"].fast_concat_len()
			),
			(b"onetwothreefourfivesix".to_vec(), 22_usize)
		);

		assert_eq!(
			(
				[&b"one"[..], b"two", b"three", b"four", b"", b"six"].fast_concat(),
				[&b"one"[..], b"two", b"three", b"four", b"", b"six"].fast_concat_len()
			),
			(b"onetwothreefoursix".to_vec(), 18_usize)
		);

		assert_eq!(
			(
				[&b"one"[..], b"two", b"three", b"four", b"", b"six", b"seven"].fast_concat(),
				[&b"one"[..], b"two", b"three", b"four", b"", b"six", b"seven"].fast_concat_len()
			),
			(b"onetwothreefoursixseven".to_vec(), 23_usize)
		);

		assert_eq!(
			(
				[&b"one"[..], b"two", b"three", b"four", b"", b"six", b"seven", b"eight"].fast_concat(),
				[&b"one"[..], b"two", b"three", b"four", b"", b"six", b"seven", b"eight"].fast_concat_len()
			),
			(b"onetwothreefoursixseveneight".to_vec(), 28_usize)
		);
	}
}
