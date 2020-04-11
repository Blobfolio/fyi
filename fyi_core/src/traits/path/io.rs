/*!
# FYI Core: Miscellany: Operations
*/

use crate::{
	Error,
	Result,
	traits::path::{
		FYIPath,
		FYIPathFormat,
	},
};
use std::{
	env,
	fs::{
		self,
		File,
	},
	io::prelude::*,
	os::unix::fs::MetadataExt,
	path::{
		Path,
		PathBuf,
	},
};



/// Format/Conversion/Mutation Helpers!
pub trait FYIPathIO {
	/// Byte for Byte Copy.
	fn fyi_copy<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path>;

	/// Copy To Temporary Location.
	fn fyi_copy_tmp(&self) -> Result<PathBuf>;

	/// Delete.
	fn fyi_delete(&self) -> Result<()>;

	/// Move.
	fn fyi_move<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path>;

	/// Read Bytes.
	fn fyi_read(&self) -> Result<Vec<u8>>;

	/// Clone permissions and ownership from a path.
	fn fyi_reference<P> (&self, src: P) -> Result<()>
	where P: AsRef<Path>;

	/// Write Bytes.
	fn fyi_write(&self, data: &[u8]) -> Result<()>;
}

impl FYIPathIO for Path {
	/// Byte for Byte Copy.
	fn fyi_copy<P> (&self, to: P) -> Result<()>
	where P: AsRef<Path> {
		if self.is_file() {
			let data: Vec<u8> = self.fyi_read()?;
			let to = to.as_ref().to_path_buf();
			to.fyi_write(&data)?;

			Ok(())
		}
		else {
			Err(Error::PathFailed("copy", self.to_path_buf()))
		}
	}

	/// Copy To Temporary Location.
	fn fyi_copy_tmp(&self) -> Result<PathBuf> {
		let mut to: PathBuf = env::temp_dir();
		to.push(&self.fyi_file_name());
		to = to.fyi_to_path_buf_unique()?;
		self.fyi_copy(&to)?;

		Ok(to)
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
			Err(Error::PathFailed("delete", self.to_path_buf()))
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
			Err(Error::PathFailed("read", self.to_path_buf()))
		}
	}

	/// Clone permissions and ownership from a path.
	fn fyi_reference<P> (&self, src: P) -> Result<()>
	where P: AsRef<Path> {
		if self.is_file() {
			if let Ok(meta) = src.as_ref().metadata() {
				if meta.is_file() {
					use nix::unistd::{self, Uid, Gid};

					// Permissions are easy.
					fs::set_permissions(&self, meta.permissions())?;

					// Ownership is a little more annoying.
					unistd::chown(
						self,
						Some(Uid::from_raw(meta.uid())),
						Some(Gid::from_raw(meta.gid()))
					)?;

					return Ok(());
				}
			}
		}

		Err(Error::PathFailed("owner/perms", self.to_path_buf()))
	}

	/// Write Bytes.
	fn fyi_write(&self, data: &[u8]) -> Result<()> {
		if false == self.is_dir() {
			{
				let mut output = File::create(&self)?;

				output.set_len(data.len() as u64)?;
				output.write_all(&data)?;
				output.flush().unwrap();
			}

			Ok(())
		}
		else {
			Err(Error::PathFailed("write", self.to_path_buf()))
		}
	}
}
