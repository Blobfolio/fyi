/*!
# FYI Num: "Nice" Integer
*/

pub(super) mod nice_u8;
pub(super) mod nice_u16;
pub(super) mod nice_u32;
pub(super) mod nice_u64;
pub(super) mod nice_percent;

use crate::DOUBLE;
use std::ptr;


#[macro_export]
/// # Helper: Generic NiceU* traits.
macro_rules! impl_nice_int {
	($lhs:ty) => {
		impl std::ops::Deref for $lhs {
			type Target = [u8];
			#[inline]
			fn deref(&self) -> &Self::Target { &self.inner[self.from..] }
		}

		impl std::fmt::Display for $lhs {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.write_str(self.as_str())
			}
		}

		/// ## Casting.
		///
		/// This section provides methods for converting instances into other
		/// types.
		///
		/// Note: this struct can also be dereferenced to `&[u8]`.
		impl $lhs {
			#[must_use]
			#[inline]
			/// # As Bytes.
			///
			/// Return the value as a byte string.
			pub fn as_bytes(&self) -> &[u8] { self }

			#[must_use]
			#[inline]
			/// # As Str.
			///
			/// Return the value as a string slice.
			pub fn as_str(&self) -> &str {
				unsafe { std::str::from_utf8_unchecked(self) }
			}
		}
	};
}



/// # Write `u8` x 3
///
/// ## Safety
///
/// The destination pointer must have at least 3 bytes free or undefined
/// things may happen!
pub(super) unsafe fn write_u8_3(buf: *mut u8, num: usize) {
	let (div, rem) = num_integer::div_mod_floor(num, 100);
	let ptr = DOUBLE.as_ptr();
	ptr::copy_nonoverlapping(ptr.add((div << 1) + 1), buf, 1);
	ptr::copy_nonoverlapping(ptr.add(rem << 1), buf.add(1), 2);
}

/// # Write `u8` x 2
///
/// ## Safety
///
/// The destination pointer must have at least 2 bytes free or undefined
/// things may happen!
pub(super) unsafe fn write_u8_2(buf: *mut u8, num: usize) {
	ptr::copy_nonoverlapping(DOUBLE.as_ptr().add(num << 1), buf, 2);
}

/// # Write `u8` x 1
///
/// ## Safety
///
/// The destination pointer must have at least 1 byte free or undefined
/// things may happen!
pub(super) unsafe fn write_u8_1(buf: *mut u8, num: usize) {
	ptr::copy_nonoverlapping(DOUBLE.as_ptr().add((num << 1) + 1), buf, 1);
}
