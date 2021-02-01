/*!
# FYI Num

This crate contains some numeric helpers, namely around decimal/byte conversion
and formatting.

The most generally useful member of this library is [`NiceInt`], which converts
`u8`, `u16`, `u32`, `usize`, and `u64` values into formatted (comma-separated)
ASCII strings very quickly.



## Stability

Release versions of this library should be in a working state, but as this
project is under perpetual development, code might change from version to
version.
*/

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



mod nice_elapsed;
mod nice_int;

pub use nice_elapsed::NiceElapsed;
pub use nice_int::NiceInt;



/// # Decimals, 00-99.
pub(crate) static DOUBLE: &[u8; 200] = b"\
	0001020304050607080910111213141516171819\
	2021222324252627282930313233343536373839\
	4041424344454647484950515253545556575859\
	6061626364656667686970717273747576777879\
	8081828384858687888990919293949596979899";



/// # Write u8.
///
/// This will quickly write a `u8` number as a UTF-8 byte slice to the provided
/// pointer.
///
/// ## Safety
///
/// The pointer must have enough space for the value, i.e. 1-3 digits.
pub unsafe fn write_u8(buf: *mut u8, num: u8) {
	use std::ptr;

	if num > 99 {
		let (div, rem) = num_integer::div_mod_floor(usize::from(num), 100);
		let ptr = DOUBLE.as_ptr();
		ptr::copy_nonoverlapping(ptr.add((div << 1) + 1), buf, 1);
		ptr::copy_nonoverlapping(ptr.add(rem << 1), buf.add(1), 2);
	}
	else if num > 9 {
		ptr::copy_nonoverlapping(DOUBLE.as_ptr().add(usize::from(num) << 1), buf, 2);
	}
	else {
		ptr::copy_nonoverlapping(DOUBLE.as_ptr().add((usize::from(num) << 1) + 1), buf, 1);
	}
}

/// # Write Time.
///
/// This writes HH:MM:SS to the provided pointer.
///
/// ## Safety
///
/// The pointer must have 8 bytes free or undefined things will happen.
pub unsafe fn write_time(buf: *mut u8, h: u8, m: u8, s: u8) {
	use std::ptr;

	assert!(h < 60 && m < 60 && s < 60);

	let ptr = DOUBLE.as_ptr();
	ptr::copy_nonoverlapping(ptr.add(usize::from(h) << 1), buf, 2);
	ptr::write(buf.add(2), b':');
	ptr::copy_nonoverlapping(ptr.add(usize::from(m) << 1), buf.add(3), 2);
	ptr::write(buf.add(5), b':');
	ptr::copy_nonoverlapping(ptr.add(usize::from(s) << 1), buf.add(6), 2);
}



#[cfg(test)]
mod tests {
	use super::*;
	use criterion as _;

	#[test]
	fn t_write_u8() {
		for i in 0..10 {
			let mut buf = [0_u8];
			unsafe {
				write_u8(buf.as_mut_ptr(), i);
				assert_eq!(buf, format!("{}", i).as_bytes());
			}
		}

		for i in 10..100 {
			let mut buf = [0_u8, 0_u8];
			unsafe {
				write_u8(buf.as_mut_ptr(), i);
				assert_eq!(buf, format!("{}", i).as_bytes());
			}
		}

		for i in 100..u8::MAX {
			let mut buf = [0_u8, 0_u8, 0_u8];
			unsafe {
				write_u8(buf.as_mut_ptr(), i);
				assert_eq!(buf, format!("{}", i).as_bytes());
			}
		}
	}

	#[test]
	fn t_write_time() {
		let mut buf = [0_u8; 8];
		unsafe {
			write_time(buf.as_mut_ptr(), 1, 2, 3);
			assert_eq!(buf, *b"01:02:03");
		}
	}
}
