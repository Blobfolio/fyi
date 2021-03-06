/*!
# FYI

FYI is a simple CLI tool for x86-64 Linux machines that prints an arbitrary
status-style message, optionally indented, timestamped, and/or prefixed.
You know, stuff like:

* **Error:** Something broke!
* **Warning:** I can't keep doing this!
* **Success:** Life is good!

That's it!



## Why?

The main idea is to make it easier to communicate successes and errors from
within shell scripts, particularly in regards to color. ANSI formatting isn't
difficult, but all that extra code is difficult to read.

Take for example:
```bash
# The manual way.
echo "\033[1;91mError:\033[0m Something broke!"

# Using FYI:
fyi error "Something broke!"
```



## Installation

This application is written in [Rust](https://www.rust-lang.org/) and can be installed using [Cargo](https://github.com/rust-lang/cargo).

For stable Rust (>= `1.51.0`), run:
```bash
RUSTFLAGS="-C link-arg=-s" cargo install \
    --git https://github.com/Blobfolio/fyi.git \
    --bin fyi \
    --target x86_64-unknown-linux-gnu
```

Pre-built `.deb` packages are also added for each [release](https://github.com/Blobfolio/fyi/releases/latest). They should always work for the latest stable Debian and Ubuntu.



## Usage

The primary usage is to generate a message with one of the default prefixes,
like: `fyi <PREFIX> [flags] [options] <MSG>`, where the prefix is one of:
* <span style="color: #27ae60">crunched</span>
* <span style="color: #5dade2">debug</span>
* <span style="color: #27ae60">done</span>
* <span style="color: #ff0000">error</span>
* <span style="color: #9b59b6">info</span>
* <span style="color: #9b59b6">notice </span>
* <span style="color: #27ae60">success</span>
* <span style="color: #ff1493">task</span>
* <span style="color: #f1c40f">warning</span>

The following flags and options are available.
```bash
-e, --exit <num>   Exit with this status code after printing. [default: 0]
-h, --help         Print this screen.
-i, --indent       Indent the line.
    --stderr       Print to STDERR instead of STDOUT.
-t, --timestamp    Include a timestamp.
```

### Custom Prefix:

To use a custom prefix (or no prefix), run `fyi print [flags] [options] <MSG>`,
using the following additional options:
```bash
-p, --prefix <txt>          Set a custom prefix. [default: ]
-c, --prefix-color <num>    Prefix color. [default: 199]
```

The color should be a `u8` corresponding to a [BASH color number](https://misc.flogisoft.com/bash/tip_colors_and_formatting#colors1).
Note: to avoid the cost of re-alignment, only values in the range of `1..=255` are supported.

### Confirmation Prompt:

To prompt a user for a Y/N (and exit with a corresponding status code), run
`fyi confirm [flags] [options] <MSG>`. Confirmation supports the same flags as
the other built-in prefixes.

### Blank Lines:

And finally, there is a convenient `blank` subcommand that does nothing but
print a certain number of blank lines for you. Run
`fyi blank [flags] [options]`, which supports the following:
```bash
-h, --help           Print this screen.
    --stderr         Print to STDERR instead of STDOUT.
-c, --count <num>    Number of empty lines to print. [default: 1]
```
*/

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]



use argyle::{
	Argue,
	ArgyleError,
	FLAG_DYNAMIC_HELP,
	FLAG_REQUIRED,
	FLAG_SUBCOMMAND,
	FLAG_VERSION,
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

	match MsgKind::from(unsafe { args.peek_unchecked() }) {
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
			.and_then(|x| std::str::from_utf8(x).ok())
			.and_then(|x| x.parse::<usize>().ok())
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
	if Msg::new(
		MsgKind::Confirm,
		std::str::from_utf8(args.first_arg()?).map_err(|_| ArgyleError::NoArg)?
	)
		.with_flags(parse_flags(args))
		.prompt()
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
		.and_then(|x| std::str::from_utf8(x).ok())
		.and_then(|x| x.parse::<i32>().ok());

	// Build the message.
	let msg: Msg =
		// Custom message prefix.
		if MsgKind::Custom == kind {
			Msg::custom(
				args.option2(b"-p", b"--prefix")
					.and_then(|x| std::str::from_utf8(x).ok())
					.unwrap_or_default(),
				args.option2(b"-c", b"--prefix-color")
					.and_then(|x| std::str::from_utf8(x).ok())
					.and_then(|x| x.parse::<u8>().ok())
					.unwrap_or(199_u8),
				std::str::from_utf8(args.first_arg()?)
					.map_err(|_| ArgyleError::NoArg)?
			)
		}
		// Built-in prefix.
		else {
			Msg::new(
				kind,
				std::str::from_utf8(args.first_arg()?)
					.map_err(|_| ArgyleError::NoArg)?
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
			write_help!($path).and_then(|_| write_help!("generic-bottom"))
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
