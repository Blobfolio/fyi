/*!
# FYI Witcher: "Nice" Elapsed
*/

use std::{
	fmt,
	mem::{
		self,
		MaybeUninit,
	},
	ops::Deref,
};



/// # Helper: Generate Impl
macro_rules! elapsed_from {
	($type:ty) => {
		impl From<$type> for NiceElapsed {
			fn from(num: $type) -> Self {
				// Nothing!
				if 0 == num { Self::min() }
				// Hours, and maybe minutes and/or seconds.
				else if num < 86400 {
					let [h, m, s] = Self::hms(num as u32);
					unsafe { Self::from_hms(h, m, s) }
				}
				// We're into days, which we don't do.
				else { Self::max() }
			}
		}
	};
}



#[derive(Clone, Copy)]
/// This is a very simple struct for efficiently converting a given number of
/// seconds (`u32`) into a nice, human-readable Oxford-joined byte string, like
/// `3 hours, 2 minutes, and 1 second`.
///
/// Note: days are unsupported, or more specifically, any value over `23:59:59`
/// (or `86400+` seconds) will return a fixed value of `>1 day`.
///
/// ## Examples
///
/// ```no_run
/// use fyi_num::NiceElapsed;
/// assert_eq!(
///     NiceElapsed::from(61_u32).as_str(),
///     "1 minute and 1 second"
/// );
/// ```
pub struct NiceElapsed {
	inner: [u8; 36],
	len: usize,
}

impl Default for NiceElapsed {
	#[inline]
	fn default() -> Self {
		Self {
			inner: [0; 36],
			len: 0,
		}
	}
}

impl Deref for NiceElapsed {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.inner[0..self.len] }
}

impl fmt::Debug for NiceElapsed {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("NiceElapsed")
		 .field("inner", &self.inner.to_vec())
		 .field("len", &self.len)
		 .finish()
	}
}

