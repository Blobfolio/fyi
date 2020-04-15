/*!
# FYI Core: Miscellany: Operations
*/

use crate::{
	Error,
	Result,
};
use std::{
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
	fn fyi_copy_tmp(&self) -> Result<NamedTempFile>;

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
	fn fyi_copy_tmp(&self) -> Result<NamedTempFile> {
		use nix::unistd::{self, Uid, Gid};

		let meta = self.metadata()?;
		if meta.is_file() {
			let parent = self.parent()
				.ok_or(Error::PathCopy(self.to_path_buf()))?;
			let target = NamedTempFile::new_in(parent)?;

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
