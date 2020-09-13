/*!
# FYI Witcher: "Nice" Integer
*/

use std::{
	fmt,
	mem::{
		self,
		MaybeUninit,
	},
	ops::Deref,
	ptr,
};



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// `NiceInt` provides a quick way to convert an integer — any unsigned value
/// under a trillion — into a formatted byte string for e.g. printing. Commas
/// are added for every thousand.
///
/// That's it!
///
/// ## Examples
///
/// ```no_run
/// use fyi_witcher::NiceInt;
/// assert_eq!(
///     NiceInt::from(33231).as_str(),
///     "33,231"
/// );
pub struct NiceInt {
	inner: [u8; 15],
	len: usize,
}

impl Deref for NiceInt {
	type Target = [u8];
	fn deref(&self) -> &Self::Target { &self.inner[..self.len] }
}

impl Default for NiceInt {
	fn default() -> Self {
		Self {
			inner: [48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			len: 1,
		}
	}
}

impl fmt::Display for NiceInt {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<u8> for NiceInt {
	#[allow(clippy::integer_division)]
	fn from(num: u8) -> Self {
		use fyi_msg::utility::write_u8;
		unsafe {
			let mut buf = [MaybeUninit::<u8>::uninit(); 15];
			let len: usize = write_u8(buf.as_mut_ptr() as *mut u8, num);
			Self {
				inner: mem::transmute::<_, [u8; 15]>(buf),
				len
			}
		}
	}
}

impl From<u16> for NiceInt {
	fn from(num: u16) -> Self {
		if num <= 255 { Self::from(num as u8) }
		else {
			unsafe {
				let mut buf = [MaybeUninit::<u8>::uninit(); 15];

				let len = write_16_from(
					num,
					if num >= 10_000 { 10_000_u16 }
						else if num >= 1_000 { 1_000_u16 }
						else { 100_u16 },
					buf.as_mut_ptr() as *mut u8,
					0
				);

				Self {
					inner: mem::transmute::<_, [u8; 15]>(buf),
					len
				}
			}
		}
	}
}

impl From<u32> for NiceInt {
	fn from(num: u32) -> Self {
		if num <= 255 { Self::from(num as u8) }
		else if num <= 65_535 { Self::from(num as u16) }
		else {
			unsafe {
				let mut buf = [MaybeUninit::<u8>::uninit(); 15];

				let len = write_32_from(
					num,
					if num >= 1_000_000_000 { 1_000_000_000_u32 }
						else if num >= 100_000_000 { 100_000_000_u32 }
						else if num >= 10_000_000 { 10_000_000_u32 }
						else if num >= 1_000_000 { 1_000_000_u32 }
						else if num >= 100_000 { 100_000_u32 }
						else { 10_000_u32 },
					buf.as_mut_ptr() as *mut u8,
					0
				);

				Self {
					inner: mem::transmute::<_, [u8; 15]>(buf),
					len
				}
			}
		}
	}
}

impl From<u64> for NiceInt {
	fn from(num: u64) -> Self {
		if num <= 255 { Self::from(num as u8) }
		else if num <= 65_535 { Self::from(num as u16) }
		else if num <= 4_294_967_295 { Self::from(num as u32) }
		else {
			unsafe {
				let mut buf = [MaybeUninit::<u8>::uninit(); 15];

				let len = write_64_from(
					999_999_999_999.min(num),
					if num >= 100_000_000_000 { 100_000_000_000_u64 }
						else if num >= 10_000_000_000 { 10_000_000_000_u64 }
						else { 1_000_000_000_u64 },
					buf.as_mut_ptr() as *mut u8,
					0
				);

				Self {
					inner: mem::transmute::<_, [u8; 15]>(buf),
					len
				}
			}
		}
	}
}

impl From<usize> for NiceInt {
	fn from(num: usize) -> Self {
		if num <= 255 { Self::from(num as u8) }
		else if num <= 65_535 { Self::from(num as u16) }
		else if num <= 4_294_967_295 { Self::from(num as u32) }
		else {
			unsafe {
				let mut buf = [MaybeUninit::<u8>::uninit(); 15];

				let len = write_64_from(
					999_999_999_999.min(num) as u64,
					if num >= 100_000_000_000 { 100_000_000_000_u64 }
						else if num >= 10_000_000_000 { 10_000_000_000_u64 }
						else { 1_000_000_000_u64 },
					buf.as_mut_ptr() as *mut u8,
					0
				);

				Self {
					inner: mem::transmute::<_, [u8; 15]>(buf),
					len
				}
			}
		}
	}
}

impl From<u128> for NiceInt {
	fn from(num: u128) -> Self {
		Self::from(999_999_999_999.min(num) as u64)
	}
}

impl NiceInt {
	#[must_use]
	/// # As Bytes.
	///
	/// Return the value as a byte string.
	pub fn as_bytes(&self) -> &[u8] { &self }

