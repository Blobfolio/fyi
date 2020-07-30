/*!
# FYI Witcher: Witcher

`Witcher` stores the results of a recursive file-search with all paths
canonicalized, deduplicated, and validated against a regular expression.

It provides several multi-threaded looping helpers — `process()`, `progress()`,
and `progress_crunch()` — to easily work through files with optional progress
bar output, or you can dereference the object to work directly with its inner
`IndexSet`.
*/



use ahash::{
	AHasher,
	AHashSet
};
use crate::utility::{
	du,
	inflect,
};
use fyi_msg::{
	Msg,
	MsgKind,
};
use fyi_progress::{
	NiceElapsed,
	NiceInt,
	Progress,
};
use rayon::prelude::*;
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
	},
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
	thread,
	time::Duration,
};



// Helper: Make an Arc<Progress> for the loops.
macro_rules! make_progress {
	($name:expr, $len:expr) => (
		Arc::new(Progress::new(
			$len,
			Some(Msg::new($name, 199, "Reticulating splines\u{2026}")),
		))
	);
}

/// Helper: Loop the progress loop inline.
macro_rules! make_progress_loop {
	($paths:ident, $progress:ident, $cb:ident) => {
		// Spawn a thread to steadily tick the progress bar. This is useful
		// when processes might be too long-running.
		let pbar2 = $progress.clone();
		rayon::spawn(move || {
			let sleep = Duration::from_millis(60);

			loop {
				pbar2.clone().tick();
				thread::sleep(sleep);

				// Are we done?
				if ! pbar2.clone().is_running() {
					break;
				}
			}
		});

		// Loop the paths!
		$paths.par_iter().for_each(|x| {
			let file: &str = x.to_str().unwrap_or_default();
			$progress.clone().add_task(file);
			$cb(x);
			$progress.clone().update(1, None::<String>, Some(file));
		});
	};
}



#[derive(Debug, Clone)]
/// Witching Stuff.
pub struct Witcher {
	/// Files found.
	files: Vec<PathBuf>,
	/// Directories found (to be scanned later).
	dirs: Vec<PathBuf>,
	/// Unique path hashes.
	unique: AHashSet<u64>,
}

impl Default for Witcher {
	fn default() -> Self {
		Self {
			files: Vec::with_capacity(64),
			dirs: Vec::with_capacity(16),
			unique: AHashSet::with_capacity(2048),
		}
	}
}

impl From<&str> for Witcher {
	fn from(src: &str) -> Self {
		Self::default()
			.with_path(PathBuf::from(src))
	}
}

impl From<&Path> for Witcher {
	fn from(src: &Path) -> Self {
		Self::default()
			.with_path(src.to_path_buf())
	}
}

impl From<PathBuf> for Witcher {
	fn from(src: PathBuf) -> Self {
		Self::default()
			.with_path(src)
	}
}

impl From<Vec<&str>> for Witcher {
	fn from(src: Vec<&str>) -> Self {
		src.iter()
			.fold(Self::default(), Self::with_path)
	}
}

impl From<Vec<&Path>> for Witcher {
	fn from(src: Vec<&Path>) -> Self {
		src.iter()
			.fold(Self::default(), Self::with_path)
	}
}

impl From<Vec<PathBuf>> for Witcher {
	fn from(src: Vec<PathBuf>) -> Self {
		src.iter()
			.fold(Self::default(), Self::with_path)
	}
}

impl Iterator for Witcher {
	type Item = PathBuf;

	fn next(&mut self) -> Option<Self::Item> {
		match self.files.pop() {
			Some(f) => Some(f),
			None => match self.dirs.pop() {
				Some(d) => {
					self.scan(&d);
					self.next()
				}
				None => None,
			},
		}
	}
}

impl Witcher {
	/// From File List.
	///
	/// Seed the `Witcher` from values stored in a text file.
	///
	/// Note: all paths within the text file must be absolute or they probably
	/// won't be resolvable.
	pub fn read_paths_from_file<P> (path: P) -> Self
	where P: AsRef<Path> {
		if let Ok(file) = File::open(path.as_ref()) {
			Self::from(
				io::BufReader::new(file).lines()
					.filter_map(|x| x.ok()
						.and_then(|x| match x.trim() {
							"" => None,
							y => Some(PathBuf::from(y)),
						})
					)
					.collect::<Vec<PathBuf>>()
			)
		}
		else { Self::default() }
	}

