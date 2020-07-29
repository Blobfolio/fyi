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
const FLAG_HELP: u8      = 0b0000_0001;
/// -i | --indent
const FLAG_INDENT: u8    = 0b0000_0010;
/// --stderr
const FLAG_STDERR: u8    = 0b0000_0100;
/// -t | --timestamp
const FLAG_TIMESTAMP: u8 = 0b0000_1000;
/// Prompt rather than Print.
const FLAG_PROMPT: u8    = 0b0001_0000;



fn main() {
	let mut args = ArgList::default();

	// The app might be called with version or help flags instead of a command.
	match args.pluck_next().unwrap().as_str() {
		"-h" | "--help" | "help" => _help(include_bytes!("../help/help.txt")),
		"-V" | "--version" => _version(),
		"blank" => _blank(&mut args),
		"print" => {
			let flags = _flags(&mut args);
			if 0 == flags & FLAG_HELP {
				let (exit, color, prefix, msg) = _print_opts(args);
				_msg(Msg::new(prefix, color, msg), flags, exit);
			}
			// Show help instead.
			else {
				_help(include_bytes!("../help/print.txt"));
			}
		},
		other => match MsgKind::from(other) {
			MsgKind::None => fyi_menu::die(b"Invalid subcommand."),
			mkind => {
				let flags =
					if other == "confirm" || other == "prompt" {
						_flags(&mut args) | FLAG_PROMPT
					}
					else {
						_flags(&mut args)
					};

				if 0 == flags & FLAG_HELP {
					let (exit, msg) = _msg_opts(args);
					_msg(mkind.as_msg(msg), flags, exit);
				}
				// Show other help.
				else if 0 == flags & FLAG_PROMPT {
					 _help(format!(
						include_str!("../help/generic.txt"),
						mkind.as_str(),
						mkind.as_str().to_lowercase(),
					).as_bytes());
				}
				// Show confirm help.
				else {
					_help(include_bytes!("../help/confirm.txt"));
				}
			}
		},
	}
}

/// Generic Message Opts.
fn _msg_opts(mut args: ArgList) -> (i32, String) {
	let mut out = (0_i32, String::new());

	// Pull the remaining options and/or message. We don't need
	// to clean up after ourselves here; args gets dropped at
	// the end.
	let len = args.len();
	let v = args.as_mut_vec();
	let mut idx = 0;
	while idx < len {
		if idx + 1 == len {
			out.1 = v.remove(idx);
			break;
		}

		match v[idx].as_str() {
			"-e" | "--exit" => {
				out.0 = v[idx + 1].parse::<i32>().unwrap_or_default();
				idx += 2;
			},
			_ => {
				out.1 = v.remove(idx);
				break;
			},
		}
	}

	if out.1.is_empty() {
		fyi_menu::die(b"Missing message.");
	}

	out
}

/// Generic Print Opts.
fn _print_opts(mut args: ArgList) -> (i32, u8, String, String) {
	let mut out = (0_i32, 199_u8, String::new(), String::new());

	// Pull the remaining options and/or message. We don't need to
	// clean up after ourselves here; args gets dropped at the end.
	let mut len = args.len();
	let v = args.as_mut_vec();
	let mut idx = 0;
	while idx < len {
		if idx + 1 == len {
			out.3 = v.remove(idx);
			break;
		}

		match v[idx].as_str() {
			"-e" | "--exit" => {
				out.0 = v[idx + 1].parse::<i32>().unwrap_or_default();
				idx += 2;
			},
			"-c" | "--prefix-color" => {
				out.1 = 1.max(v[idx + 1].parse::<u8>().unwrap_or(199));
				idx += 2;
			},
			"-p" | "--prefix" => {
				out.2 = v.remove(idx + 1);
				idx += 1;
				len -= 1;
			},
			_ => {
				out.3 = v.remove(idx);
				break;
			},
		}
	}

	if out.3.is_empty() {
		fyi_menu::die(b"Missing message.");
	}

	out
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
	let flags: u8 = _flags(args);
	if 0 == flags & FLAG_HELP {
		// How many lines should we print?
		let count: usize = 100.min(1.max(
			args.pluck_opt(|x| x == "-c" || x == "--count")
				.map_or(1, |x| x.parse::<usize>().unwrap_or(1))
		));

		// Print it to `Stdout`.
		if 0 == flags & FLAG_STDERR {
			io::stdout().write_all(&[10].repeat(count)).unwrap();
		}
		// Print it to `Stderr`.
		else {
			io::stderr().write_all(&[10].repeat(count)).unwrap();
		}
	}
	// Show help instead.
	else {
		_help(include_bytes!("../help/blank.txt"));
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

	// Do a confirmation.
	if 0 != flags & FLAG_PROMPT {
		if ! msg.prompt() {
			process::exit(1);
		}

		return;
	}
	// Print it to `Stdout`.
	else if 0 == flags & FLAG_STDERR { msg.println(); }
	// Print it to `Stderr`.
	else { msg.eprintln(); }

	// We might have a custom exit code.
	if 0 != exit {
		process::exit(exit);
	}
}

#[cfg(not(feature = "man"))]
#[cold]
/// Print Help.
fn _help(txt: &[u8]) {
	io::stdout().write_fmt(format_args!(
		r#"
                      ;\
                     |' \
  _                  ; : ;
 / `-.              /: : |
|  ,-.`-.          ,': : |
\  :  `. `.       ,'-. : |
 \ ;    ;  `-.__,'    `-.|         {}{}{}
  \ ;   ;  :::  ,::'`:.  `.        Simple CLI status messages.
   \ `-. :  `    :.    `.  \
    \   \    ,   ;   ,:    (\
     \   :., :.    ,'o)): ` `-.
    ,/,' ;' ,::"'`.`---'   `.  `-._
  ,/  :  ; '"      `;'          ,--`.
 ;/   :; ;             ,:'     (   ,:)
   ,.,:.    ; ,:.,  ,-._ `.     \""'/
   '::'     `:'`  ,'(  \`._____.-'"'
      ;,   ;  `.  `. `._`-.  \\
      ;:.  ;:       `-._`-.\  \`.
       '`:. :        |' `. `\  ) \
          ` ;:       |    `--\__,'
            '`      ,'
                 ,-'

{}
"#,
		"\x1b[38;5;199mFYI\x1b[0;38;5;69m v",
		env!("CARGO_PKG_VERSION"),
		"\x1b[0m",
		unsafe { std::str::from_utf8_unchecked(txt) }
	)).unwrap();
}

#[cfg(feature = "man")]
#[cold]
/// Print Help.
///
/// This is a stripped-down version of the help screen made specifically for
/// `help2man`, which gets run during the Debian package release build task.
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
