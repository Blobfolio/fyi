/*!
# FYI Witcher: Utility Methods
*/

use std::{
	path::{
		Path,
		PathBuf,
	},
};



#[must_use]
/// Total File(s) Size.
///
/// Add up the size of all files in a set. Calculations are run in parallel so
/// should be fairly fast depending on the file system.
pub fn du(paths: &[PathBuf]) -> u64 {
	use rayon::prelude::*;
	paths.par_iter()
		.map(|x| x.metadata().map_or(0, |m| m.len()))
		.sum()
}

/// Ergonomical File Extension.
///
/// This one-liner returns the file extension as a lower-cased `String` for
/// easier comparisons. If the path is not a file or has no extension, an empty
/// string is returned instead.
pub fn file_extension<P> (path: P) -> String
where P: AsRef<Path> {
	let path = path.as_ref();
	if path.is_dir() { String::new() }
	else {
		path.extension().map_or(
			String::new(),
			|ext| ext.to_str()
				.unwrap_or_default()
				.to_lowercase()
		)
	}
}

/// Ergonomical File Size.
///
/// This method always returns a `u64`, either the file's size or `0` if the
/// path is invalid.
pub fn file_size<P> (path: P) -> u64
where P: AsRef<Path> {
	path.as_ref()
		.metadata()
		.map_or(
			0,
			|meta|
				if meta.is_dir() { 0 }
				else { meta.len() }
		)
}

/// Is File Executable?
///
/// This method attempts to determine whether or not a file has executable
/// permissions (generally). If the path is not a file, `false` is returned.
pub fn is_executable<P> (path: P) -> bool
where P: AsRef<Path> {
	use std::os::unix::fs::PermissionsExt;

	if let Ok(meta) = path.as_ref().metadata() {
		if meta.is_file() {
			let permissions = meta.permissions();
			return permissions.mode() & 0o111 != 0;
		}
	}

	false
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_file_extension() {
		_file_extension("/dev/null", "");
		_file_extension(env!("CARGO_MANIFEST_DIR"), "");
		_file_extension("/a/file/foo.JPG", "jpg");
		_file_extension("src/lib.rs", "rs");
	}

	fn _file_extension<P> (path: P, expected: &str)
	where P: AsRef<Path> {
		assert_eq!(
			&file_extension(&path),
			expected,
			"{:?} should have a file extension of {:?}",
			path.as_ref(),
			expected
		);
	}

	#[test]
	fn t_file_size() {
		_file_size("/dev/null", 0);
		_file_size(env!("CARGO_MANIFEST_DIR"), 0);
		_file_size("/a/file/foo.JPG", 0);
		_file_size("tests/assets/file.txt", 26);
	}

	fn _file_size<P> (path: P, expected: u64)
	where P: AsRef<Path> {
		assert_eq!(
			file_size(&path),
			expected,
			"{:?} should have a file size of {:?}",
			path.as_ref(),
			expected
		);
	}

	#[test]
	fn t_is_executable() {
		_is_executable("/dev/null", false);
		_is_executable(env!("CARGO_MANIFEST_DIR"), false);
		_is_executable("/a/file/foo.JPG", false);
		_is_executable("tests/assets/file.txt", false);
		_is_executable("tests/assets/is-executable.sh", true);
	}

	fn _is_executable<P> (path: P, expected: bool)
	where P: AsRef<Path> {
		assert_eq!(
			is_executable(&path),
			expected,
			"expected is_executable({:?}) = {:?}",
			path.as_ref(),
			expected
		);
	}
}
