/*!
# FYI Core: Path Walking
*/

use crate::witcher::{
	formats::FYIPathFormat,
	mass::FYIPathMIO,
};
use regex::Regex;
use std::{
	collections::HashSet,
	ffi::OsStr,
	fs::File,
	io::{
		BufReader,
		BufRead,
	},
	path::{
		Path,
		PathBuf,
	},
};
use walkdir::WalkDir;



/// Path walking!
pub trait FYIPathWalk {
	/// Recursive walk.
	fn fyi_walk(&self) -> Vec<PathBuf>;

	/// Careful walk.
	fn fyi_walk_filtered(&self, pattern: &Regex) -> Vec<PathBuf>;

	/// Walk contents (e.g. txt file).
	fn fyi_walk_file_lines(&self, pat: Option<Regex>) -> Vec<PathBuf>;

	/// Recursive walk (HashSet version).
	fn fyi_walk_hs(&self) -> HashSet<PathBuf>;

	/// Careful walk (HashSet version).
	fn fyi_walk_filtered_hs(&self, pattern: &Regex) -> HashSet<PathBuf>;

	/// Walk contents (e.g. txt file).
	fn fyi_walk_file_lines_hs(&self, pat: Option<Regex>) -> HashSet<PathBuf>;
}

impl FYIPathWalk for Path {
	/// Recursive walk.
	fn fyi_walk(&self) -> Vec<PathBuf> {
		if self.is_dir() {
			let walked: Vec<PathBuf> = WalkDir::new(self.fyi_to_path_buf_abs())
				.follow_links(true)
				.into_iter()
				.filter_map(|x| match x {
					Ok(y) => {
						if y.file_type().is_file() {
							Some(y.path().fyi_to_path_buf_abs())
						}
						else {
							None
						}
					},
					_ => None,
				})
				.collect();

			walked
		}
		else if self.is_file() {
			vec![self.fyi_to_path_buf_abs()]
		}
		else {
			Vec::new()
		}
	}

	/// Careful walk.
	fn fyi_walk_filtered(&self, pattern: &Regex) -> Vec<PathBuf> {
		if self.is_dir() {
			let walked: Vec<PathBuf> = WalkDir::new(self.fyi_to_path_buf_abs())
				.follow_links(true)
				.into_iter()
				.filter_map(|x| match x {
					Ok(y) => {
						if y.file_type().is_file() {
							let name = y.file_name()
								.to_str()
								.unwrap_or("");

							match pattern.is_match(&name) {
								true => Some(y.path().fyi_to_path_buf_abs()),
								false => None,
							}
						}
						// Ignore directories.
						else {
							None
						}
					},
					_ => None,
				})
				.collect();

			walked
		}
		else if self.is_file() {
			let name = self.file_name()
				.unwrap_or(OsStr::new(""))
				.to_str()
				.unwrap_or("");

			match pattern.is_match(&name) {
				true => vec![self.fyi_to_path_buf_abs()],
				false => Vec::new(),
			}
		}
		else {
			Vec::new()
		}
	}

	/// Walk contents (e.g. txt file).
	fn fyi_walk_file_lines(&self, pat: Option<Regex>) -> Vec<PathBuf> {
		if self.is_file() {
			let input = File::open(&self).expect("Unable to open file.");
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

			match pat {
				Some(pat) => out.fyi_walk_filtered(&pat),
				None => out.fyi_walk(),
			}
		}
		else {
			Vec::new()
		}
	}

	/// Recursive walk.
	fn fyi_walk_hs(&self) -> HashSet<PathBuf> {
		if self.is_dir() {
			let walked: HashSet<PathBuf> = WalkDir::new(self.fyi_to_path_buf_abs())
				.follow_links(true)
				.into_iter()
				.filter_map(|x| match x {
					Ok(y) => {
						if y.file_type().is_file() {
							Some(y.path().fyi_to_path_buf_abs())
						}
						else {
							None
						}
					},
					_ => None,
				})
				.collect();

			walked
		}
		else if self.is_file() {
			let mut walked = HashSet::new();
			walked.insert(self.fyi_to_path_buf_abs());
			walked
		}
		else {
			HashSet::new()
		}
	}

	/// Careful walk.
	fn fyi_walk_filtered_hs(&self, pattern: &Regex) -> HashSet<PathBuf> {
		if self.is_dir() {
			let walked: HashSet<PathBuf> = WalkDir::new(self.fyi_to_path_buf_abs())
				.follow_links(true)
				.into_iter()
				.filter_map(|x| match x {
					Ok(y) => {
						if y.file_type().is_file() {
							let name = y.file_name()
								.to_str()
								.unwrap_or("");

							match pattern.is_match(&name) {
								true => Some(y.path().fyi_to_path_buf_abs()),
								false => None,
							}
						}
						// Ignore directories.
						else {
							None
						}
					},
					_ => None,
				})
				.collect();

			walked
		}
		else if self.is_file() {
			let name = self.file_name()
				.unwrap_or(OsStr::new(""))
				.to_str()
				.unwrap_or("");

			match pattern.is_match(&name) {
				true => {
					let mut walked = HashSet::new();
					walked.insert(self.fyi_to_path_buf_abs());
					walked
				},
				false => HashSet::new(),
			}
		}
		else {
			HashSet::new()
		}
	}

	/// Walk contents (e.g. txt file).
	fn fyi_walk_file_lines_hs(&self, pat: Option<Regex>) -> HashSet<PathBuf> {
		if self.is_file() {
			let input = File::open(&self).expect("Unable to open file.");
			let buffered = BufReader::new(input);

			let out: HashSet<PathBuf> = buffered.lines()
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

			match pat {
				Some(pat) => out.fyi_walk_filtered(&pat),
				None => out.fyi_walk(),
			}
		}
		else {
			HashSet::new()
		}
	}
}
