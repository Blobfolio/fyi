/*!
# FYI Witcher Traits: Path I/O

This trait adds a few useful I/O methods to `Path`:
* `witch_copy()` Copy a file byte-for-byte to a new location.
* `witch_copy_tmp()` The same as above, but a tempfile is used (and returned) instead.
* `witch_move()` Move a file, byte for byte, to a new location.
* `witch_reference_from()` Copy ownership and permissions from another `Path`.
* `witch_write()` Write or rewrite data to a file, using a tempfile in the middle for safety.
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



/// Witch I/O
pub trait WitchIO {
	/// Byte for Byte Copy.
	fn witch_copy<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path>;

	/// Copy To Temporary Location.
	fn witch_copy_tmp(&self, suffix: Option<String>) -> Result<NamedTempFile>;

	/// Move.
	fn witch_move<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path>;

	/// Chown/Chmod to Reference.
	fn witch_reference_from<P> (&self, from: P) -> Result<(bool, bool)>
	where P: AsRef<Path>;

	/// Write Bytes.
	fn witch_write(&self, data: &[u8]) -> Result<()>;
}

impl WitchIO for Path {
	/// Copy File.
	///
	/// Copy the byte content of a file to a new location, using a tempfile in
	/// the middle for safety.
	///
	/// It is worth noting that this does *not* copy ownership or
	/// permissions to the new target. If that file already existed, it will
	/// keep its original attributes, otherwise the system will set whatever
	/// it deems appropriate.
	///
	/// Panics if `self` is not a file or `to` is a directory.
	fn witch_copy<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path> {
		assert!(
			self.exists() &&
			! self.is_dir() &&
			! to.as_ref().is_dir()
		);

		let to = to.as_ref().to_path_buf();
		to.witch_write(&fs::read(&self).map_err(|e| e.to_string())?)?;

		Ok(())
	}

	/// Copy To Temporary Location.
	///
	/// This behaves similarly to `witch_copy()`, except the destination *is*
	/// a tempfile, which gets returned at the end so it can be persisted or
	/// forgotten accordingly.
	///
	/// Unlike `witch_copy()` or `witch_move()`, this method *does* attempt to
	/// copy ownership and permission to the tempfile. (Its path is always
	/// unoccupied, so might as well choose its default.)
	///
	/// Panics if `self` is not a file.
	fn witch_copy_tmp(&self, suffix: Option<String>) -> Result<NamedTempFile> {
		assert!(self.exists() && ! self.is_dir());

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
		file.write_all(&fs::read(&self).map_err(|e| e.to_string())?)
			.map_err(|e| e.to_string())?;
		file.flush().map_err(|e| e.to_string())?;

		Ok(target)
	}

	/// Move File.
	///
	/// Move the byte content of a file to new location, using a tempfile in
	/// the middle for safety.
	///
	/// This is exactly like `witch_copy()`, except `self` is deleted
	/// afterwards.
	///
	/// Panics if `self` is not a file, or `to` is a directory.
	fn witch_move<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path> {
		assert!(
			self.exists() &&
			! self.is_dir() &&
			! to.as_ref().is_dir()
		);

		self.witch_copy(&to)?;
		fs::remove_file(&self).map_err(|e| e.to_string())?;

		Ok(())
	}

	/// Chown/Chmod From Reference.
	///
	/// Copy ownership and permissions from `from` to `self`, if possible.
	///
	/// On *nix systems, programs can only do what the users running them are
	/// allowed to do, so depending on the particulars, these operations may
	/// work only partially or fail entirely.
	///
	/// Unless a hard error is hit while trying to read the metadata of the
	/// reference file, the method will return separate `boolean` values for
	/// each part, permissions first, then ownership.
	///
	/// Panics if either path is not a file.
	fn witch_reference_from<P> (&self, from: P) -> Result<(bool, bool)>
	where P: AsRef<Path> {
		assert!(
			self.exists() &&
			from.as_ref().exists() &&
			! self.is_dir() &&
			! from.as_ref().is_dir()
		);

		let meta = from.as_ref().metadata().map_err(|e| e.to_string())?;
		Ok((
			fs::set_permissions(&self, meta.permissions()).is_ok(),
			unistd::chown(
				self,
				Some(Uid::from_raw(meta.uid())),
				Some(Gid::from_raw(meta.gid()))
			).is_ok()
		))
	}

	/// Write Bytes to File.
	///
	/// This is a small wrapper around `tempfile_fast`, which writes data to a
	/// tempfile before committing them to the chosen destination.
	///
	/// If `self` doesn't exist, it will be created.
	///
	/// Panics if `self` is a directory.
	fn witch_write(&self, data: &[u8]) -> Result<()> {
		assert!(! self.is_dir());

		let mut tmp = tempfile_fast::Sponge::new_for(&self)
			.map_err(|e| e.to_string())?;
		tmp.write_all(data)
			.map_err(|e| e.to_string())?;
		tmp.commit()
			.map_err(|e| e.to_string())?;

		Ok(())
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
			fs::remove_file(&to).unwrap();
		}

		assert!(from.is_file());
		assert!(! to.is_file());

		from.witch_copy(&to).unwrap();
		assert!(to.is_file());
		assert_eq!(file_size(&from), file_size(&to));

		// And remove it.
		fs::remove_file(&to).unwrap();
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

	#[test]
	fn witch_move() {
		let src: PathBuf = PathBuf::from("tests/assets/file.txt");
		let from: PathBuf = PathBuf::from("tests/assets/witch_move.1");
		let to: PathBuf = PathBuf::from("tests/assets/witch_move.2");

		// Copy the original so we don't mess it up.
		src.witch_copy(&from).unwrap();

		// Make sure the destination is empty before starting.
		if to.is_file() {
			fs::remove_file(&to).unwrap();
		}

		assert!(from.is_file());
		assert!(! to.is_file());

		from.witch_move(&to).unwrap();
		assert!(! from.is_file());
		assert!(to.is_file());

		// We can compare it against our original.
		assert_eq!(file_size(&src), file_size(&to));

		// And remove it.
		fs::remove_file(&to).unwrap();
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

		let data2: Vec<u8> = fs::read(&path).unwrap();
		let human: String = String::from_utf8(data2).unwrap();

		assert_eq!(data, human);

		drop(tmp);
		assert!(! path.is_file());
	}
}
