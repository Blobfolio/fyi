/*!
# FYI Witcher: Utility Methods.
*/

use std::path::PathBuf;



/// # Trusting Canonicalize
///
/// This method is solely intended for use in cases where a path has been
/// constructed from a known-canonical path, i.e. during directory traversal
/// (where the directory was canonicalized before reading).
///
/// In order to avoid the expense of `realpath` syscalls, this method will
/// first do an `lstat` to see if the path is a symlink. If not, the in-path is
/// transparently returned.
pub(crate) fn trusting_canonicalize(path: PathBuf) -> Result<PathBuf, std::io::Error> {
	let meta = std::fs::symlink_metadata(&path)?;
	if meta.file_type().is_symlink() {
		std::fs::canonicalize(path)
	}
	else { Ok(path) }
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



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_trusting_canonicalize() {
		let test_dir = std::fs::canonicalize("./tests/links").expect("Missing witcher link directory.");

		let raw = vec![
			test_dir.join("01"),
			test_dir.join("02"),
			test_dir.join("03"),
			test_dir.join("04"),
			test_dir.join("05"),
			test_dir.join("06"),
			test_dir.join("07"), // Sym to six.
			test_dir.join("06/08"),
			test_dir.join("06/09"),
			test_dir.join("06/10"), // Sym to one.
		];

		let canon = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|x| std::fs::canonicalize(x).ok())
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		// There should be two fewer entries as two are symlinks.
		assert_eq!(raw.len(), 10);
		assert_eq!(canon.len(), 8);
		assert!(! canon.contains(&raw[6]));
		assert!(! canon.contains(&raw[9]));

		let trusting = {
			let mut tmp: Vec<PathBuf> = raw.iter()
				.filter_map(|x| trusting_canonicalize(x.clone()).ok())
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		assert_eq!(trusting, canon);
	}
}
