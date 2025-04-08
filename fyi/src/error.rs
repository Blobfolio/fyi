/*!
# FYI: Errors
*/

use fyi_msg::MsgKind;
use std::{
	fmt,
	process::ExitCode,
};



#[derive(Debug, Clone, Copy, PartialEq)]
/// # Errors!
pub(super) enum FyiError {
	/// # Unrecognized CLI.
	InvalidCli(MsgKind),

	/// # No Message.
	NoMessage,

	/// # Passthrough.
	Passthrough(ExitCode),

	/// # Print Help (Not an Error).
	PrintHelp(MsgKind),

	/// # Print Version (Not an Error).
	PrintVersion,
}

impl std::error::Error for FyiError {}

impl fmt::Display for FyiError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Self::InvalidCli(s) = self {
			write!(
				f,
				concat!(
					"Invalid CLI argument(s); run ",
					fyi_ansi::dim!("fyi {} --help"),
					" for usage.",
				),
				s.command(),
			)
		}
		else { f.write_str(self.as_str()) }
	}
}

impl FyiError {
	/// # As String Slice.
	pub(super) const fn as_str(self) -> &'static str {
		match self {
			Self::InvalidCli(_) => "Invalid CLI argument(s).",
			Self::NoMessage => "Missing message.",
			Self::Passthrough(_) | Self::PrintHelp(_) => "",
			Self::PrintVersion => concat!("FYI v", env!("CARGO_PKG_VERSION")),
		}
	}
}
