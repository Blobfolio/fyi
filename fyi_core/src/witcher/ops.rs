/*!
# FYI Core: Miscellany: Operations
*/

use crate::witcher::{
	formats::FYIPathFormat,
	props::FYIPath,
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
	fn fyi_copy<P> (&self, to: P) -> Result<(), String>
	where P: AsRef<Path>;

	/// Copy To Temporary Location.
	fn fyi_copy_tmp(&self) -> Result<PathBuf, String>;

	/// Delete.
	fn fyi_delete(&self) -> Result<(), String>;

	/// Move.
	fn fyi_move<P> (&self, to: P) -> Result<(), String>
	where P: AsRef<Path>;

	/// Read Bytes.
	fn fyi_read(&self) -> Result<Vec<u8>, String>;

	/// Clone permissions and ownership from a path.
	fn fyi_reference<P> (&self, src: P) -> Result<(), String>
	where P: AsRef<Path>;

	/// Write Bytes.
	fn fyi_write(&self, data: &[u8]) -> Result<(), String>;
}

impl FYIPathIO for Path {
	/// Byte for Byte Copy.
	fn fyi_copy<P> (&self, to: P) -> Result<(), String>
	where P: AsRef<Path> {
		if self.is_file() {
			let data: Vec<u8> = self.fyi_read()?;
			let to = to.as_ref().to_path_buf();
			to.fyi_write(&data)?;

			Ok(())
		}
		else {
			Err(format!("Unable to copy: {}", self.fyi_to_string()).to_string())
		}
	}

	/// Copy To Temporary Location.
	fn fyi_copy_tmp(&self) -> Result<PathBuf, String> {
		let mut to: PathBuf = env::temp_dir();
		to.push(&self.fyi_file_name());
		to = to.fyi_to_path_buf_unique()?;
		self.fyi_copy(&to)?;

		Ok(to)
	}

	/// Delete.
	fn fyi_delete(&self) -> Result<(), String> {
		if self.is_file() {
			let _ = fs::remove_file(&self).map_err(|x| x.to_string())?;
			Ok(())
		}
		else if false == self.exists() {
			Ok(())
		}
		else {
			Err(format!("Could not delete: {}", self.fyi_to_string()).to_string())
		}
	}

	/// Move.
	fn fyi_move<P> (&self, to: P) -> Result<(), String>
	where P: AsRef<Path> {
		self.fyi_copy(&to)?;
		self.fyi_delete()?;

		Ok(())
	}

	/// Read Bytes.
	fn fyi_read(&self) -> Result<Vec<u8>, String> {
		if self.is_file() {
			match fs::read(&self) {
				Ok(data) => Ok(data),
				Err(e) => Err(e.to_string())
			}
		}
		else {
			Err(format!("Could not read: {}", self.fyi_to_string()).to_string())
		}
	}

	/// Clone permissions and ownership from a path.
	fn fyi_reference<P> (&self, src: P) -> Result<(), String>
	where P: AsRef<Path> {
		if self.is_file() {
			if let Ok(meta) = src.as_ref().metadata() {
				if meta.is_file() {
					use nix::unistd::{self, Uid, Gid};

					// Permissions are easy.
					fs::set_permissions(&self, meta.permissions())
						.map_err(|x| x.to_string())?;

					// Ownership is a little more annoying.
					unistd::chown(
						self,
						Some(Uid::from_raw(meta.uid())),
						Some(Gid::from_raw(meta.gid()))
					).map_err(|x| x.to_string())?;

					return Ok(());
				}
			}
		}

		Err(format!("Could not set ownership/permissions: {}", self.fyi_to_string()).to_string())
	}

	/// Write Bytes.
	fn fyi_write(&self, data: &[u8]) -> Result<(), String> {
		if false == self.is_dir() {
			{
				let mut output = File::create(&self)
					.map_err(|e| e.to_string())?;

				output.set_len(data.len() as u64).map_err(|e| e.to_string())?;
				output.write_all(&data).map_err(|e| e.to_string())?;
				output.flush().unwrap();
			}

			Ok(())
		}
		else {
			Err(format!("Could not write to: {}", self.fyi_to_string()).to_string())
		}
	}
}
