/*!
# FYI: CLI
*/

use argyle::{
	Argue,
	Argument,
};
use crate::FyiError;
use dactyl::traits::{
	BytesToSigned,
	BytesToUnsigned,
};
use fyi_msg::{
	Msg,
	MsgKind,
};
use std::num::NonZeroUsize;



#[derive(Debug, Clone, Copy)]
/// # Message Settings.
///
/// Most of these settings will get embedded into the `Msg` itself, but there
/// are a couple parts that do not get referenced until print time.
pub(super) struct Settings {
	/// # Flags.
	flags: u8,

	/// # Exit.
	exit: i32,
}

impl Settings {
	/// # Indent Message.
	const FLAG_INDENT: u8 =    0b0001;

	/// # Print to STDERR.
	const FLAG_STDERR: u8 =    0b0010;

	/// # Include Timestamp.
	const FLAG_TIMESTAMP: u8 = 0b0100;

	/// # Default Yes (for Prompt).
	const FLAG_YES: u8 =       0b1000;

	/// # Exit Code.
	pub(super) const fn exit(self) -> Result<(), FyiError> {
		if self.exit == 0 { Ok(()) }
		else { Err(FyiError::Passthrough(self.exit)) }
	}

	/// # Stderr?
	pub(super) const fn stderr(self) -> bool {
		Self::FLAG_STDERR == self.flags & Self::FLAG_STDERR
	}

	/// # Default Yes?
	pub(super) const fn yes(self) -> bool {
		Self::FLAG_YES == self.flags & Self::FLAG_YES
	}

	/// # Convert to `Msg` Flags.
	const fn msg_flags(self) -> u8 {
		let mut flags: u8 = fyi_msg::FLAG_NEWLINE;
		if Self::FLAG_INDENT == self.flags & Self::FLAG_INDENT {
			flags |= fyi_msg::FLAG_INDENT;
		}
		if Self::FLAG_TIMESTAMP == self.flags & Self::FLAG_TIMESTAMP {
			flags |= fyi_msg::FLAG_TIMESTAMP;
		}
		flags
	}

	/// # New.
	const fn new() -> Self {
		Self { flags: 0, exit: 0 }
	}

	/// # Set Indent.
	fn set_indent(&mut self) { self.flags |= Self::FLAG_INDENT; }

	/// # Set Stderr.
	fn set_stderr(&mut self) { self.flags |= Self::FLAG_STDERR; }

	/// # Set Timestamp.
	fn set_timestamp(&mut self) { self.flags |= Self::FLAG_TIMESTAMP; }

	/// # Set Yes.
	fn set_yes(&mut self) { self.flags |= Self::FLAG_YES; }
}



/// # Parse Message Kind.
pub(super) fn parse_kind() -> Result<MsgKind, FyiError> {
	let mut args = argyle::args().with_keywords(
		include!(concat!(env!("OUT_DIR"), "/argyle-kind.rs"))
	);

	// The first result must be a subcommand or help/version flag.
	let kind = match args.next() {
		Some(Argument::Key("-V" | "--version")) => return Err(FyiError::PrintVersion),
		Some(Argument::Command(s)) => MsgKind::from(s.as_bytes()),
		_ => return Err(FyiError::PrintHelp(MsgKind::None)),
	};

	// Force the help screen if no kind was parsed.
	if matches!(kind, MsgKind::None) { Err(FyiError::PrintHelp(MsgKind::None)) }
	// Otherwise return it!
	else { Ok(kind) }
}

/// # Parse and Print Blanks!
pub(super) fn parse_blank() -> Result<(), FyiError> {
	// The first arg is always skipped, the second we read earlier.
	let args = Argue::from(std::env::args_os().skip(2))
		.with_keywords(include!(concat!(env!("OUT_DIR"), "/argyle-blank.rs")));

	let mut stderr = false;
	let mut count = NonZeroUsize::MIN;
	for arg in args {
		match arg {
			Argument::Key("-h" | "--help") => return Err(FyiError::PrintHelp(MsgKind::Blank)),
			Argument::Key("--stderr") => { stderr = true; },
			Argument::KeyWithValue("-c" | "--count", s) =>
				if let Some(s) = NonZeroUsize::btou(s.trim().as_bytes()) {
					count = NonZeroUsize::max(count, s);
				},

			// Nothing else is relevant here.
			_ => {},
		}
	}

	// Print it!
	let lines = "\n".repeat(count.get());
	if stderr { eprint!("{lines}"); }
	else { print!("{lines}"); }

	Ok(())
}

/// # Parse Message.
pub(super) fn parse_msg(kind: MsgKind) -> Result<(Msg, Settings), FyiError> {
	// The first arg is always skipped, the second we read earlier.
	let args = Argue::from(std::env::args_os().skip(2))
		.with_keywords(include!(concat!(env!("OUT_DIR"), "/argyle-msg.rs")));

	let mut msg = None;
	let mut prefix = String::new();
	let mut color = 199_u8;
	let mut flags = Settings::new();
	for arg in args {
		match arg {
			Argument::Key("-h" | "--help") => return Err(FyiError::PrintHelp(kind)),
			Argument::Key("-i" | "--indent") => { flags.set_indent(); },
			Argument::Key("--stderr") => { flags.set_stderr(); },
			Argument::Key("-t" | "--timestamp") => { flags.set_timestamp(); },
			Argument::Key("-y" | "--yes") => { flags.set_yes(); },

			Argument::KeyWithValue("-c" | "--prefix-color", s) =>
				if let Some(s) = u8::btou(s.trim().as_bytes()) { color = s; },
			Argument::KeyWithValue("-p" | "--prefix", s) => { prefix = s; },
			Argument::KeyWithValue("-e" | "--exit", s) =>
				if let Some(s) = i32::btoi(s.trim().as_bytes()) { flags.exit = s; },

			Argument::Other(s) =>
				if msg.is_none() { msg.replace(s); }
				else { return Err(FyiError::InvalidCli(kind)); },

			Argument::End(_) => {},
			_ => return Err(FyiError::InvalidCli(kind)),
		}
	}

	let msg = msg.ok_or(FyiError::NoMessage)?;
	let msg =
		if matches!(kind, MsgKind::Custom) { Msg::custom(prefix, color, msg) }
		else { Msg::new(kind, msg) }
			.with_flags(flags.msg_flags());

	Ok((msg, flags))
}
