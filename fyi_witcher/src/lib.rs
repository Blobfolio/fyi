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
#![warn(clippy::pedantic)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod witcher;
pub mod traits;

/// The Witcher!
pub use witcher::Witcher;

/// Generic result type.
pub type Result<T, E = String> = std::result::Result<T, E>;

/// Utilities.
pub mod utility {
	use std::path::Path;

	/// Ergonomical file size.
	pub fn file_extension<P> (path: P) -> String
	where P: AsRef<Path> {
		let path = path.as_ref();
		if path.is_dir() {
			"".to_string()
		}
		else if let Some(ext) = path.extension() {
			ext.to_str()
				.unwrap_or("")
				.to_lowercase()
		}
		else {
			"".to_string()
		}
	}

	/// Ergonomical file size.
	pub fn file_size<P> (path: P) -> u64
	where P: AsRef<Path> {
		if let Ok(meta) = path.as_ref().metadata() {
			meta.len()
		}
		else { 0 }
	}
}
