/*!
# FYI Witcher: Witcher
*/

mod witch;

use ahash::AHashSet;
use crate::utility;
use rayon::iter::{
	ParallelBridge,
	ParallelDrainRange,
	ParallelIterator,
};
use std::{
	convert::TryFrom,
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
use witch::Witch;



/// Helper: Unlock the inner Mutex, handling poisonings inasmuch as is
/// possible.
macro_rules! mutex_ptr {
	($mutex:expr) => (
		$mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
	);
}



/// # Lowercase Mask.
///
/// An uppercase ASCII byte can be made lowercase by BIT-ORing its value
/// against this, like `b'J' | (1 << 5) == b'j'`.
///
/// This has no effect against digits or `-` or `a-z`, so can be used here
/// without the usual range checking.
const LOWER: u8 = 1 << 5;



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
/// ```
/// Box<dyn Fn(&PathBuf) -> bool + 'static + Send + Sync>
/// ```
///
/// ## Examples
///
/// ```no_run
/// use fyi_witcher::Witcher;
/// use fyi_witcher::utility;
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
	seen: AHashSet<Witch>,
	/// Filter callback.
	cb: Option<Box<dyn Fn(&PathBuf) -> bool + 'static + Send + Sync>>,
}

impl Default for Witcher {
	fn default() -> Self {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(2048),
			seen: AHashSet::with_capacity(2048),
			cb: None,
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
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_filter(|p: &PathBuf| { ... })
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_filter<F>(mut self, cb: F) -> Self
	where F: Fn(&PathBuf) -> bool + 'static + Send + Sync {
		self.cb = Some(Box::new(cb));
		self
	}

