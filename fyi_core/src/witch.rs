/*!
# FYI Core: Witch
*/

use rayon::prelude::*;
use regex::Regex;
use std::{
	borrow::Cow,
	collections::HashSet,
	ffi::OsStr,
	fs::{
		self,
		DirEntry,
		File,
	},
	io::{
		BufReader,
		BufRead,
		Error,
		ErrorKind,
	},
	path::PathBuf,
};



#[derive(Debug, Defaults, Clone)]
/// A witch.
pub struct Witch {
	#[def = "HashSet::with_capacity(1024)"]
	files: HashSet<PathBuf>,

	#[def = "HashSet::with_capacity(1024)"]
	dirs: HashSet<PathBuf>,
}

impl Witch {
	/// New.
	pub fn new(paths: &[PathBuf], pattern: Option<String>) -> Self {
		let mut me: Witch = Witch::default();

		match pattern {
			Some(p) => {
				let pattern = Regex::new(p.as_str()).expect("Invalid pattern.");
				for path in paths {
					me._walk_route_filtered(path, &pattern);
				}
			},
			_ => {
				for path in paths {
					me._walk_route(path);
				}
			}
		}

		me.dirs.clear();
		me.dirs.shrink_to_fit();

		me
	}

	/// From File List
	pub fn from_file(path: &PathBuf, pattern: Option<String>) -> Self {
		if false == path.is_file() {
			return Witch::default();
		}

		let input = File::open(&path).expect("Unable to open file.");
		let buffered = BufReader::new(input);

		let out: Vec<PathBuf> = buffered.lines()
			.filter_map(|x| match x.ok() {
				Some(x) => {
					let x = x.trim();
					match x.is_empty() {
						true => None,
						false => Some(PathBuf::from(x)),
					}
				},
				_ => None,
			})
			.collect();

		Witch::new(&out, pattern)
	}

	/// Get Disk Size.
	pub fn du(&self) -> u64 {
		use rayon::prelude::*;

		self.files().par_iter()
			.map(|ref x| match x.metadata() {
				Ok(meta) => meta.len(),
				_ => 0,
			})
			.sum()
	}

	/// Get Files.
	pub fn files(&self) -> Cow<HashSet<PathBuf>> {
		Cow::Borrowed(&self.files)
	}

	/// Is Empty.
	pub fn is_empty(&self) -> bool {
		self.files.is_empty()
	}

	/// Get Length.
	pub fn len(&self) -> usize {
		self.files.len()
	}



	// -----------------------------------------------------------------
	// Loopers
	// -----------------------------------------------------------------

	/// Parallel Loop.
	pub fn process<F> (&self, cb: F)
	where F: Fn(&PathBuf) + Send + Sync {
		self.files.par_iter().for_each(cb);
	}

	#[cfg(feature = "progress")]
	/// Common Progress Tasks.
	fn _progress<S, F> (&self, name: S, cb: F)
	where
		S: Into<Cow<'static, str>>,
		F: Fn(&PathBuf) + Send + Sync {
		use crate::{
			arc::progress as parc,
			Msg,
			Prefix,
			Progress,
			PROGRESS_CLEAR_ON_FINISH,
		};

		let bar = Progress::new(
			Msg::new("Reticulating splines…")
				.with_prefix(Prefix::Custom(name.into().as_ref(), 199))
				.to_string(),
			self.files.len() as u64,
			PROGRESS_CLEAR_ON_FINISH
		);
		let looper = parc::looper(&bar, 60);
		self.files().par_iter().for_each(|ref x| {
			parc::add_working(&bar, &x);
			cb(x);
			parc::update(&bar, 1, None, Some(x.to_path_buf()));
		});
		parc::finish(&bar);
		looper.join().unwrap();
	}

	#[cfg(feature = "progress")]
	/// Parallel Loop w/ Progress.
	pub fn progress<S, F> (&self, name: S, cb: F)
	where
		S: Into<Cow<'static, str>>,
		F: Fn(&PathBuf) + Send + Sync {
		use crate::{
			arc::progress as parc,
			Msg,
			Prefix,
			Progress,
			PROGRESS_CLEAR_ON_FINISH,
		};
		use std::time::Instant;

		let time = Instant::now();

		{
			let bar = Progress::new(
				Msg::new("Reticulating splines…")
					.with_prefix(Prefix::Custom(name.into().as_ref(), 199))
					.to_string(),
				self.files.len() as u64,
				PROGRESS_CLEAR_ON_FINISH
			);
			let looper = parc::looper(&bar, 60);
			self.files().par_iter().for_each(|ref x| {
				parc::add_working(&bar, &x);
				cb(x);
				parc::update(&bar, 1, None, Some(x.to_path_buf()));
			});
			parc::finish(&bar);
			looper.join().unwrap();
		}

		Msg::msg_crunched_in(self.files.len() as u64, time, None)
			.print();
	}

