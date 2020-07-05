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
use fyi_msg::Msg;
use std::{
	io::{
		self,
		Write,
	},
	process,
};



/// Short Circuit: If a help switch is included on a subcommand, drop
/// everything and show the info.
macro_rules! get_help {
	($com:expr, $opts:ident) => {
		if $opts.pluck_help() { return _help(Some($com)); }
	};
}



fn main() {
	let mut args = ArgList::default();
	args.expect();

	// The app might be called with version or help flags instead of a command.
	match args.peek().unwrap() {
		"-V" | "--version" => _version(),
		"-h" | "--help" | "help" => _help(None),
		// Otherwise just go off into the appropriate subcommand action.
		_ => match args.expect_command().as_str() {
			"blank" => _blank(&mut args),
			"confirm" | "prompt" => _confirm(&mut args),
			"print" => _custom(&mut args),
			other => _builtin(other, &mut args),
		},
	}
}

#[allow(clippy::ptr_arg)]
/// Match: Exit Code
fn match_exit(txt: &String) -> bool {
	txt == "-e" || txt == "--exit"
}

#[allow(clippy::ptr_arg)]
/// Match: Indentation
fn match_indent(txt: &String) -> bool {
	txt != "-i" && txt != "--indent"
}

#[allow(clippy::ptr_arg)]
/// Match: Stderr
fn match_stderr(txt: &String) -> bool {
	txt != "--stderr"
}

#[allow(clippy::ptr_arg)]
/// Match: Timestamp
fn match_timestamp(txt: &String) -> bool {
	txt != "-t" && txt != "--timestamp"
}

#[cold]
/// Handle Blank
fn _blank(opts: &mut ArgList) {
	get_help!("blank", opts);

	let count: usize = match opts.pluck_opt_usize(|x| x == "-c" || x == "--count") {
		Some(c) => usize::min(10, usize::max(1, c)),
		None => 1,
	};

	if opts.pluck_switch(match_stderr) {
		io::stderr().write_all(&[10].repeat(count)).unwrap();
	}
	else {
		io::stdout().write_all(&[10].repeat(count)).unwrap();
	}
}

/// Handle Built-In Prefix.
fn _builtin(com: &str, opts: &mut ArgList) {
	get_help!(com, opts);

	// Pull switches.
	let indent = opts.pluck_switch(match_indent);
	let timestamp = opts.pluck_switch(match_timestamp);
	let stderr = opts.pluck_switch(match_stderr);

	// Exit is an option.
	let exit: u8 = opts.pluck_opt_usize(match_exit).unwrap_or(0) as u8;

	// And finally the message bit!
	_msg(
		match com {
			"crunched" => Msg::crunched(opts.expect_arg()),
			"debug" => Msg::debug(opts.expect_arg()),
			"done" => Msg::done(opts.expect_arg()),
			"error" => Msg::error(opts.expect_arg()),
			"info" => Msg::info(opts.expect_arg()),
			"notice" => Msg::notice(opts.expect_arg()),
			"success" => Msg::success(opts.expect_arg()),
			"task" => Msg::task(opts.expect_arg()),
			"warning" => Msg::warning(opts.expect_arg()),
			_ => {
				ArgList::die("Invalid subcommand.");
				unreachable!();
			},
		},
		indent,
		timestamp,
		stderr,
		i32::from(exit)
	);
}

/// Handle Confirmation.
fn _confirm(opts: &mut ArgList) {
	get_help!("confirm", opts);

	let indent = opts.pluck_switch(match_indent);
	let timestamp = opts.pluck_switch(match_timestamp);

	let mut msg = Msg::confirm(opts.expect_arg());

	if indent {
		msg.set_indent(1);
	}

	if timestamp {
		msg.set_timestamp();
	}

	if ! msg.prompt() {
		process::exit(1);
	}
}

/// Custom Prefix.
fn _custom(opts: &mut ArgList) {
	get_help!("print", opts);

	// Pull switches.
	let indent = opts.pluck_switch(match_indent);
	let timestamp = opts.pluck_switch(match_timestamp);
	let stderr = opts.pluck_switch(match_stderr);

	// Pull the options.
	let exit: u8 = opts.pluck_opt_usize(match_exit).unwrap_or(0) as u8;
	let color: u8 = usize::min(255, opts.pluck_opt_usize(|x| x == "-c" || x == "--prefix-color").unwrap_or(199)) as u8;
	let prefix = opts.pluck_opt(|x| x == "-p" || x == "--prefix").unwrap_or_default();

	// And finally the message bit!
	_msg(
		Msg::new(prefix, color, opts.expect_arg()),
		indent,
		timestamp,
		stderr,
		i32::from(exit)
	)
}

#[cold]
/// Print help and exit.
fn _help(com: Option<&str>) {
	match com.unwrap_or_default() {
		"blank" => _helpful(include_str!("../help/blank.txt")),
		"confirm" | "prompt" => _helpful(include_str!("../help/confirm.txt")),
		"crunched" => _help_generic("crunched", "Crunched"),
		"debug" => _help_generic("debug", "Debug"),
		"done" => _help_generic("done", "Done"),
		"error" => _help_generic("error", "Error"),
		"info" => _help_generic("info", "Info"),
		"notice" => _help_generic("notice", "Notice"),
		"print" => _helpful(include_str!("../help/print.txt")),
		"success" => _help_generic("success", "Success"),
		"task" => _help_generic("task", "Task"),
		"warning" => _help_generic("warning", "Warning"),
		_ => _helpful(include_str!("../help/help.txt"))
	}
}

#[cold]
/// Generic Subcommand Help
///
/// Most of the built-ins work exactly the same way.
fn _help_generic(com: &str, name: &str) {
	_helpful(&format!(include_str!("../help/generic.txt"), name, com));
}

#[cold]
/// Print full help.
fn _helpful(help: &str) {
	io::stdout().write_all(format!(
		"FYI {}\n{}\n\n{}\n",
		env!("CARGO_PKG_VERSION"),
		env!("CARGO_PKG_DESCRIPTION"),
		help,
	).as_bytes()).unwrap();
}

/// Print Message.
fn _msg(mut msg: Msg, indent: bool, timestamp: bool, stderr: bool, exit: i32) {
	if indent {
		msg.set_indent(1);
	}

	if timestamp {
		msg.set_timestamp();
	}

	// Print it to `Stderr`.
	if stderr {
		msg.eprintln();
	}
	// Print it to `Stdout`.
	else {
		msg.println();
	}

	// We might have a custom exit code.
	if 0 != exit {
		process::exit(exit);
	}
}

#[cold]
/// Print version and exit.
fn _version() {
	io::stdout().write_all(["FYI", env!("CARGO_PKG_VERSION")].join(" ").as_bytes()).unwrap();
}
