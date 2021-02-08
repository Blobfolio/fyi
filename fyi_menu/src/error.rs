/*!
# FYI Menu: Errors
*/

use std::{
	error::Error,
	fmt,
};



#[derive(Debug, Clone)]
/// # Error Struct.
pub enum ArgueError {
	/// Missing anything.
	Empty,
	/// No trailing args.
	NoArg,
	/// Expected subcommand.
	NoSubCmd,
	/// Other.
	Other(&'static str),
	/// Miscellaneous Silent Failure.
	///
	/// This has no corresponding error text, but does have its own exit code.
	Passthru(i32),
	/// Too many options defined.
	TooManyKeys,
	/// Wants subcommand help.
	WantsDynamicHelp(Option<Vec<u8>>),
	/// Wants help.
	WantsHelp,
	/// Wants version.
	WantsVersion,
}

impl AsRef<str> for ArgueError {
	fn as_ref(&self) -> &str {
		match self {
			Self::Empty => "Missing options, flags, arguments, and/or ketchup.",
			Self::NoArg => "Missing required trailing argument.",
			Self::NoSubCmd => "Missing/invalid subcommand.",
			Self::Other(s) => s,
			Self::Passthru(_)
				| Self::WantsDynamicHelp(_)
				| Self::WantsHelp
				| Self::WantsVersion => "",
			Self::TooManyKeys => "Too many keys.",
		}
	}
}

impl Error for ArgueError {}

impl fmt::Display for ArgueError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}

impl ArgueError {
	#[must_use]
	/// # Exit code.
	pub const fn exit_code(&self) -> i32 {
		match self {
			Self::Passthru(c) => *c,
			Self::WantsDynamicHelp(_)
				| Self::WantsHelp
				| Self::WantsVersion => 0,
			_ => 1,
		}
	}
}
