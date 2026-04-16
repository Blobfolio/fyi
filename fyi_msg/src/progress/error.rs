/*!
# FYI Msg - Progless Error
*/

use std::{
	error::Error,
	fmt,
};



#[cfg_attr(docsrs, doc(cfg(feature = "progress")))]
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
		<str as fmt::Display>::fmt(self.as_str(), f)
	}
}

impl Error for ProglessError {}

impl ProglessError {
	#[must_use]
	#[inline]
	/// # As Str.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::EmptyTotal => "At least one task is required.",

			Self::TotalOverflow => concat!(
				"Progress can only be displayed for up to ",
				cfg_select! {
					target_pointer_width = "16" => "65,535",
					_ => "4,294,967,295",
				},
				" items.",
			),
		}
	}
}
