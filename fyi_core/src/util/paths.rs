/*!
# FYI Core: Paths
*/

use crate::{
	Error,
	Result,
};
use std::{
	borrow::Cow,
	fs,
	os::unix::fs::PermissionsExt,
	path::{
		Path,
		PathBuf,
	},
};



/// Extension.
pub fn file_extension<P> (path: P) -> Cow<'static, str>
where P: AsRef<Path> {
	if path.as_ref().is_dir() {
		Cow::Borrowed("")
	}
	else {
		match path.as_ref().extension() {
			Some(ext) => Cow::Owned(ext.to_str().unwrap_or("").to_lowercase()),
			_ => Cow::Borrowed(""),
		}
	}
}

/// File name.
pub fn file_name<P> (path: P) -> Cow<'static, str>
where P: AsRef<Path> {
	if path.as_ref().is_dir() {
		Cow::Borrowed("")
	}
	else {
		match path.as_ref().file_name() {
			Some(name) => Cow::Owned(name.to_str().unwrap_or("").to_string()),
			_ => Cow::Borrowed(""),
		}
	}
}

/// File Size.
pub fn file_size<P> (path: P) -> u64
where P: AsRef<Path> {
	if let Ok(meta) = path.as_ref().metadata() {
		if meta.is_file() {
			return meta.len();
		}
	}

	0
}

/// Is Executable?
pub fn is_executable<P> (path: P) -> bool
where P: AsRef<Path> {
	if let Ok(meta) = path.as_ref().metadata() {
		if meta.is_file() {
			let permissions = meta.permissions();
			return permissions.mode() & 0o111 != 0;
		}
	}

	return false;
}

/// Parent Directory.
pub fn parent<P> (path: P) -> Result<PathBuf>
where P: AsRef<Path> {
	if let Some(dir) = path.as_ref().parent() {
		if dir.is_dir() {
			return Ok(to_path_buf_abs(&dir));
		}
	}

	Err(Error::PathInvalid(path.as_ref().to_path_buf(), "has no parent"))
}

/// Absolute PathBuf.
pub fn to_path_buf_abs<P> (path: P) -> PathBuf
where P: AsRef<Path> {
	match fs::canonicalize(path.as_ref()) {
		Ok(path) => path,
		_ => path.as_ref().to_path_buf(),
	}
}

/// To String.
pub fn to_string<P> (path: P) -> Cow<'static, str>
where P: AsRef<Path> {
	Cow::Owned(path.as_ref().to_str().unwrap_or("").to_string())
}

/// To String.
pub fn to_string_abs<P> (path: P) -> Cow<'static, str>
where P: AsRef<Path> {
	to_string(to_path_buf_abs(path))
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn file_extension() {
		assert_eq!(super::file_extension(&PathBuf::from("foo/bar.JS")), "js");
		assert_eq!(super::file_extension(&PathBuf::from("src/lib.rs")), "rs");

		assert_eq!(super::file_extension(&PathBuf::from("foo/bar")), "");
		assert_eq!(super::file_extension(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))), "");
	}

	#[test]
	fn file_name() {
		assert_eq!(super::file_name(&PathBuf::from("foo/bar.JS")), "bar.JS");
		assert_eq!(super::file_name(&PathBuf::from("src/lib.rs")), "lib.rs");

		// Should return "bar" since the path doesn't exist and might be
		// intended to hold a file at some point.
		assert_eq!(super::file_name(&PathBuf::from("foo/bar")), "bar");

		// This is definitely a directory, though, so shouldn't return
		// anything.
		assert_eq!(super::file_name(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))), "");
	}

	#[test]
	fn file_size() {
		// These should come up zero.
		assert_eq!(super::file_size(&PathBuf::from("foo/bar.JS")), 0);
		assert_eq!(super::file_size(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))), 0);

		// And something we know.
		assert_eq!(super::file_size(&PathBuf::from("tests/assets/file.txt")), 26);
	}

	#[test]
	fn is_executable() {
		// These should come up false.
		assert_eq!(super::is_executable(&PathBuf::from("foo/bar.JS")), false);
		assert_eq!(super::is_executable(&PathBuf::from("tests/assets/file.txt")), false);
		assert_eq!(super::is_executable(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))), false);

		// But this should come up true.
		assert_eq!(super::is_executable(&PathBuf::from("tests/assets/is-executable.sh")), true);
	}

	#[test]
	fn parent() {
		// A known file.
		let file: PathBuf = PathBuf::from("./src/lib.rs");
		assert!(file.is_file());

		// The canonical parent.
		let dir: PathBuf = PathBuf::from("./src")
			.canonicalize()
			.expect("Parent, damn it.");
		assert!(dir.is_dir());

		// The two should match.
		assert_eq!(super::parent(&file).unwrap(), dir);

		// This should also work on directories.
		let dir2: PathBuf = PathBuf::from(".")
			.canonicalize()
			.expect("Parent, damn it.");
		assert!(dir2.is_dir());
		assert_eq!(super::parent(&dir).unwrap(), dir2);
	}

	#[test]
	fn to_path_buf_abs() {
		let path = PathBuf::from("src/lib.rs");
		let canon = path.canonicalize().expect("Canon, damn it!");

		assert_eq!(
			path.to_str().expect("Strings, damn it!"),
			"src/lib.rs",
		);
		assert_eq!(
			super::to_path_buf_abs(&path).to_str().expect("Strings, damn it!"),
			canon.to_str().expect("Strings, damn it!"),
		);
	}

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
