/*!
# FYI Core: Witch
*/

use jwalk::WalkDir;
use rayon::prelude::*;
use regex::Regex;
use std::{
	borrow::Cow,
	collections::HashSet,
	fs::File,
	io::{
		BufReader,
		BufRead,
	},
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
};



#[derive(Debug, Clone)]
/// The Witch!
pub struct Witch(HashSet<PathBuf>);

impl Witch {
	/// New.
	pub fn new<P>(paths: &[P], pattern: Option<String>) -> Self
	where P: AsRef<Path> {
		let paths: Vec<PathBuf> = paths.into_iter()
			.map(|p| p.as_ref().to_path_buf())
			.collect();

		match pattern {
			Some(p) => {
				let pattern = Regex::new(&p)
					.expect("Invalid pattern.");
				Witch::new_filtered(&paths, &pattern)
			},
			None => Witch::new_straight(&paths),
		}
	}

	/// New.
	pub fn new_filtered(paths: &[PathBuf], pattern: &Regex) -> Self {
		Witch(
			paths.into_iter()
				.par_bridge()
				.flat_map(|ref i| WalkDir::new(i)
					.follow_links(true)
					.skip_hidden(false)
					.into_iter()
					.filter_map(|e| {
						if let Ok(path) = e {
							if path.file_type().is_file() {
								if let Ok(path) = path.path().canonicalize() {
									if pattern.is_match(path.to_str().unwrap_or("")) {
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
					.collect::<HashSet<PathBuf>>()
				)
				.collect()
		)
	}

	/// New.
	pub fn new_straight(paths: &[PathBuf]) -> Self {
		Witch(
			paths.into_iter()
				.par_bridge()
				.flat_map(|ref i| WalkDir::new(i)
					.follow_links(true)
					.skip_hidden(false)
					.into_iter()
					.filter_map(|e| {
						if let Ok(path) = e {
							if path.file_type().is_file() {
								if let Ok(path) = path.path().canonicalize() {
									Some(path)
								}
								else { None }
							}
							else { None }
						}
						else { None }
					})
					.collect::<HashSet<PathBuf>>()
				)
				.collect()
		)
	}

	/// From File List
	pub fn from_file<P> (path: P, pattern: Option<String>) -> Self
	where P: AsRef<Path> {
		let path = path.as_ref().to_path_buf();
		if false == path.is_file() {
			return Witch(HashSet::new());
		}

		let input = File::open(&path).expect("Unable to open file.");
		let buffered = BufReader::new(input);

		let paths: Vec<PathBuf> = buffered.lines()
			.filter_map(|x| match x {
				Ok(x) => {
					let x = x.trim();
					match x.is_empty() {
						true => None,
						false => Some(PathBuf::from(x)),
					}
				},
				_ => None,
			})
			.collect();

		match pattern {
			Some(p) => {
				let pattern = Regex::new(&p)
					.expect("Invalid pattern.");
				Witch::new_filtered(&paths, &pattern)
			},
			None => Witch::new_straight(&paths),
		}
	}

	/// Get Disk Size.
	pub fn du(&self) -> u64 {
		use rayon::prelude::*;

		self.0.par_iter()
			.map(|ref x| match x.metadata() {
				Ok(meta) => meta.len(),
				_ => 0,
			})
			.sum()
	}

	/// Files.
	pub fn files(&self) -> HashSet<PathBuf> {
		self.0.clone()
	}

	/// Is Empty.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Get Length.
	pub fn len(&self) -> usize {
		self.0.len()
	}



	// -----------------------------------------------------------------
	// Loopers
	// -----------------------------------------------------------------

	/// Parallel Loop.
	pub fn process<F> (&self, cb: F)
	where F: Fn(&PathBuf) + Send + Sync {
		self.0.par_iter().for_each(cb);
	}

	#[cfg(feature = "progress")]
	/// Parallel Loop w/ Progress.
	pub fn progress<S, F> (&self, name: S, cb: F)
	where
		S: Into<Cow<'static, str>>,
		F: Fn(&Arc<PathBuf>) + Send + Sync
	{
		let bar = self._progress_bar(name.into());
		self._progress_loop(&bar, cb);
		bar.crunched_in(None);
	}

	#[cfg(feature = "progress")]
	/// Parallel Loop w/ Progress and Size Comparison.
	pub fn progress_crunch<S, F> (&self, name: S, cb: F)
	where
		S: Into<Cow<'static, str>>,
		F: Fn(&Arc<PathBuf>) + Send + Sync
	{
		let bar = self._progress_bar(name.into());
		let before: u64 = self.du();

		self._progress_loop(&bar, cb);

		let after: u64 = self.du();
		bar.crunched_in(Some((before, after)));
	}

	#[cfg(feature = "progress")]
	#[inline(always)]
	fn _progress_bar(&self, name: Cow<'_, str>) -> Arc<crate::Progress<'static>> {
		use crate::{
			Msg,
			Prefix,
			Progress,
		};

		Arc::new(Progress::new(
			Msg::new("Reticulating splines…")
				.with_prefix(Prefix::new(name, 199))
				.to_string(),
			self.0.len() as u64,
			0
		))
	}

	#[cfg(feature = "progress")]
	#[inline(always)]
	fn _progress_loop<F> (&self, bar: &Arc<crate::Progress<'static>>, cb: F)
	where F: Fn(&Arc<PathBuf>) + Send + Sync {
		use crate::{
			Progress,
			traits::path::FYIPathFormat,
		};

		// Loop!
		let handle = Progress::steady_tick(&bar, None);
		self.0.par_iter().for_each(|x| {
			let file: String = x.fyi_to_string();
			bar.clone().add_task(&file);
			cb(&Arc::new(x.clone()));
			bar.clone().update(1, None, Some(file));
		});
		handle.join().unwrap();
	}
}
