/*!
# FYI
*/

#![forbid(unsafe_code)]

#![warn(
	clippy::filetype_is_file,
	clippy::integer_division,
	clippy::needless_borrow,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::suboptimal_flops,
	clippy::unneeded_field_pattern,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]



use argyle::{
	Argue,
	ArgyleError,
	FLAG_DYNAMIC_HELP,
	FLAG_REQUIRED,
	FLAG_SUBCOMMAND,
	FLAG_VERSION,
};
use dactyl::traits::{
	BytesToSigned,
	BytesToUnsigned,
};
use fyi_msg::{
	Msg,
	MsgKind,
	FLAG_INDENT,
	FLAG_NEWLINE,
	FLAG_TIMESTAMP,
};



#[doc(hidden)]
/// # Main.
fn main() {
	// Handle errors.
	if let Err(e) = _main() {
		match e {
			ArgyleError::Passthru(_) => {},
			ArgyleError::WantsDynamicHelp(x) => {
				helper(x);
				return;
			},
			ArgyleError::WantsVersion => {
				println!(concat!("FYI v", env!("CARGO_PKG_VERSION")));
				return;
			},
			_ => {
				Msg::error(&e).eprint();
			},
		}

		std::process::exit(e.exit_code());
	}
}

#[doc(hidden)]
#[inline]
/// # Actual Main.
///
/// This lets us more easily bubble errors, which are printed and handled
/// specially.
fn _main() -> Result<(), ArgyleError> {
	// Parse CLI arguments.
	let args = Argue::new(
		FLAG_DYNAMIC_HELP | FLAG_REQUIRED | FLAG_SUBCOMMAND | FLAG_VERSION
	)?;

	match MsgKind::from(&args[0]) {
		MsgKind::Blank => {
			blank(&args);
			Ok(())
		},
		MsgKind::None => Err(ArgyleError::NoSubCmd),
		MsgKind::Confirm => confirm(&args),
		kind => msg(kind, &args),
	}
}

#[doc(hidden)]
#[cold]
/// # Shoot Blanks.
///
/// Print one or more blank lines to `STDOUT` or `STDERR`.
fn blank(args: &Argue) {
	// How many lines should we print?
	let msg = Msg::plain("\n".repeat(
		args.option2(b"-c", b"--count")
			.and_then(usize::btou)
			.map_or(1, |x| 1_usize.max(x))
	));

	// Print it to `Stderr`.
	if args.switch(b"--stderr") { msg.eprint(); }
	// Print it to `Stdout`.
	else { msg.print(); }
}

#[doc(hidden)]
/// # Confirmation.
///
/// This prompts a message and exits with `0` or `1` depending on the
/// positivity of the response.
fn confirm(args: &Argue) -> Result<(), ArgyleError> {
	let default = args.switch2(b"-y", b"--yes");
	if Msg::new(
		MsgKind::Confirm,
		args.arg(0)
			.and_then(|x| std::str::from_utf8(x).ok())
			.ok_or(ArgyleError::Empty)?
	)
		.with_flags(parse_flags(args))
		.prompt_with_default(default)
	{
		Ok(())
	}
	else {
		Err(ArgyleError::Passthru(1))
	}
}

#[doc(hidden)]
/// # Parse Flags.
///
/// Most subcommands support the same two flags — indentation and timestamping.
/// This parses those from the arguments, and adds the newline flag since all
/// message types need a trailing line break.
fn parse_flags(args: &Argue) -> u8 {
	let mut flags: u8 = FLAG_NEWLINE;
	if args.switch2(b"-i", b"--indent") { flags |= FLAG_INDENT; }
	if args.switch2(b"-t", b"--timestamp") { flags |= FLAG_TIMESTAMP; }
	flags
}

#[doc(hidden)]
/// # Basic Message.
///
/// This prints the message and exits accordingly.
fn msg(kind: MsgKind, args: &Argue) -> Result<(), ArgyleError> {
	// We need to discover the exit flag before forming the message as its
	// position could affect Argyle's understanding of where the trailing args
	// begin.
	let exit: Option<i32> = args.option2(b"-e", b"--exit")
		.and_then(i32::btoi);

	// Build the message.
	let msg: Msg =
		// Custom message prefix.
		if MsgKind::Custom == kind {
			Msg::custom(
				args.option2(b"-p", b"--prefix")
					.and_then(|x| std::str::from_utf8(x).ok())
					.unwrap_or_default(),
				args.option2(b"-c", b"--prefix-color")
					.and_then(u8::btou)
					.unwrap_or(199_u8),
				args.arg(0)
					.and_then(|x| std::str::from_utf8(x).ok())
					.ok_or(ArgyleError::Empty)?
			)
		}
		// Built-in prefix.
		else {
			Msg::new(
				kind,
				args.arg(0)
					.and_then(|x| std::str::from_utf8(x).ok())
					.ok_or(ArgyleError::Empty)?
			)
		}
		.with_flags(parse_flags(args));

	// Print to `STDERR`.
	if args.switch(b"--stderr") { msg.eprint(); }
	// Print to `STDOUT`.
	else { msg.print(); }

	// Special exit?
	exit.map_or(Ok(()), |e| Err(ArgyleError::Passthru(e)))
}

#[doc(hidden)]
#[cold]
/// # Help Page.
///
/// Print the appropriate help screen given the call details. Most of the sub-
/// commands work the same way, but a few have their own distinct messages.
///
/// The contents are generated via `build.rs`, which lowers the runtime cost
/// and shrinks the binary a touch.
fn helper(cmd: Option<Box<[u8]>>) {
	use std::io::Write;

	let writer = std::io::stdout();
	let mut handle = writer.lock();

	// The built-in message types have a variable part, and a static part.
	macro_rules! write_help {
		($path:literal) => {
			handle.write_all(include_bytes!(concat!(env!("OUT_DIR"), "/help-", $path, ".txt")))
		};
		($path:literal, true) => {
			write_help!($path).and_then(|()| write_help!("generic-bottom"))
		};
	}

	// The top is always the same.
	write_help!("top").unwrap();

	// The middle section varies by subcommand.
	if let Some(cmd) = cmd {
		match &*cmd {
			b"blank" => write_help!("blank"),
			b"confirm" | b"prompt" => write_help!("confirm"),
			b"crunched" => write_help!("crunched", true),
			b"debug" => write_help!("debug", true),
			b"done" => write_help!("done", true),
			b"error" => write_help!("error", true),
			b"info" => write_help!("info", true),
			b"notice" => write_help!("notice", true),
			b"print" => write_help!("print"),
			b"review" => write_help!("review", true),
			b"success" => write_help!("success", true),
			b"task" => write_help!("task", true),
			b"warning" => write_help!("warning", true),
			_ => write_help!("help"),
		}.unwrap();
	}
	else {
		write_help!("help").unwrap();
	}

	handle.flush().unwrap();
}