impl fmt::Display for NiceElapsed {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<u32> for NiceElapsed {
	fn from(num: u32) -> Self {
		// Nothing!
		if 0 == num { Self::min() }
		// Hours, and maybe minutes and/or seconds.
		else if num < 86400 {
			let [h, m, s] = Self::hms(num);
			unsafe { Self::from_hms(h, m, s) }
		}
		// We're into days, which we don't do.
		else { Self::max() }
	}
}

// These all work the same way.
elapsed_from!(usize);
elapsed_from!(u64);
elapsed_from!(u128);

impl NiceElapsed {
	#[must_use]
	/// # Minimum Value
	///
	/// We can save some processing time by hard-coding the value for `0`,
	/// which comes out to `0 seconds`.
	pub const fn min() -> Self {
		Self {
			//       0   •    s    e   c    o    n    d    s
			inner: [48, 32, 115, 101, 99, 111, 110, 100, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			len: 9,
		}
	}

	#[must_use]
	/// # Maximum Value
	///
	/// We can save some processing time by hard-coding the maximum value.
	/// Because `NiceElapsed` does not support days, this is equivalent to
	/// `86400`, which comes out to `>1 day`.
	pub const fn max() -> Self {
		Self {
			//       >   1   •    d   a    y
			inner: [62, 49, 32, 100, 97, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			len: 6,
		}
	}

	/// # From Hours, Minutes, Seconds.
	///
	/// Fill the buffer with the appropriate output given all the different bits.
	///
	/// ## Safety
	///
	/// All numbers must be — but should be — less than 60 or undefined things
	/// may happen.
	unsafe fn from_hms(h: u8, m: u8, s: u8) -> Self {
		let mut buf = [MaybeUninit::<u8>::uninit(); 36];
		let count: u8 = h.ne(&0) as u8 + m.ne(&0) as u8 + s.ne(&0) as u8;

		let len: usize = {
			let mut dst = buf.as_mut_ptr() as *mut u8;

			// Hours.
			if h > 0 {
				dst = write_u8_advance(dst, h);
				if h == 1 {
					dst = write_advance(dst, b" hour".as_ptr(), 5);
				}
				else {
					dst = write_advance(dst, b" hours".as_ptr(), 6);
				}

				if 3 == count {
					dst = write_advance(dst, b", ".as_ptr(), 2);
				}
				else if 2 == count {
					dst = write_advance(dst, b" and ".as_ptr(), 5);
				}
			}

			// Minutes.
			if m > 0 {
				dst = write_u8_advance(dst, m);
				if m == 1 {
					dst = write_advance(dst, b" minute".as_ptr(), 7);
				}
				else {
					dst = write_advance(dst, b" minutes".as_ptr(), 8);
				}

				if 3 == count {
					dst = write_advance(dst, b", and ".as_ptr(), 6);
				}
				else if 2 == count && h == 0 {
					dst = write_advance(dst, b" and ".as_ptr(), 5);
				}
			}

			// Seconds.
			if s > 0 {
				dst = write_u8_advance(dst, s);
				if s == 1 {
					dst = write_advance(dst, b" second".as_ptr(), 7);
				}
				else {
					dst = write_advance(dst, b" seconds".as_ptr(), 8);
				}
			}

			dst.offset_from(buf.as_ptr() as *const u8) as usize
		};

		// Put it all together!
		Self {
			inner: mem::transmute::<_, [u8; 36]>(buf),
			len
		}
	}

	#[must_use]
	/// # Time Chunks.
	///
	/// This method splits seconds into hours, minutes, and seconds. Days are not
	/// supported; the maximum return value is `(23, 59, 59)`.
	pub const fn hms(mut num: u32) -> [u8; 3] {
		if num < 60 { [0, 0, num as u8] }
		else if num < 86399 {
			let mut buf = [0_u8; 3];

			if num >= 3600 {
				buf[0] = ((num * 0x91A3) >> 27) as u8;
				num -= buf[0] as u32 * 3600;
			}
			if num >= 60 {
				buf[1] = ((num * 0x889) >> 17) as u8;
				buf[2] = (num - buf[1] as u32 * 60) as u8;
			}
			else if num > 0 { buf[2] = num as u8; }

			buf
		}
		else { [23, 59, 59] }
	}

	#[must_use]
	#[inline]
	/// # As Bytes.
	///
	/// Return the nice value as a byte string.
	pub fn as_bytes(&self) -> &[u8] { self }

	#[must_use]
	#[inline]
	/// # As Str.
	///
	/// Return the nice value as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
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
unsafe fn write_advance(dst: *mut u8, src: *const u8, len: usize) -> *mut u8 {
	std::ptr::copy_nonoverlapping(src, dst, len);
	dst.add(len)
}

/// # Write u8.
///
/// This will quickly write a `u8` number as a UTF-8 byte slice to the provided
/// pointer.
///
/// ## Safety
///
/// The pointer must have enough space for the value, i.e. 1-3 digits.
unsafe fn write_u8_advance(buf: *mut u8, num: u8) -> *mut u8 {
	use std::ptr;

	if num > 99 {
		let (div, rem) = num_integer::div_mod_floor(usize::from(num), 100);
		let ptr = crate::DOUBLE.as_ptr();
		ptr::copy_nonoverlapping(ptr.add((div << 1) + 1), buf, 1);
		ptr::copy_nonoverlapping(ptr.add(rem << 1), buf.add(1), 2);
		buf.add(3)
	}
	else if num > 9 {
		ptr::copy_nonoverlapping(crate::DOUBLE.as_ptr().add(usize::from(num) << 1), buf, 2);
		buf.add(2)
	}
	else {
		ptr::copy_nonoverlapping(crate::DOUBLE.as_ptr().add((usize::from(num) << 1) + 1), buf, 1);
		buf.add(1)
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_from() {
		_from(0, "0 seconds");
		_from(1, "1 second");
		_from(50, "50 seconds");

		_from(60, "1 minute");
		_from(61, "1 minute and 1 second");
		_from(100, "1 minute and 40 seconds");
		_from(2101, "35 minutes and 1 second");
		_from(2121, "35 minutes and 21 seconds");

		_from(3600, "1 hour");
		_from(3601, "1 hour and 1 second");
		_from(3602, "1 hour and 2 seconds");
		_from(3660, "1 hour and 1 minute");
		_from(3661, "1 hour, 1 minute, and 1 second");
		_from(3662, "1 hour, 1 minute, and 2 seconds");
		_from(3720, "1 hour and 2 minutes");
		_from(3721, "1 hour, 2 minutes, and 1 second");
		_from(3723, "1 hour, 2 minutes, and 3 seconds");
		_from(36001, "10 hours and 1 second");
		_from(36015, "10 hours and 15 seconds");
		_from(36060, "10 hours and 1 minute");
		_from(37732, "10 hours, 28 minutes, and 52 seconds");
		_from(37740, "10 hours and 29 minutes");

		_from(428390, ">1 day");
	}

	fn _from(num: u32, expected: &str) {
		assert_eq!(
			&*NiceElapsed::from(num),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}
}
