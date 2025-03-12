/*!
# FYI: CLI
*/

use argyle::{
	Argue,
	Argument,
};
use crate::FyiError;
use dactyl::traits::BytesToUnsigned;
use fyi_msg::{
	Msg,
	MsgKind,
};
use std::{
	num::NonZeroUsize,
	process::ExitCode,
};



// Flags generated by build.rs.
include!(concat!(env!("OUT_DIR"), "/flags.rs"));



#[derive(Debug, Clone, Copy)]
/// # Message Settings.
///
/// Most of these settings will get embedded into the `Msg` itself, but there
/// are a couple parts that do not get referenced until print time.
pub(super) struct Settings {
	/// # Flags.
	flags: Flags,

	/// # Exit.
	exit: ExitCode,
}

impl Settings {
	/// # Exit Code.
	pub(super) fn exit(self) -> Result<(), FyiError> {
		if self.exit == ExitCode::SUCCESS { Ok(()) }
		else { Err(FyiError::Passthrough(self.exit)) }
	}

	/// # Stderr?
	pub(super) const fn stderr(self) -> bool { self.flags.contains(Flags::Stderr) }

	/// # Default Yes?
	pub(super) const fn yes(self) -> bool { self.flags.contains(Flags::Yes) }

	/// # Convert to `Msg` Flags.
	const fn msg_flags(self) -> u8 {
		let mut flags: u8 = fyi_msg::FLAG_NEWLINE;
		if self.flags.contains(Flags::Indent) {
			flags |= fyi_msg::FLAG_INDENT;
		}
		if self.flags.contains(Flags::Timestamp) {
			flags |= fyi_msg::FLAG_TIMESTAMP;
		}
		flags
	}

	/// # New.
	const fn new() -> Self {
		Self { flags: Flags::None, exit: ExitCode::SUCCESS }
	}

	/// # Set Indent.
	fn set_indent(&mut self) { self.flags |= Flags::Indent; }

	/// # Set Stderr.
	fn set_stderr(&mut self) { self.flags |= Flags::Stderr; }

	/// # Set Timestamp.
	fn set_timestamp(&mut self) { self.flags |= Flags::Timestamp; }

	/// # Set Yes.
	fn set_yes(&mut self) { self.flags |= Flags::Yes; }
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
				if let Some(s) = u8::btou(s.trim().as_bytes()) { flags.exit = s.into(); },

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
