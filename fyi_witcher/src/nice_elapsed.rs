/*!
# FYI Witcher: "Nice" Elapsed
*/

use std::{
	fmt,
	ops::Deref,
};



/// # Helper: Generate Impl
macro_rules! elapsed_from {
	($type:ty) => {
		impl From<$type> for NiceElapsed {
			fn from(num: $type) -> Self {
				// Nothing!
				if 0 == num { Self::min() }
				// Just seconds.
				else if num < 60 { Self::from_s(num) }
				// Minutes and maybe seconds.
				else if num < 3600 {
					// Break up the parts.
					let m: $type = num_integer::div_floor(num, 60);
					let s: $type = num - m * 60;

					// Minutes and seconds.
					if s > 0 { Self::from_ms(m, s) }
					// Just minutes.
					else { Self::from_m(m) }
				}
				// Hours, and maybe minutes and/or seconds.
				else if num < 86400 {
					// Break up the parts.
					let h: $type = num_integer::div_floor(num, 3600);
					let mut s: $type = num - h * 3600;
					let mut m: $type = 0;
					if s >= 60 {
						m = num_integer::div_floor(s, 60);
						s -= m * 60;
					}

					// Figure out which pieces need adding.
					match (m == 0, s == 0) {
						// All three parts.
						(false, false) => Self::from_hms(h, m, s),
						// Hours and Minutes.
						(false, true) => Self::from_hm(h, m),
						// Hours and Seconds.
						(true, false) => Self::from_hs(h, s),
						// Only hours.
						(true, true) => Self::from_h(h),
					}
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
	fn default() -> Self {
		Self {
			inner: [0; 36],
			len: 0,
		}
	}
}

impl Deref for NiceElapsed {
	type Target = [u8];
	fn deref(&self) -> &Self::Target { self.as_bytes() }
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

elapsed_from!(usize);
elapsed_from!(u32);
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

	/// # Write Number.
	///
	/// This writes any old number to the buffer.
	fn write_int<N> (&mut self, num: N)
	where N: itoa::Integer {
		self.len += itoa::write(&mut self.inner[self.len..], num).unwrap();
	}

	/// # Write Bytes.
	///
	/// This writes a byte string to the buffer (e.g. a unit).
	fn write_bytes(&mut self, buf: &[u8]) {
		let end: usize = self.len + buf.len();
		self.inner[self.len..end].copy_from_slice(buf);
		self.len = end;
	}

	/// # Write Hour And.
	///
	/// This writes "x hour(s) and", which comes up in a few combinations.
	fn write_hour_and<N>(&mut self, h: N)
	where N: itoa::Integer + num_traits::One + PartialEq {
		if h.is_one() {
			self.write_bytes(b"1 hour and ");
		}
		else {
			self.write_int(h);
			self.write_bytes(b" hours and ");
		}
	}

	/// # Write Minutes.
	///
	/// This writes "x minute(s)", which comes up in a few combinations.
	fn write_minutes<N>(&mut self, m: N)
	where N: itoa::Integer + num_traits::One + PartialEq {
		if m.is_one() {
			self.write_bytes(b"1 minute");
		}
		else {
			self.write_int(m);
			self.write_bytes(b" minutes");
		}
	}

	/// # Write Seconds.
	///
	/// This writes "x second(s)", which comes up in a few combinations.
	fn write_seconds<N>(&mut self, s: N)
	where N: itoa::Integer + num_traits::One + PartialEq {
		if s.is_one() {
			self.write_bytes(b"1 second");
		}
		else {
			self.write_int(s);
			self.write_bytes(b" seconds");
		}
	}

	/// # From Hours, Minutes, Seconds.
	///
	/// Fill the buffer with all three units (hours, minutes, and seconds).
	fn from_hms<N> (h: N, m: N, s: N) -> Self
	where N: itoa::Integer + num_traits::One + PartialEq {
		let mut out = Self::default();

		// Hours.
		if h.is_one() {
			out.write_bytes(b"1 hour, ");
		}
		else {
			out.write_int(h);
			out.write_bytes(b" hours, ");
		}

		// Minutes.
		if m.is_one() {
			out.write_bytes(b"1 minute, and ");
		}
		else {
			out.write_int(m);
			out.write_bytes(b" minutes, and ");
		}

		// Seconds.
		out.write_seconds(s);

		out
	}

	/// # From Hours, Minutes.
	///
	/// Fill the buffer with two units, hours and minutes.
	fn from_hm<N>(h: N, m: N) -> Self
	where N: itoa::Integer + num_traits::One + PartialEq {
		let mut out = Self::default();

		out.write_hour_and(h);
		out.write_minutes(m);

		out
	}

	/// # From Hours, Seconds.
	///
	/// Fill the buffer with two units, hours and seconds.
	fn from_hs<N>(h: N, s: N) -> Self
	where N: itoa::Integer + num_traits::One + PartialEq {
		let mut out = Self::default();

		out.write_hour_and(h);
		out.write_seconds(s);

		out
	}

	/// # From Hours.
	///
	/// Fill the buffer using only hours.
	fn from_h<N>(h: N) -> Self
	where N: itoa::Integer + num_traits::One + PartialEq {
		let mut out = Self::default();

		if h.is_one() {
			out.write_bytes(b"1 hour");
		}
		else {
			out.write_int(h);
			out.write_bytes(b" hours");
		}

		out
	}

	/// # From Minutes, Seconds.
	///
	/// Fill the buffer using two units, minutes and seconds.
	fn from_ms<N>(m: N, s: N) -> Self
	where N: itoa::Integer + num_traits::One + PartialEq {
		let mut out = Self::default();

		if m.is_one() {
			out.write_bytes(b"1 minute and ");
		}
		else {
			out.write_int(m);
			out.write_bytes(b" minutes and ");
		}

		out.write_seconds(s);

		out
	}

	/// # From Minutes.
	///
	/// Fill the buffer using only minutes.
	fn from_m<N>(m: N) -> Self
	where N: itoa::Integer + num_traits::One + PartialEq {
		let mut out = Self::default();
		out.write_minutes(m);
		out
	}

	/// # From Seconds.
	///
	/// Fill the buffer using only seconds.
	fn from_s<N>(s: N) -> Self
	where N: itoa::Integer + num_traits::One + PartialEq {
		let mut out = Self::default();
		out.write_seconds(s);
		out
	}

	#[must_use]
	/// # As Bytes.
	///
	/// Return the nice value as a byte string.
	pub fn as_bytes(&self) -> &[u8] { &self.inner[0..self.len] }

	#[must_use]
	/// # As Str.
	///
	/// Return the nice value as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.inner[0..self.len]) }
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
