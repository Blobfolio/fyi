/*!
# FYI Witcher: Witcher

`Witcher` is a very simple recursive file searching library that returns all
file paths within the tree(s), nice and canonicalized. Duplicates are weeded
out, symlinks are resolved and followed, hidden files are picked up like any
other file.

Short and sweet.

While `Witcher` is light on options — there aren't any! — it can be seeded with
multiple starting paths using the `Witcher::with_path()` builder pattern. This,
combined with the general stripped-to-basics codebase, make this a more
performant option than using crates such as `jwalk` or `walkdir` in many cases.

## Examples

`Witcher` implements `Iterator`, so you can simply initiate it and loop/map/
filter your way to a better tomorrow:

```no_run
use fyi_witcher::Witcher;
use std::path::PathBuf;

let paths: Vec<PathBuf> = Witcher.from(PathBuf::from(.))
    .filter(|x| x.as_str().unwrap_or_default().ends_with('.jpg'))
    .collect();
```

Two collection convenience methods exist to short-circuit the `Iterator`
process if you don't need it:

```no_run
// Just make it a Vec of PathBufs.
let paths: Vec<PathBuf> = Witcher.from(PathBuf::from(.)).to_vec();

// Filter (file) paths by regular expression, returning the matches as a Vec.
let paths: Vec<PathBuf> = Witcher.from(PathBuf::from(.))
    .filter_and_collect("(?i).+\.jpg$");
*/



use ahash::{
	AHasher,
	AHashSet
};
use crate::utility::du;
use fyi_msg::{
	Msg,
	MsgKind,
};
use fyi_progress::{
	NiceElapsed,
	NiceInt,
	Progress,
	utility::inflect,
};
use std::{
	borrow::Borrow,
	ffi::OsStr,
	fs::{
		self,
		File,
	},
	hash::Hasher,
	io::{
		self,
		BufRead,
		BufReader,
		Lines,
	},
	path::{
		Path,
		PathBuf,
	},
};



/// Helper: Generate "impl From" for Iterator<AsRef<Path>> types.
macro_rules! from_many {
	($type:ty) => {
		impl From<$type> for Witcher {
			fn from(src: $type) -> Self {
				src.iter()
					.fold(Self::default(), Self::with_path)
			}
		}
	};
}



#[derive(Debug, Clone)]
/// Witcher.
///
/// This is the it, folks! See the library documentation for more information.
pub struct Witcher {
	/// Paths waiting return or traversal.
	stack: Vec<PathBuf>,
	/// Unique path hashes found.
	hash: AHashSet<u64>,
}

impl Default for Witcher {
	fn default() -> Self {
		Self {
			stack: Vec::with_capacity(64),
			hash: AHashSet::with_capacity(2048),
		}
	}
}

impl From<&str> for Witcher {
	fn from(src: &str) -> Self {
		Self::default().with_path(src)
	}
}

impl From<&Path> for Witcher {
	fn from(src: &Path) -> Self {
		Self::default().with_path(src)
	}
}

impl From<PathBuf> for Witcher {
	fn from(src: PathBuf) -> Self {
		Self::default().with_path(src)
	}
}

from_many!(&[&str]);
from_many!(&[String]);
from_many!(&[&Path]);
from_many!(&[PathBuf]);

from_many!(Vec<&str>);
from_many!(Vec<String>);
from_many!(Vec<&Path>);
from_many!(Vec<PathBuf>);

impl From<Lines<BufReader<File>>> for Witcher {
	fn from(src: Lines<BufReader<File>>) -> Self {
		src.fold(
			Self::default(),
			|acc, line| match line.unwrap_or_default().trim() {
				"" => acc,
				l => acc.with_path(l),
			}
		)
	}
}

impl Iterator for Witcher {
	type Item = PathBuf;

	fn next(&mut self) -> Option<Self::Item> {
		match self.stack.pop() {
			Some(path) =>
				// Recurse directories.
				if path.is_dir() {
					self.push_dir(path);
					self.next()
				}
				// Return files.
				else { Some(path) },
			None => None,
		}
	}
}

impl Witcher {
	/// From File List.
	///
	/// Seed the `Witcher` from values stored in a text file, one path per
	/// line.
	///
	/// Note: relative paths parsed in this manner probably won't resolve
	/// correctly; it is recommended only absolute paths be used.
	pub fn read_paths_from_file<P> (path: P) -> Self
	where P: AsRef<Path> {
		if let Ok(file) = File::open(path.as_ref()) {
			Self::from(io::BufReader::new(file).lines())
		}
		else { Self::default() }
	}

