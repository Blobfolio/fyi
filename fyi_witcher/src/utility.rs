/*!
# FYI Witcher: Utility Methods.
*/

#[cfg(feature = "simd")]
use packed_simd::{
	u8x4,
	u8x8,
	u8x16,
	u8x32,
	u8x64,
};

use std::{
	ops::Range,
	path::Path,
};
use unicode_width::UnicodeWidthChar;



#[cfg(not(feature = "simd"))]
#[must_use]
/// # Count Line Breaks.
///
/// This simply adds up the occurrences of `\n` within a byte string.
pub fn count_nl(src: &[u8]) -> usize {
	src.iter().filter(|v| **v == b'\n').count()
}

#[cfg(feature = "simd")]
#[must_use]
/// # Count Line Breaks.
///
/// This simply adds up the occurrences of `\n` within a byte string.
pub fn count_nl(src: &[u8]) -> usize {
	let len: usize = src.len();
	let mut offset: usize = 0;
	let mut total: usize = 0;

	// We're checking lengths all along the way, so this isn't really unsafe.
	unsafe {
		// Break indefinitely long strings into chunks of 64 characters, counting
		// newlines as we go.
		if offset + 64 <= len {
			let mut tmp = u8x64::splat(0);
			loop {
				tmp += u8x64::from_slice_unaligned_unchecked(&src[offset..offset+64])
					.eq(u8x64::splat(b'\n'))
					.select(u8x64::splat(1), u8x64::splat(0));

				offset += 64;
				if len < offset + 64 { break; }
			}
			total += tmp.wrapping_sum() as usize;
		}

		// We can use the same trick for progressively smaller power-of-two-sized
		// chunks, but none of these will hit more than once, so their totals can
		// be added directly without looping.
		if offset + 32 <= len {
			total += u8x32::from_slice_unaligned_unchecked(&src[offset..offset+32])
				.eq(u8x32::splat(b'\n'))
				.select(u8x32::splat(1), u8x32::splat(0))
				.wrapping_sum() as usize;
			offset += 32;
		}

		if offset + 16 <= len {
			total += u8x16::from_slice_unaligned_unchecked(&src[offset..offset+16])
				.eq(u8x16::splat(b'\n'))
				.select(u8x16::splat(1), u8x16::splat(0))
				.wrapping_sum() as usize;
			offset += 16;
		}

		if offset + 8 <= len {
			total += u8x8::from_slice_unaligned_unchecked(&src[offset..offset+8])
				.eq(u8x8::splat(b'\n'))
				.select(u8x8::splat(1), u8x8::splat(0))
				.wrapping_sum() as usize;
			offset += 8;
		}

		if offset + 4 <= len {
			total += u8x4::from_slice_unaligned_unchecked(&src[offset..offset+4])
				.eq(u8x4::splat(b'\n'))
				.select(u8x4::splat(1), u8x4::splat(0))
				.wrapping_sum() as usize;
			offset += 4;
		}
	}

	// The last few bytes have to be checked manually, but that's fine. The
	// remainder can't be much.
	while offset < len {
		if src[offset] == b'\n' { total += 1; }
		offset += 1;
	}

	total
}

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

#[allow(clippy::integer_division)]
#[must_use]
/// # Time Chunks.
///
/// This method splits seconds into hours, minutes, and seconds. Days are not
/// supported; the maximum return value is `(23, 59, 59)`.
pub const fn hms_u64(num: u64) -> [u8; 3] { hms_u32(num as u32) }

#[allow(clippy::integer_division)]
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

#[must_use]
#[allow(trivial_casts)] // We need triviality!
/// # Path to Bytes.
///
/// This is exactly the way [`std::path::PathBuf`] handles it.
pub fn path_as_bytes(p: &std::path::PathBuf) -> &[u8] {
    unsafe { &*(p.as_os_str() as *const std::ffi::OsStr as *const [u8]) }
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
	fn t_count_nl() {
		assert_eq!(count_nl(b"This has no line breaks."), 0);
		assert_eq!(count_nl(b"This\nhas\ntwo line breaks."), 2);
		assert_eq!(count_nl(&[10_u8; 63]), 63);
	}

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
