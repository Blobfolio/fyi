use num_format::{
	Locale,
	ToFormattedString,
};
use std::borrow::Cow;



/// Thousands separator.
pub trait Thousands {
	/// Thousands Separator.
	fn thousands(&self) -> Cow<'_, str>;
}



macro_rules! impl_int_thousands {
	($type:ty) => {
		impl Thousands for $type {
			#[inline]
			/// Thousands Separator.
			fn thousands(&self) -> Cow<'_, str> {
				if *self < 1000 {
					let mut cache = [0u8; 20];
					let n = itoa::write(&mut cache[..], *self).unwrap();
					unsafe { String::from_utf8_unchecked(cache[0..n].to_vec()).into() }
				}
				else {
					self.to_formatted_string(&Locale::en).into()
				}
			}
		}
	};
}

macro_rules! impl_smallint_thousands {
	($type:ty) => {
		impl Thousands for $type {
			#[inline]
			/// Thousands Separator.
			fn thousands(&self) -> Cow<'_, str> {
				self.to_formatted_string(&Locale::en).into()
			}
		}
	};
}

impl_int_thousands!(usize);
impl_int_thousands!(u16);
impl_int_thousands!(u32);
impl_int_thousands!(u64);
impl_int_thousands!(u128);
impl_int_thousands!(i16);
impl_int_thousands!(i32);
impl_int_thousands!(i64);

impl_smallint_thousands!(u8);
impl_smallint_thousands!(i8);



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn human_int() {
		assert_eq!(500u64.thousands(), Cow::Borrowed("500"));
		assert_eq!(5000u64.thousands(), Cow::Borrowed("5,000"));
		assert_eq!(50000u64.thousands(), Cow::Borrowed("50,000"));
		assert_eq!(500000u64.thousands(), Cow::Borrowed("500,000"));
		assert_eq!(5000000u64.thousands(), Cow::Borrowed("5,000,000"));
	}
}
