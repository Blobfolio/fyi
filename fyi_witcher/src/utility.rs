/*!
# FYI Witcher: Utility

This mod contains miscellaneous utility functions for the crate.
*/

use std::{
	borrow::Borrow,
	path::Path,
};



/// Ergonomical File Extension.
///
/// This one-liner returns the file extension as a lower-cased `String` for
/// easier comparisons. If the path is not a file or has no extension, an empty
/// string is returned instead.
pub fn file_extension<P> (path: P) -> String
where P: AsRef<Path> {
	let path = path.as_ref();
	if path.is_dir() {
		"".to_string()
	}
	else if let Some(ext) = path.extension() {
		ext.to_str()
			.unwrap_or_default()
			.to_lowercase()
	}
	else {
		"".to_string()
	}
}

/// Ergonomical File Size.
///
/// This method always returns a `u64`, either the file's size or `0` if the
/// path is invalid.
pub fn file_size<P> (path: P) -> u64
where P: AsRef<Path> {
	if let Ok(meta) = path.as_ref().metadata() {
		if meta.is_dir() { 0 }
		else { meta.len() }
	}
	else { 0 }
}

/// String Inflection
///
/// Given a number, come up with a byte string like "1 thing" or "2 things".
pub fn inflect<T1, T2> (num: u64, one: T1, many: T2) -> Vec<u8>
where
	T1: Borrow<str>,
	T2: Borrow<str> {
	if 1 == num {
		[49, 32].iter()
			.chain(one.borrow().as_bytes())
			.copied()
			.collect::<Vec<u8>>()
	}
	else if num < 1000 {
		let mut buf = itoa::Buffer::new();
		buf.format(num).as_bytes().iter()
			.chain(&[32])
			.chain(many.borrow().as_bytes())
			.copied()
			.collect::<Vec<u8>>()
	}
	else {
		let mut buf = num_format::Buffer::default();
		buf.write_formatted(&num, &num_format::Locale::en);
		buf.as_bytes().iter()
			.chain(&[32])
			.chain(many.borrow().as_bytes())
			.copied()
			.collect::<Vec<u8>>()
	}
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
	fn t_inflect() {
		assert_eq!(&inflect(0, "book", "books").as_ref(), b"0 books");
		assert_eq!(&inflect(1, "book", "books").as_ref(), b"1 book");
		assert_eq!(&inflect(1000, "book", "books").as_ref(), b"1,000 books");
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
