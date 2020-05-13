/*!
# FYI Witcher: Witcher

`Witcher` stores the results of a recursive file-search with all paths
canonicalized and validated against a regular expression.

*/

use fyi_msg::{
	Flags,
	Msg,
	traits::Printable,
	utility,
};
use fyi_progress::{
	lapsed,
	Progress,
};
use indexmap::set::IndexSet;
use jwalk::WalkDir;
use num_format::{
	Locale,
	ToFormattedString,
};
use rayon::prelude::*;
use regex::Regex;
use std::{
	borrow::Borrow,
	fs::{
		self,
		File,
	},
	io::{
		self,
		BufRead,
	},
	ops::{
		Deref,
		DerefMut,
	},
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
};



macro_rules! make_progress {
	($name:expr, $len:expr) => (
		Arc::new(Progress::new(
			Some(Msg::new($name, 199, "Reticulating splines\u{2026}")),
			$len
		))
	);
}

macro_rules! make_progress_loop {
	($witcher:ident, $progress:ident, $cb:ident) => {
		let handle = Progress::steady_tick(&$progress, None);
		$witcher.par_iter().for_each(|x| {
			let file: &str = x.to_str().unwrap_or("");
			$progress.clone().add_task(file);
			$cb(&Arc::new(x.clone()));
			$progress.clone().update(1, None, Some(file));
		});
		handle.join().unwrap();
	};
}

macro_rules! crunched_in {
	($total:expr, $secs:expr) => (
		Msg::crunched(unsafe {
			std::str::from_utf8_unchecked(&[
				&utility::inflect($total, "file in ", "files in "),
				&lapsed::full($secs),
				&b"."[..],
			].concat())
		})
	);

	($total:expr, $secs:expr, $before:expr, $after:expr) => (
		if 0 == $after || $before <= $after {
			Msg::crunched(unsafe {
				std::str::from_utf8_unchecked(&[
					&utility::inflect($total, "file in ", "files in "),
					&lapsed::full($secs),
					&b", but nothing doing."[..],
				].concat())
			})
		}
		else {
			Msg::crunched(unsafe {
				std::str::from_utf8_unchecked(&[
					&utility::inflect($total, "file in ", "files in "),
					&lapsed::full($secs),
					&b", saving "[..],
					($before - $after).to_formatted_string(&Locale::en).as_bytes(),
					format!(
						" bytes ({:3.*}%).",
						2,
						(1.0 - ($after as f64 / $before as f64)) * 100.0
					).as_bytes()
				].concat())
			})
		}
	);
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

impl DerefMut for Witcher {
	/// Deref.
	fn deref_mut(&mut self) -> &mut IndexSet<PathBuf> {
		&mut self.0
	}
}

impl Witcher {
	/// New.
	///
	/// Recursively search for files and store the result.
	pub fn new<P, R> (paths: &[P], pattern: R) -> Self
	where
		P: AsRef<Path>,
		R: Borrow<str> {
		let pattern: Regex = Regex::new(pattern.borrow()).expect("Invalid Regex.");

		Witcher(paths.iter()
			.filter_map(|p| fs::canonicalize(p.as_ref()).ok())
			.collect::<IndexSet<PathBuf>>()
			.into_par_iter()
			.flat_map(|i| WalkDir::new(i)
				.follow_links(true)
				.skip_hidden(false)
				.into_iter()
				.filter_map(|p| {
					if let Ok(path) = p {
						if path.file_type().is_dir() { None }
						else if let Ok(path) = fs::canonicalize(&path.path()) {
							if let Some(path_str) = path.to_str() {
								if pattern.is_match(path_str) {
									Some(path)
								}
								else { None }
							}
							else { None }
						}
						else { None }
					}
					else { None }
				})
				.collect::<IndexSet<PathBuf>>()
			)
			.collect()
		)
	}

	/// From File.
	///
	/// Read the contents of a text file containing a list of paths to search,
	/// one per line.
	pub fn from_file<P, R> (path: P, pattern: R) -> Self
	where
		P: AsRef<Path>,
		R: Borrow<str> {
			if let Ok(file) = File::open(path.as_ref()) {
				Witcher::new(
					&io::BufReader::new(file).lines()
						.filter_map(|x| match x {
							Ok(x) => {
								let x = x.trim();
								if x.is_empty() { None }
								else { Some(PathBuf::from(x)) }
							},
							_ => None,
						})
						.collect::<Vec<PathBuf>>(),
					pattern,
				)
			}
    		else { Witcher::default() }
	}

	#[must_use]
	/// Get Disk Size.
	pub fn du(&self) -> u64 {
		self.0.par_iter()
			.map(|x| match x.metadata() {
				Ok(meta) => meta.len(),
				_ => 0,
			})
			.sum()
	}



	// -----------------------------------------------------------------
	// Loopers
	// -----------------------------------------------------------------

	/// Parallel Loop.
	pub fn process<F> (&self, cb: F)
	where F: Fn(&PathBuf) + Send + Sync {
		self.0.par_iter().for_each(cb);
	}

	/// Parallel Loop w/ Progress.
	pub fn progress<S, F> (&self, name: S, cb: F)
	where
		S: Borrow<str>,
		F: Fn(&Arc<PathBuf>) + Send + Sync
	{
		let pbar = make_progress!(name, self.0.len() as u64);
		make_progress_loop!(self, pbar, cb);
		crunched_in!(
			pbar.total(),
			pbar.time().elapsed().as_secs() as u32
		).print(0, Flags::TO_STDERR);
	}

	/// Parallel Loop w/ Progress.
	pub fn progress_crunch<S, F> (&self, name: S, cb: F)
	where
		S: Borrow<str>,
		F: Fn(&Arc<PathBuf>) + Send + Sync
	{
		let pbar = make_progress!(name, self.0.len() as u64);
		let before: u64 = self.du();

		make_progress_loop!(self, pbar, cb);

		crunched_in!(
			pbar.total(),
			pbar.time().elapsed().as_secs() as u32,
			before,
			self.du()
		).print(0, Flags::TO_STDERR);
	}
}
