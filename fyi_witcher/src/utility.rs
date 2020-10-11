/*!
# FYI Witcher: Utility Methods.
*/

#![allow(clippy::wildcard_imports)]

#[cfg(target_arch = "x86")]    use std::arch::x86::*;
#[cfg(target_arch = "x86_64")] use std::arch::x86_64::*;

use std::{
	ops::Range,
	path::Path,
};
use unicode_width::UnicodeWidthChar;



#[allow(clippy::naive_bytecount)] // This is a fallback.
#[must_use]
/// # Count Line Breaks.
///
/// This simply adds up the occurrences of `\n` within a byte string.
pub fn count_nl(src: &[u8]) -> usize {
	let len: usize = src.len();

	if 32 <= len && is_x86_feature_detected!("avx2") {
		unsafe { count_nl_avx2(src) }
	}
	else if 16 <= len && is_x86_feature_detected!("sse2") {
		unsafe { count_nl_sse2(src) }
	}
	else {
		src.iter().filter(|&&x| x == b'\n').count()
	}
}

#[allow(clippy::integer_division)] // It's fine.
#[allow(clippy::cast_possible_wrap)] // It's fine.
#[allow(clippy::cast_ptr_alignment)] // It's fine.
#[target_feature(enable = "avx2")]
/// # Count Line Breaks (AVX2).
///
/// This is an AVX2/SIMD-optimized implementation of the line counter. It is
/// used for strings that are at least 32 bytes.
unsafe fn count_nl_avx2(src: &[u8]) -> usize {
	const MASK: [u8; 64] = [
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
		255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	];

	let len: usize = src.len();
	let ptr = src.as_ptr();
	let needle = _mm256_set1_epi8(b'\n' as i8);

	let mut offset: usize = 0;
	let mut total = _mm256_setzero_si256();
	for _ in 0..len/32 {
		total = _mm256_sub_epi8(
			total,
			_mm256_cmpeq_epi8(_mm256_loadu_si256(ptr.add(offset) as *const _), needle)
		);
		offset += 32;
	}

	if offset < len {
		total = _mm256_sub_epi8(
			total,
			_mm256_and_si256(
				_mm256_cmpeq_epi8(_mm256_loadu_si256(ptr.add(len - 32) as *const _), needle),
				_mm256_loadu_si256(MASK.as_ptr().add(len - offset) as *const _)
			)
		);
	}

	let sums = _mm256_sad_epu8(total, _mm256_setzero_si256());
	(
		_mm256_extract_epi64(sums, 0) + _mm256_extract_epi64(sums, 1) +
		_mm256_extract_epi64(sums, 2) + _mm256_extract_epi64(sums, 3)
	) as usize
}

#[allow(clippy::integer_division)] // It's fine.
#[allow(clippy::cast_possible_wrap)] // It's fine.
#[allow(clippy::cast_ptr_alignment)] // It's fine.
#[target_feature(enable = "sse2")]
/// # Count Line Breaks (SSE2).
///
/// This is an SSE2/SIMD-optimized implementation of the line counter. It is
/// used for strings that are at least 16 bytes.
unsafe fn count_nl_sse2(src: &[u8]) -> usize {
	const MASK: [u8; 32] = [
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	];

	let len: usize = src.len();
	let ptr = src.as_ptr();
	let needle = _mm_set1_epi8(b'\n' as i8);

	let mut offset: usize = 0;
	let mut total = _mm_setzero_si128();
	for _ in 0..len/16 {
		total = _mm_sub_epi8(
			total,
			_mm_cmpeq_epi8(_mm_loadu_si128(ptr.add(offset) as *const _), needle)
		);
		offset += 16;
	}

	if offset < len {
		total = _mm_sub_epi8(
			total,
			_mm_and_si128(
				_mm_cmpeq_epi8(_mm_loadu_si128(ptr.add(len - 16) as *const _), needle),
				_mm_loadu_si128(MASK.as_ptr().add(len - offset) as *const _)
			)
		);
	}

	let sums = _mm_sad_epu8(total, _mm_setzero_si128());
	(_mm_extract_epi32(sums, 0) + _mm_extract_epi32(sums, 2)) as usize
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
	fn t_count_nl() {
		assert_eq!(count_nl(b"This has no line breaks."), 0);
		assert_eq!(count_nl(b"This\nhas\ntwo line breaks."), 2);
		assert_eq!(count_nl(&[10_u8; 63]), 63);
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
