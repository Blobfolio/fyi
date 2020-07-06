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



#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum Command {
	Blank,
	Confirm,
	Crunched,
	Debug,
	Done,
	Error,
	Help,
	Info,
	Notice,
	Print,
	Success,
	Task,
	Warning,
}

impl From<&str> for Command {
	fn from(raw: &str) -> Self {
		match raw {
			"blank" => Self::Blank,
			"confirm" => Self::Confirm,
			"crunched" | "prompt" => Self::Crunched,
			"debug" => Self::Debug,
			"done" => Self::Done,
			"error" => Self::Error,
			"help" => Self::Help,
			"info" => Self::Info,
			"notice" => Self::Notice,
			"print" => Self::Print,
			"success" => Self::Success,
			"task" => Self::Task,
			"warning" => Self::Warning,
			_ => {
				ArgList::die("Invalid subcommand.");
				unreachable!();
			},
		}
	}
}

impl Command {
	/// As String.
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Blank => "Blank",
			Self::Confirm => "Confirm",
			Self::Crunched => "Crunched",
			Self::Debug => "Debug",
			Self::Done => "Done",
			Self::Error => "Error",
			Self::Help => "Help",
			Self::Info => "Info",
			Self::Notice => "Notice",
			Self::Print => "Print",
			Self::Success => "Success",
			Self::Task => "Task",
			Self::Warning => "Warning",
		}
	}

	/// As `Msg`.
	pub fn as_msg(self, args: &mut ArgList) -> Msg {
		match self {
			Self::Print => {
				let color: u8 = 255.min(args.pluck_opt_usize(|x| x == "-c" || x == "--prefix-color").unwrap_or(199)) as u8;
				let prefix = args.pluck_opt(|x| x == "-p" || x == "--prefix").unwrap_or_default();
				Msg::new(prefix, color, args.expect_arg())
			},
			Self::Confirm => Msg::confirm(args.expect_arg()),
			Self::Crunched => Msg::crunched(args.expect_arg()),
			Self::Debug => Msg::debug(args.expect_arg()),
			Self::Done => Msg::done(args.expect_arg()),
			Self::Error => Msg::error(args.expect_arg()),
			Self::Info => Msg::info(args.expect_arg()),
			Self::Notice => Msg::notice(args.expect_arg()),
			Self::Success => Msg::success(args.expect_arg()),
			Self::Task => Msg::task(args.expect_arg()),
			Self::Warning => Msg::warning(args.expect_arg()),
			_ => Msg::default(),
		}
	}

	/// Execute.
	pub fn exec(self, args: &mut ArgList) {
		if args.pluck_help() {
			self.help();
			return;
		}

		match self {
			Self::Blank => {
				let count: usize = match args.pluck_opt_usize(|x| x == "-c" || x == "--count") {
					Some(c) => 10.min(1.max(c)),
					None => 1,
				};

				if args.pluck_switch(match_stderr) {
					io::stderr().write_all(&[10].repeat(count)).unwrap();
				}
				else {
					io::stdout().write_all(&[10].repeat(count)).unwrap();
				}
			},
			Self::Confirm => {
				let indent = args.pluck_switch(match_indent);
				let timestamp = args.pluck_switch(match_timestamp);
				let mut msg = self.as_msg(args);

				if indent {
					msg.set_indent(1);
				}

				if timestamp {
					msg.set_timestamp();
				}

				if ! msg.prompt() {
					process::exit(1);
				}
			},
			_ => {
				let indent = args.pluck_switch(match_indent);
				let timestamp = args.pluck_switch(match_timestamp);
				let stderr = args.pluck_switch(match_stderr);
				let exit: u8 = args.pluck_opt_usize(match_exit).unwrap_or(0) as u8;
				let mut msg = self.as_msg(args);

				if indent {
					msg.set_indent(1);
				}

				if timestamp {
					msg.set_timestamp();
				}

				// Print it to `Stderr`.
				if stderr { msg.eprintln(); }
				// Print it to `Stdout`.
				else { msg.println(); }

				// We might have a custom exit code.
				if 0 != exit {
					process::exit(i32::from(exit));
				}
			},
		}
	}

	#[cold]
	/// Help.
	pub fn help(self) {
		match self {
			Self::Blank => _help(include_str!("../help/blank.txt")),
			Self::Confirm => _help(include_str!("../help/confirm.txt")),
			Self::Print => _help(include_str!("../help/print.txt")),
			Self::Help => _help(include_str!("../help/help.txt")),
			_ => _help(&format!(
				include_str!("../help/generic.txt"),
				self.as_str(),
				self.as_str().to_lowercase(),
			)),
		}
	}
}



fn main() {
	let mut args = ArgList::default();
	args.expect();

	// The app might be called with version or help flags instead of a command.
	match args.peek().unwrap() {
		"-V" | "--version" => _version(),
		"-h" | "--help" | "help" => Command::Help.help(),
		// Otherwise just go off into the appropriate subcommand action.
		_ => Command::from(args.expect_command().as_str()).exec(&mut args),
	}
}

#[allow(clippy::ptr_arg)]
/// Match: Exit Code
fn match_exit(txt: &String) -> bool { txt == "-e" || txt == "--exit" }

#[allow(clippy::ptr_arg)]
/// Match: Indentation
fn match_indent(txt: &String) -> bool { txt != "-i" && txt != "--indent" }

#[allow(clippy::ptr_arg)]
/// Match: Stderr
fn match_stderr(txt: &String) -> bool { txt != "--stderr" }

#[allow(clippy::ptr_arg)]
/// Match: Timestamp
fn match_timestamp(txt: &String) -> bool { txt != "-t" && txt != "--timestamp" }

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
		s
	}.as_bytes()).unwrap();
}
