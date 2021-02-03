/*!
# FYI Num: Nice u32.
*/



/// # Total Buffer Size.
const SIZE: usize = 13;



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// `NiceU32` provides a quick way to convert a `u32` into a formatted byte
/// string for e.g. printing. Commas are added for every thousand.
///
/// That's it!
///
/// ## Examples
///
/// ```no_run
/// use fyi_num::NiceU32;
/// assert_eq!(
///     NiceU32::from(33231).as_str(),
///     "33,231"
/// );
pub struct NiceU32 {
	inner: [u8; SIZE],
	from: usize,
}

crate::impl_nice_int!(NiceU32);

impl Default for NiceU32 {
	#[inline]
	fn default() -> Self {
		Self {
			inner: [0, b',', 0, 0, 0, b',', 0, 0, 0, b',', 0, 0, 0],
			from: SIZE,
		}
	}
}

impl From<u32> for NiceU32 {
	fn from(num: u32) -> Self {
		// Recast as usize to avoid all the subsequent index casts.
		let mut num = num as usize;
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



#[cfg(test)]
mod tests {
	use super::*;
	use num_format::{ToFormattedString, Locale};
	use rand::Rng;

	#[test]
	#[ignore] // This takes a very long time to run.
	fn t_nice_u32() {
		for i in 0..=u32::MAX {
			assert_eq!(
				NiceU32::from(i).as_str(),
				i.to_formatted_string(&Locale::en),
			);
		}
	}

	#[test]
	fn t_nice_u32_sample() {
		let mut rng = rand::thread_rng();

		for _ in 0..1_000_000 {
			let num: u32 = rng.gen();
			assert_eq!(
				NiceU32::from(num).as_str(),
				num.to_formatted_string(&Locale::en),
			);
		}

		assert_eq!(NiceU32::from(0).as_str(), "0");

		assert_eq!(
			NiceU32::from(u32::MAX).as_str(),
			u32::MAX.to_formatted_string(&Locale::en),
		);
	}
}
