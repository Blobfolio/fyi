/*!
# FYI Core: Witch
*/

use jwalk::WalkDir;
use rayon::prelude::*;
use regex::Regex;
use std::{
	borrow::Cow,
	collections::HashSet,
	fs::{
		self,
		File,
	},
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
								if let Ok(path) = fs::canonicalize(&path.path()) {
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
								if let Ok(path) = fs::canonicalize(&path.path()) {
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
		let path: PathBuf = path.as_ref().to_path_buf();
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
		use crate::Progress;

		// Loop!
		let handle = Progress::steady_tick(&bar, None);
		self.0.par_iter().for_each(|x| {
			let file: String = x.to_str()
				.unwrap_or("")
				.to_string();
			bar.clone().add_task(&file);
			cb(&Arc::new(x.clone()));
			bar.clone().update(1, None, Some(file));
		});
		handle.join().unwrap();
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use crate::util::paths;
	use std::io::Write;
	use tempfile::NamedTempFile;

	#[test]
	fn witch_new() {
		let paths: Vec<&str> = vec![
			"tests",
			"tests/assets",
		];

		// Matches will be canonicalized.
		let canon: Vec<PathBuf> = vec![
			paths::to_path_buf_abs("tests/assets/is-executable.sh"),
			paths::to_path_buf_abs("tests/assets/file.txt"),
		];

		// Casual walk.
		let walk: Witch = Witch::new(&paths, None);
		let files: HashSet<PathBuf> = walk.files();

		assert!(! walk.is_empty());
		assert_eq!(files.len(), 2);
		assert_eq!(files.len(), walk.len());

		for i in &canon {
			assert!(files.contains(i));
		}

		// Check with a file path.
		let walk: Witch = Witch::new(&canon[..1], None);
		let files: HashSet<PathBuf> = walk.files();

		assert_eq!(files.len(), 1);
		assert!(files.contains(&canon[0]));

		// Filtered walk.
		let walk: Witch = Witch::new(&paths, Some(r"(?i).+\.sh$".into()));
		let files: HashSet<PathBuf> = walk.files();

		assert_eq!(files.len(), 1);
		assert!(files.contains(&canon[0]));

		// Sad walk.
		let walk: Witch = Witch::new(&paths, Some(r"(?i).+\.exe$".into()));
		let files: HashSet<PathBuf> = walk.files();

		assert!(walk.is_empty());
		assert_eq!(files.len(), 0);
	}

	#[test]
	fn witch_from_file() {
		// Populate a list real quick.
		let list: NamedTempFile = NamedTempFile::new()
			.expect("Tempfile, damn it.");
		let paths: String = format!(
			"{}\n{}\n",
			paths::to_string_abs(&PathBuf::from("tests")),
			paths::to_string_abs(&PathBuf::from("tests/assets")),
		);
		let path: PathBuf = list.path().to_path_buf();

		{
			let mut file = list.as_file();
			file.write_all(paths.as_bytes()).expect("Write, damn it.");
			file.flush().unwrap();
			drop(file);
		}

		assert!(path.is_file());

		// Matches will be canonicalized.
		let canon: Vec<PathBuf> = vec![
			paths::to_path_buf_abs("tests/assets/is-executable.sh"),
			paths::to_path_buf_abs("tests/assets/file.txt"),
		];

		// Casual walk.
		let walk: Witch = Witch::from_file(&path, None);
		let files: HashSet<PathBuf> = walk.files();

		assert_eq!(files.len(), 2);
		for i in &canon {
			assert!(files.contains(i));
		}

		// Filtered walk.
		let walk: Witch = Witch::from_file(&path, Some(r"(?i).+\.sh$".into()));
		let files: HashSet<PathBuf> = walk.files();

		assert_eq!(files.len(), 1);
		assert!(files.contains(&canon[0]));

		drop(list);
		assert!(! path.is_file());
	}

	// Note: Witch::files(), ::len(), and ::is_empty() are repeatedly
	// covered by the other tests so are not drawn out separately.

	#[test]
	fn walk_du() {
		let walk: Witch = Witch::new(
			&["tests/assets/file.txt"],
			Some(r"(?i).+\.txt$".into()),
		);
		assert_eq!(walk.du(), 26);
	}
}
