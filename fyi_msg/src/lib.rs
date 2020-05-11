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
pub use print::{
	Flags,
	print,
	print_to,
};

#[cfg(feature = "interactive")]
pub use print::prompt;

pub use timestamp::Timestamp;

/// Miscellaneous utility classes.
pub mod utility {
	#[must_use]
	#[inline]
	/// Hash Calc.
	pub fn hash(t: &[u8]) -> u64 {
		seahash::hash(t)
	}

	/// In-Place u8 Slice Replacement
	///
	/// Note, the two sides must have the same length.
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
	pub fn term_width() -> usize {
		// Reserve one space at the end "just in case".
		if let Some((w, _)) = term_size::dimensions() { w.saturating_sub(1) }
		else { 0 }
	}

	#[must_use]
	/// Whitespace maker.
	pub fn whitespace(num: usize) -> &'static [u8] {
		static WHITES: &[u8; 255] = &[b' '; 255];

		if num >= 255 { &WHITES[..] }
		else { &WHITES[0..num] }
	}
}