	/// With Path.
	///
	/// Add a path to the current Witcher queue.
	pub fn with_path<P> (mut self, path: P) -> Self
	where P: AsRef<Path> {
		if let Ok(path) = fs::canonicalize(path) { self.push(path); }
		self
	}

	#[allow(trivial_casts)] // Doesn't work without it.
	/// Filter and Collect
	///
	/// Find everything, filter according to the provided regex pattern, and
	/// return the results as a straight Vec.
	pub fn filter_and_collect<R> (&mut self, pattern: R) -> Vec<PathBuf>
	where R: Borrow<str> {
		use regex::bytes::Regex;

		let pattern: Regex = Regex::new(pattern.borrow()).expect("Invalid Regex.");
		self.filter(|p| pattern.is_match(unsafe {
			&*(p.as_os_str() as *const OsStr as *const [u8])
		}))
			.collect::<Vec<PathBuf>>()
	}

	#[allow(clippy::wrong_self_convention)] // I mean it is what it is.
	/// To Vec
	///
	/// Find everything and return the results as a straight Vec.
	pub fn to_vec(&mut self) -> Vec<PathBuf> {
		self.collect()
	}

	/// Read Dir.
	///
	/// Read the immediate paths under a directory and push any unique results
	/// back to the stack.
	fn push_dir(&mut self, path: PathBuf) {
		if let Ok(paths) = fs::read_dir(path) {
			paths.filter_map(|p| p.ok().and_then(|p| fs::canonicalize(p.path()).ok()))
				.for_each(|p| self.push(p));
		}
	}

	/// Push if Previously Unseen.
	///
	/// Push a (canonical) path to the stack unless it has already been seen.
	fn push(&mut self, path: PathBuf) {
		if self.hash.insert(hash_path_buf(&path)) {
			self.stack.push(path);
		}
	}
}

#[allow(clippy::pedantic)] // It gets eaten!
/// Parallel Loop w/ Progress.
///
/// This is an add-on for `Progress` that tallies the before and after sizes,
/// useful for applications that modify the files being iterated.
///
/// See the documentation for `fyi_progress::Progress` for more details.
pub fn progress_crunch<F> (paths: Progress<PathBuf>, cb: F)
where F: Fn(&PathBuf) + Send + Sync + Copy {
	// Abort if missing paths.
	if paths.is_empty() {
		MsgKind::Warning.into_msg("No matching files were found.\n")
			.eprint();

		return;
	}

	// Add up the space used before the run.
	let before: u64 = du(&*paths);

	paths.run(cb);

	crunched_in(
		paths.total().into(),
		paths.elapsed(),
		before,
		du(&*paths)
	);
}

/// Crunched In Msg
///
/// This is an alternative progress summary that includes the number of bytes
/// saved. It is called after `progress_crunch()`.
fn crunched_in(total: u64, time: u32, before: u64, after: u64) {
	// No savings or weird values.
	if 0 == after || before <= after {
		Msg::from([
			&inflect(total, "file in ", "files in "),
			&*NiceElapsed::from(time),
			b", but nothing doing.\n",
		].concat())
			.with_prefix(MsgKind::Crunched)
			.eprint();
	}
	// Something happened!
	else {
		MsgKind::Crunched.into_msg(format!(
			"{} in {}, saving {} bytes ({:3.*}%).\n",
			unsafe { std::str::from_utf8_unchecked(&inflect(total, "file", "files")) },
			NiceElapsed::from(time).as_str(),
			NiceInt::from(before - after).as_str(),
			2,
			(1.0 - (after as f64 / before as f64)) * 100.0
		)).eprint();
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
		let mut w1 = Witcher::from(PathBuf::from("tests/")).to_vec();
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 2);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
		assert_eq!(du(&w1), 162_u64);

		// Look only for .txt files.
		w1 = Witcher::from(vec![PathBuf::from("tests/")]).filter_and_collect(r"(?i)\.txt$");
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 1);
		assert!(w1.contains(&abs_p1));
		assert!(! w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
		assert_eq!(du(&w1), 26_u64);

		// Look for something that doesn't exist.
		w1 = Witcher::from(PathBuf::from("tests/")).filter_and_collect(r"(?i)\.exe$");
		assert!(w1.is_empty());
		assert_eq!(w1.len(), 0);
		assert!(! w1.contains(&abs_p1));
		assert!(! w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
		assert_eq!(du(&w1), 0_u64);
	}
}
