/*!
# FYI Core: Miscellany: Path Formatting
*/

use crate::util::strings;
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
		Cow::Owned(strings::from_os_string(
			self.fyi_to_path_buf_abs().into_os_string()
		))
	}
}
