/*!
# FYI Msg Traits: `FastConcat`
*/

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
		let len: usize = a + b;
		let mut buf: Vec<u8> = Vec::with_capacity(len);

		unsafe {
			let dst = buf.as_mut_ptr();

			ptr::copy_nonoverlapping(self[0].as_ptr(), dst, a);
			ptr::copy_nonoverlapping(self[1].as_ptr(), dst.add(a), b);

			buf.set_len(len);
		}

		buf
	}
}

/// # Helper: Concatenation
///
/// The actual concatenation works the same regardless of array size; the loop
/// just needs a different breakpoint.
macro_rules! impl_fast_concat {
	($max:literal) => {
		fn fast_concat(&self) -> Vec<u8> {
			let len: usize = self.fast_concat_len();
			let mut buf: Vec<u8> = Vec::with_capacity(len);

			unsafe {
				let mut dst = buf.as_mut_ptr();
				let mut idx: usize = 0;

				loop {
					let s_len: usize = self[idx].len();
					ptr::copy_nonoverlapping(self[idx].as_ptr(), dst, s_len);
					if idx == $max { break; }
					else {
						dst = dst.add(s_len);
						idx += 1;
					}
				}

				buf.set_len(len);
			}

			buf
		}
	};
}

impl FastConcat for [&[u8]; 3] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len()
	}

	impl_fast_concat!(2);
}

impl FastConcat for [&[u8]; 4] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len()
	}

	impl_fast_concat!(3);
}

impl FastConcat for [&[u8]; 5] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len() + self[4].len()
	}

	impl_fast_concat!(4);
}

impl FastConcat for [&[u8]; 6] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len() + self[4].len() + self[5].len()
	}

	impl_fast_concat!(5);
}

impl FastConcat for [&[u8]; 7] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len() + self[4].len() + self[5].len() + self[6].len()
	}

	impl_fast_concat!(6);
}

impl FastConcat for [&[u8]; 8] {
	#[inline]
	fn fast_concat_len(&self) -> usize {
		self[0].len() + self[1].len() + self[2].len() + self[3].len() + self[4].len() + self[5].len() + self[6].len() + self[7].len()
	}

	impl_fast_concat!(7);
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
	}
}
