/*!
# FYI Core: Obligatory Error Type
*/

use std::{
	fmt,
	io,
	path::PathBuf,
};



#[derive(thiserror::Error)]
/// Error!
pub enum Error {
	#[error(transparent)]
	/// Passthru IO.
	File(#[from] io::Error),

	#[cfg(feature = "witcher")]
	#[error(transparent)]
	/// Passthru IO.
	Nix(#[from] nix::Error),

	#[error("Error: {0}")]
	/// Miscellaneous.
	Other(String),

	#[error("No {0} were found.")]
	/// No paths.
	NoPaths(String),

	#[error("Failed to copy: {0}.")]
	/// Copy failed.
	PathCopy(PathBuf),

	#[error("Failed to delete: {0}.")]
	/// Delete failed.
	PathDelete(PathBuf),

	#[error("Invalid path: {0} {1}.")]
	/// Invalid path (for miscellaneous reasons).
	PathInvalid(PathBuf, &'static str),

	#[error("Failed to read: {0}.")]
	/// Read failed.
	PathRead(PathBuf),

	#[error("Failed to set owner/perms: {0}.")]
	/// Reference failed.
	PathReference(PathBuf),

	#[error("Failed to create unique path: {0}.")]
	/// Could not create unique path.
	PathUnique(PathBuf),

	#[error("Failed to write: {0}.")]
	/// Write failed.
	PathWrite(PathBuf),
}

impl fmt::Debug for Error {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self)
	}
}

/// Result wrapper.
pub type Result<T, E = Error> = std::result::Result<T, E>;
