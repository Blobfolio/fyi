/*!
# FYI Witcher

This crate provides two main components, both of them file-related:

* [`Witcher`] is a simple, minimally configurable file system traversal library.
* [`Witching`] is a lightweight, automatic progress bar wrapper that can be used while iterating through/operating on a set of paths.

For simple get-all-files-recursively purposes, there's also [`witch`].



## Optional Features

| Feature | Description |
| ------- | ----------- |
| regexp | Enables the [`Witcher::with_regex`] path filter. |
| witching | Enables the [`Witching`] progress struct. |



## Stability

Release versions of this library should be in a working state, but as this
project is under perpetual development, code might change from version to
version.
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
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



mod witch;
mod witcher;
#[cfg(feature = "witching")] mod witching;
pub mod utility;

pub use witch::witch;
pub use witcher::Witcher;

#[cfg(feature = "witching")]
pub use witching::{
	Witching,
	WITCHING_DIFF,
	WITCHING_QUIET,
	WITCHING_SUMMARIZE,
};



/// # (Not) Random State.
///
/// Using a fixed seed value for `AHashSet`/`AHashMap` drops a few dependencies
/// and prevents Valgrind complaining about 64 lingering bytes from the runtime
/// static that would be used otherwise.
///
/// For our purposes, the variability of truly random keys isn't really needed.
pub(crate) const AHASH_STATE: ahash::RandomState = ahash::RandomState::with_seeds(13, 19, 23, 71);



#[macro_export]
/// Helper: Mutex Unlock.
///
/// This just moves tedious code out of the way.
macro_rules! mutex_ptr {
	($mutex:expr) => (
		$mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
	);
}
