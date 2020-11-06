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
#[inline]
/// # Time Chunks.
///
/// This method splits seconds into hours, minutes, and seconds. Days are not
/// supported; the maximum return value is `(23, 59, 59)`.
pub const fn hms_u64(num: u64) -> [u8; 3] { hms_u32(num as u32) }

#[must_use]
/// # Time Chunks.
///
/// This method splits seconds into hours, minutes, and seconds. Days are not
/// supported; the maximum return value is `(23, 59, 59)`.
pub const fn hms_u32(mut num: u32) -> [u8; 3] {
	if num < 60 { [0, 0, num as u8] }
	else if num < 86399 {
		let mut buf = [0_u8; 3];

		if num >= 3600 {
			buf[0] = ((num * 0x91A3) >> 27) as u8;
			num -= buf[0] as u32 * 3600;
		}
		if num >= 60 {
			buf[1] = ((num * 0x889) >> 17) as u8;
			buf[2] = (num - buf[1] as u32 * 60) as u8;
		}
		else if num > 0 { buf[2] = num as u8; }

		buf
	}
	else { [23, 59, 59] }
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

	#[test]
	fn t_hms_u64() {
		assert_eq!(hms_u64(1), [0_u8, 0_u8, 1_u8]);
		assert_eq!(hms_u64(30), [0_u8, 0_u8, 30_u8]);
		assert_eq!(hms_u64(90), [0_u8, 1_u8, 30_u8]);
		assert_eq!(hms_u64(3600), [1_u8, 0_u8, 0_u8]);

		// Make sure the numbers add up.
		for i in 0..86400_u32 {
			let test = hms_u32(i);
			assert_eq!(i, test[0] as u32 * 3600 + test[1] as u32 * 60 + test[2] as u32);
		}

		for i in 0..86400_u64 {
			let test = hms_u64(i);
			assert_eq!(i, test[0] as u64 * 3600 + test[1] as u64 * 60 + test[2] as u64);
		}
	}
}
