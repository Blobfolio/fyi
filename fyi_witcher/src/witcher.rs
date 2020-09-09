/*!
# FYI Witcher: Witcher
*/

use ahash::{
	AHasher,
	AHashSet
};

use crate::{
	utility,
	Witching,
};

#[cfg(feature = "simd")] use packed_simd::u8x8;

use rayon::prelude::*;
use std::{
	borrow::Borrow,
	fs,
	hash::Hasher,
	path::{
		Path,
		PathBuf,
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
///
/// It is important to define the filter *before* adding any paths, because if
/// those paths are files, they'll need to be filtered. Right? Right.
///
/// Filter callbacks should accept a `&PathBuf` and return `true` to keep it,
/// `false` to discard it.
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
	dirs: Vec<PathBuf>,
	/// Files found.
	files: Vec<PathBuf>,
	/// Unique path hashes (to prevent duplicate scans, results).
	seen: AHashSet<u64>,
	/// Filter callback.
	cb: Box<dyn Fn(&PathBuf) -> bool + 'static>,
}

impl Default for Witcher {
	fn default() -> Self {
		Self {
			dirs: Vec::new(),
			files: Vec::with_capacity(2048),
			seen: AHashSet::with_capacity(2048),
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
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_filter(|p: &PathBuf| { ... })
	///     .build();
	/// ```
	pub fn with_filter<F>(mut self, cb: F) -> Self
	where F: Fn(&PathBuf) -> bool + 'static {
		self.cb = Box::new(cb);
		self
	}

	#[cfg(not(feature = "simd"))]
	#[must_use]
	/// # With Extension Filter.
	///
	/// This method — and [`with_ext2()`](Witcher::with_ext2), [`with_ext3()`](Witcher::with_ext3) — can be faster for
	/// matching simple file extensions than [`with_regex()`](Witcher::with_regex),
	/// particularly if regular expressions are not used anywhere else.
	///
	/// Note: The extension should include the leading period and be in lower case.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext1(b".jpg")
	///     .build();
	/// ```
	pub fn with_ext1(mut self, ext: &'static [u8]) -> Self {
		self.cb = Box::new(move |p: &PathBuf|
			utility::ends_with_ignore_ascii_case(utility::path_as_bytes(p), ext)
		);
		self
	}

	#[cfg(not(feature = "simd"))]
	#[must_use]
	/// # With Extensions (2) Filter.
	///
	/// Note: The extension should include the leading period and be in lower case.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext2(b".jpg", b".jpeg")
	///     .build();
	/// ```
	pub fn with_ext2(mut self, ext1: &'static [u8], ext2: &'static [u8]) -> Self {
		self.cb = Box::new(move |p: &PathBuf| {
			let bytes: &[u8] = utility::path_as_bytes(p);
			utility::ends_with_ignore_ascii_case(bytes, ext1) ||
			utility::ends_with_ignore_ascii_case(bytes, ext2)
		});
		self
	}

	#[cfg(not(feature = "simd"))]
	#[must_use]
	/// # With Extensions (3) Filter.
	///
	/// Note: The extension should include the leading period and be in lower case.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext3(b".jpg", b".jpeg", b".png")
	///     .build();
	/// ```
	pub fn with_ext3(
		mut self,
		ext1: &'static [u8],
		ext2: &'static [u8],
		ext3: &'static [u8]
	) -> Self {
		self.cb = Box::new(move |p: &PathBuf| {
			let bytes: utility::path_as_bytes(p);
			utility::ends_with_ignore_ascii_case(bytes, ext1) ||
			utility::ends_with_ignore_ascii_case(bytes, ext2) ||
			utility::ends_with_ignore_ascii_case(bytes, ext3)
		});
		self
	}

	#[cfg(feature = "simd")]
	#[must_use]
	/// # With Extension Filter.
	///
	/// This method — and [`with_ext2()`](Witcher::with_ext2), [`with_ext3()`](Witcher::with_ext3) — can be faster for
	/// matching simple file extensions than [`with_regex()`](Witcher::with_regex),
	/// particularly if regular expressions are not used anywhere else.
	///
	/// Note: The extension should include the leading period and be in lower case.
	///
	/// ## Panics
	///
	/// This method will panic if the extension is less than 2 bytes.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext1(b".jpg")
	///     .build();
	/// ```
	pub fn with_ext1(mut self, ext1: &[u8]) -> Self {
		self.cb = {
			let splat = u8x8::splat(0);
			let ext1 = with_ext_key(ext1);
			let mask1 = m8x8::from_cast(ext1);
			Box::new(move |p: &PathBuf|
				mask1.select(
					with_ext_key(
						unsafe { &*(p.as_os_str() as *const OsStr as *const [u8]) }
					),
					splat
				) == ext1
			)
		};

		self
	}

	#[cfg(feature = "simd")]
	#[must_use]
	/// # With Extensions (2) Filter.
	///
	/// Note: The extension should include the leading period and be in lower case.
	///
	/// ## Panics
	///
	/// This method will panic if any extension is less than 2 bytes.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext2(b".jpg", b".jpeg")
	///     .build();
	/// ```
	pub fn with_ext2(mut self, ext1: &[u8], ext2: &[u8]) -> Self {
		self.cb = {
			let splat = u8x8::splat(0);
			let ext1 = with_ext_key(ext1);
			let ext2 = with_ext_key(ext2);
			let mask1 = m8x8::from_cast(ext1);
			let mask2 = m8x8::from_cast(ext2);

			Box::new(move |p: &PathBuf| {
				let src = with_ext_key(
					unsafe { &*(p.as_os_str() as *const OsStr as *const [u8]) }
				);

				mask1.select(src, splat) == ext1 ||
				mask2.select(src, splat) == ext2
			})
		};

		self
	}

	#[cfg(feature = "simd")]
	#[must_use]
	/// # With Extensions (3) Filter.
	///
	/// Note: The extension should include the leading period and be in lower case.
	///
	/// ## Panics
	///
	/// This method will panic if any extension is less than 2 bytes.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext3(b".jpg", b".jpeg", b".png")
	///     .build();
	/// ```
	pub fn with_ext3(
		mut self,
		ext1: &'static [u8],
		ext2: &'static [u8],
		ext3: &'static [u8]
	) -> Self {
		self.cb = {
			let splat = u8x8::splat(0);
			let ext1 = with_ext_key(ext1);
			let ext2 = with_ext_key(ext2);
			let ext3 = with_ext_key(ext3);
			let mask1 = m8x8::from_cast(ext1);
			let mask2 = m8x8::from_cast(ext2);
			let mask3 = m8x8::from_cast(ext3);

			Box::new(move |p: &PathBuf| {
				let src = with_ext_key(
					unsafe { &*(p.as_os_str() as *const OsStr as *const [u8]) }
				);

				mask1.select(src, splat) == ext1 ||
				mask2.select(src, splat) == ext2 ||
				mask3.select(src, splat) == ext3
			})
		};

		self
	}

	/// # With a Regex Callback.
	///
	/// This is a convenience method for filtering files by regular expression.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_regex(r"(?i).+\.jpe?g$")
	///     .build();
	/// ```
	pub fn with_regex<R>(mut self, reg: R) -> Self
	where R: Borrow<str> {
		use regex::bytes::Regex;
		let pattern: Regex = Regex::new(reg.borrow()).expect("Invalid Regex.");
		self.cb = Box::new(move|p: &PathBuf| pattern.is_match(utility::path_as_bytes(p)));
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
	///     .with_ext1(b".jpg")
	///     .build();
	/// ```
	pub fn with_paths<P>(self, paths: &[P]) -> Self
	where P: AsRef<Path> {
		paths.iter().fold(self, Self::with_path)
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
	///     .with_ext1(b".jpg")
	///     .build();
	/// ```
	pub fn with_path<P>(mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Ok(path) = fs::canonicalize(path.as_ref()) {
			if self.seen.insert(hash_path_buf(&path)) {
				if path.is_dir() {
					self.dirs.push(path);
				}
				else if (self.cb)(&path) {
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
	/// collect the files into a `Vec<PathBuf>`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext1(b".jpg")
	///     .build();
	/// ```
	pub fn build(mut self) -> Vec<PathBuf> {
		self.digest();
		self.files
	}

	#[must_use]
	/// # Build (into Progress)!
	///
	/// This is identical to [`build()`](Witcher::build), except a ready-to-go
	/// [`Witching`] struct is returned instead of a vector.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witcher;
	///
	/// let files = Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext1(b".jpg")
	///     .into_witching()
	///     .run(|p| { ... });
	/// ```
	pub fn into_witching(self) -> Witching { Witching::from(self.build()) }

	/// # Digest.
	///
	/// This method drains and scans all queued directories, compiling a list
	/// of files as it goes.
	///
	/// If additional directories are discovered during a run, the process is
	/// repeated. Once all directories have been scanned, it's done!
	fn digest(&mut self) {
		while ! self.dirs.is_empty() {
			// Read each directory.
			let (tx, rx) = crossbeam_channel::unbounded();
			self.dirs.par_iter()
				.filter_map(|p| fs::read_dir(p).ok())
				.for_each(|paths| {
					paths.filter_map(|p| p.ok().and_then(|p| fs::canonicalize(p.path()).ok()))
						.for_each(|p| tx.send(p).unwrap());
				});

			// Clear the queue.
			self.dirs.truncate(0);
			drop(tx);

			// Collect the paths found.
			rx.iter().for_each(|p| {
				if self.seen.insert(hash_path_buf(&p)) {
					if p.is_dir() {
						self.dirs.push(p);
					}
					else if (self.cb)(&p) {
						self.files.push(p);
					}
				}
			});
		}
	}
}



#[must_use]
/// # Hash Path.
///
/// This method calculates a unique `u64` hash from a canonical `PathBuf` using
/// the [`AHash`](https://crates.io/crates/ahash) algorithm. It is faster than the default `Hash` implementation
/// because it works against the full byte string, rather than crunching each
/// path component individually.
///
/// Speed aside, the main reason for this is it allows us to track uniqueness
/// as simple, `Copy`able `u64`s instead of having to redundantly store owned
/// copies of all the `PathBuf`s.
fn hash_path_buf(path: &PathBuf) -> u64 {
	let mut hasher = AHasher::default();
	hasher.write(utility::path_as_bytes(path));
	hasher.finish()
}

#[cfg(feature = "simd")]
/// # SIMD Haystack
///
/// This method converts a path into a SIMD-optimized haystack to match
/// against a needle, converting the bytes to lower case as needed.
///
/// The result is zero-padded from the left in cases where the path is
/// shorter than 8, otherwise the last eight bytes of the path are used.
///
/// Note: any value shorter than
fn with_ext_key(ext: &[u8]) -> u8x8 {
	let ext = match ext.len() {
		0 | 1 => u8x8::splat(0),
		2 => u8x8::new(0, 0, 0, 0, 0, 0, ext[0], ext[1]),
		3 => u8x8::new(0, 0, 0, 0, 0, ext[0], ext[1], ext[2]),
		4 => u8x8::new(0, 0, 0, 0, ext[0], ext[1], ext[2], ext[3]),
		5 => u8x8::new(0, 0, 0, ext[0], ext[1], ext[2], ext[3], ext[4]),
		6 => u8x8::new(0, 0, ext[0], ext[1], ext[2], ext[3], ext[4], ext[5]),
		7 => u8x8::new(0, ext[0], ext[1], ext[2], ext[3], ext[4], ext[5], ext[6]),
		len => unsafe { u8x8::from_slice_unaligned_unchecked(&ext[len - 8..]) },
	};

	// Lower-case the result by adding "32" to any bytes in `65..=90`.
	(ext.le(u8x8::splat(90)) & ext.ge(u8x8::splat(65))).select(ext + u8x8::splat(32), ext)
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
		assert_eq!(w1.len(), 3);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));

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

		// One Extension.
		w1 = Witcher::default()
			.with_path(PathBuf::from("tests/"))
			.with_ext1(b".txt")
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 1);

		// Two Extensions.
		w1 = Witcher::default()
			.with_path(PathBuf::from("tests/"))
			.with_ext2(b".txt", b".sh")
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 2);

		// Three Extensions.
		w1 = Witcher::default()
			.with_path(PathBuf::from("tests/"))
			.with_ext3(b".txt", b".sh", b".jpeg")
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 3);
	}
}