	#[cfg(feature = "progress")]
	/// Parallel Loop w/ Progress and Size Comparison.
	pub fn progress_crunch<S, F> (&self, name: S, cb: F)
	where
		S: Into<Cow<'static, str>>,
		F: Fn(&PathBuf) + Send + Sync {
		use crate::Msg;
		use std::time::Instant;

		let time = Instant::now();
		let before: u64 = self.du();

		self._progress(name, cb);

		let after: u64 = self.du();
		Msg::msg_crunched_in(self.files.len() as u64, time, Some((before, after)))
			.print();
	}



	// -----------------------------------------------------------------
	// Walk All Files
	// -----------------------------------------------------------------

	/// Route path.
	fn _walk_route(&mut self, path: &PathBuf) {
		if let Ok(meta) = path.symlink_metadata() {
			if meta.is_file() {
				if let Ok(p) = path.canonicalize() {
					self.files.insert(p);
				}
			}
			else if meta.is_dir() {
				if let Ok(p) = path.canonicalize() {
					let _ = self._walk_dir(&p).is_ok();
				}
			}
			// Recurse for symlinks.
			else if let Ok(path) = fs::read_link(&path) {
				if path.is_file() {
					if let Ok(p) = path.canonicalize() {
						self.files.insert(p);
					}
				}
				else if path.is_dir() {
					if let Ok(p) = path.canonicalize() {
						let _ = self._walk_dir(&p).is_ok();
					}
				}
			}
		}
	}

	/// Walk Dir.
	fn _walk_dir(&mut self, path: &PathBuf) -> Result<(), Error> {
		if self.dirs.insert(path.to_path_buf()) {
			for entry in fs::read_dir(&path)? {
				if let Ok(e) = entry {
					if self._walk_dir_entry(Cow::Borrowed(&&e)).is_err() {
						self._walk_route(&e.path());
					}
				}
			}
		}

		Ok(())
	}

	/// Walk Dir Entry.
	fn _walk_dir_entry(&mut self, path: Cow<&DirEntry>) -> Result<(), Error> {
		if let Ok(ft) = path.file_type() {
			if ft.is_file() {
				self.files.insert(path.path().canonicalize()?);
				return Ok(());
			}
			else if ft.is_dir() {
				let _ = self._walk_dir(&path.path()).is_ok();
				return Ok(());
			}
		}

		Err(Error::new(ErrorKind::NotFound, "Unable to process path."))
	}



	// -----------------------------------------------------------------
	// Walk All Files
	// -----------------------------------------------------------------

	/// Route path.
	fn _walk_route_filtered(&mut self, path: &PathBuf, pattern: &Regex) {
		if let Ok(meta) = path.symlink_metadata() {
			if meta.is_file() {
				if let Ok(p) = path.canonicalize() {
					self._walk_file_filtered(&p, &pattern);
				}
			}
			else if meta.is_dir() {
				if let Ok(p) = path.canonicalize() {
					let _ = self._walk_dir_filtered(&p, &pattern).is_ok();
				}
			}
			// Recurse for symlinks.
			else if let Ok(path) = fs::read_link(&path) {
				if path.is_file() {
					if let Ok(p) = path.canonicalize() {
						self._walk_file_filtered(&p, &pattern);
					}
				}
				else if path.is_dir() {
					if let Ok(p) = path.canonicalize() {
						let _ = self._walk_dir_filtered(&p, &pattern).is_ok();
					}
				}
			}
		}
	}

	/// Walk Dir.
	fn _walk_dir_filtered(&mut self, path: &PathBuf, pattern: &Regex) -> Result<(), Error> {
		if self.dirs.insert(path.to_path_buf()) {
			for entry in fs::read_dir(&path)? {
				if let Ok(e) = entry {
					if self._walk_dir_entry_filtered(Cow::Borrowed(&&e), &pattern).is_err() {
						self._walk_route_filtered(&e.path(), &pattern);
					}
				}
			}
		}

		Ok(())
	}

	/// Walk Dir Entry.
	fn _walk_dir_entry_filtered(&mut self, path: Cow<&DirEntry>, pattern: &Regex) -> Result<(), Error> {
		if let Ok(ft) = path.file_type() {
			if ft.is_file() {
				self._walk_file_filtered(&path.path(), &pattern);
				return Ok(());
			}
			else if ft.is_dir() {
				let _ = self._walk_dir_filtered(&path.path(), &pattern).is_ok();
				return Ok(());
			}
		}

		Err(Error::new(ErrorKind::NotFound, "Unable to process path."))
	}

	/// Push file.
	fn _walk_file_filtered(&mut self, path: &PathBuf, pattern: &Regex) {
		let name = path.file_name()
			.unwrap_or(OsStr::new(""))
			.to_str()
			.unwrap_or("");

		if pattern.is_match(&name) {
			self.files.insert(path.to_path_buf());
		}
	}
}
