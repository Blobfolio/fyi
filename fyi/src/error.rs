/*!
# FYI: Errors
*/

use fyi_msg::MsgKind;
use std::fmt;



#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # Errors!
pub(super) enum FyiError {
	/// # Unrecognized CLI.
	InvalidCli(MsgKind),

	/// # No Message.
	NoMessage,

	/// # Passthrough.
	Passthrough(i32),

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
				"Invalid CLI argument(s); run \x1b[2mfyi {} --help\x1b[0m for usage.",
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

	/// # Exit Code.
	pub(super) const fn exit_code(self) -> i32 {
		match self {
			Self::Passthrough(e) => e,
			Self::PrintHelp(_) | Self::PrintVersion => 0,
			_ => 1,
		}
	}
}