/*!
# FYI Num: Nice u16.
*/



/// # Total Buffer Size.
const SIZE: usize = 6;



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// `NiceU16` provides a quick way to convert a `u16` into a formatted byte
/// string for e.g. printing. Commas are added for every thousand.
///
/// That's it!
///
/// ## Examples
///
/// ```no_run
/// use fyi_num::NiceU16;
/// assert_eq!(
///     NiceU16::from(33231).as_str(),
///     "33,231"
/// );
pub struct NiceU16 {
	inner: [u8; SIZE],
	from: usize,
}

crate::impl_nice_int!(NiceU16);

impl Default for NiceU16 {
	#[inline]
	fn default() -> Self {
		Self {
			inner: [0, 0, b',', 0, 0, 0],
			from: SIZE,
		}
	}
}

impl From<u16> for NiceU16 {
	fn from(mut num: u16) -> Self {
		let mut out = Self::default();
		let ptr = out.inner.as_mut_ptr();

		// For `u16` this can only trigger once.
		if num >= 1000 {
			let (div, rem) = num_integer::div_mod_floor(num, 1000);
			unsafe { super::write_u8_3(ptr.add(out.from - 3), usize::from(rem)); }
			num = div;
			out.from -= 4;
		}

		if num >= 100 {
			out.from -= 3;
			unsafe { super::write_u8_3(ptr.add(out.from), usize::from(num)); }
		}
		else if num >= 10 {
			out.from -= 2;
			unsafe { super::write_u8_2(ptr.add(out.from), usize::from(num)); }
		}
		else {
			out.from -= 1;
			unsafe { super::write_u8_1(ptr.add(out.from), usize::from(num)); }
		}

		out
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use num_format::{ToFormattedString, Locale};

	#[test]
	fn t_nice_u16() {
		for i in 0..=u16::MAX {
			assert_eq!(
				NiceU16::from(i).as_str(),
				i.to_formatted_string(&Locale::en),
			);
		}
	}
}