	#[must_use]
	/// # With Extension Filter.
	///
	/// This method can be faster for matching simple file extensions than
	/// [`with_regex()`](Witcher::with_regex), particularly if regular
	/// expressions are not used anywhere else.
	///
	/// ## Panics
	///
	/// The extension must include the leading period and be at least three
	/// characters in length.
	///
	/// ## Safety
	///
	/// This method uses some "unsafe" pointer-casting tricks that would be
	/// unsuitable in nearly any other context, but as we're comparing bytes
	/// and numbers — rather than strings — it works A-OK here.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_ext(b".jpg")
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_ext(mut self, ext: &[u8]) -> Self {
		let len: usize = ext.len();
		assert!(len > 2 && ext[0] == b'.', "Invalid extension.");

		// Specialize the matching given the length of the target extension.
		// Where possible, we'll manipulate the provided value outside the
		// closure to avoid loop/callback overhead.
		match len {
			// Like: .gz
			3 => {
				// Separate the dot and characters, comparing the latter as a
				// single u16.
				let (ext, mask) = unsafe {
					let m: u16 = *([LOWER, LOWER].as_ptr().cast::<u16>());
					let e: u16 = *(ext.as_ptr().add(1).cast::<u16>()) | m;
					(e, m)
				};

				self.cb = Some(Box::new(move |p: &PathBuf| {
					let path: &[u8] = utility::path_as_bytes(p);
					let p_len: usize = path.len();

					p_len > 3 &&
					path[p_len - 3] == b'.' &&
					ext == unsafe { *(path[p_len - 2..].as_ptr().cast::<u16>()) | mask }
				}));
			},
			// Like: .jpg
			4 => {
				// Convert the extension, dot and all, to a u32 for comparison.
				let (ext, mask) = unsafe {
					let m: u32 = *([0, LOWER, LOWER, LOWER].as_ptr().cast::<u32>());
					let e: u32 = *(ext.as_ptr().cast::<u32>()) | m;
					(e, m)
				};

				self.cb = Some(Box::new(move |p: &PathBuf| {
					let path: &[u8] = utility::path_as_bytes(p);
					let p_len: usize = path.len();

					p_len > 4 &&
					ext == unsafe { *(path[p_len - 4..].as_ptr().cast::<u32>()) | mask }
				}));
			},
			// Like: .html
			5 => {
				// Separate the dot and characters, comparing the latter as a
				// single u32.
				let (ext, mask) = unsafe {
					let m: u32 = *([LOWER, LOWER, LOWER, LOWER].as_ptr().cast::<u32>());
					let e: u32 = *(ext.as_ptr().add(1).cast::<u32>()) | m;
					(e, m)
				};

				self.cb = Some(Box::new(move |p: &PathBuf| {
					let path: &[u8] = utility::path_as_bytes(p);
					let p_len: usize = path.len();

					p_len > 5 &&
					path[p_len - 5] == b'.' &&
					ext == unsafe { *(path[p_len - 4..].as_ptr().cast::<u32>()) | mask }
				}));
			},
			// Like: .xhtml
			_ => {
				// While we could use u64 to specialize larger extensions, they
				// aren't really common enough to be worth it. Instead, we'll
				// just merge the strategies of [`slice::ends_with`] and
				// [`slice::eq_ignore_ascii_case`].
				let ext: Box<[u8]> = Box::from(ext.to_ascii_lowercase());
				self.cb = Some(Box::new(move |p: &PathBuf| {
					let path: &[u8] = utility::path_as_bytes(p);
					let p_len: usize = path.len();

					p_len > len &&
					path.iter()
						.skip(p_len - len)
						.zip(ext.iter())
						.all(|(a, b)| a.to_ascii_lowercase() == *b)
				}));
			}
		}

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
		self.cb = Some(Box::new(move|p: &PathBuf| pattern.is_match(utility::path_as_bytes(p))));
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
	///     .with_ext(b".jpg")
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
	///     .with_ext(b".jpg")
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn with_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Ok(path) = fs::canonicalize(path.as_ref()) {
			if let Ok(w) = Witch::try_from(&path) {
				if self.seen.insert(w) {
					if w.is_dir() {
						if let Ok(rd) = fs::read_dir(path) {
							self.dirs.push(rd);
						}
					}
					else {
						match self.cb {
							Some(ref cb) => if cb(&path) { self.files.push(path); },
							None => { self.files.push(path); }
						}
					}
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
	///     .with_ext(b".jpg")
	///     .with_path("/my/dir")
	///     .build();
	/// ```
	pub fn build(self) -> Vec<PathBuf> {
		// We don't have to do anything!
		if self.dirs.is_empty() {
			self.files
		}
		// Traverse without extra filtering.
		else if self.cb.is_none() {
			self.build_no_cb()
		}
		// Traverse with filtering.
		else {
			self.build_cb()
		}
	}

	/// # Build With Callback.
	fn build_cb(self) -> Vec<PathBuf> {
		// Break up the data.
		let Self { mut dirs, files, seen, cb } = self;
		let seen = Arc::from(Mutex::new(seen));
		let files = Arc::from(Mutex::new(files));
		let cb = cb.unwrap();

		// Process until we're our of directories.
		loop {
			dirs = dirs.par_drain(..)
				.flat_map(ParallelBridge::par_bridge)
				.filter_map(Witch::from_dent)
				.filter_map(|(w, p)|
					if mutex_ptr!(seen).insert(w) {
						if w.is_dir() { fs::read_dir(p).ok() }
						else {
							if cb(&p) {
								mutex_ptr!(files).push(p);
							}
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

	/// # Build Without Callback.
	fn build_no_cb(self) -> Vec<PathBuf> {
		// Break up the data.
		let Self { mut dirs, files, seen, .. } = self;
		let seen = Arc::from(Mutex::new(seen));
		let files = Arc::from(Mutex::new(files));

		// Process until we're our of directories.
		loop {
			dirs = dirs.par_drain(..)
				.flat_map(ParallelBridge::par_bridge)
				.filter_map(Witch::from_dent)
				.filter_map(|(w, p)|
					if mutex_ptr!(seen).insert(w) {
						if w.is_dir() { fs::read_dir(p).ok() }
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
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_ext(b".jpg")
	///     .with_path("/my/dir")
	///     .into_witching()
	///     .run(|p| { ... });
	/// ```
	pub fn into_witching(self) -> crate::Witching { crate::Witching::from(self.build()) }
}



#[cfg(test)]
mod tests {
	use super::*;
	use criterion as _;

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
			.with_ext(b".txt")
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 1);
	}
}
