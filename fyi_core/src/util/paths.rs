/*!
# FYI Core: Paths
*/

use crate::traits::AbsPath;
use std::{
	borrow::Cow,
	path::Path,
};



#[inline]
/// To String.
pub fn to_string<P> (path: P) -> Cow<'static, str>
where P: AsRef<Path> {
	path.as_ref()
		.to_str()
		.unwrap_or("")
		.to_string()
		.into()
}

/// To String.
pub fn to_string_abs<P> (path: P) -> Cow<'static, str>
where P: AsRef<Path> {
	to_string(path.to_path_buf_abs())
}



#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;



	#[test]
	fn to_string() {
		let path = PathBuf::from("src/lib.rs");
		assert_eq!(
			&super::to_string(&path),
			"src/lib.rs",
		);
	}

	#[test]
	fn to_string_abs() {
		let path = PathBuf::from("src/lib.rs");
		let canon = path.canonicalize().expect("Canon, damn it!");

		assert_eq!(
			path.to_str().expect("Strings, damn it!"),
			"src/lib.rs",
		);
		assert_eq!(
			&super::to_string_abs(&path),
			canon.to_str().expect("Strings, damn it!"),
		);
	}
}