	/// With Path.
	///
	/// Add a path to the current Witcher queue.
	pub fn with_path<P> (mut self, path: P) -> Self
	where P: Into<PathBuf> {
		if let Ok(path) = path.into().canonicalize() {
			self.enqueue_unique(path);
		}

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

	/// Enqueue If.
	///
	/// Insert a path into the right spot if it hasn't already been seen.
	fn enqueue_unique(&mut self, path: PathBuf) {
		if self.unique.insert(hash_path_buf(&path)) {
			if path.is_dir() {
				self.dirs.push(path);
			}
			else {
				self.files.push(path);
			}
		}
	}

	/// Scan Directory.
	///
	/// This runs any time the inbox is empty, scanning the directory, and
	/// pushing results into the appropriate places.
	fn scan(&mut self, path: &PathBuf) {
		if let Ok(paths) = fs::read_dir(path) {
			paths
				.for_each(|p|
					if let Some(p) = p.ok().and_then(|p| p.path().canonicalize().ok()) {
						self.enqueue_unique(p);
					}
				)
		}
	}
}



/// Silent Loop.
///
/// This will execute a callback once for each file in the result set using
/// multiple threads.
///
/// This is just a convenience wrapper to prevent the implementing library
/// having to queue up `rayon` directly.
pub fn process<F> (paths: &[PathBuf], cb: F)
where F: Fn(&PathBuf) + Send + Sync {
	paths.par_iter().for_each(cb);
}

/// Parallel Loop w/ Progress.
///
/// Execute your callback once for each file while displaying a `Progress`
/// progress bar from `fyi_progress`.
///
/// Each thread automatically adds the current file name as a "task" at the
/// start and removes it at the end, so i.e. the progress bar will show which
/// entries are actively being worked on.
///
/// At the end, the progress bar will be cleared and a message will be printed
/// like: "Crunched: Finished 30 files in 1 minute and 3 seconds."
pub fn progress<S, F> (paths: &[PathBuf], name: S, cb: F)
where
	S: Borrow<str>,
	F: Fn(&PathBuf) + Send + Sync
{
	let pbar = make_progress!(name, paths.len() as u64);
	make_progress_loop!(paths, pbar, cb);
	finished_in(pbar.total(), pbar.time().elapsed().as_secs() as u32);
}

/// Parallel Loop w/ Progress.
///
/// This is the same as calling `progress()`, except that it will add up the
/// total disk size of files before and after and report any savings in the
/// final message.
///
/// If your operation doesn't affect file sizes, or doesn't need a size summary
/// at the end, use one of the other loops (or write your own) instead.
pub fn progress_crunch<S, F> (paths: &[PathBuf], name: S, cb: F)
where
	S: Borrow<str>,
	F: Fn(&PathBuf) + Send + Sync
{
	let pbar = make_progress!(name, paths.len() as u64);
	let before: u64 = du(paths);

	make_progress_loop!(paths, pbar, cb);

	crunched_in(
		pbar.total(),
		pbar.time().elapsed().as_secs() as u32,
		before,
		du(paths)
	);
}

/// Crunched In Msg
///
/// This is similar to `finished_in()`, except before/after disk usage is
/// included in the summary. If no bytes are saved, the message will end
/// with "…but nothing doing" instead of "…saving X bytes".
///
/// Like the progress bar, this prints to `Stderr`.
fn crunched_in(total: u64, time: u32, before: u64, after: u64) {
	if 0 == after || before <= after {
		MsgKind::Crunched.as_msg(unsafe {
			std::str::from_utf8_unchecked(&[
				&inflect(total, "file in ", "files in "),
				&*NiceElapsed::from(time),
				b", but nothing doing.\n",
			].concat())
		}).eprint();
	}
	else {
		MsgKind::Crunched.as_msg(format!(
			"{} in {}, saving {} bytes ({:3.*}%).\n",
			unsafe { std::str::from_utf8_unchecked(&inflect(total, "file", "files")) },
			NiceElapsed::from(time).as_str(),
			NiceInt::from(before - after).as_str(),
			2,
			(1.0 - (after as f64 / before as f64)) * 100.0
		)).eprint();
	}
}

/// Finished In Msg
///
/// Print a simple summary like "Crunched: 1 file in 2 seconds.".
///
/// Like the progress bar, this prints to `Stderr`.
fn finished_in(total: u64, time: u32) {
	MsgKind::Crunched.as_msg(unsafe {
		std::str::from_utf8_unchecked(&[
			&inflect(total, "file in ", "files in "),
			&*NiceElapsed::from(time),
			&[46, 10],
		].concat())
	}).eprint();
}

#[must_use]
#[allow(trivial_casts)] // Doesn't work without it.
/// Hash Path.
///
/// We want to make sure we don't go over the same file twice. The fastest
/// solution seems to be hashing the (canonicalized) path, storing that `u64`
/// in a `HashSet` for reference.
///
/// Ultimately we'll probably want to use Arcs or something so the
/// authoritative path can just live directly in the set, with the queues
/// merely sharing a reference.
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
