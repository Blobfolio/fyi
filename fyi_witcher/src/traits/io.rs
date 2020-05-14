/*!
# FYI Witcher Traits: Path I/O

This trait adds a handful of read/write/copy/move-type methods to `Path`
objects.
*/

use crate::Result;
use nix::unistd::{self, Uid, Gid};
use std::{
	ffi::OsStr,
	fs,
	io::prelude::*,
	os::unix::fs::MetadataExt,
	path::Path,
};
use tempfile::NamedTempFile;



/// Format/Conversion/Mutation Helpers!
pub trait WitchIO {
	/// Byte for Byte Copy.
	fn witch_copy<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path>;

	/// Copy To Temporary Location.
	fn witch_copy_tmp(&self, suffix: Option<String>) -> Result<NamedTempFile>;

	/// Delete.
	fn witch_delete(&self) -> Result<()>;

	/// Move.
	fn witch_move<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path>;

	/// Read Bytes.
	fn witch_read(&self) -> Result<Vec<u8>>;

	/// Chown/Chmod to Reference.
	fn witch_reference_from<P> (&self, from: P) -> Result<(bool, bool)>
	where P: AsRef<Path>;

	/// Write Bytes.
	fn witch_write(&self, data: &[u8]) -> Result<()>;
}

impl WitchIO for Path {
	/// Byte for Byte Copy.
	fn witch_copy<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path> {
		if self.is_file() {
			let to = to.as_ref().to_path_buf();
			to.witch_write(&fs::read(&self).map_err(|e| e.to_string())?)?;

			Ok(())
		}
		else {
			Err(format!(
				"Unable to copy {:?} to {:?}.",
				self,
				to.as_ref(),
			))
		}
	}

	/// Copy To Temporary Location.
	fn witch_copy_tmp(&self, suffix: Option<String>) -> Result<NamedTempFile> {
		let meta = self.metadata().map_err(|e| e.to_string())?;
		if meta.is_file() {
			let parent = self.parent()
				.ok_or_else(|| format!(
					"Unable to copy {:?} to tmpfile.",
					self,
				))?;


			// Allocate a tempfile.
			let target = match suffix {
				Some(x) => tempfile::Builder::new()
					.suffix(OsStr::new(x.as_str()))
					.tempfile_in(parent).map_err(|e| e.to_string())?,
				None => NamedTempFile::new_in(parent).map_err(|e| e.to_string())?,
			};

			// Copy references.
			target.path().witch_reference_from(&self)?;

			// Write data.
			let mut file = target.as_file();
			let data: Vec<u8> = fs::read(&self).map_err(|e| e.to_string())?;
			file.write_all(&data).map_err(|e| e.to_string())?;
			file.flush().unwrap();

			Ok(target)
		}
		else {
			Err(format!(
				"Unable to copy {:?} to tmpfile.",
				self,
			))
		}
	}

	/// Delete.
	fn witch_delete(&self) -> Result<()> {
		if self.is_file() {
			fs::remove_file(&self).map_err(|e| e.to_string())?;
			Ok(())
		}
		else if self.exists() {
			Err(format!(
				"Unable to delete {:?}.",
				self,
			))
		}
		else {
			Ok(())
		}
	}

	/// Move.
	fn witch_move<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path> {
		self.witch_copy(&to)?;
		self.witch_delete()?;

		Ok(())
	}

	/// Read Bytes.
	fn witch_read(&self) -> Result<Vec<u8>> {
		if self.is_file() {
			Ok(fs::read(&self).map_err(|e| e.to_string())?)
		}
		else {
			Err(format!(
				"Unable to read {:?}.",
				self,
			))
		}
	}

	/// Chown/Chmod to Reference.
	fn witch_reference_from<P> (&self, from: P) -> Result<(bool, bool)>
	where P: AsRef<Path> {
		let meta = from.as_ref().metadata().map_err(|e| e.to_string())?;
		if meta.is_file() && self.is_file() {
			Ok((
				fs::set_permissions(&self, meta.permissions()).is_ok(),
				unistd::chown(
					self,
					Some(Uid::from_raw(meta.uid())),
					Some(Gid::from_raw(meta.gid()))
				).is_ok()
			))
		}
		else {
			Err(format!(
				"Unable to set owner/perms for {:?} using {:?}.",
				self,
				from.as_ref(),
			))
		}
	}

