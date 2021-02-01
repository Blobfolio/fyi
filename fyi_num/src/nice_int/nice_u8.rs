/*!
# FYI Num: Nice u8.
*/



/// # Total Buffer Size.
const SIZE: usize = 3;



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// `NiceU8` provides a quick way to convert a `u8` into a formatted byte
/// string for e.g. printing.
///
/// That's it!
///
/// ## Examples
///
/// ```no_run
/// use fyi_num::NiceU8;
/// assert_eq!(
///     NiceU8::from(231).as_str(),
///     "231"
/// );
pub struct NiceU8 {
	inner: [u8; SIZE],
	from: usize,
}

crate::impl_nice_int!(NiceU8);

impl Default for NiceU8 {
	#[inline]
	fn default() -> Self {
		Self {
			inner: [0, 0, 0],
			from: SIZE,
		}
	}
}

impl From<u8> for NiceU8 {
	fn from(num: u8) -> Self {
		let mut out = Self::default();

		if num >= 100 {
			out.from -= 3;
			unsafe { super::write_u8_3(out.inner.as_mut_ptr().add(out.from), usize::from(num)); }
		}
		else if num >= 10 {
			out.from -= 2;
			unsafe { super::write_u8_2(out.inner.as_mut_ptr().add(out.from), usize::from(num)); }
		}
		else {
			out.from -= 1;
			unsafe { super::write_u8_1(out.inner.as_mut_ptr().add(out.from), usize::from(num)); }
		}

		out
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_nice_u8() {
		for i in 0..=u8::MAX {
			assert_eq!(
				NiceU8::from(i).as_str(),
				format!("{}", i),
			);
		}
	}
}
