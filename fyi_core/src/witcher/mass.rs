/*!
# FYI Core: Miscellany: Mass Operations
*/

use crate::witcher::formats::FYIFormats;
use crate::witcher::ops::FYIOps;
use crate::witcher::props::FYIProps;
use rayon::prelude::*;
use std::path::PathBuf;
use regex::Regex;



/// Vector Business.
pub trait FYIMassOps {
	/// File sizes.
	fn fyi_file_sizes(&self) -> u64;

	/// To Abs PathBuf.
	fn fyi_to_path_buf_abs(&self) -> Vec<PathBuf>;

	/// To Abs PathBuf.
	fn fyi_into_path_buf_abs(&mut self);

	/// Recursive walk.
	fn fyi_walk(&self) -> Vec<PathBuf>;

	/// Mute walk.
	fn fyi_walk_mut(&mut self);

	/// Careful walk.
	fn fyi_walk_filtered(&self, pattern: &Regex) -> Vec<PathBuf>;

	/// Mute walk.
	fn fyi_walk_filtered_mut(&mut self, pattern: &Regex);
}

impl FYIMassOps for Vec<PathBuf> {
	/// File sizes.
	fn fyi_file_sizes(&self) -> u64 {
		self.par_iter()
			.map(|ref x| match x.is_dir() {
				true => {
					let tmp = x.fyi_walk();
					tmp.fyi_file_sizes()
				},
				false => x.fyi_file_size(),
			})
			.sum()
	}

	/// To Abs PathBuf.
	fn fyi_to_path_buf_abs(&self) -> Vec<PathBuf> {
		let mut out: Vec<PathBuf> = self.iter()
		.filter_map(|ref x| match x.exists() {
			true => Some(x.fyi_to_path_buf_abs()),
			false => None,
		})
		.collect();

		if 1 < out.len() {
			out.par_sort();
			out.dedup();
		}

		out
	}

	/// To Abs PathBuf.
	fn fyi_into_path_buf_abs(&mut self) {
		self.retain(|ref x| x.exists());
		self.par_iter_mut().for_each(|x| *x = x.fyi_to_path_buf_abs());

		if 1 < self.len() {
			self.par_sort();
			self.dedup();
		}
	}

	/// Recursive walk.
	fn fyi_walk(&self) -> Vec<PathBuf> {
		let mut out: Vec<PathBuf> = Vec::new();

		self.fyi_to_path_buf_abs()
			.iter()
			.for_each(|ref x| {
				if x.is_dir() {
					out.par_extend(x.fyi_walk());
				}
				else {
					out.push(x.to_path_buf());
				}
			});

		if 1 < out.len() {
			out.par_sort();
			out.dedup();
		}

		out
	}

	/// Mute walk.
	fn fyi_walk_mut(&mut self) {
		*self = self.fyi_walk();
	}

	/// Careful walk.
	fn fyi_walk_filtered(&self, pattern: &Regex) -> Vec<PathBuf> {
		let mut out: Vec<PathBuf> = Vec::new();

		self.fyi_to_path_buf_abs()
			.iter()
			.for_each(|ref x| {
				out.par_extend(x.fyi_walk_filtered(&pattern));
			});

		if 1 < out.len() {
			out.par_sort();
			out.dedup();
		}

		out
	}

	/// Mute walk.
	fn fyi_walk_filtered_mut(&mut self, pattern: &Regex) {
		*self = self.fyi_walk_filtered(&pattern);
	}
}
