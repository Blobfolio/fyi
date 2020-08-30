/*!
# FYI Witcher: Utility Methods.
*/

use std::path::Path;



#[must_use]
/// # Ends With Ignore ASCII Case.
///
/// This combines `ends_with()` and `eq_ignore_ascii_case()`, but skips an
/// operation by assuming the needle `end` is already in lower case.
///
/// ## Examples
///
/// ```no_run
/// assert!(
///     fyi_witcher::utility::ends_with_ignore_ascii_case(
///         b"/home/usr/Images/picture.JPG",
///         b".jpg"
///     )
/// );
/// ```
pub fn ends_with_ignore_ascii_case(src: &[u8], end: &[u8]) -> bool {
	let (m, n) = (src.len(), end.len());
	m >= n && src.iter().skip(m - n).zip(end).all(|(a, b)| a.to_ascii_lowercase() == *b)
}

/// # Ergonomical File Extension.
///
/// This one-liner returns the file extension as a lower-cased `String` for
/// easier comparisons. If the path is not a file or has no extension, an empty
/// string is returned instead.
///
/// ## Examples
///
/// ```no_run
/// assert_eq!(
///     fyi_witcher::utility::file_extension("../file.JPEG"),
///     String::from(".jpeg")
/// );
/// ```
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

/// # Ergonomical File Size.
///
/// This method always returns a `u64`, either the file's size or `0` if the
/// path is invalid.
///
/// ## Examples
///
/// ```no_run
/// let size: u64 = fyi_witcher::utility::file_size("../file.jpeg");
/// ```
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

/// # Is File Executable?
///
/// This method attempts to determine whether or not a file has executable
/// permissions (generally). If the path is not a file, `false` is returned.
///
/// ```no_run
/// if fyi_witcher::utility::is_executable("./my-script.sh") { ... }
/// ```
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

#[must_use]
/// # Chunked Seconds.
///
/// This method converts seconds into hours, minutes, and seconds, returning
/// a fixed-length array with each value in order, e.g. `[h, m, s]`.
///
/// As with the rest of the methods in this crate, days and beyond are not
/// considered. Large values are simply truncated to `86399`, i.e. one second
/// shy of a full day.
///
/// ## Examples
///
/// ```no_run
/// use fyi_witcher::utility::secs_chunks;
///
/// assert_eq!(secs_chunks(1), [0, 0, 1]);
/// assert_eq!(secs_chunks(90), [0, 1, 30]);
/// ```
pub fn secs_chunks(num: u32) -> [u32; 3] {
	let mut out: [u32; 3] = [0, 0, u32::min(86399, num)];

	// Hours.
	if out[2] >= 3600 {
		out[0] = num_integer::div_floor(out[2], 3600);
		out[2] -= out[0] * 3600;
	}

	// Minutes.
	if out[2] >= 60 {
		out[1] = num_integer::div_floor(out[2], 60);
		out[2] -= out[1] * 60;
	}

	out
}

#[must_use]
/// # Term Width.
///
/// This is a simple wrapper around `term_size::dimensions()` to provide
/// the current terminal column width. We don't have any use for height,
/// so that property is ignored.
///
/// Note: The width returned will be `1` less than the actual value to mitigate
/// any whitespace weirdness that might be lurking at the edge.
pub fn term_width() -> usize {
	term_size::dimensions().map_or(0, |(w, _)| w.saturating_sub(1))
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_ends_with_ignore_ascii_case() {
		assert!(
			ends_with_ignore_ascii_case(b"/path/to/file.jpg", b".jpg")
		);
		assert!(
			ends_with_ignore_ascii_case(b"/path/to/file.JPG", b".jpg")
		);
		assert!(
			! ends_with_ignore_ascii_case(b"/path/to/file.jpeg", b".jpg")
		);
	}

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

	#[test]
	fn t_secs_chunks() {
		assert_eq!(secs_chunks(1), [0, 0, 1]);
		assert_eq!(secs_chunks(30), [0, 0, 30]);
		assert_eq!(secs_chunks(90), [0, 1, 30]);
		assert_eq!(secs_chunks(3600), [1, 0, 0]);
	}
}
