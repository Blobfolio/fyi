/*!
# FYI Progress: "Nice" Elapsed

Convert seconds into an oxford-joined byte string like, "3 hours, 2 minutes,
and 1 second".

Note: days are unsupported, or more specifically, any value over 23:59:59 will
return ">1 day".
*/

use std::{
	fmt,
	ops::Deref,
};



/// Helper: Append a number and label to the buffer.
macro_rules! ne_push {
	($lhs:ident, $num:expr, $one:expr, $many:expr) => {
		// Singular shortcut.
		if 1 == $num {
			let end: usize = $lhs.len + $one.len();
			$lhs.inner[$lhs.len..end].copy_from_slice($one);
			$lhs.len = end;
		}
		// The rest!
		else {
			// Write the number.
			$lhs.len += itoa::write(&mut $lhs.inner[$lhs.len..], $num).unwrap();

			// Write the label.
			let end: usize = $lhs.len + $many.len();
			$lhs.inner[$lhs.len..end].copy_from_slice($many);
			$lhs.len = end;
		}
	};
}



#[derive(Clone, Copy)]
/// Nice Elapsed
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

	fn deref(&self) -> &Self::Target {
		&self.inner[0..self.len]
	}
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
		f.write_str(unsafe { std::str::from_utf8_unchecked(&*self) })
	}
}

impl From<u32> for NiceElapsed {
	fn from(num: u32) -> Self {
		// Nothing!
		if 0 == num { Self::min() }
		// Just seconds.
		else if num < 60 { Self::from_s(num) }
		// Minutes and maybe seconds.
		else if num < 3600 {
			// Break up the parts.
			let m: u32 = num_integer::div_floor(num, 60);
			let s: u32 = num - m * 60;

			// Minutes and seconds.
			if s > 0 { Self::from_ms(m, s) }
			// Just minutes.
			else { Self::from_m(m) }
		}
		// Hours, and maybe minutes and/or seconds.
		else if num < 86400 {
			// Break up the parts.
			let h: u32 = num_integer::div_floor(num, 3600);
			let mut s: u32 = num - h * 3600;
			let mut m: u32 = 0;
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



impl NiceElapsed {
	#[must_use]
	/// Minimum Value
	pub const fn min() -> Self {
		Self {
			//       0   •    s    e   c    o    n    d    s
			inner: [48, 32, 115, 101, 99, 111, 110, 100, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			len: 9,
		}
	}

	#[must_use]
	/// Maximum Value
	pub const fn max() -> Self {
		Self {
			//       >   1   •    d   a    y
			inner: [62, 49, 32, 100, 97, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			len: 6,
		}
	}

	/// From Hours, Minutes, Seconds.
	fn from_hms(h: u32, m: u32, s: u32) -> Self {
		let mut out = Self::default();

		// Write hours.
		ne_push!(
			out, h,
			// 1   •    h    o    u    r   ,   •
			&[49, 32, 104, 111, 117, 114, 44, 32],
			// •    h    o    u    r    s   ,   •
			&[32, 104, 111, 117, 114, 115, 44, 32]
		);

		// Write minutes.
		ne_push!(
			out, m,
			// 1   •    m    i    n    u    t    e   ,   •   a    n    d   •
			&[49, 32, 109, 105, 110, 117, 116, 101, 44, 32, 97, 110, 100, 32],
			// •    m    i    n    u    t    e    s   ,   •   a    n    d   •
			&[32, 109, 105, 110, 117, 116, 101, 115, 44, 32, 97, 110, 100, 32]
		);

		// Write seconds.
		ne_push!(
			out, s,
			// 1   •    s    e   c    o    n    d
			&[49, 32, 115, 101, 99, 111, 110, 100],
			// •    s    e   c    o    n    d    s
			&[32, 115, 101, 99, 111, 110, 100, 115]
		);

		out
	}

	/// From Hours, Minutes.
	fn from_hm(h: u32, m: u32) -> Self {
		let mut out = Self::default();

		// Write hours.
		ne_push!(
			out, h,
			// 1   •    h    o    u    r   •   a    n    d   •
			&[49, 32, 104, 111, 117, 114, 32, 97, 110, 100, 32],
			// •    h    o    u    r    s   •   a    n    d   •
			&[32, 104, 111, 117, 114, 115, 32, 97, 110, 100, 32]
		);

		// Write minutes.
		ne_push!(
			out, m,
			// 1   •    m    i    n    u    t    e
			&[49, 32, 109, 105, 110, 117, 116, 101],
			// •    m    i    n    u    t    e    s
			&[32, 109, 105, 110, 117, 116, 101, 115]
		);

		out
	}

	/// From Hours, Seconds.
	fn from_hs(h: u32, s: u32) -> Self {
		let mut out = Self::default();

		// Write hours.
		ne_push!(
			out, h,
			// 1   •    h    o    u    r   •   a    n    d   •
			&[49, 32, 104, 111, 117, 114, 32, 97, 110, 100, 32],
			// •    h    o    u    r    s   •   a    n    d   •
			&[32, 104, 111, 117, 114, 115, 32, 97, 110, 100, 32]
		);

		// Write seconds.
		ne_push!(
			out, s,
			// 1   •    s    e   c    o    n    d
			&[49, 32, 115, 101, 99, 111, 110, 100],
			// •    s    e   c    o    n    d    s
			&[32, 115, 101, 99, 111, 110, 100, 115]
		);

		out
	}

	/// From Hours.
	fn from_h(h: u32) -> Self {
		let mut out = Self::default();

		// Write hours.
		ne_push!(
			out, h,
			// 1   •    h    o    u    r
			&[49, 32, 104, 111, 117, 114],
			// •    h    o    u    r    s
			&[32, 104, 111, 117, 114, 115]
		);

		out
	}

	/// From Minutes, Seconds.
	fn from_ms(m: u32, s: u32) -> Self {
		let mut out = Self::default();

		// Write minutes.
		ne_push!(
			out, m,
			// 1   •    m    i    n    u    t    e   •   a    n    d   •
			&[49, 32, 109, 105, 110, 117, 116, 101, 32, 97, 110, 100, 32],
			// •    m    i    n    u    t    e    s   •   a    n    d   •
			&[32, 109, 105, 110, 117, 116, 101, 115, 32, 97, 110, 100, 32]
		);

		// Write seconds.
		ne_push!(
			out, s,
			// 1   •    s    e   c    o    n    d
			&[49, 32, 115, 101, 99, 111, 110, 100],
			// •    s    e   c    o    n    d    s
			&[32, 115, 101, 99, 111, 110, 100, 115]
		);

		out
	}

	/// From Minutes.
	fn from_m(m: u32) -> Self {
		let mut out = Self::default();

		// Write minutes.
		ne_push!(
			out, m,
			// 1   •    m    i    n    u    t    e
			&[49, 32, 109, 105, 110, 117, 116, 101],
			// •    m    i    n    u    t    e    s
			&[32, 109, 105, 110, 117, 116, 101, 115]
		);

		out
	}

	/// From Seconds.
	fn from_s(s: u32) -> Self {
		let mut out = Self::default();

		// Write seconds.
		ne_push!(
			out, s,
			// 1   •    s    e   c    o    n    d
			&[49, 32, 115, 101, 99, 111, 110, 100],
			// •    s    e   c    o    n    d    s
			&[32, 115, 101, 99, 111, 110, 100, 115]
		);

		out
	}

	#[must_use]
	/// As String.
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
