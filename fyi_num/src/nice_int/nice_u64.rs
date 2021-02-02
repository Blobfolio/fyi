/*!
# FYI Num: Nice u64.
*/



/// # Total Buffer Size.
const SIZE: usize = 26;



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// `NiceU64` provides a quick way to convert a `u64` into a formatted byte
/// string for e.g. printing. Commas are added for every thousand.
///
/// That's it!
///
/// ## Examples
///
/// ```no_run
/// use fyi_num::NiceU64;
/// assert_eq!(
///     NiceU64::from(33231_u64).as_str(),
///     "33,231"
/// );
pub struct NiceU64 {
	inner: [u8; SIZE],
	from: usize,
}

crate::impl_nice_int!(NiceU64);

impl Default for NiceU64 {
	#[inline]
	fn default() -> Self {
		Self {
			inner: [0, 0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0],
			from: SIZE,
		}
	}
}

impl From<usize> for NiceU64 {
	fn from(mut num: usize) -> Self {
		let mut out = Self::default();

		let ptr = out.inner.as_mut_ptr();

		while num >= 1000 {
			let (div, rem) = num_integer::div_mod_floor(num, 1000);
			unsafe { super::write_u8_3(ptr.add(out.from - 3), rem); }
			num = div;
			out.from -= 4;
		}

		if num >= 100 {
			out.from -= 3;
			unsafe { super::write_u8_3(ptr.add(out.from), num); }
		}
		else if num >= 10 {
			out.from -= 2;
			unsafe { super::write_u8_2(ptr.add(out.from), num); }
		}
		else {
			out.from -= 1;
			unsafe { super::write_u8_1(ptr.add(out.from), num); }
		}

		out
	}
}

impl From<u64> for NiceU64 {
	#[cfg(target_pointer_width = "64")]
	fn from(num: u64) -> Self {
		// Skip all the index casts.
		Self::from(num as usize)
	}

	#[cfg(not(target_pointer_width = "64"))]
	fn from(mut num: u64) -> Self {
		let mut out = Self::default();

		let ptr = out.inner.as_mut_ptr();

		while num >= 1000 {
			let (div, rem) = num_integer::div_mod_floor(num, 1000);
			unsafe { super::write_u8_3(ptr.add(out.from - 3), rem as usize); }
			num = div;
			out.from -= 4;
		}

		if num >= 100 {
			out.from -= 3;
			unsafe { super::write_u8_3(ptr.add(out.from), num as usize); }
		}
		else if num >= 10 {
			out.from -= 2;
			unsafe { super::write_u8_2(ptr.add(out.from), num as usize); }
		}
		else {
			out.from -= 1;
			unsafe { super::write_u8_1(ptr.add(out.from), num as usize); }
		}

		out
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use num_format::{ToFormattedString, Locale};
	use rand::Rng;

	#[test]
	#[ignore] // This takes a very long time to run.
	fn t_nice_u64() {
		for i in 0..=u64::MAX {
			assert_eq!(
				NiceU64::from(i).as_str(),
				i.to_formatted_string(&Locale::en),
			);
		}
	}

	#[test]
	fn t_nice_u64_sample() {
		let mut rng = rand::thread_rng();

		for _ in 0..1_000_000 {
			let num: u64 = rng.gen();
			assert_eq!(
				NiceU64::from(num).as_str(),
				num.to_formatted_string(&Locale::en),
			);
		}

		assert_eq!(NiceU64::from(0_u64).as_str(), "0");

		assert_eq!(
			NiceU64::from(u64::MAX).as_str(),
			u64::MAX.to_formatted_string(&Locale::en),
		);
	}
}
