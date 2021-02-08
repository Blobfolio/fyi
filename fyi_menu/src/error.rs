/*!
# FYI Menu: Errors
*/

use std::{
	error::Error,
	fmt,
};



#[derive(Debug, Clone, Copy)]
/// # Error Struct.
pub enum ArgueError {
	/// Missing anything.
	Empty,
	/// No trailing args.
	NoArg,
	/// Other.
	NoSubCmd,
	/// Miscellaneous Silent Failure.
	///
	/// This has no corresponding error text, but does have its own exit code.
	Passthru(i32),
	/// Too many options defined.
	TooManyKeys,
}

impl AsRef<str> for ArgueError {
	fn as_ref(&self) -> &str {
		match self {
			Self::Empty => "Missing options, flags, arguments, and/or ketchup.",
			Self::NoArg => "Missing required trailing argument.",
			Self::NoSubCmd => "Missing/invalid subcommand.",
			Self::Passthru(_) => "",
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
	pub const fn exit_code(self) -> i32 {
		match self {
			Self::Passthru(c) => c,
			_ => 1,
		}
	}
}
