/*!
# FYI Msg: Utility Methods
*/

use std::ptr;



/// # 48.
///
/// This is a simple mask that can be applied against a decimal between `0..10`
/// to turn it into the equivalent ASCII. This is the same thing as adding `48`
/// (for this particular range) but is minutely faster because it's bitwise!
///
/// ```no_run
/// let x: u8 = 5;
/// assert_eq!(x | MASK_U8, x + 48);
/// ```
pub const MASK_U8: u8 = 0b11_0000;

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

/// # Write ANSI Color Sequence (Bold)
///
/// This method writes a bold ANSI color sequence (e.g. `\x1b[1;38;5;2m`) directly
/// to a pointer.
///
/// ## Safety
///
/// This will write between 11 and 13 bytes to a mutable pointer. That pointer
/// must be valid and sized correctly or undefined things will happen.
pub unsafe fn write_ansi_code_bold(buf: *mut u8, num: u8) -> usize {
	// Bad Data/Overflow.
	if num == 0 {
		ptr::copy_nonoverlapping(b"\x1b[0m".as_ptr(), buf, 4);
		return 4;
	}

	// Otherwise they all start the same.
	ptr::copy_nonoverlapping(b"\x1b[1;38;5;".as_ptr(), buf, 9);

	// Add the color.
	let len: usize = write_u8(buf.add(9), num) + 9;

	// And finish off with the "m".
	ptr::write(buf.add(len), b'm');

	len + 1
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

#[inline]
/// # Write Double-Digit Time Value.
///
/// This writes a number `0..60` as ASCII-fied bytes, e.g. "00" or "13". Any
/// value over `59` is simply written as "59".
///
/// ## Safety
///
/// This writes two bytes to a mutable pointer; that pointer must be valid and
/// allocated accordingly or undefined things will happen.
pub unsafe fn write_time_dd(buf: *mut u8, num: u8) {
	write_u8_2(buf, 59.min(num))
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
	static INTS: [u8; 478] = *b"0123456789\
		100101102103104105106107108109110111112113114115116117118119120121122123124125126127128129130131132133134135136137138139140141142143144145146147148149\
		150151152153154155156157158159160161162163164165166167168169170171172173174175176177178179180181182183184185186187188189190191192193194195196197198199\
		200201202203204205206207208209210211212213214215216217218219220221222223224225226227228229230231232233234235236237238239240241242243244245246247248249\
		250251252253254255";

	if num < 10 {
		ptr::copy_nonoverlapping(INTS.as_ptr().add(num as usize), buf, 1);
		1
	}
	else if num < 100 {
		write_u8_2(buf, num);
		2
	}
	else {
		ptr::copy_nonoverlapping(INTS.as_ptr().add(10 + (num as usize - 100) * 3), buf, 3);
		3
	}
}

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
	static INTS: [u8; 200] = *b"\
		0001020304050607080910111213141516171819\
		2021222324252627282930313233343536373839\
		4041424344454647484950515253545556575859\
		6061626364656667686970717273747576777879\
		8081828384858687888990919293949596979899";

	ptr::copy_nonoverlapping(
		INTS.as_ptr().add((num << 1) as usize),
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
		if num <= 255 {
			write_u8(buf, num as u8);
		}
		else {
			ptr::write(buf, (num / 100) as u8 | MASK_U8);
			write_u8_2(buf.add(1), (num % 100) as u8);
		}
	}
	else {
		ptr::write(buf, MASK_U8);
		write_u8_2(buf.add(1), num as u8);
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_ansi_code_bold() {
		let mut buf = [0_u8; 13];
		let ptr = buf.as_mut_ptr();

		assert_eq!(unsafe { write_ansi_code_bold(ptr, 0) }, 4);
		assert_eq!(&buf[0..4], b"\x1b[0m");

		for i in 1..10 {
			assert_eq!(unsafe { write_ansi_code_bold(ptr, i) }, 11);
			assert_eq!(&buf[0..11], format!("\x1B[1;38;5;{}m", i).as_bytes());
		}

		for i in 10..100 {
			assert_eq!(unsafe { write_ansi_code_bold(ptr, i) }, 12);
			assert_eq!(&buf[0..12], format!("\x1B[1;38;5;{}m", i).as_bytes());
		}

		for i in 100..=255 {
			assert_eq!(unsafe { write_ansi_code_bold(ptr, i) }, 13);
			assert_eq!(&buf[0..13], format!("\x1B[1;38;5;{}m", i).as_bytes());
		}
	}

	#[test]
	fn t_time_format_dd() {
		// Test the supported values.
		for i in 0..=59 {
			let mut buf = [0_u8, 0_u8];
			unsafe { write_time_dd(buf.as_mut_ptr(), i); }
			assert_eq!(
				buf,
				format!("{:02}", i).as_bytes(),
				"DD for {} is incorrect: {:?}",
				i,
				buf
			);
		}

		// And make sure overflow works.
		let mut buf = [0_u8, 0_u8];
		unsafe { write_time_dd(buf.as_mut_ptr(), 60); }
		assert_eq!(buf, &b"59"[..]);
	}

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
