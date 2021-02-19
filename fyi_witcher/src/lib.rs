/*!
# FYI Witcher

This crate provides [`Witcher`], a simple, minimally configurable file system traversal library.

For simple get-all-files-recursively purposes, there's also the method [`witch`], which avoids the filtering overhead and setup of [`Witcher`].



## Optional Features

| Feature | Description |
| ------- | ----------- |
| regexp | Enables the [`Witcher::with_regex`] path filter. |



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
pub mod utility;

pub use witch::witch;
pub use witcher::Witcher;



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
