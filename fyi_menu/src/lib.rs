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

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



mod argue;
mod keykind;
#[cfg(not(feature = "simd"))] mod keymaster;
#[cfg(feature = "simd")]      mod simd;
pub mod utility;

use fyi_msg::{
	Msg,
	MsgKind,
};
pub use keykind::KeyKind;
#[cfg(not(feature = "simd"))] pub use keymaster::KeyMaster;
#[cfg(feature = "simd")]      pub use simd::KeyMaster;
pub use argue::Argue;



/// Print an Error and Exit.
pub fn die(msg: &[u8]) {
	Msg::from(msg)
		.with_prefix(MsgKind::Error)
		.eprintln();
	std::process::exit(1);
}

#[must_use]
/// Hash Key.
pub fn hash_arg_key(key: &str) -> u64 {
	use std::hash::Hasher;
	let mut hasher = ahash::AHasher::default();
	hasher.write(key.as_bytes());
	hasher.finish()
}
