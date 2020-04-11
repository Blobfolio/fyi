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

	#[error("Invalid path: {1} {0}.")]
	/// Expecting an absolute value.
	InvalidPath(&'static str, PathBuf),

	#[error("Operation failed: {0} on {1}.")]
	/// Expecting an absolute value.
	PathFailed(&'static str, PathBuf),
}

impl fmt::Debug for Error {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self)
	}
}

/// Result wrapper.
pub type Result<T, E = Error> = std::result::Result<T, E>;
