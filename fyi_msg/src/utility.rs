/*!
# FYI Msg: Utility Methods
*/

use std::ptr;



#[must_use]
#[inline]
/// # `AHash` Byte Hash.
///
/// This is a convenience method for quickly hashing bytes using the
/// [`AHash`](https://crates.io/crates/ahash) crate. Check out that project's
/// home page for more details. Otherwise, TL;DR it is very fast.
///
/// ## Examples
///
/// ```no_run
/// let hash = fyi_msg::utility::hash64(b"Hello World");
/// ```
pub fn hash64(src: &[u8]) -> u64 {
	use std::hash::Hasher;
	let mut hasher = ahash::AHasher::default();
	hasher.write(src);
	hasher.finish()
}

/// # Grow `Vec<u8>` From Middle.
///
/// This works like [`std::vec::Vec::resize`], except it supports expansion from the
/// middle, like [`std::vec::Vec::insert`]. The expanded indexes will
/// never be undefined, but may contain copies of data previously occupying
/// those spots (rather than a bunch of zeroes).
///
/// It might seem counter-intuitive to split the resizing and writing
/// operations, but this approach is generally faster than trying to do both at
/// once using [`std::vec::Vec::splice`].
///
/// ## Examples
///
/// ```no_run
/// let mut test: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
/// vec_resize_at(&mut test, 4, 5);
/// assert_eq!(
///     test,
///     vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 4, 5, 6, 7, 8, 9],
/// );
/// ```
pub fn vec_resize_at(src: &mut Vec<u8>, idx: usize, adj: usize) {
	let old_len: usize = src.len();
	if idx >= old_len {
		src.resize(old_len + adj, 0);
	}
	else {
		src.reserve(adj);
		unsafe {
			{
				let ptr = src.as_mut_ptr().add(idx);
				let after: usize = old_len - idx;

				// Shift the data over.
				ptr::copy(ptr, ptr.add(adj), after);

				// If we're adding more than we just copied, we'll need to
				// initialize those values.
				if adj > after {
					ptr::write_bytes(ptr.add(after), 0, adj - after);
				}
			}
			src.set_len(old_len + adj);
		}
	}
}

#[must_use]
/// # Write and Advance.
///
/// Write data to a pointer, then return a new pointer advanced that many
/// places.
///
/// ## Safety
///
/// The pointer must have enough room to hold the new data!
pub unsafe fn write_advance(dst: *mut u8, src: *const u8, len: usize) -> *mut u8 {
	ptr::copy_nonoverlapping(src, dst, len);
	dst.add(len)
}

/// # Write Date or Time.
///
/// This is used to quickly write date/time values to a buffer, each number
/// presented as two digits, separated by `sep`.
///
/// ## Safety
///
/// The pointer must have 8 bytes available, and hours, minutes, and seconds
/// must all be in valid ranges or undefined things will happen.
pub unsafe fn write_time(buf: *mut u8, n1: u8, n2: u8, n3: u8, sep: u8) {
	write_u8_2(buf, n1);
	ptr::write(buf.add(2), sep);
	write_u8_2(buf.add(3), n2);
	ptr::write(buf.add(5), sep);
	write_u8_2(buf.add(6), n3);
}

/// # Write `u8` as ASCII.
///
/// This method references a quick lookup table to efficiently write a number
/// between `0..=255` to a buffer in string format.
///
/// ## Safety
///
/// This will write between 1 and 3 bytes to a mutable pointer. That pointer
/// must be valid and sized correctly or undefined things will happen.
pub unsafe fn write_u8(buf: *mut u8, num: u8) -> usize {
	if num < 10 {
		ptr::copy_nonoverlapping(
			crate::NUMD.as_ptr().add(num as usize),
			buf,
			1
		);
		1
	}
	else if num < 100 {
		ptr::copy_nonoverlapping(
			crate::NUMDD.as_ptr().add((num << 1) as usize),
			buf,
			2
		);
		2
	}
	else {
		ptr::copy_nonoverlapping(
			crate::NUMDDD.as_ptr().add((num - 100) as usize * 3),
			buf,
			3
		);
		3
	}
}

#[inline]
/// # Write 2 Digits.
///
/// This will always write two digits to the pointer, zero-padding on the left
/// as necessary.
///
/// ## Safety
///
/// The number must be in `0..=99`, and the pointer must be allocated for two
/// bytes, or undefined things will happen.
pub unsafe fn write_u8_2(buf: *mut u8, num: u8) {
	ptr::copy_nonoverlapping(
		crate::NUMDD.as_ptr().add((num << 1) as usize),
		buf,
		2
	);
}

#[allow(clippy::integer_division)]
/// # Write 3 Digits.
///
/// This will always write three digits to the pointer, zero-padding on the
/// left as necessary.
///
/// ## Safety
///
/// The number must be in `0..=999`, and the pointer must be allocated for
/// three bytes, or undefined things will happen.
pub unsafe fn write_u8_3(buf: *mut u8, num: u16) {
	if num >= 100 {
		ptr::copy_nonoverlapping(
			crate::NUMDDD.as_ptr().add((num - 100) as usize * 3),
			buf,
			3
		);
	}
	else {
		ptr::copy_nonoverlapping(
			crate::NUMD.as_ptr().add((num / 100) as usize),
			buf,
			1
		);
		write_u8_2(buf.add(1), (num % 100) as u8);
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_write_datetime() {
		let mut buf = [50, 48, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0];
		unsafe {
			write_time(buf.as_mut_ptr().add(2), 20, 9, 18, b'-');
			write_time(buf.as_mut_ptr().add(11), 18, 37, 5, b':');
		}

		assert_eq!(buf, *b"2020-09-18 18:37:05");
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

		// Test possible uninit space.
		test = vec![1, 2, 3, 4];
		vec_resize_at(&mut test, 2, 10);
		assert_eq!(
			test,
			vec![1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4],
		);
	}
}
