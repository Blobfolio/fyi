/*!
# FYI Witcher: Table of Contents
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
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]



#[cfg(feature = "witch_io")] mod witchio;
mod witcher;
pub mod utility;

/// The Witcher!
pub use witcher::Witcher;

/// Generic result type.
pub type Result<T, E = String> = std::result::Result<T, E>;

#[cfg(feature = "witch_io")]
/// The `WitchIO` trait.
pub mod traits {
	pub use crate::witchio::WitchIO;
}
