/*!
# FYI Witcher: Utility Methods.
*/

use rayon::{
	iter::ParallelIterator,
	prelude::IntoParallelIterator,
};
use std::path::{
	Path,
	PathBuf,
};



#[must_use]
/// # Disk Usage
///
/// This sums the provided file sizes in parallel.
///
/// ## Examples
///
/// ```no_run
/// let paths = [ "/path/one", "path/two", "path/three" ];
/// let size = fyi_witcher::utility::du(&paths);
/// ```
pub fn du<I, P>(src: I) -> u64
where P: AsRef<Path>, I: IntoParallelIterator<Item=P> {
	src.into_par_iter()
		.map(|p| std::fs::metadata(p).map_or(0, |m| m.len()))
		.sum()
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



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_du() {
		let files = vec![
			std::fs::canonicalize("./tests/assets/file.txt").unwrap(),
			std::fs::canonicalize("./tests/assets/functioning.JPEG").unwrap(),
			std::fs::canonicalize("./tests/assets/is-executable.sh").unwrap(),
		];

		let size = du(&files);
		assert_eq!(size, 30_383_u64);

		// Make sure ownership is still OK.
		assert_eq!(files.len(), 3);
	}

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
