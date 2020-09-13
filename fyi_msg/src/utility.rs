/*!
# FYI Msg: Utility Methods
*/

use std::ptr;



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
	// 0..4     Reset.
	// 4..13    Opener.
	// 13..14   m.
	static ANSI: [u8; 14] = *b"\x1b[0m\x1b[1;38;5;m";

	// Bad Data/Overflow.
	if num == 0 {
		ptr::copy_nonoverlapping(ANSI.as_ptr(), buf, 4);
		return 4;
	}

	// Grab the pointer.
	let ptr = ANSI.as_ptr();

	// Otherwise they all start the same.
	ptr::copy_nonoverlapping(ptr.add(4), buf, 9);

	// Add the color.
	let len: usize = write_u8(buf.add(9), num) + 9;

	// And finish off with the "m".
	ptr::copy_nonoverlapping(ptr.add(13), buf.add(len), 1);

	len + 1
}

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
	if num < 10 {
		ptr::write(buf, 48_u8);
		write_u8(buf.add(1), num);
	}
	else {
		write_u8(buf, 59.min(num));
	}
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
	// 0..10    0, 1, 2, ... 9.
	// 10..190  10, 11, 12 ... 99.
	// 190..658 100, 101, ... 255.
	static INTS: [u8; 658] = *b"0123456789\
		10111213141516171819202122232425262728293031323334353637383940414243444546474849\
		5051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899\
		100101102103104105106107108109110111112113114115116117118119120121122123124125126127128129130131132133134135136137138139140141142143144145146147148149\
		150151152153154155156157158159160161162163164165166167168169170171172173174175176177178179180181182183184185186187188189190191192193194195196197198199\
		200201202203204205206207208209210211212213214215216217218219220221222223224225226227228229230231232233234235236237238239240241242243244245246247248249\
		250251252253254255";

	if num < 10 {
		ptr::copy_nonoverlapping(INTS.as_ptr().add(num as usize), buf, 1);
		1
	}
	else if num < 100 {
		ptr::copy_nonoverlapping(INTS.as_ptr().add((10 + (num - 10) * 2) as usize), buf, 2);
		2
	}
	else {
		ptr::copy_nonoverlapping(INTS.as_ptr().add(190 + (num as usize - 100) * 3), buf, 3);
		3
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
