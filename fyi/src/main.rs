/*!
# FYI
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use fyi_menu::ArgList;
use fyi_msg::{
	Msg,
	MsgKind,
};
use std::{
	io::{
		self,
		Write,
	},
	process,
};



/// -h | --help
const FLAG_HELP: u8      = 0b0001;
/// -i | --indent
const FLAG_INDENT: u8    = 0b0010;
/// --stderr
const FLAG_STDERR: u8    = 0b0100;
/// -t | --timestamp
const FLAG_TIMESTAMP: u8 = 0b1000;



fn main() {
	let mut args = ArgList::default();
	args.expect();

	// The app might be called with version or help flags instead of a command.
	match args.peek().unwrap() {
		"-V" | "--version" => _version(),
		"-h" | "--help" | "help" => _help(include_bytes!("../help/help.txt")),
		// Otherwise just go off into the appropriate subcommand action.
		_ => match args.expect_command().as_str() {
			"blank" => _blank(&mut args),
			"confirm" | "prompt" => _confirm(&mut args),
			"print" => {
				let flags = _flags(&mut args);
				if 0 == flags & FLAG_HELP {
					let exit: i32 = _exit(&mut args);
					let color: u8 = 255.min(args.pluck_opt_usize(|x| x == "-c" || x == "--prefix-color")
						.unwrap_or(199)) as u8;
					let prefix = args.pluck_opt(|x| x == "-p" || x == "--prefix")
						.unwrap_or_default();

					_msg(Msg::new(prefix, color, args.expect_arg()), flags, exit);
				}
				// Show help instead.
				else {
					_help(include_bytes!("../help/print.txt"));
				}
			},
			other => match MsgKind::from(other) {
				MsgKind::None => ArgList::die("Invalid subcommand."),
				other => {
					let flags = _flags(&mut args);
					if 0 == flags & FLAG_HELP {
						let exit: i32 = _exit(&mut args);
						_msg(other.as_msg(args.expect_arg()), flags, exit);
					}
					// Show help instead.
					else {
						 _help(format!(
							include_str!("../help/generic.txt"),
							other.as_str(),
							other.as_str().to_lowercase(),
						).as_bytes());
					}
				}
			}
		}
	}
}

/// Fetch Exit Code.
///
/// Many of the subcommands accept an optional alternative exit status. This
/// fetches it in a centralized way.
fn _exit(args: &mut ArgList) -> i32 {
	match args.pluck_opt(|x| x == "-e" || x == "--exit") {
		Some(x) => x.parse::<i32>().unwrap_or_default(),
		None => 0,
	}
}

/// Fetch Common Flags.
///
/// Most subcommands accept the same set of flags to control help, indentation,
/// timestamp, and destination. This looks for and crunches all of those in one
/// go to reduce the number of iterations that would be required to check each
/// individually.
fn _flags(args: &mut ArgList) -> u8 {
	let mut flags: u8 = 0;
	args.pluck_flags(
		&mut flags,
		&[
			"-i", "--indent",
			"-t", "--timestamp",
			"--stderr",
			"-h", "--help",
		],
		&[
			FLAG_INDENT, FLAG_INDENT,
			FLAG_TIMESTAMP, FLAG_TIMESTAMP,
			FLAG_STDERR,
			FLAG_HELP, FLAG_HELP,
		],
	);
	flags
}

/// Shoot Blanks.
fn _blank(args: &mut ArgList) {
	if args.pluck_help() {
		_help(include_bytes!("../help/blank.txt"));
		return;
	}

	// How many lines should we print?
	let count: usize = match args.pluck_opt_usize(|x| x == "-c" || x == "--count") {
		Some(c) => 100.min(1.max(c)),
		None => 1,
	};

	// Print to `STDERR` instead of `STDOUT`.
	if args.pluck_switch(|x| x != "--stderr") {
		io::stderr().write_all(&[10].repeat(count)).unwrap();
	}
	else {
		io::stdout().write_all(&[10].repeat(count)).unwrap();
	}
}

/// Pop a Confirmation Prompt.
fn _confirm(args: &mut ArgList) {
	let flags: u8 = _flags(args);
	if 0 == flags & FLAG_HELP {
		let mut msg = MsgKind::Confirm.as_msg(args.expect_arg());

		if 0 != flags & FLAG_INDENT {
			msg.set_indent(1);
		}

		if 0 != flags & FLAG_TIMESTAMP {
			msg.set_timestamp();
		}

		if ! msg.prompt() {
			process::exit(1);
		}
	}
	// Show help instead.
	else {
		_help(include_bytes!("../help/confirm.txt"));
	}
}

/// Print Regular Message.
fn _msg(mut msg: Msg, flags: u8, exit: i32) {
	if 0 != flags & FLAG_INDENT {
		msg.set_indent(1);
	}

	if 0 != flags & FLAG_TIMESTAMP {
		msg.set_timestamp();
	}

	// Print it to `Stdout`.
	if 0 == flags & FLAG_STDERR { msg.println(); }
	// Print it to `Stderr`.
	else { msg.eprintln(); }

	// We might have a custom exit code.
	if 0 != exit {
		process::exit(exit);
	}
}

#[cold]
/// Print Help.
fn _help(txt: &[u8]) {
	io::stdout().write_all(&[
		b"FYI ",
		env!("CARGO_PKG_VERSION").as_bytes(),
		b"\n",
		env!("CARGO_PKG_DESCRIPTION").as_bytes(),
		b"\n\n",
		txt,
		b"\n",
	].concat()).unwrap();
}

#[cold]
/// Print version and exit.
fn _version() {
	io::stdout().write_all(&[
		b"FYI ",
		env!("CARGO_PKG_VERSION").as_bytes(),
		b"\n"
	].concat()).unwrap();
}
