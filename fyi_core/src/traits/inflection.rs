use crate::traits::Thousands;
use std::borrow::{
	Borrow,
	Cow,
};



/// String inflection.
pub trait Inflection {
	/// Inflect.
	fn inflect<S1, S2> (&self, one: S1, two: S2) -> Cow<str>
	where
		S1: Borrow<str>,
		S2: Borrow<str>;
}



macro_rules! impl_int_inflection {
	($type:ty) => {
		impl Inflection for $type {
			/// Inflect.
			fn inflect<S1, S2> (&self, one: S1, two: S2) -> Cow<str>
			where
				S1: Borrow<str>,
				S2: Borrow<str>
			{
				if 1 == *self {
					let noun = one.borrow();
					let mut out: String = String::with_capacity(noun.len() + 2);
					out.push_str("1 ");
					out.push_str(noun);
					out.into()
				}
				else if *self < 1000 {
					let mut cache = [0u8; 20];
					let n = itoa::write(&mut cache[..], *self).unwrap();
					let noun = two.borrow();

					unsafe {
						let mut out: String = String::from_utf8_unchecked(cache[0..n].to_vec());
						out.reserve_exact(1 + noun.len());
						out.push(' ');
						out.push_str(noun);
						out.into()
					}
				}
				else {
					let num = self.thousands();
					let noun = two.borrow();
					let mut out: String = String::with_capacity(num.len() + noun.len() + 1);
					out.push_str(&num);
					out.push(' ');
					out.push_str(noun);
					out.into()
				}
			}
		}
	};
}

macro_rules! impl_smallint_inflection {
	($type:ty) => {
		impl Inflection for $type {
			/// Inflect.
			fn inflect<S1, S2> (&self, one: S1, two: S2) -> Cow<str>
			where
				S1: Borrow<str>,
				S2: Borrow<str>
			{
				if 1 == *self {
					let noun = one.borrow();
					let mut out: String = String::with_capacity(noun.len() + 2);
					out.push_str("1 ");
					out.push_str(noun);
					out.into()
				}
				else {
					let num = self.thousands();
					let noun = two.borrow();
					let mut out: String = String::with_capacity(num.len() + noun.len() + 1);
					out.push_str(&num);
					out.push(' ');
					out.push_str(noun);
					out.into()
				}
			}
		}
	};
}



impl_int_inflection!(usize);
impl_int_inflection!(u16);
impl_int_inflection!(u32);
impl_int_inflection!(u64);
impl_int_inflection!(u128);
impl_int_inflection!(i16);
impl_int_inflection!(i32);
impl_int_inflection!(i64);

impl_smallint_inflection!(u8);
impl_smallint_inflection!(i8);



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn inflect() {
		assert_eq!(0.inflect("book", "books"), Cow::Borrowed("0 books"));
		assert_eq!(1.inflect("book", "books"), Cow::Borrowed("1 book"));
		assert_eq!(2.inflect("book", "books"), Cow::Borrowed("2 books"));
		assert_eq!(5000.inflect("book", "books"), Cow::Borrowed("5,000 books"));
	}
}
