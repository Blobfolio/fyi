/*!
# FYI Witcher: Witcher
*/

use ahash::AHashSet;
use crate::utility::{
	resolve_dir_entry,
	resolve_path,
};
use rayon::iter::{
	ParallelBridge,
	ParallelDrainRange,
	ParallelIterator,
};
use std::{
	fs::{
		self,
		ReadDir,
	},
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		Mutex,
	},
};



/// Helper: Unlock the inner Mutex, handling poisonings inasmuch as is
/// possible.
macro_rules! mutex_ptr {
	($mutex:expr) => (
		$mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
	);
}



#[allow(missing_debug_implementations)]
/// `Witcher` is a very simple recursive file finder. Directories are read in
/// parallel. Files are canonicalized and deduped. Symlinks are followed.
/// Hidden files and directories are read like any other.
///
/// ## Filtering
///
/// This "lite" version specifically omits filtering. It will simply crawl and
/// return all files found.
///
/// ## Examples
///
/// ```no_run
/// use fyi_witcher::lite::Witcher;
///
/// // Return all files under "/usr/share/man".
/// let res: Vec<PathBuf> = Witcher::default()
///     .with_path("/usr/share/man")
///     .build();
/// ```
pub struct Witcher {
	/// Directories to scan.
	dirs: Vec<ReadDir>,
	/// Files found.
	files: Vec<PathBuf>,
	/// Unique path hashes (to prevent duplicate scans, results).
	seen: AHashSet<u128>,
}

impl Default for Witcher {
	fn default() -> Self {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(2048),
			seen: AHashSet::with_capacity(2048),
		}
	}
}

impl Witcher {
	/// # With Paths.
	///
	/// Append files and/or directories to the finder. File paths will be
	/// checked against the filter callback (if any) and added straight to the
	/// results if they pass. Directories will be queued for later scanning.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::lite::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_paths(&["/my/dir"])
	///     .build();
	/// ```
	pub fn with_paths<P, I>(self, paths: I) -> Self
	where
		P: AsRef<Path>,
		I: IntoIterator<Item=P> {
		paths.into_iter().fold(self, Self::with_path)
	}

	/// # With Path.
	///
	/// Add a path to the finder. If the path is a file, it will be checked
	/// against the filter callback (if any) before being added to the results.
	/// If it is a directory, it will be queued for later scanning.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::lite::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Some((hash, dir, path)) = resolve_path(PathBuf::from(path.as_ref()), false) {
			if self.seen.insert(hash) {
				if dir {
					if let Ok(rd) = fs::read_dir(path) {
						self.dirs.push(rd);
					}
				}
				else {
					self.files.push(path);
				}
			}
		}

		self
	}

	#[must_use]
	/// # Build!
	///
	/// Once everything is set up, call this method to consume the queue and
	/// collect the files into a `Vec<PathBuf>`, consuming `self`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::lite::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn build(self) -> Vec<PathBuf> {
		// We don't have to do anything!
		if self.dirs.is_empty() {
			return self.files;
		}

		// Break up the data.
		let Self { mut dirs, files, seen } = self;
		let seen = Arc::from(Mutex::new(seen));
		let files = Arc::from(Mutex::new(files));

		// Process until we're our of directories.
		loop {
			dirs = dirs.par_drain(..)
				.flat_map(ParallelBridge::par_bridge)
				.filter_map(resolve_dir_entry)
				.filter_map(|(hash, dir, p)|
					if mutex_ptr!(seen).insert(hash) {
						if dir { fs::read_dir(p).ok() }
						else {
							mutex_ptr!(files).push(p);
							None
						}
					}
					else { None }
				)
				.collect();

			if dirs.is_empty() { break; }
		}

		Arc::<Mutex<Vec<PathBuf>>>::try_unwrap(files)
			.ok()
			.and_then(|x| x.into_inner().ok())
			.unwrap_or_default()
	}

	#[cfg(feature = "witching")]
	#[must_use]
	/// # Build (into Progress)!
	///
	/// This is identical to [`build()`](Witcher::build), except a ready-to-go
	/// [`Witching`] struct is returned instead of a vector.
	///
	/// This method requires the crate feature `witching` be enabled.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::lite::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .into_witching()
	///     .run(|p| { ... });
	/// ```
	pub fn into_witching(self) -> crate::Witching { crate::Witching::from(self.build()) }
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_new() {
		let mut abs_dir = fs::canonicalize("tests/assets/").unwrap();
		abs_dir.push("_.txt");
		let abs_p1 = abs_dir.with_file_name("file.txt");
		let abs_p2 = abs_dir.with_file_name("is-executable.sh");
		let abs_perr = abs_dir.with_file_name("foo.bar");

		// Do a non-search search.
		let w1 = Witcher::default()
			.with_path(PathBuf::from("tests/"))
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 9);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
	}
}
