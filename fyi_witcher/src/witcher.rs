/*!
# FYI Witcher: Witcher

This is a very simple recursive file finder. Directories are read in parallel.
Files are canonicalized and deduped. Symlinks are followed. Hidden files and
directories are read like any other.

The struct uses a builder pattern.

## Filtering

Results can be filtered prior to being yielded with the use of either
`with_filter()` — specifying a custom callback method — or `with_regex()` — to
match against a pattern.

It is important to define the filter *before* adding any paths, because if
those paths are files, they'll need to be filtered. Right? Right.

Filter callbacks should accept an `&PathBuf` and return `true` to keep it,
`false` to discard it.

## Examples

```no_run
use fyi_witcher::Witcher;

// Return all files under "/usr/share/man".
let res: Vec<PathBuf> = Witcher::default()
    .with_path("/usr/share/man")
    .build();

// Return only Gzipped files.
let res: Vec<PathBuf> = Witcher::default()
    .with_regex(r"(?i).+\.gz$")
    .with_path("/usr/share/man")
    .build();

// If you're just matching one pattern, it can be faster to not use Regex:
let res: Vec<PathBuf> = Witcher::default()
    .with_filter(|p: &PathBuf| {
        let bytes: &[u8] = unsafe { &*(p.as_os_str() as *const OsStr as *const [u8]) };
        bytes.len() > 3 && bytes[bytes.len()-3..].eq_ignore_ascii_case(b".gz")
    })
    .with_path("/usr/share/man")
    .build();
```
*/

use ahash::{
	AHasher,
	AHashSet
};
use crate::Witching;
use rayon::prelude::*;
use std::{
	borrow::Borrow,
	ffi::OsStr,
	fs,
	hash::Hasher,
	path::{
		Path,
		PathBuf,
	},
};



#[allow(missing_debug_implementations)]
/// Witcher.
///
/// This is the main file finder struct. See the module reference for more
/// details.
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
	/// With Callback.
	///
	/// Define a custom filter callback to determine whether or not a given
	/// file path should be yielded.
	pub fn with_filter<F>(mut self, cb: F) -> Self
	where F: Fn(&PathBuf) -> bool + 'static {
		self.cb = Box::new(cb);
		self
	}

	#[allow(trivial_casts)] // Triviality is required!
	/// With a Regex Callback.
	///
	/// This is a convenience method for filtering files by regular expression.
	pub fn with_regex<R>(mut self, reg: R) -> Self
	where R: Borrow<str> {
		use regex::bytes::Regex;
		let pattern: Regex = Regex::new(reg.borrow()).expect("Invalid Regex.");
		self.cb = Box::new(move|p: &PathBuf| pattern.is_match(
			unsafe { &*(p.as_os_str() as *const OsStr as *const [u8]) }
		));
		self
	}

	/// With Paths.
	///
	/// Append files and/or directories to the finder. File paths will be
	/// checked against the filter callback (if any) and added straight to the
	/// results if they pass. Directories will be queued for later scanning.
	pub fn with_paths<P>(self, paths: &[P]) -> Self
	where P: AsRef<Path> {
		paths.iter().fold(self, Self::with_path)
	}

	/// With Paths From File.
	///
	/// This method reads paths from a text file — one path per line — and adds
	/// them to the finder.
	///
	/// Paths added in this method should be absolute as relativity might not
	/// work correctly coming from a text file.
	pub fn with_paths_from_file<P>(self, path: P) -> Self
	where P: AsRef<Path> {
		use std::{
			fs::File,
			io::{
				BufRead,
				BufReader,
			},
		};

		if let Ok(file) = File::open(path.as_ref()) {
			BufReader::new(file).lines()
				.filter_map(|line| match line.unwrap_or_default().trim() {
					"" => None,
					x => Some(PathBuf::from(x)),
				})
				.fold(self, Self::with_path)
		}
		else { self }
	}

	/// With Path.
	///
	/// Add a path to the finder. If the path is a file, it will be checked
	/// against the filter callback (if any) before being added to the results.
	/// If it is a directory, it will be queued for later scanning.
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

	/// With Path(s) Helper
	///
	/// This is a convenience method that triggers either `with_paths()` or
	/// `with_paths_from_file()`, depending on whether or not `list` is set.
	pub fn with<P>(self, paths: &[P], list: bool) -> Self
	where P: AsRef<Path> {
		if list && ! paths.is_empty() {
			self.with_paths_from_file(&paths[0])
		}
		else { self.with_paths(paths) }
	}

	#[must_use]
	/// Build!
	///
	/// Once everything is set up, call this method to consume the queue and
	/// collect the files into a `Vec<PathBuf>`.
	pub fn build(mut self) -> Vec<PathBuf> {
		self.digest();
		self.files
	}

	#[must_use]
	/// Build (into Progress)!
	///
	/// This is identical to `build()`, except a ready-to-go `Progress` struct
	/// is returned instead.
	pub fn into_witching(self) -> Witching { Witching::from(self.build()) }

	/// Digest.
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
				.for_each(|p| {
					if let Ok(paths) = fs::read_dir(p) {
						paths.filter_map(|p| p.ok().and_then(|p| fs::canonicalize(p.path()).ok()))
							.for_each(|p| tx.send(p).unwrap());
					}
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
#[allow(trivial_casts)] // Doesn't work without it.
/// Hash Path.
///
/// This method calculates a unique `u64` hash from a canonical `PathBuf` using
/// the `AHash` algorithm. It is faster than the default `Hash` implementation
/// because it works against the full byte string, rather than crunching each
/// path component individually.
///
/// Speed aside, the main reason for this is it allows us to track uniqueness
/// as simple, `Copy`able `u64`s instead of full-blown `PathBuf`s.
fn hash_path_buf(path: &PathBuf) -> u64 {
	let mut hasher = AHasher::default();
	hasher.write(unsafe { &*(path.as_os_str() as *const OsStr as *const [u8]) });
	hasher.finish()
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
		let mut w1 = Witcher::default()
			.with_path(PathBuf::from("tests/"))
			.build();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 2);
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
	}
}
