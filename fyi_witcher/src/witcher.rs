/*!
# FYI Witcher: Witcher

`Witcher` stores the results of a recursive file-search with all paths
canonicalized and validated against a regular expression.

*/

use indexmap::set::IndexSet;
use jwalk::WalkDir;
use rayon::prelude::*;
use regex::Regex;
use std::{
	borrow::Borrow,
	fs,
	ops::{
		Deref,
		DerefMut,
	},
	path::{
		Path,
		PathBuf,
	},
};



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
}
