/*!
# FYI Witcher: Utility Methods.
*/

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
/// let hash = fyi_witcher::utility::hash64(b"Hello World");
/// ```
pub fn hash64(src: &[u8]) -> u64 {
	use std::hash::Hasher;
	let mut hasher = ahash::AHasher::default();
	hasher.write(src);
	hasher.finish()
}

#[allow(trivial_casts)] // We need triviality!
#[must_use]
#[inline]
/// # Path to Bytes.
///
/// This is exactly the way [`std::path::PathBuf`] handles it.
///
/// ## Examples
///
/// ```no_run
/// let path = fyi_witcher::utility::path_as_bytes(PathBuf::from("/path/to/file.jpg"));
/// ```
pub fn path_as_bytes(p: &std::path::PathBuf) -> &[u8] {
	unsafe { &*(p.as_os_str() as *const std::ffi::OsStr as *const [u8]) }
}

#[cfg(feature = "witching")]
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
///
/// This method requires the `witching` crate feature be enabled.
pub fn term_width() -> usize {
	term_size::dimensions().map_or(0, |(w, _)| w.saturating_sub(1))
}
