/*!
# FYI

This is the bootstrap for the FYI binary, a CLI tool for printing simple status
messages.
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

use fyi_menu::{
	die,
	FLAG_ALL,
	parse_env_args,
};
use fyi_msg::MsgKind;
use std::{
	io::{
		self,
		Write,
	},
	process,
};



/// -i | --indent
const FLAG_INDENT: u8    = 0b0001;
/// --stderr
const FLAG_STDERR: u8    = 0b0010;
/// -t | --timestamp
const FLAG_TIMESTAMP: u8 = 0b0100;



fn main() {
	let mut args = parse_env_args(FLAG_ALL);

	// There are a number of different modes here, some of which require extra
	// work, some of which immediately short-circuit. Best to match the first
	// value early to see what's what.
	match args.remove(0).as_str() {
		"-h" | "--help" | "help" => _help(include_bytes!("../help/help.txt")),
		"-V" | "--version" => _version(),
		"blank" => _blank(&args),
		x => _msg(x, &args),
	}
}

/// Shoot Blanks.
///
/// This prints one or more blank lines to `Stdout` or `Stderr`. That's it!
fn _blank(args: &[String]) {
	// The command defaults.
	let mut count: usize = 1;
	let mut err: bool = false;

	// Run through the args to see what's what.
	let mut idx: usize = 0;
	let len: usize = args.len();
	while idx < len {
		match args[idx].as_str() {
			"--stderr" => {
				err = true;
				idx += 1;
			},
			"-c" | "--count" =>
				if idx + 1 < len {
					count = 100.min(1.max(args[idx + 1].parse::<usize>().unwrap_or(1)));
					idx += 2;
				}
				else { idx += 1; },
			"-h" | "--help" => {
				_help(include_bytes!("../help/blank.txt"));
				return;
			},
			_ => { break; },
		}
	}

	// Print it to `Stderr`.
	if err {
		io::stderr().write_all(&[10].repeat(count)).unwrap();
	}
	// Print it to `Stdout`.
	else {
		io::stdout().write_all(&[10].repeat(count)).unwrap();
	}
}

/// Process Message.
///
/// Most of the subcommands work pretty much the same way. Enough that we can
/// group their handling under one parent method, anyway.
fn _msg(com: &str, args: &[String]) {
	// Set up the default options.
	let mut exit: i32 = 0;
	let mut flags: u8 = 0;
	let mut color: u8 = 199;
	let mut prefix: &str = "";
	let mut kind: MsgKind = MsgKind::from(com);
	let custom: bool = com == "print";

	// Parse the arguments to see what we've got.
	let mut idx: usize = 0;
	let len: usize = args.len();
	while idx < len {
		match args[idx].as_str() {
			"-i" | "--indent" => {
				flags |= FLAG_INDENT;
				idx += 1;
			},
			"--stderr" => {
				flags |= FLAG_STDERR;
				idx += 1;
			},
			"-t" | "--timestamp" => {
				flags |= FLAG_TIMESTAMP;
				idx += 1;
			},
			"-e" | "--exit" =>
				if idx + 1 < len {
					exit = args[idx + 1].parse::<i32>().unwrap_or(0);
					idx += 2;
				}
				else { idx += 1; },
			"-h" | "--help" => {
				if custom {
					_help(
						include_bytes!("../help/print.txt")
					);
				}
				else if MsgKind::Confirm == kind {
					_help(include_bytes!("../help/confirm.txt"));
				}
				else {
					 _help(format!(
						include_str!("../help/generic.txt"),
						com,
						kind.into_msg("Hello World").as_str(),
						com.to_lowercase(),
					).as_bytes());
				}

				return;
			},
			"-c" | "--prefix-color" if custom =>
				if idx + 1 < len {
					color = args[idx + 1].parse::<u8>().unwrap_or(199);
					idx += 2;
				}
				else { idx += 1; },
			"-p" | "--prefix" if custom =>
				if idx + 1 < len {
					prefix = args[idx + 1].as_str();
					idx += 2;
				}
				else { idx += 1; },
			_ => { break; },
		}
	}

	// Are we missing a message?
	if idx + 1 != len {
		die(b"Missing message.");
	}

	// If we're custom and have a prefix, we need to update the kind.
	if custom && ! prefix.is_empty() {
		kind = MsgKind::new(prefix, color);
	}

	// Let's build the message!
	let msg = kind.into_msg(&args[idx])
		.with_indent((0 != flags & FLAG_INDENT) as u8)
		.with_timestamp(0 != flags & FLAG_TIMESTAMP);

	// It's a prompt!
	if MsgKind::Confirm == kind {
		if ! msg.prompt() {
			process::exit(1);
		}

		return;
	}
	// Print to `Stdout`.
	else if 0 == flags & FLAG_STDERR {
		msg.println();
	}
	// Print to `Stderr`.
	else {
		msg.eprintln();
	}

	// Special exit?
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
