/*!
# FYI Witcher: Witcher

`Witcher` stores the results of a recursive file-search with all paths
canonicalized, deduplicated, and validated against a regular expression.

It provides several multi-threaded looping helpers — `process()`, `progress()`,
and `progress_crunch()` — to easily work through files with optional progress
bar output, or you can dereference the object to work directly with its inner
`IndexSet`.
*/

use crate::utility::inflect;
use fyi_msg::Msg;
use fyi_progress::{
	NiceElapsed,
	NiceInt,
	Progress,
};
use indexmap::set::IndexSet;
use jwalk::WalkDir;
use rayon::prelude::*;
use regex::bytes::Regex;
use std::{
	borrow::Borrow,
	ffi::OsStr,
	fs::{
		self,
		File,
	},
	io::{
		self,
		BufRead,
	},
	ops::Deref,
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
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

// Helper: Loop the progress loop inline.
macro_rules! make_progress_loop {
	($witcher:ident, $progress:ident, $cb:ident) => {
		let handle = Progress::steady_tick(&$progress, None);
		$witcher.0.par_iter().for_each(|x| {
			let file: &str = x.to_str().unwrap_or_default();
			$progress.clone().add_task(file);
			$cb(x);
			$progress.clone().update(1, None::<String>, Some(file));
		});
		handle.join().unwrap();
	};
}



#[derive(Debug, Default, Clone)]
/// Witcher.
pub struct Witcher(IndexSet<PathBuf>);

impl Deref for Witcher {
	type Target = IndexSet<PathBuf>;

	/// Deref.
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Witcher {
	#[allow(trivial_casts)]
	/// New.
	///
	/// Recursively search for files within the specified paths, filtered
	/// according to a regular expression — applied against the canonical path
	/// — and store the results.
	///
	/// All results are canonicalized and deduped for minimum confusion.
	pub fn new<P, R> (paths: &[P], pattern: R) -> Self
	where
		P: AsRef<Path>,
		R: Borrow<str> {
		let pattern: Regex = Regex::new(pattern.borrow()).expect("Invalid Regex.");

		Self(paths.iter()
			// Canonicalize the search paths.
			.filter_map(|p| fs::canonicalize(p).ok())
			.collect::<IndexSet<PathBuf>>()
			.into_par_iter()
			// Walk each search path.
			.flat_map(|i| WalkDir::new(i)
				.follow_links(true)
				.skip_hidden(false)
				.into_iter()
				.filter_map(|p| p.ok()
					.and_then(|p| if p.file_type().is_dir() { None } else { Some(p) })
					.and_then(|p| fs::canonicalize(p.path()).ok())
					.and_then(|p| if pattern.is_match(unsafe {
						&*(p.as_os_str() as *const OsStr as *const [u8])
					}) {
						Some(p)
					}
						else { None })
				)
				.collect::<IndexSet<PathBuf>>()
			)
			.collect()
		)
	}

	/// New Custom
	///
	/// This method can be used to perform a file search with arbitrary
	/// filtering via a callback method. This method should accept a
	/// `Result<jwalk::DirEntry>` and return a `Some(PathBuf)` if the file is
	/// worth keeping, or `None` to reject the entry.
	///
	/// # Safety
	///
	/// The responsibility for validating and canonicalizing file paths falls
	/// falls to callback, so be sure you handle that, otherwise things like
	/// `du()` might be a bit weird.
	pub unsafe fn custom<P, F> (paths: &[P], cb: F) -> Self
	where
		P: AsRef<Path>,
		F: FnMut(Result<jwalk::DirEntry<((), ())>, jwalk::Error>) -> Option<PathBuf> + Send + Sync + Copy {
		Self(paths.iter()
			// Canonicalize the search paths.
			.filter_map(|p| fs::canonicalize(p).ok())
			.collect::<IndexSet<PathBuf>>()
			.into_par_iter()
			// Walk each search path.
			.flat_map(|i| WalkDir::new(i)
				.follow_links(true)
				.skip_hidden(false)
				.into_iter()
				.filter_map(cb)
				.collect::<IndexSet<PathBuf>>()
			)
			.collect()
		)
	}

	/// Simple.
	///
	/// Recursively search for files within the specified paths. That's it.
	/// Unfiltered! All files in the result set will be canonicalized and
	/// deduped for minimum confusion.
	pub fn simple<P> (paths: &[P]) -> Self
	where P: AsRef<Path> {
		Self(paths.iter()
			// Canonicalize the search paths.
			.filter_map(|p| fs::canonicalize(p).ok())
			.collect::<IndexSet<PathBuf>>()
			.into_par_iter()
			// Walk each search path.
			.flat_map(|i| WalkDir::new(i)
				.follow_links(true)
				.skip_hidden(false)
				.into_iter()
				.filter_map(|p| p.ok()
					.and_then(|p| if p.file_type().is_dir() { None } else { Some(p) })
					.and_then(|p| fs::canonicalize(p.path()).ok())
				)
				.collect::<IndexSet<PathBuf>>()
			)
			.collect()
		)
	}

	/// From File.
	///
	/// This works just like `new()`, except the list of paths to search are
	/// read from the text file at `path`.
	pub fn from_file<P, R> (path: P, pattern: R) -> Self
	where
		P: AsRef<Path>,
		R: Borrow<str> {
			if let Ok(file) = File::open(path.as_ref()) {
				Self::new(
					&io::BufReader::new(file).lines()
						.filter_map(|x| x.ok()
							.and_then(|x| match x.trim() {
								"" => None,
								y => Some(PathBuf::from(y)),
							})
						)
						.collect::<Vec<PathBuf>>(),
					pattern,
				)
			}
    		else { Self::default() }
	}

	#[must_use]
	/// Get Disk Size.
	///
	/// Add up all the file sizes in the result set, using multiple threads
	/// when possible.
	///
	/// Note: this value is not cached; you should be able to call it once, do
	/// some stuff, then call it again and get a different result.
	pub fn du(&self) -> u64 {
		self.0.par_iter()
			.map(|x| match x.metadata() {
				Ok(meta) => meta.len(),
				Err(_) => 0,
			})
			.sum()
	}



	// ------------------------------------------------------------------------
	// Loopers
	// ------------------------------------------------------------------------

	/// Parallel Loop.
	///
	/// Execute your callback once for each file in the result set. Calls will
	/// not necessarily be in a predictable order, but everything will be hit.
	///
	/// As the source code betrays, this literally just forwards your callback
	/// to `par_iter().for_each()`, but it saves you having to import `rayon`
	/// into the local scope.
	pub fn process<F> (&self, cb: F)
	where F: Fn(&PathBuf) + Send + Sync {
		self.0.par_iter().for_each(cb);
	}

	/// Parallel Loop w/ Progress.
	///
	/// Execute your callback once for each file while displaying a `Progress`
	/// progress bar from `fyi_progress`.
	///
	/// Each thread automatically adds the current file name as a "task" at the
	/// start and removes it at the end, so i.e. the progress bar will show
	/// which entries are actively being worked on.
	///
	/// At the end, the progress bar will be cleared and a message will be
	/// printed like: "Crunched: Finished 30 files in 1 minute and 3 seconds."
	pub fn progress<S, F> (&self, name: S, cb: F)
	where
		S: Borrow<str>,
		F: Fn(&PathBuf) + Send + Sync
	{
		let pbar = make_progress!(name, self.0.len() as u64);

		make_progress_loop!(self, pbar, cb);

		Self::finished_in(pbar.total(), pbar.time().elapsed().as_secs() as u32);
	}

	/// Parallel Loop w/ Progress.
	///
	/// This is the same as calling `progress()`, except that it will add up
	/// the total disk size of files before and after and report any savings
	/// in the final message.
	///
	/// If your operation doesn't affect file sizes, or doesn't need a size
	/// summary at the end, use one of the other loops (or write your own)
	/// instead.
	pub fn progress_crunch<S, F> (&self, name: S, cb: F)
	where
		S: Borrow<str>,
		F: Fn(&PathBuf) + Send + Sync
	{
		let pbar = make_progress!(name, self.0.len() as u64);
		let before: u64 = self.du();

		make_progress_loop!(self, pbar, cb);

		Self::crunched_in(
			pbar.total(),
			pbar.time().elapsed().as_secs() as u32,
			before,
			self.du()
		);
	}

	/// Finished In Msg
	///
	/// Print a simple summary like "Crunched: 1 file in 2 seconds.".
	///
	/// Like the progress bar, this prints to `Stderr`.
	fn finished_in(total: u64, time: u32) {
		Msg::crunched(unsafe {
			std::str::from_utf8_unchecked(
				&inflect(total, "file in ", "files in ").iter()
					.chain(&*NiceElapsed::from(time))
					.chain(&[46, 10])
					.copied()
					.collect::<Vec<u8>>()
			)
		}).eprint();
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
			Msg::crunched(unsafe {
				std::str::from_utf8_unchecked(
					&inflect(total, "file in ", "files in ").iter()
						.chain(&*NiceElapsed::from(time))
						.chain(b", but nothing doing.\n")
						.copied()
						.collect::<Vec<u8>>()

				)
			}).eprint();
		}
		else {
			Msg::crunched(unsafe {
				std::str::from_utf8_unchecked(
					&inflect(total, "file in ", "files in ").iter()
						.chain(&*NiceElapsed::from(time))
						.chain(b", saving ")
						.chain(&*NiceInt::from(before - after))
						.chain(format!(
							" bytes ({:3.*}%).\n",
							2,
							(1.0 - (after as f64 / before as f64)) * 100.0
						).as_bytes())
						.copied()
						.collect::<Vec<u8>>()
				)
			}).eprint();
		}
	}
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
		let mut w1 = Witcher::new(&[PathBuf::from("tests/")], ".");
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 2);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
		assert_eq!(w1.du(), 162_u64);

		// Look only for .txt files.
		w1 = Witcher::new(&[PathBuf::from("tests/")], r"(?i)\.txt$");
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 1);
		assert!(w1.contains(&abs_p1));
		assert!(! w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
		assert_eq!(w1.du(), 26_u64);

		// Look for something that doesn't exist.
		w1 = Witcher::new(&[PathBuf::from("tests/")], r"(?i)\.exe$");
		assert!(w1.is_empty());
		assert_eq!(w1.len(), 0);
		assert!(! w1.contains(&abs_p1));
		assert!(! w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
		assert_eq!(w1.du(), 0_u64);
	}

	pub fn t_custom_cb (p: Result<jwalk::DirEntry<((), ())>, jwalk::Error>) -> Option<PathBuf> {
		// Skip errors, duh.
		if let Ok(path) = p {
			// We don't want directories.
			if path.file_type().is_dir() { None }
			// We need to canonicalize again because symlinks might
			// not actually be living with the parent directory.
			else if let Ok(path) = fs::canonicalize(&path.path()) {
				// Do a simple `ends_with()` check to filter just .sh files.
				unsafe {
					let p_str: *const OsStr = path.as_os_str();
					if (&*(p_str as *const [u8])).ends_with(&[46, 115, 104]) { Some(path) }
					else { None }
				}
			}
			else { None }
		}
		else { None }
	}

	#[test]
	fn t_custom() {
		let mut abs_dir = fs::canonicalize("tests/assets/").unwrap();
		abs_dir.push("_.txt");
		let abs_p1 = abs_dir.with_file_name("file.txt");
		let abs_p2 = abs_dir.with_file_name("is-executable.sh");
		let abs_perr = abs_dir.with_file_name("foo.bar");

		// Search for .sh files.
		unsafe {
			let w1 = Witcher::custom(&[PathBuf::from("tests/")], t_custom_cb);
			assert!(! w1.is_empty(), "{:?}", &b".sh"[..]);
			assert_eq!(w1.len(), 1);
			assert!(! w1.contains(&abs_p1));
			assert!(w1.contains(&abs_p2));
			assert!(! w1.contains(&abs_perr));
			assert_eq!(w1.du(), 136_u64);
		}
	}

	#[test]
	fn t_simple() {
		let mut abs_dir = fs::canonicalize("tests/assets/").unwrap();
		abs_dir.push("_.txt");
		let abs_p1 = abs_dir.with_file_name("file.txt");
		let abs_p2 = abs_dir.with_file_name("is-executable.sh");
		let abs_perr = abs_dir.with_file_name("foo.bar");

		// Search for all files.
		let w1 = Witcher::simple(&[PathBuf::from("tests/")]);
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 2);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
		assert_eq!(w1.du(), 162_u64);
	}
}
