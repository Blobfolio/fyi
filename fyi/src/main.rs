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
		"-h" | "--help" | "help" => _help(include_str!("../help/help.txt")),
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
					_help(include_str!("../help/print.txt"));
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
						 _help(&format!(
							include_str!("../help/generic.txt"),
							other.as_str(),
							other.as_str().to_lowercase(),
						));
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
	let len: usize = args.len();
	if 0 == len { 0 }
	else {
		let mut flags: u8 = 0;
		let mut del = 0;
		let raw = args.as_mut_vec();

		// This is basically what `Vec.retain()` does, except we're hitting
		// multiple patterns at once and sending back the results.
		let ptr = raw.as_mut_ptr();
		unsafe {
			let mut idx: usize = 0;
			while idx < len {
				match (*ptr.add(idx)).as_str() {
					"-i" | "--indent" => {
						flags |= FLAG_INDENT;
						del += 1;
					},
					"--stderr" => {
						flags |= FLAG_STDERR;
						del += 1;
					},
					"-t" | "--timestamp" => {
						flags |= FLAG_TIMESTAMP;
						del += 1;
					},
					"-h" | "--help" => {
						flags |= FLAG_HELP;
						del += 1;
					},
					_ => if del > 0 {
						ptr.add(idx).swap(ptr.add(idx - del));
					}
				}

				idx += 1;
			}
		}

		// Did we find anything? If so, run `truncate()` to free the memory
		// and return the flags.
		if del > 0 {
			raw.truncate(len - del);
			flags
		}
		else { 0 }
	}
}

/// Shoot Blanks.
fn _blank(args: &mut ArgList) {
	if args.pluck_help() {
		_help(include_str!("../help/blank.txt"));
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
		_help(include_str!("../help/confirm.txt"));
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
fn _help(txt: &str) {
	io::stdout().write_all({
		let mut s = String::with_capacity(1024);
		s.push_str("FYI ");
		s.push_str(env!("CARGO_PKG_VERSION"));
		s.push('\n');
		s.push_str(env!("CARGO_PKG_DESCRIPTION"));
		s.push('\n');
		s.push('\n');
		s.push_str(txt);
		s.push('\n');
		s
	}.as_bytes()).unwrap();
}

#[cold]
/// Print version and exit.
fn _version() {
	io::stdout().write_all({
		let mut s = String::from("FYI ");
		s.push_str(env!("CARGO_PKG_VERSION"));
		s.push('\n');
		s
	}.as_bytes()).unwrap();
}
