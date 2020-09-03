/*!
# FYI Witcher

This crate provides two main components, both of them file-related:

* [`Witcher`] is a simple, minimally configurable file system traversal library.
* [`Witching`] is a lightweight, automatic progress bar wrapper that can be used while iterating through/operating on a set of paths.

Out of necessity, this crate also contains a few random odds and ends that
might be independently useful, namely:

* [`NiceElapsed`] converts a given number of seconds into a human-readable, Oxford-joined list of units, like "1 hour, 2 minutes, and 3 seconds", suitable for summaries and the like.
* [`NiceInt`] is a fast, (US) formatting-aware integer stringifier. (It turns numbers into byte strings for e.g. printing.)



## Stability: Alpha

This project is under heavy development and subject to change. While the code
in the `master` branch should always be in a "working" state, breaking changes
and major refactors may be introduced between releases.

(This should probably *not* be used in production-ready applications.)



## Crate Features

* `simd`: This feature enables some SIMD optimizations courtesy of [`packed_simd`](https://crates.io/crates/packed_simd) for minor performance gains. This requires Rust nightly.
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



mod nice_elapsed;
mod nice_int;
mod witcher;
mod witching;
pub mod utility;

pub use nice_elapsed::NiceElapsed;
pub use nice_int::NiceInt;
pub use witcher::Witcher;
pub use witching::{
	Witching,
	WITCHING_DIFF,
	WITCHING_QUIET,
	WITCHING_SUMMARIZE,
};
