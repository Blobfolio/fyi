/*!
# FYI Core: Miscellany: Operations
*/

use crate::{
	Error,
	Result,
};
use std::{
	ffi::OsStr,
	fs,
	io::prelude::*,
	os::unix::fs::MetadataExt,
	path::Path,
};
use tempfile::NamedTempFile;



/// Format/Conversion/Mutation Helpers!
pub trait FYIPathIO {
	/// Byte for Byte Copy.
	fn fyi_copy<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path>;

	/// Copy To Temporary Location.
	fn fyi_copy_tmp(&self, suffix: Option<String>) -> Result<NamedTempFile>;

	/// Delete.
	fn fyi_delete(&self) -> Result<()>;

	/// Move.
	fn fyi_move<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path>;

	/// Read Bytes.
	fn fyi_read(&self) -> Result<Vec<u8>>;

	/// Write Bytes.
	fn fyi_write(&self, data: &[u8]) -> Result<()>;
}

impl FYIPathIO for Path {
	/// Byte for Byte Copy.
	fn fyi_copy<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path> {
		if self.is_file() {
			let to = to.as_ref().to_path_buf();
			to.fyi_write(&self.fyi_read()?)?;

			Ok(())
		}
		else {
			Err(Error::PathCopy(self.to_path_buf()))
		}
	}

	/// Copy To Temporary Location.
	fn fyi_copy_tmp(&self, suffix: Option<String>) -> Result<NamedTempFile> {
		use nix::unistd::{self, Uid, Gid};

		let meta = self.metadata()?;
		if meta.is_file() {
			let parent = self.parent()
				.ok_or(Error::PathCopy(self.to_path_buf()))?;

			let target = match suffix {
				Some(x) => tempfile::Builder::new()
					.suffix(OsStr::new(x.as_str()))
					.tempfile_in(parent)?,
				None => NamedTempFile::new_in(parent)?,
			};

			let file = target.as_file();
			file.set_permissions(meta.permissions())?;
			unistd::chown(
				target.path(),
				Some(Uid::from_raw(meta.uid())),
				Some(Gid::from_raw(meta.gid()))
			)?;

			target.path().fyi_write(&self.fyi_read()?)?;
			Ok(target)
		}
		else {
			Err(Error::PathCopy(self.to_path_buf()))
		}
	}

	/// Delete.
	fn fyi_delete(&self) -> Result<()> {
		if self.is_file() {
			let _ = fs::remove_file(&self)?;
			Ok(())
		}
		else if false == self.exists() {
			Ok(())
		}
		else {
			Err(Error::PathDelete(self.to_path_buf()))
		}
	}

	/// Move.
	fn fyi_move<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path> {
		self.fyi_copy(&to)?;
		self.fyi_delete()?;

		Ok(())
	}

	/// Read Bytes.
	fn fyi_read(&self) -> Result<Vec<u8>> {
		if self.is_file() {
			Ok(fs::read(&self)?)
		}
		else {
			Err(Error::PathRead(self.to_path_buf()))
		}
	}

	/// Write Bytes.
	fn fyi_write(&self, data: &[u8]) -> Result<()> {
		if false == self.is_dir() {
			let mut tmp = tempfile_fast::Sponge::new_for(&self)?;
			tmp.write_all(&data)?;
			tmp.commit()?;

			Ok(())
		}
		else {
			Err(Error::PathWrite(self.to_path_buf()))
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::PathProps;
	use std::path::PathBuf;

	#[test]
	fn fyi_copy() {
		let from: PathBuf = PathBuf::from("tests/assets/file.txt");
		let to: PathBuf = PathBuf::from("tests/assets/fyi_copy.bak");

		// Make sure the destination is empty before starting.
		if to.is_file() {
			to.fyi_delete().expect("Delete, damn it.");
		}

		assert!(from.is_file());
		assert!(! to.is_file());

		from.fyi_copy(&to).expect("Copy, damn it.");
		assert!(to.is_file());
		assert_eq!(from.file_size(), to.file_size());

		// And remove it.
		to.fyi_delete().expect("Delete, damn it.");
	}

	#[test]
	fn fyi_copy_tmp() {
		let from: PathBuf = PathBuf::from("tests/assets/file.txt");
		assert!(from.is_file());

		{
			// First without a suffix.
			let tmp: NamedTempFile = from.fyi_copy_tmp(None)
				.expect("Tempfile, damn it.");
			let path: PathBuf = tmp.path().to_path_buf();

			assert!(path.is_file());
			assert_eq!(from.file_size(), path.file_size());

			drop(tmp);
			assert!(! path.is_file());
		}

		{
			// Now with one.
			let tmp: NamedTempFile = from.fyi_copy_tmp(Some(".bak".into()))
				.expect("Tempfile, damn it.");
			let path: PathBuf = tmp.path().to_path_buf();

			assert!(path.is_file());
			assert_eq!(path.file_extension(), "bak");
			assert_eq!(from.file_size(), path.file_size());

			drop(tmp);
			assert!(! path.is_file());
		}
	}

	// Note: fyi_delete() is covered multiple times by other tests.

	#[test]
	fn fyi_move() {
		let src: PathBuf = PathBuf::from("tests/assets/file.txt");
		let from: PathBuf = PathBuf::from("tests/assets/fyi_move.1");
		let to: PathBuf = PathBuf::from("tests/assets/fyi_move.2");

		// Copy the original so we don't mess it up.
		src.fyi_copy(&from).expect("Copy, damn it.");

		// Make sure the destination is empty before starting.
		if to.is_file() {
			to.fyi_delete().expect("Delete, damn it.");
		}

		assert!(from.is_file());
		assert!(! to.is_file());

		from.fyi_move(&to).expect("Move, damn it.");
		assert!(! from.is_file());
		assert!(to.is_file());

		// We can compare it against our original.
		assert_eq!(src.file_size(), to.file_size());

		// And remove it.
		to.fyi_delete().expect("Delete, damn it.");
	}

	#[test]
	fn fyi_read() {
		let path: PathBuf = PathBuf::from("tests/assets/file.txt");
		assert!(path.is_file());

		let data: Vec<u8> = path.fyi_read().expect("Read, damn it.");
		let human: String = String::from_utf8(data).expect("String, damn it.");

		assert_eq!(&human, "This is just a text file.\n");
	}

	#[test]
	fn fyi_write() {
		let data: String = "Hello World".to_string();

		let tmp: NamedTempFile = NamedTempFile::new()
			.expect("Tempfile, damn it.");
		let path: PathBuf = tmp.path().to_path_buf();

		assert!(path.is_file());
		assert_eq!(path.file_size(), 0);

		// Write it.
		path.fyi_write(data.as_bytes()).expect("Write, damn it.");

		let data2: Vec<u8> = path.fyi_read().expect("Read, damn it.");
		let human: String = String::from_utf8(data2).expect("String, damn it.");

		assert_eq!(data, human);

		drop(tmp);
		assert!(! path.is_file());
	}
}
