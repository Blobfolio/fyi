/*!
# FYI Menu

This crate provides a very simple CLI argument parser. It differs from
`std::env::args()` primarily in that key/value formatting is normalized,
splitting joint entries like `-kValue` and `--key=Value`.

Actual processing and interpretation of the arguments is left to the coder.

This sort of approach can significantly reduce the overhead of simple CLI apps
compared to using (excellent but heavy) crates like `clap`.

See the flag documentation below for handling options.
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
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unknown_clippy_lints)]

pub mod utility;
mod argue;

pub use argue::{
	Argue,
	die,
	KeyKind,
};
