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
use std::process;



/// Short Circuit: If a help switch is included on a subcommand, drop
/// everything and show the info.
macro_rules! get_help {
	($com:expr, $opts:ident) => {
		if $opts.extract_switch(&["-h", "--help"]) {
			return _help(Some($com));
		}
	};
}



fn main() {
	let mut args = ArgList::default();
	args.expect_any();

	// The app might be called with version or help flags instead of a command.
	match args.peek_first().unwrap() {
		"-V" | "--version" => _version(),
		"-h" | "--help" | "help" => _help(None),
		// Otherwise just go off into the appropriate subcommand action.
		_ => match args.expect_command().into_owned().as_str() {
			"blank" => _blank(&mut args),
			"confirm" | "prompt" => _confirm(&mut args),
			"print" => _custom(&mut args),
			other => _builtin(other, &mut args),
		},
	}
}

/// Handle Blank
fn _blank(opts: &mut ArgList) {
	get_help!("blank", opts);

	let count: usize = match opts.extract_opt_usize(&["-c", "--count"]) {
		Some(c) => usize::min(10, usize::max(1, c)),
		None => 1,
	};

	if opts.extract_switch(&["--stderr"]) {
		eprint!("{}", "\n".repeat(count));
	}
	else {
		print!("{}", "\n".repeat(count));
	}
}

/// Handle Built-In Prefix.
fn _builtin(com: &str, opts: &mut ArgList) {
	get_help!(com, opts);

	// Pull switches.
	let indent = opts.extract_switch(&["-i", "--indent"]);
	let timestamp = opts.extract_switch(&["-t", "--timestamp"]);
	let stderr = opts.extract_switch(&["--stderr"]);

	// Exit is an option.
	let exit: u8 = opts.extract_opt_usize(&["-e", "--exit"]).unwrap_or(0) as u8;

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

	let indent = opts.extract_switch(&["-i", "--indent"]);
	let timestamp = opts.extract_switch(&["-t", "--timestamp"]);

	let mut msg = Msg::confirm(opts.expect_arg());

	if indent {
		msg.set_indent(1);
	}

	if timestamp {
		msg.set_timestamp();
	}

	if ! casual::confirm(msg) {
		process::exit(1);
	}
}

/// Custom Prefix.
fn _custom(opts: &mut ArgList) {
	get_help!("print", opts);

	// Pull switches.
	let indent = opts.extract_switch(&["-i", "--indent"]);
	let timestamp = opts.extract_switch(&["-t", "--timestamp"]);
	let stderr = opts.extract_switch(&["--stderr"]);

	// Pull the options.
	let exit: u8 = opts.extract_opt_usize(&["-e", "--exit"]).unwrap_or(0) as u8;
	let color: u8 = usize::min(255, opts.extract_opt_usize(&["-c", "--prefix-color"]).unwrap_or(199)) as u8;
	let prefix = opts.extract_opt(&["-p", "--prefix"]).unwrap_or_default().into_owned();

	// And finally the message bit!
	_msg(
		Msg::new(prefix, color, opts.expect_arg()),
		indent,
		timestamp,
		stderr,
		i32::from(exit)
	)
}

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

/// Generic Subcommand Help
///
/// Most of the built-ins work exactly the same way.
fn _help_generic(com: &str, name: &str) {
	_helpful(&format!(include_str!("../help/generic.txt"), name, com));
}

/// Print full help.
fn _helpful(help: &str) {
	println!(
		"FYI {}\n{}\n\n{}",
		env!("CARGO_PKG_VERSION"),
		env!("CARGO_PKG_DESCRIPTION"),
		help,
	);
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
		eprintln!("{}", &msg);
	}
	// Print it to `Stdout`.
	else {
		println!("{}", &msg);
	}

	// We might have a custom exit code.
	if 0 != exit {
		process::exit(exit);
	}
}

/// Print version and exit.
fn _version() {
	println!("FYI {}", env!("CARGO_PKG_VERSION"));
}