	/// Write Bytes.
	fn witch_write(&self, data: &[u8]) -> Result<()> {
		if self.is_dir() {
			Err(format!(
				"Unable to write to {:?}.",
				self,
			))
		}
		else {
			let mut tmp = tempfile_fast::Sponge::new_for(&self)
				.map_err(|e| e.to_string())?;
			tmp.write_all(data)
				.map_err(|e| e.to_string())?;
			tmp.commit()
				.map_err(|e| e.to_string())?;

			Ok(())
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use crate::utility::{
		file_extension,
		file_size,
	};
	use std::path::PathBuf;

	#[test]
	fn witch_copy() {
		let from: PathBuf = PathBuf::from("tests/assets/file.txt");
		let to: PathBuf = PathBuf::from("tests/assets/witch_copy.bak");

		// Make sure the destination is empty before starting.
		if to.is_file() {
			to.witch_delete().unwrap();
		}

		assert!(from.is_file());
		assert!(! to.is_file());

		from.witch_copy(&to).unwrap();
		assert!(to.is_file());
		assert_eq!(file_size(&from), file_size(&to));

		// And remove it.
		to.witch_delete().unwrap();
	}

	#[test]
	fn witch_copy_tmp() {
		let from: PathBuf = PathBuf::from("tests/assets/file.txt");
		assert!(from.is_file());

		{
			// First without a suffix.
			let tmp: NamedTempFile = from.witch_copy_tmp(None).unwrap();
			let path: PathBuf = tmp.path().to_path_buf();

			assert!(path.is_file());
			assert_eq!(file_size(&from), file_size(&path));

			drop(tmp);
			assert!(! path.is_file());
		}

		{
			// Now with one.
			let tmp: NamedTempFile = from.witch_copy_tmp(Some(".bak".into()))
				.unwrap();
			let path: PathBuf = tmp.path().to_path_buf();

			assert!(path.is_file());
			assert_eq!(file_extension(&path), "bak");
			assert_eq!(file_size(&from), file_size(&path));

			drop(tmp);
			assert!(! path.is_file());
		}
	}

	// Note: witch_delete() is covered multiple times by other tests.

	#[test]
	fn witch_move() {
		let src: PathBuf = PathBuf::from("tests/assets/file.txt");
		let from: PathBuf = PathBuf::from("tests/assets/witch_move.1");
		let to: PathBuf = PathBuf::from("tests/assets/witch_move.2");

		// Copy the original so we don't mess it up.
		src.witch_copy(&from).unwrap();

		// Make sure the destination is empty before starting.
		if to.is_file() {
			to.witch_delete().unwrap();
		}

		assert!(from.is_file());
		assert!(! to.is_file());

		from.witch_move(&to).unwrap();
		assert!(! from.is_file());
		assert!(to.is_file());

		// We can compare it against our original.
		assert_eq!(file_size(&src), file_size(&to));

		// And remove it.
		to.witch_delete().unwrap();
	}

	#[test]
	fn witch_read() {
		let path: PathBuf = PathBuf::from("tests/assets/file.txt");
		assert!(path.is_file());

		let data: Vec<u8> = path.witch_read().unwrap();
		let human: String = String::from_utf8(data).unwrap();

		assert_eq!(&human, "This is just a text file.\n");
	}

	#[test]
	fn witch_write() {
		let data: String = "Hello World".to_string();

		let tmp: NamedTempFile = NamedTempFile::new().unwrap();
		let path: PathBuf = tmp.path().to_path_buf();

		assert!(path.is_file());
		assert_eq!(file_size(&path), 0);

		// Write it.
		path.witch_write(data.as_bytes()).unwrap();

		let data2: Vec<u8> = path.witch_read().unwrap();
		let human: String = String::from_utf8(data2).unwrap();

		assert_eq!(data, human);

		drop(tmp);
		assert!(! path.is_file());
	}
}
