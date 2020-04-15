/*!
# FYI Core: Miscellany: Path Formatting
*/

use crate::{
	Error,
	Result,
	traits::path::FYIPath,
	util::strings,
};
use std::{
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
	fn fyi_to_string(&self) -> String;

	/// With File Name.
	fn fyi_with_file_name<S> (&self, name: S) -> PathBuf
	where S: Into<String>;
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
	fn fyi_to_string(&self) -> String {
		strings::from_os_string(
			self.fyi_to_path_buf_abs().into_os_string()
		)
	}

	/// With File Name.
	fn fyi_with_file_name<S> (&self, name: S) -> PathBuf
	where S: Into<String> {
		if self.is_dir() {
			let mut clone: PathBuf = self.fyi_to_path_buf_abs();
			clone.push(name.into());
			clone
		}
		else {
			self.with_file_name(format!(
				"{}{}",
				self.fyi_file_name(),
				name.into()
			))
		}
	}
}