	#[must_use]
	/// # As Str.
	///
	/// Return the value as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self) }
	}
}

#[allow(clippy::integer_division)]
/// # Write `u64` Portions.
///
/// This writes the leading decimals of a "big number" to the buffer, then
/// recasts the value as a `u32` to continue from the middle.
///
/// The explicit `if x == y` conditions are rather unsightly but execute much
/// faster than loops with `/=` operations on the divisors. Ah the benefits of
/// capping infinity…
unsafe fn write_64_from(mut src: u64, mut from: u64, buf: *mut u8, mut len: usize) -> usize {
	if from == 100_000_000_000 {
		if src >= 100_000_000_000 {
			ptr::write(buf.add(len), (src / 100_000_000_000) as u8 + 48);
			src %= 100_000_000_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		len += 1;
		from = 10_000_000_000;
	}

	if from == 10_000_000_000 {
		if src >= 10_000_000_000 {
			ptr::write(buf.add(len), (src / 10_000_000_000) as u8 + 48);
			src %= 10_000_000_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		len += 1;
		from = 1_000_000_000;
	}

	// We only need to crunch this as a u64 if the source is too big to be
	// represented as a u32.
	if from == 1_000_000_000 && src > 4_294_967_295 {
		ptr::write(buf.add(len), (src / 1_000_000_000) as u8 + 48);
		src %= 1_000_000_000;

		ptr::write(buf.add(len + 1), b',');
		len += 2;
		from = 100_000_000;
	}

	write_32_from(src as u32, from as u32, buf, len)
}

#[allow(clippy::integer_division)]
/// # Write `u32` Portions.
///
/// This writes the leading decimals of a "mid-sized number" to the buffer,
/// then recasts the value as a `u16` to finish it off.
unsafe fn write_32_from(mut src: u32, mut from: u32, buf: *mut u8, mut len: usize) -> usize {
	if from == 1_000_000_000 {
		if src >= 1_000_000_000 {
			ptr::write(buf.add(len), (src / 1_000_000_000) as u8 + 48);
			src %= 1_000_000_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		ptr::write(buf.add(len + 1), b',');
		len += 2;
		from = 100_000_000;
	}

	if from == 100_000_000 {
		if src >= 100_000_000 {
			ptr::write(buf.add(len), (src / 100_000_000) as u8 + 48);
			src %= 100_000_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		len += 1;
		from = 10_000_000;
	}

	if from == 10_000_000 {
		if src >= 10_000_000 {
			ptr::write(buf.add(len), (src / 10_000_000) as u8 + 48);
			src %= 10_000_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		len += 1;
		from = 1_000_000;
	}

	if from == 1_000_000 {
		if src >= 1_000_000 {
			ptr::write(buf.add(len), (src / 1_000_000) as u8 + 48);
			src %= 1_000_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		ptr::write(buf.add(len + 1), b',');
		len += 2;
		from = 100_000;
	}

	if from == 100_000 {
		if src >= 100_000 {
			ptr::write(buf.add(len), (src / 100_000) as u8 + 48);
			src %= 100_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		len += 1;
		from = 10_000;
	}

	// We only need to crunch this as a u32 if the source is too big to be
	// represented as a u16.
	if from == 10_000 && src > 65_535 {
		ptr::write(buf.add(len), (src / 10_000) as u8 + 48);
		src %= 10_000;

		len += 1;
		from = 1_000;
	}

	write_16_from(src as u16, from as u16, buf, len)
}

#[allow(clippy::integer_division)]
/// # Write `u16` Portions.
///
/// This writes all remaining decimals to the buffer. Numbers small enough to
/// be represented as a `u8` won't arrive here, but bigger numbers might after
/// their leading bits have been written.
unsafe fn write_16_from(mut src: u16, mut from: u16, buf: *mut u8, mut len: usize) -> usize {
	if from == 10_000 {
		if src >= 10_000 {
			ptr::write(buf.add(len), (src / 10_000) as u8 + 48);
			src %= 10_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		from = 1_000;
		len += 1;
	}

	if from == 1_000 {
		if src >= 1_000 {
			ptr::write(buf.add(len), (src / 1_000) as u8 + 48);
			src %= 1_000;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		ptr::write(buf.add(len + 1), b',');
		len += 2;
		from = 100;
	}

	if from == 100 {
		if src >= 100 {
			ptr::write(buf.add(len), (src / 100) as u8 + 48);
			src %= 100;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		len += 1;
		from = 10;
	}

	if from == 10 {
		if src >= 10 {
			ptr::write(buf.add(len), (src / 10) as u8 + 48);
			src %= 10;
		}
		else { ptr::write(buf.add(len), 48_u8); }

		len += 1;
	}

	ptr::write(buf.add(len), src as u8 + 48);
	len + 1
}



#[test]
fn t_nice_int() {
	use num_format::{ToFormattedString, Locale};

	for i in [
		1_u64,
		5_u64,
		9_u64,
		10_u64,
		98_u64,
		99_u64,
		100_u64,
		101_u64,
		678_u64,
		999_u64,
		1000_u64,
		1001_u64,
		9999_u64,
		10000_u64,
		10001_u64,
		99999_u64,
		100000_u64,
		100001_u64,
		999999_u64,
		1000000_u64,
		1000001_u64,
		9999999_u64,
		10000000_u64,
		10000001_u64,
		99999999_u64,
		100000000_u64,
		100000001_u64,
		999999999_u64,
		1000000000_u64,
		1000000001_u64,
		9999999999_u64,
		10000000000_u64,
		10000000001_u64,
		99999999999_u64,
		100000000000_u64,
		100000000001_u64,
	].iter() {
		assert_eq!(&*NiceInt::from(*i), i.to_formatted_string(&Locale::en).as_bytes(), "{:?}", *i);
	}
}
