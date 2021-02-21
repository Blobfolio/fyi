/*!
# FYI Witcher: Witcher
*/

use ahash::AHashSet;
use crate::{
	AHASH_STATE,
	utility::{
		path_as_bytes,
		resolve_dir_entry,
		resolve_path,
	},
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



#[allow(missing_debug_implementations)]
/// `Witcher` is a very simple recursive file finder. Directories are read in
/// parallel. Files are canonicalized and deduped. Symlinks are followed.
/// Hidden files and directories are read like any other.
///
/// ## Filtering
///
/// Results can be filtered prior to being yielded with the use of either
/// [`with_filter()`](Witcher::with_filter) — specifying a custom callback method
/// — or [`with_regex()`](Witcher::with_regex) — to match against a pattern.
/// (The latter requires the `regexp` crate feature be enabled.)
///
/// It is important to define the filter *before* adding any paths, because if
/// those paths are files, they'll need to be filtered. Right? Right.
///
/// Filter callbacks should accept a `&PathBuf` and return `true` to keep it,
/// `false` to discard it. Ultimately, they get stored in the struct with the
/// following type:
///
/// ```ignore
/// Box<dyn Fn(&std::path::PathBuf) -> bool + 'static + Send + Sync>
/// ```
///
/// ## Examples
///
/// ```no_run
/// use fyi_witcher::Witcher;
/// use fyi_witcher::utility;
/// use std::path::PathBuf;
///
/// // Return all files under "/usr/share/man".
/// let res: Vec<PathBuf> = Witcher::default()
///     .with_path("/usr/share/man")
///     .build();
///
/// // Return only Gzipped files.
/// let res: Vec<PathBuf> = Witcher::default()
///     .with_regex(r"(?i).+\.gz$")
///     .with_path("/usr/share/man")
///     .build();
///
/// // If you're just matching one pattern, it can be faster to not use Regex:
/// let res: Vec<PathBuf> = Witcher::default()
///     .with_filter(|p: &PathBuf| {
///         let bytes: &[u8] = utility::path_as_bytes(p);
///         bytes.len() > 3 && bytes[bytes.len()-3..].eq_ignore_ascii_case(b".gz")
///     })
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
	/// Filter callback.
	cb: Box<dyn Fn(&PathBuf) -> bool + 'static + Send + Sync>,
}

impl Default for Witcher {
	fn default() -> Self {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(2048),
			seen: AHashSet::with_capacity_and_hasher(2048, AHASH_STATE),
			cb: Box::new(|_: &PathBuf| true),
		}
	}
}

impl Witcher {
	/// # With Callback.
	///
	/// Define a custom filter callback to determine whether or not a given
	/// file path should be yielded.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_filter(|p: &PathBuf| { ... })
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_filter<F>(mut self, cb: F) -> Self
	where F: Fn(&PathBuf) -> bool + 'static + Send + Sync {
		self.cb = Box::new(cb);
		self
	}

	#[cfg(feature = "regexp")]
	/// # With a Regex Callback.
	///
	/// This is a convenience method for filtering files by regular expression.
	///
	/// This method is only available when the `regexp` crate feature is
	/// enabled.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_regex(r"(?i).+\.jpe?g$")
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_regex<R>(mut self, reg: R) -> Self
	where R: std::borrow::Borrow<str> {
		use regex::bytes::Regex;
		let pattern: Regex = Regex::new(reg.borrow()).expect("Invalid Regex.");
		self.cb = Box::new(move|p: &PathBuf| pattern.is_match(path_as_bytes(p)));
		self
	}

	/// # With Paths.
	///
	/// Append files and/or directories to the finder. File paths will be
	/// checked against the filter callback (if any) and added straight to the
	/// results if they pass. Directories will be queued for later scanning.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
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
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Some((h, is_dir, p)) = resolve_path(PathBuf::from(path.as_ref()), false) {
			if self.seen.insert(h) {
				if is_dir {
					if let Ok(rd) = fs::read_dir(p) {
						self.dirs.push(rd);
					}
				}
				else if (self.cb)(&p) {
					self.files.push(p);
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
	/// use fyi_witcher::Witcher;
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
		let Self { mut dirs, files, seen, cb } = self;
		let seen = Arc::from(Mutex::new(seen));
		let files = Arc::from(Mutex::new(files));

		// Process until we're our of directories.
		loop {
			dirs = dirs.par_drain(..)
				.flat_map(ParallelBridge::par_bridge)
				.filter_map(resolve_dir_entry)
				.filter_map(|(h, is_dir, p)|
					if crate::mutex_ptr!(seen).insert(h) {
						if is_dir { fs::read_dir(p).ok() }
						else {
							if cb(&p) { crate::mutex_ptr!(files).push(p); }
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
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;
	use std::os::unix::ffi::OsStrExt;

	#[test]
	fn t_new() {
		let mut abs_dir = fs::canonicalize("tests/assets/").unwrap();
		abs_dir.push("_.txt");
		let abs_p1 = abs_dir.with_file_name("file.txt");
		let abs_p2 = abs_dir.with_file_name("is-executable.sh");
		let abs_perr = abs_dir.with_file_name("foo.bar");

		// Do a non-search search.
		let mut w1 = Witcher::default()
			.with_path(PathBuf::from("tests/"))
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 9);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));

		#[cfg(feature = "regexp")]
		{
			// Look only for .txt files.
			w1 = Witcher::default()
				.with_regex(r"(?i)\.txt$")
				.with_paths(&[PathBuf::from("tests/")])
				.build();
			assert!(! w1.is_empty());
			assert_eq!(w1.len(), 1);
			assert!(w1.contains(&abs_p1));
			assert!(! w1.contains(&abs_p2));
			assert!(! w1.contains(&abs_perr));

			// Look for something that doesn't exist.
			w1 = Witcher::default()
				.with_regex(r"(?i)\.exe$")
				.with_path(PathBuf::from("tests/"))
				.build();
			assert!(w1.is_empty());
			assert_eq!(w1.len(), 0);
			assert!(! w1.contains(&abs_p1));
			assert!(! w1.contains(&abs_p2));
			assert!(! w1.contains(&abs_perr));
		}

		// One Extension.
		w1 = Witcher::default()
			.with_path(PathBuf::from("tests/"))
			.with_filter(|p: &PathBuf| p.extension()
				.map_or(
					false,
					|e| e.as_bytes().eq_ignore_ascii_case(b"txt")
				)
			)
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 1);
	}
}
