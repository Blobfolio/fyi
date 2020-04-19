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



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fyi_file_extension() {
		assert_eq!(&PathBuf::from("foo/bar.JS").fyi_file_extension(), "js");
		assert_eq!(&PathBuf::from("src/lib.rs").fyi_file_extension(), "rs");

		assert_eq!(&PathBuf::from("foo/bar").fyi_file_extension(), "");
		assert_eq!(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).fyi_file_extension(), "");
	}

	#[test]
	fn fyi_file_name() {
		assert_eq!(&PathBuf::from("foo/bar.JS").fyi_file_name(), "bar.JS");
		assert_eq!(&PathBuf::from("src/lib.rs").fyi_file_name(), "lib.rs");

		// Should return "bar" since the path doesn't exist and might be
		// intended to hold a file at some point.
		assert_eq!(&PathBuf::from("foo/bar").fyi_file_name(), "bar");

		// This is definitely a directory, though, so shouldn't return
		// anything.
		assert_eq!(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).fyi_file_name(), "");
	}

	#[test]
	fn fyi_file_size() {
		// These should come up zero.
		assert_eq!(PathBuf::from("foo/bar.JS").fyi_file_size(), 0);
		assert_eq!(PathBuf::from(env!("CARGO_MANIFEST_DIR")).fyi_file_size(), 0);

		// And something we know.
		assert_eq!(PathBuf::from("tests/assets/file.txt").fyi_file_size(), 26);
	}

	#[test]
	fn fyi_is_executable() {
		// These should come up false.
		assert_eq!(PathBuf::from("foo/bar.JS").fyi_is_executable(), false);
		assert_eq!(PathBuf::from("tests/assets/file.txt").fyi_is_executable(), false);
		assert_eq!(PathBuf::from(env!("CARGO_MANIFEST_DIR")).fyi_is_executable(), false);

		// But this should come up true.
		assert_eq!(PathBuf::from("tests/assets/is-executable.sh").fyi_is_executable(), true);
	}

	#[test]
	fn fyi_parent() {
		// A known file.
		let file: PathBuf = PathBuf::from("./src/lib.rs");
		assert!(file.is_file());

		// The canonical parent.
		let dir: PathBuf = PathBuf::from("./src")
			.canonicalize()
			.expect("Parent, damn it.");
		assert!(dir.is_dir());

		// The two should match.
		assert_eq!(file.fyi_parent().unwrap(), dir);

		// This should also work on directories.
		let dir2: PathBuf = PathBuf::from(".")
			.canonicalize()
			.expect("Parent, damn it.");
		assert!(dir2.is_dir());
		assert_eq!(dir.fyi_parent().unwrap(), dir2);
	}
}
