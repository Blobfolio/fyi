/*!
# FYI Witcher: Utility Methods.
*/

use std::{
	ops::Range,
	path::Path,
};
use unicode_width::UnicodeWidthChar;



#[must_use]
/// # Fit Length
///
/// This method returns the maximum slice range that will "fit" a given
/// printable "width". It could be the entire thing, or it might be some
/// smaller chunk.
///
/// This is at best an approximation as the concept of "width" is mysterious
/// and unknowable, apparently. See [`unicode_width`](https://crates.io/crates/unicode-width) for a
/// list of gotchas.
pub fn fitted_range(src: &[u8], width: usize) -> Range<usize> {
	// Width cannot exceed length, so we only need to dig deeper if the length
	// is bigger.
	let len: usize = src.len();
	if len > width {
		let mut total_len: usize = 0;
		let mut total_width: usize = 0;

		// For our purposes, basic ANSI markup (of the kind used by `FYI`) is
		// considered zero-width.
		let mut in_ansi: bool = false;

		// Convert to a string slice so we can iterate over individual chars.
		for c in unsafe { std::str::from_utf8_unchecked(src) }.chars() {
			// Find the "length" of this char.
			let ch_len: usize = c.len_utf8();
			total_len += ch_len;

			// If we're in the middle of an ANSI sequence nothing counts, but
			// we need to watch for the end marker so we can start paying
			// attention again.
			if in_ansi {
				// We're only interested in A/K/m signals.
				if c == 'A' || c == 'K' || c == 'm' { in_ansi = false; }
				continue;
			}
			// Are we entering an ANSI sequence?
			else if c == '\x1b' {
				in_ansi = true;
				continue;
			}

			// The width matters!
			let ch_width: usize = UnicodeWidthChar::width(c).unwrap_or_default();
			total_width += ch_width;

			// Widths can creep up unevenly. If we've gone over, we need to
			// back up a step and exit.
			if total_width > width {
				return 0..total_len-ch_len
			}
		}
	}

	0..len
}

#[must_use]
#[inline]
/// # `AHash` Byte Hash.
///
/// This is a convenience method for quickly hashing bytes using the
/// [`AHash`](https://crates.io/crates/ahash) crate. Check out that project's
/// home page for more details. Otherwise, TL;DR it is very fast.
///
/// ## Examples
///
/// ```no_run
/// let hash = fyi_msg::utility::hash64(b"Hello World");
/// ```
pub fn hash64(src: &[u8]) -> u64 {
	use std::hash::Hasher;
	let mut hasher = ahash::AHasher::default();
	hasher.write(src);
	hasher.finish()
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

	path.as_ref()
		.metadata()
		.ok()
		.filter(std::fs::Metadata::is_file)
		.map_or(false, |m| m.permissions().mode() & 0o111 != 0)
}

#[allow(trivial_casts)] // We need triviality!
#[must_use]
#[inline]
/// # Path to Bytes.
///
/// This is exactly the way [`std::path::PathBuf`] handles it.
pub fn path_as_bytes(p: &std::path::PathBuf) -> &[u8] {
	unsafe { &*(p.as_os_str() as *const std::ffi::OsStr as *const [u8]) }
}

#[must_use]
#[inline]
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
	fn t_fitted_range() {
		assert_eq!(fitted_range(b"Hello World", 15), 0..11);
		assert_eq!(fitted_range(b"Hello \x1b[1mWorld\x1b[0m", 15), 0..19);
		assert_eq!(fitted_range(b"Hello \x1b[1mWorld\x1b[0m", 7), 0..11);
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
