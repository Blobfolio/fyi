/*!
# FYI Witcher: Utility Methods.
*/

use std::path::{
	Path,
	PathBuf,
};



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
	let mut hasher = ahash::AHasher::new_with_keys(1319, 2371);
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
/// use std::path::PathBuf;
/// let path = fyi_witcher::utility::path_as_bytes(&PathBuf::from("/path/to/file.jpg"));
/// ```
pub fn path_as_bytes(p: &Path) -> &[u8] {
	unsafe { &*(p.as_os_str() as *const std::ffi::OsStr as *const [u8]) }
}

/// # Resolve `DirEntry`.
///
/// This is a convenience callback for [`Witcher`] and [`lite::Witcher`] used
/// during `ReadDir` traversal.
///
/// See [`resolve_path`] for more information.
pub(crate) fn resolve_dir_entry(entry: Result<std::fs::DirEntry, std::io::Error>) -> Option<(u128, bool, PathBuf)> {
	let entry = entry.ok()?;
	resolve_path(entry.path(), true)
}

/// # Resolve Path.
///
/// This attempts to cheaply resolve a given path, returning:
/// * A unique hash derived from the path's device and inode.
/// * A bool indicating whether or not the path is a directory.
/// * The canonicalized path.
///
/// As [`std::fs::canonicalize`] is an expensive operation, this method allows
/// a "trusted" bypass, which will only canonicalize the path if it is a
/// symlink.
///
/// The trusted mode is only appropriate in cases like `ReadDir` where the
/// directory seed was canonicalized. The idea is that since `DirEntry` paths
/// are joined to the seed, they'll be canonical so long as the seed was,
/// except in cases of symlinks.
pub(crate) fn resolve_path(path: PathBuf, trusted: bool) -> Option<(u128, bool, PathBuf)> {
	use std::os::unix::fs::MetadataExt;

	let meta = std::fs::metadata(&path).ok()?;
	let hash: u128 = unsafe { *([meta.dev(), meta.ino()].as_ptr().cast::<u128>()) };
	let dir: bool = meta.is_dir();

	if trusted {
		let meta = std::fs::symlink_metadata(&path).ok()?;
		if ! meta.file_type().is_symlink() {
			return Some((hash, dir, path));
		}
	}

	let path = std::fs::canonicalize(path).ok()?;
	Some((hash, dir, path))
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
pub fn term_width() -> u32 {
	term_size::dimensions().map_or(0, |(w, _)| (w as u32).saturating_sub(1))
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_resolve_path() {
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
				.filter_map(|x| resolve_path(x.clone(), true).map(|(_, _, p)| p))
				.collect();
			tmp.sort();
			tmp.dedup();
			tmp
		};

		assert_eq!(trusting, canon);
	}
}
