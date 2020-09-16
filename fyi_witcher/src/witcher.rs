/*!
# FYI Witcher: Witcher
*/

use ahash::AHashSet;
use crate::{
	utility,
	Witching,
};
use fyi_msg::utility::hash64;
#[cfg(feature = "simd")] use packed_simd::u8x8;
use rayon::prelude::*;
use std::{
	borrow::Borrow,
	fs,
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
			let bytes: &[u8] = utility::path_as_bytes(p);
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
	/// This method — and [`Witcher::with_ext2`], [`Witcher::with_ext3`] — can be faster for
	/// matching simple file extensions than [`Witcher::with_regex`],
	/// particularly if regular expressions are not used anywhere else.
	///
	/// Comparisons are done case-insensitively from the leading periods in the
	/// needle — `ext1` — and haystack — the path. Extensions must be at least
	/// 2 bytes (e.g. ".h") and no longer than 8 bytes (e.g. ".longone").
	///
	/// Note: The extension must include a leading period or nothing will match.
	///
	/// ## Panics
	///
	/// This method will panic if the extension is greater than 8 bytes.
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
			let ext1 = with_ext_key(ext1);
			Box::new(move |p: &PathBuf| ext1.eq(with_ext_path_key(utility::path_as_bytes(p))).all())
		};

		self
	}

	#[cfg(feature = "simd")]
	#[must_use]
	/// # With Extensions (2) Filter.
	///
	/// Note: The extensions should include their leading periods.
	///
	/// ## Panics
	///
	/// This method will panic if any extension is greater than 8 bytes.
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
			let ext1 = with_ext_key(ext1);
			let ext2 = with_ext_key(ext2);

			Box::new(move |p: &PathBuf| {
				let src = with_ext_path_key(utility::path_as_bytes(p));
				ext1.eq(src).all() || ext2.eq(src).all()
			})
		};

		self
	}

	#[cfg(feature = "simd")]
	#[must_use]
	/// # With Extensions (3) Filter.
	///
	/// Note: The extensions should include their leading periods.
	///
	/// ## Panics
	///
	/// This method will panic if any extension is greater than 8 bytes.
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
			let ext1 = with_ext_key(ext1);
			let ext2 = with_ext_key(ext2);
			let ext3 = with_ext_key(ext3);

			Box::new(move |p: &PathBuf| {
				let src = with_ext_path_key(utility::path_as_bytes(p));
				ext1.eq(src).all() || ext2.eq(src).all() || ext3.eq(src).all()
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
			if self.seen.insert(hash64(utility::path_as_bytes(&path))) {
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
				if self.seen.insert(hash64(utility::path_as_bytes(&p))) {
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



#[cfg(feature = "simd")]
/// # SIMD Haystack
///
/// This method converts a path extension (e.g. `b".jpg"`) into an 8-lane SIMD
/// vector for easy comparison.
///
/// Note the leading period. This should be present in passed values.
///
/// Extensions requiring fewer than 8 lanes are zero-padded on the left.
fn with_ext_key(ext: &[u8]) -> u8x8 {
	match ext.len() {
		2 => u8x8::new(0, 0, 0, 0, 0, 0, b'.', ext[1].to_ascii_lowercase()),
		3 => u8x8::new(0, 0, 0, 0, 0, b'.', ext[1].to_ascii_lowercase(), ext[2].to_ascii_lowercase()),
		4 => u8x8::new(0, 0, 0, 0, b'.', ext[1].to_ascii_lowercase(), ext[2].to_ascii_lowercase(), ext[3].to_ascii_lowercase()),
		5 => u8x8::new(0, 0, 0, b'.', ext[1].to_ascii_lowercase(), ext[2].to_ascii_lowercase(), ext[3].to_ascii_lowercase(), ext[4].to_ascii_lowercase()),
		6 => u8x8::new(0, 0, b'.', ext[1].to_ascii_lowercase(), ext[2].to_ascii_lowercase(), ext[3].to_ascii_lowercase(), ext[4].to_ascii_lowercase(), ext[5].to_ascii_lowercase()),
		7 => u8x8::new(0, b'.', ext[1].to_ascii_lowercase(), ext[2].to_ascii_lowercase(), ext[3].to_ascii_lowercase(), ext[4].to_ascii_lowercase(), ext[5].to_ascii_lowercase(), ext[6].to_ascii_lowercase()),
		8 => u8x8::new(b'.', ext[1].to_ascii_lowercase(), ext[2].to_ascii_lowercase(), ext[3].to_ascii_lowercase(), ext[4].to_ascii_lowercase(), ext[5].to_ascii_lowercase(), ext[6].to_ascii_lowercase(), ext[7].to_ascii_lowercase()),
		_ => u8x8::splat(0),
	}
}

#[cfg(feature = "simd")]
/// # SIMD Path Extension.
///
/// This method plucks the extension piece off a [`PathBuf`] and converts it
/// into a SIMD vector for comparison.
///
/// Because we're only comparing 8-lane values, if no period is found after 8
/// checks, a zeroed vector is returned instead.
fn with_ext_path_key(ext: &[u8]) -> u8x8 {
	let len: usize = ext.len();
	if len >= 8 {
		use packed_simd::m8x8;

		let mut raw = unsafe { u8x8::from_slice_unaligned_unchecked(&ext[len-8..]) };

		// Calculate the position of the extension portion — including the
		// period — from the bitmask's leading zeros. Inclusivity means taking
		// one value more than the result, unless that result is eight, which
		// means no match.
		raw = match raw.eq(u8x8::splat(b'.')).bitmask().leading_zeros() {
			1 => m8x8::new(false, false, false, false, false, false, true, true).select(raw, u8x8::splat(0)),
			2 => m8x8::new(false, false, false, false, false, true, true, true).select(raw, u8x8::splat(0)),
			3 => m8x8::new(false, false, false, false, true, true, true, true).select(raw, u8x8::splat(0)),
			4 => m8x8::new(false, false, false, true, true, true, true, true).select(raw, u8x8::splat(0)),
			5 => m8x8::new(false, false, true, true, true, true, true, true).select(raw, u8x8::splat(0)),
			6 => m8x8::new(false, true, true, true, true, true, true, true).select(raw, u8x8::splat(0)),
			7 => raw,
			_ => return u8x8::splat(0),
		};

		// This terrible bit of wizardry adds `32` to anything between `65..=90`
		// to force lower case. As we already have a SIMD vector, this is
		// cheaper than passing a sub-slice through `with_ext_key()`.
		return (raw.ge(u8x8::splat(b'A')) & raw.le(u8x8::splat(b'Z'))).select(raw | u8x8::splat(32), raw);
	}
	else if len > 1 {
		for i in 1..=8.min(len) {
			if ext[len - i] == b'.' {
				return with_ext_key(&ext[len - i..]);
			}
		}
	}

	u8x8::splat(0)
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
