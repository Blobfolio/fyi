/*!
# FYI Core: Miscellany: Operations
*/

use fyi_core::{
	Error,
	Result,
};
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
			to.witch_write(&self.witch_read()?)?;

			Ok(())
		}
		else {
			Err(Error::new(format!(
				"Unable to copy {:?} to {:?}.",
				self,
				to.as_ref(),
			)))
		}
	}

	/// Copy To Temporary Location.
	fn witch_copy_tmp(&self, suffix: Option<String>) -> Result<NamedTempFile> {
		let meta = self.metadata()?;
		if meta.is_file() {
			let parent = self.parent()
				.ok_or(Error::new(format!(
					"Unable to copy {:?} to tmpfile.",
					self,
				)))?;

			// Allocate a tempfile.
			let target = match suffix {
				Some(x) => tempfile::Builder::new()
					.suffix(OsStr::new(x.as_str()))
					.tempfile_in(parent)?,
				None => NamedTempFile::new_in(parent)?,
			};

			// Copy references.
			target.path().witch_reference_from(&self)?;

			// Write data.
			let mut file = target.as_file();
			let data: Vec<u8> = fs::read(&self)?;
			file.write_all(&data)?;
			file.flush().unwrap();

			Ok(target)
		}
		else {
			Err(Error::new(format!(
				"Unable to copy {:?} to tmpfile.",
				self,
			)))
		}
	}

	/// Delete.
	fn witch_delete(&self) -> Result<()> {
		if self.is_file() {
			fs::remove_file(&self)?;
			Ok(())
		}
		else if false == self.exists() {
			Ok(())
		}
		else {
			Err(Error::new(format!(
				"Unable to delete {:?}.",
				self,
			)))
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
			Ok(fs::read(&self)?)
		}
		else {
			Err(Error::new(format!(
				"Unable to read {:?}.",
				self,
			)))
		}
	}

	/// Chown/Chmod to Reference.
	fn witch_reference_from<P> (&self, from: P) -> Result<(bool, bool)>
	where P: AsRef<Path> {
		let meta = from.as_ref().metadata()?;
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
			Err(Error::new(format!(
				"Unable to set owner/perms for {:?} using {:?}.",
				self,
				from.as_ref(),
			)))
		}
	}

	/// Write Bytes.
	fn witch_write(&self, data: &[u8]) -> Result<()> {
		if false == self.is_dir() {
			let mut tmp = tempfile_fast::Sponge::new_for(&self)?;
			tmp.write_all(&data)?;
			tmp.commit()?;

			Ok(())
		}
		else {
			Err(Error::new(format!(
				"Unable to write to {:?}.",
				self,
			)))
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use fyi_core::traits::PathProps;
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
		assert_eq!(from.file_size(), to.file_size());

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
			assert_eq!(from.file_size(), path.file_size());

			drop(tmp);
			assert!(! path.is_file());
		}

		{
			// Now with one.
			let tmp: NamedTempFile = from.witch_copy_tmp(Some(".bak".into()))
				.unwrap();
			let path: PathBuf = tmp.path().to_path_buf();

			assert!(path.is_file());
			assert_eq!(path.file_extension(), "bak");
			assert_eq!(from.file_size(), path.file_size());

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
		assert_eq!(src.file_size(), to.file_size());

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
		assert_eq!(path.file_size(), 0);

		// Write it.
		path.witch_write(data.as_bytes()).unwrap();

		let data2: Vec<u8> = path.witch_read().unwrap();
		let human: String = String::from_utf8(data2).unwrap();

		assert_eq!(data, human);

		drop(tmp);
		assert!(! path.is_file());
	}
}
