/*!
# FYI Core: Miscellany: Path Formatting
*/

use std::{
	borrow::Cow,
	fs,
	path::{
		Path,
		PathBuf,
	},
};



/// Format/Conversion/Mutation Helpers!
pub trait FYIPathFormat {
	/// Absolute PathBuf.
	fn fyi_to_path_buf_abs(&self) -> PathBuf;

	/// To String.
	fn fyi_to_string(&self) -> Cow<'static, str>;

	/// To String.
	fn fyi_to_string_abs(&self) -> Cow<'static, str>;
}

impl FYIPathFormat for Path {
	/// Absolute PathBuf.
	fn fyi_to_path_buf_abs(&self) -> PathBuf {
		match fs::canonicalize(self) {
			Ok(path) => path,
			_ => self.into(),
		}
	}

	/// To String.
	fn fyi_to_string(&self) -> Cow<'static, str> {
		Cow::Owned(self.to_str().unwrap_or("").to_string())
	}

	/// To String.
	fn fyi_to_string_abs(&self) -> Cow<'static, str> {
		self.fyi_to_path_buf_abs().fyi_to_string()
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fyi_to_path_buf_abs() {
		let path = PathBuf::from("src/lib.rs");
		let canon = path.canonicalize().expect("Canon, damn it!");

		assert_eq!(
			path.to_str().expect("Strings, damn it!"),
			"src/lib.rs",
		);
		assert_eq!(
			path.fyi_to_path_buf_abs().to_str().expect("Strings, damn it!"),
			canon.to_str().expect("Strings, damn it!"),
		);
	}

	#[test]
	fn fyi_to_string() {
		let path = PathBuf::from("src/lib.rs");
		assert_eq!(
			&path.fyi_to_string(),
			"src/lib.rs",
		);
	}

	#[test]
	fn fyi_to_string_abs() {
		let path = PathBuf::from("src/lib.rs");
		let canon = path.canonicalize().expect("Canon, damn it!");

		assert_eq!(
			path.to_str().expect("Strings, damn it!"),
			"src/lib.rs",
		);
		assert_eq!(
			&path.fyi_to_string_abs(),
			canon.to_str().expect("Strings, damn it!"),
		);
	}
}
