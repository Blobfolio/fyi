/*!
# FYI Msg - Progless Error
*/

use std::{
	error::Error,
	fmt,
};



#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
/// # Obligatory error type.
pub enum ProglessError {
	/// # Length (total) must be non-zero.
	EmptyTotal,
	/// # Length (total) overflow.
	TotalOverflow,
}

impl AsRef<str> for ProglessError {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl fmt::Display for ProglessError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl Error for ProglessError {}

impl ProglessError {
	#[must_use]
	/// # As Str.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::EmptyTotal => "At least one task is required.",

			#[cfg(target_pointer_width = "16")]
			Self::TotalOverflow => "Progress can only be displayed for up to 65,535 items.",

			#[cfg(not(target_pointer_width = "16"))]
			Self::TotalOverflow => "Progress can only be displayed for up to 4,294,967,295 items.",
		}
	}
}
