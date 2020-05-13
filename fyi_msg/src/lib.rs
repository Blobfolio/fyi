/*!
# FYI Msg: Table of Contents
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::pedantic)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod msg;
mod timestamp;

pub mod print;
pub mod traits;

pub use msg::Msg;
pub use print::Flags;

#[cfg(feature = "interactive")]
pub use print::prompt;

pub use timestamp::Timestamp;

/// Miscellaneous utility classes.
pub mod utility {
	use num_format::{
		Locale,
		WriteFormatted,
	};
	use std::borrow::{
		Borrow,
		Cow,
	};
	pub use super::print::{
		print,
		print_to,
	};

	/// String Inflection
	///
	/// Given a number, come up with a string like "1 thing" or "2 things".
	pub fn inflect<T1, T2> (num: u64, one: T1, many: T2) -> Cow<'static, [u8]>
	where
		T1: Borrow<str>,
		T2: Borrow<str> {
		if 1 == num {
			Cow::Owned([
				b"1 ",
				one.borrow().as_bytes(),
			].concat())
		}
		else if num < 1000 {
			let noun = many.borrow();
			let mut buf: Vec<u8> = Vec::with_capacity(noun.len() + 4);
			itoa::write(&mut buf, num).expect("Invalid number.");
			buf.push(b' ');
			buf.extend_from_slice(noun.as_bytes());
			Cow::Owned(buf)
		}
		else {
			let noun = many.borrow();
			let mut buf: Vec<u8> = Vec::with_capacity(noun.len() + 4);
			buf.write_formatted(&num, &Locale::en).expect("Invalid number.");
			buf.push(b' ');
			buf.extend_from_slice(noun.as_bytes());
			Cow::Owned(buf)
		}
	}

	/// In-Place u8 Slice Replacement
	///
	/// It is often more efficient to copy bytes into the middle of an existing
	/// buffer than to run a million separate `push()` or `extend_from_slice()`
	/// operations.
	///
	/// This method can be used to do just that for any two `[u8]` slices of
	/// matching lengths. (Note: if the lengths don't match, it will panic!)
	///
	/// Each index from the right is compared against the corresponding index
	/// on the left and copied if different.
	pub fn slice_swap(lhs: &mut [u8], rhs: &[u8]) {
		let len: usize = lhs.len();
		assert_eq!(len, rhs.len());

		for i in 0..len {
			if lhs[i] != rhs[i] {
				lhs[i] = rhs[i];
			}
		}
	}

	#[must_use]
	/// Term Width
	///
	/// This is a simple wrapper around `term_size::dimensions()` to provide
	/// the current terminal column width. We don't have any use for height,
	/// so that property is ignored.
	pub fn term_width() -> usize {
		// Reserve one space at the end "just in case".
		if let Some((w, _)) = term_size::dimensions() { w.saturating_sub(1) }
		else { 0 }
	}

	#[must_use]
	/// Whitespace maker.
	///
	/// This method borrows whitespace from a static reference, useful for
	/// quickly padding strings, etc.
	pub fn whitespace(num: usize) -> &'static [u8] {
		static WHITES: &[u8; 255] = &[b' '; 255];

		if num >= 255 { &WHITES[..] }
		else { &WHITES[0..num] }
	}
}
