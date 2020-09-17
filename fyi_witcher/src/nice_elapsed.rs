/*!
# FYI Witcher: "Nice" Elapsed
*/

use crate::utility;
use std::{
	fmt,
	mem::{
		self,
		MaybeUninit,
	},
	ops::Deref,
	ptr,
};



/// # Helper: Generate Impl
macro_rules! elapsed_from {
	($type:ty) => {
		impl From<$type> for NiceElapsed {
			#[allow(clippy::integer_division)]
			fn from(num: $type) -> Self {
				// Nothing!
				if 0 == num { Self::min() }
				// Hours, and maybe minutes and/or seconds.
				else if num < 86400 {
					let [h, m, s] = utility::hms_u32(num as u32);
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
/// use fyi_witcher::NiceElapsed;
/// assert_eq!(
///     NiceElapsed::from(61).as_str(),
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
	#[allow(clippy::integer_division)]
	fn from(num: u32) -> Self {
		// Nothing!
		if 0 == num { Self::min() }
		// Hours, and maybe minutes and/or seconds.
		else if num < 86400 {
			let [h, m, s] = utility::hms_u32(num);
			unsafe { Self::from_hms(h, m, s) }
		}
		// We're into days, which we don't do.
		else { Self::max() }
	}
}

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
	/// Because `NiceInt` does not support days, this is equivalent to `86400`,
	/// which comes out to `>1 day`.
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
	/// All numbers must be — but should be — less than 99 or undefined things
	/// may happen.
	unsafe fn from_hms(h: u8, m: u8, s: u8) -> Self {
		use fyi_msg::utility::write_u8;

		let mut buf = [MaybeUninit::<u8>::uninit(); 36];
		let dst = buf.as_mut_ptr() as *mut u8;
		let count: u8 = h.ne(&0) as u8 + m.ne(&0) as u8 + s.ne(&0) as u8;
		let mut len: usize = 0;

		// Hours.
		if h > 0 {
			len += write_u8(dst, h);
			if h == 1 {
				ptr::copy_nonoverlapping(b" hour".as_ptr(), dst.add(len), 5);
				len += 5;
			}
			else {
				ptr::copy_nonoverlapping(b" hours".as_ptr(), dst.add(len), 6);
				len += 6;
			}

			if 3 == count {
				ptr::copy_nonoverlapping(b", ".as_ptr(), dst.add(len), 2);
				len += 2;
			}
			else if 2 == count {
				ptr::copy_nonoverlapping(b" and ".as_ptr(), dst.add(len), 5);
				len += 5;
			}
		}

		// Minutes.
		if m > 0 {
			len += write_u8(dst.add(len), m);
			if m == 1 {
				ptr::copy_nonoverlapping(b" minute".as_ptr(), dst.add(len), 7);
				len += 7;
			}
			else {
				ptr::copy_nonoverlapping(b" minutes".as_ptr(), dst.add(len), 8);
				len += 8;
			}

			if 3 == count {
				ptr::copy_nonoverlapping(b", and ".as_ptr(), dst.add(len), 6);
				len += 6;
			}
			else if 2 == count && h == 0 {
				ptr::copy_nonoverlapping(b" and ".as_ptr(), dst.add(len), 5);
				len += 5;
			}
		}

		// Seconds.
		if s > 0 {
			len += write_u8(dst.add(len), s);
			if s == 1 {
				ptr::copy_nonoverlapping(b" second".as_ptr(), dst.add(len), 7);
				len += 7;
			}
			else {
				ptr::copy_nonoverlapping(b" seconds".as_ptr(), dst.add(len), 8);
				len += 8;
			}
		}

		// Put it all together!
		Self {
			inner: mem::transmute::<_, [u8; 36]>(buf),
			len
		}
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
