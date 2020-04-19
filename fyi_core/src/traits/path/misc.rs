/*!
# FYI Core: Miscellany: Path Properties
*/

use crate::{
	Error,
	Result,
	traits::path::FYIPathFormat,
	util::strings,
};
use std::{
	borrow::Cow,
	ffi::OsStr,
	os::unix::fs::PermissionsExt,
	path::{
		Path,
		PathBuf,
	},
};



/// Format/Conversion/Mutation Helpers!
pub trait FYIPath {
	/// Extension.
	fn fyi_file_extension(&self) -> Cow<'static, str>;

	/// File name.
	fn fyi_file_name(&self) -> Cow<'static, str>;

	/// File Size.
	fn fyi_file_size(&self) -> u64;

	/// Is Executable?
	fn fyi_is_executable(&self) -> bool;

	/// Parent Directory.
	fn fyi_parent(&self) -> Result<PathBuf>;
}

impl FYIPath for Path {
	/// Extension.
	fn fyi_file_extension(&self) -> Cow<'static, str> {
		if self.is_dir() {
			Cow::Borrowed("")
		}
		else {
			match self.extension() {
				Some(ext) => Cow::Owned(strings::from_os_string(ext).to_lowercase()),
				_ => Cow::Borrowed(""),
			}
		}
	}

	/// File name.
	fn fyi_file_name(&self) -> Cow<'static, str> {
		match self.is_dir() {
			true => Cow::Borrowed(""),
			false => Cow::Owned(
				self.file_name()
					.unwrap_or(OsStr::new(""))
					.to_str()
					.unwrap_or("")
					.to_string()
			),
		}
	}

	/// File Size.
	fn fyi_file_size(&self) -> u64 {
		if let Ok(meta) = self.metadata() {
			if meta.is_file() {
				return meta.len();
			}
		}

		0
	}

	/// Is Executable?
	fn fyi_is_executable(&self) -> bool {
		if let Ok(meta) = self.metadata() {
			if meta.is_file() {
				let permissions = meta.permissions();
				return permissions.mode() & 0o111 != 0;
			}
		}

		return false;
	}

	/// Parent Directory.
	fn fyi_parent(&self) -> Result<PathBuf> {
		if let Some(dir) = self.parent() {
			if dir.is_dir() {
				return Ok(dir.fyi_to_path_buf_abs());
			}
		}

		Err(Error::PathInvalid(self.to_path_buf(), "has no parent"))
	}
}
