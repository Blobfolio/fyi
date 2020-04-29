/*!
# FYI Core: Obligatory Error Type
*/

use std::{
	error,
	fmt,
	io,
};


#[derive(Debug, Clone, PartialEq)]
/// Error!
pub struct Error(String);

impl fmt::Display for Error {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl From<String> for Error {
	/// Do it.
	fn from(thing: String) -> Error {
		Error(thing)
	}
}

impl From<io::Error> for Error {
	/// Do it.
	fn from(thing: io::Error) -> Error {
		Error(format!("{}", thing))
	}
}

impl Error {
	/// New.
	pub fn new<T> (msg: T) -> Error
	where T: AsRef<str> {
		Error(msg.as_ref().to_string())
	}
}

impl Default for Error {
	/// Default.
	fn default() -> Error {
		Error("Boo.".to_string())
	}
}

impl error::Error for Error {}

/// Result wrapper.
pub type Result<T, E = Error> = std::result::Result<T, E>;
