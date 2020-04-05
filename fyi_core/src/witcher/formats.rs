/*!
# FYI Core: Miscellany: Path Formatting
*/

use crate::misc::strings;
use crate::witcher::props::FYIPath;
use std::path::{
	Path,
	PathBuf,
};



/// Format/Conversion/Mutation Helpers!
pub trait FYIPathFormat {
	/// Append to Path.
	fn fyi_append<S> (&self, name: S) -> PathBuf
	where S: Into<String>;

	/// Absolute PathBuf.
	fn fyi_to_path_buf_abs(&self) -> PathBuf;

	/// To Unique PathBuf.
	fn fyi_to_path_buf_unique(&self) -> Result<PathBuf, String>;

	/// To String.
	fn fyi_to_string(&self) -> String;

	/// With File Name.
	fn fyi_with_file_name<S> (&self, name: S) -> PathBuf
	where S: Into<String>;
}

impl FYIPathFormat for Path {
	/// Append to Path.
	fn fyi_append<S> (&self, name: S) -> PathBuf
	where S: Into<String> {
		// A directory? Just push it.
		if self.is_dir() {
			self.fyi_with_file_name(name)
		}
		else {
			PathBuf::from(format!(
				"{}{}",
				self.fyi_to_string(),
				name.into()
			))
		}
	}

	/// Absolute PathBuf.
	fn fyi_to_path_buf_abs(&self) -> PathBuf {
		match self.canonicalize() {
			Ok(path) => path,
			_ => self.to_path_buf(),
		}
	}

	/// To Unique PathBuf.
	fn fyi_to_path_buf_unique(&self) -> Result<PathBuf, String> {
		if self.is_dir() {
			return Err(format!(
				"Path cannot be a directory: {}",
				self.fyi_to_string()
			));
		}

		// The parent must exist.
		let dir: PathBuf = self.fyi_parent()?;

		// We can leave early if the parent exists but not the target.
		if false == self.exists() {
			return Ok(self.fyi_to_path_buf_abs());
		}

		// Grab the name.
		let name: String = self.fyi_file_name();
		if name.is_empty() {
			return Err(format!(
				"Missing name: {}",
				self.fyi_to_string()
			));
		}

		// Split it on the first period; we'll add our uniqueness before
		// it.
		let (name, ext) = {
			let mut index = 0;
			for (k, v) in name.char_indices() {
				if 0 < k && '.' == v {
					index = k;
					break;
				}
			}

			if 0 < index {
				let (n, e) = name.split_at(index);
				(n.to_string(), e.to_string())
			}
			else {
				(name.clone(), String::new())
			}
		};

		// Increment the middle with numbers 100 times; should be plenty
		// for uniqueness without much overhead.
		for i in 0..100 {
			let alt: PathBuf = dir.fyi_with_file_name(format!(
				"{}__{}{}",
				&name,
				i,
				&ext
			));
			if false == alt.exists() {
				return Ok(alt);
			}
		}

		Err("No unique name could be created.".to_string())
	}

	/// To String.
	fn fyi_to_string(&self) -> String {
		strings::from_os_string(
			self.fyi_to_path_buf_abs().into_os_string()
		)
	}

	/// With File Name.
	fn fyi_with_file_name<S> (&self, name: S) -> PathBuf
	where S: Into<String> {
		if self.is_dir() {
			let mut clone: PathBuf = self.fyi_to_path_buf_abs();
			clone.push(name.into());
			clone
		}
		else {
			self.with_file_name(format!(
				"{}{}",
				self.fyi_file_name(),
				name.into()
			))
		}
	}
}
