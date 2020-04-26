/*!
# FYI Core: Obligatory Error Type
*/

use std::{
	error,
	fmt,
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

impl<X> From<X> for Error
where X: Into<String> {
	/// Do it.
	fn from(thing: X) -> Error {
		Error(thing.into())
	}
}

impl Error {
	/// New.
	pub fn new<T> (msg: T) -> Error
	where T: AsRef<str> {
		Error(msg.as_ref().to_string())
	}

	/// Default.
	pub fn default() -> Error {
		Error("Boo.".to_string())
	}
}

impl error::Error for Error {}

/// Result wrapper.
pub type Result<T, E = Error> = std::result::Result<T, E>;
