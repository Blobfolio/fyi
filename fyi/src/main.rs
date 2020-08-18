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
	Argue,
};
use fyi_msg::{
	MsgKind,
	FLAG_INDENT,
	FLAG_TIMESTAMP,
};
use std::{
	io::{
		self,
		Write,
	},
	process,
};



/// Main.
fn main() {
	// Parse CLI arguments.
	let mut args = Argue::new()
		.with_any()
		.with_version(versioner)
		.with_help(helper);

	// Where we going?
	match args.peek() {
		Some("blank") => blank(&mut args),
		Some("print") => {
			let kind: MsgKind = {
				let color = args.option2("-c", "--prefix-color")
					.map_or(199_u8, |x| x.parse::<u8>().unwrap_or(199));
				let prefix = args.option2("-p", "--prefix").unwrap_or_default();
				MsgKind::new(prefix, color)
			};
			message(kind, &mut args);
		},
		Some(x) if MsgKind::from(x) != MsgKind::None => message(MsgKind::from(x), &mut args),
		_ => {
			die(b"Missing subcommand.");
			unreachable!();
		},
	}
}

/// Shoot Blanks.
///
/// Print one or more blank lines to `Stdout` or `Stderr`.
fn blank(args: &mut Argue) {
	// How many lines should we print?
	let count: usize = 1.max(
		args.option2("-c", "--count")
			.map_or(1, |c| c.parse::<usize>().unwrap_or(1))
	);

	// Print it to `Stderr`.
	if args.switch("--stderr") {
		io::stderr().write_all(&[10].repeat(count)).unwrap();
	}
	// Print it to `Stdout`.
	else {
		io::stdout().write_all(&[10].repeat(count)).unwrap();
	}
}

/// Help Page.
///
/// Print the appropriate help screen given the call details. Most of the sub-
/// commands work the same way, but a few have their own distinct messages.
fn helper(cmd: Option<&str>) {
	match cmd {
		Some("blank") => _help(include_bytes!("../help/blank.txt")),
		Some("print") => _help(include_bytes!("../help/print.txt")),
		Some("confirm") | Some("prompt") => _help(include_bytes!("../help/confirm.txt")),
		Some(x) if MsgKind::from(x) != MsgKind::None => {
			_help(format!(
				include_str!("../help/generic.txt"),
				x,
				MsgKind::from(x).into_msg("Hello World").as_str(),
				x.to_lowercase(),
			).as_bytes())
		},
		_ => _help(include_bytes!("../help/help.txt")),
	}
}

/// Print Message!
///
/// Almost all roads lead to this method, which crunches the CLI args and
/// prints an appropriately formatted message.
fn message(kind: MsgKind, args: &mut Argue) {
	// Exit code.
	let exit: i32 = args.option2("-e", "--exit")
		.map_or(0, |x| x.parse::<i32>().unwrap_or(0));

	// Basic flags.
	let mut flags: u8 = 0;
	if args.switch2("-i", "--indent") { flags |= FLAG_INDENT; }
	if args.switch2("-t", "--timestamp") { flags |= FLAG_TIMESTAMP; }

	// Let's build the message!
	let msg = kind.into_msg(args.take_arg()).with_flags(flags);

	// It's a prompt!
	if MsgKind::Confirm == kind {
		if ! msg.prompt() {
			process::exit(1);
		}

		return;
	}
	// Print to `Stderr`.
	else if args.switch("--stderr") { msg.eprintln(); }
	// Print to `Stdout`.
	else { msg.println(); }

	// Special exit?
	if 0 != exit {
		process::exit(exit);
	}
}

/// Print Version.
fn versioner() {
	let writer = std::io::stdout();
	let mut handle = writer.lock();

	handle.write_all(b"FYI ").unwrap();
	handle.write_all(env!("CARGO_PKG_VERSION").as_bytes()).unwrap();
	handle.write_all(b"\n").unwrap();

	handle.flush().unwrap();
}

#[cfg(not(feature = "man"))]
#[cold]
/// Print Help.
///
/// This actually prints the help screen. This version of the method is used in
/// the compiled binary.
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
/// This actually prints the help screen. This version of the method is used
/// during build to give `help2man` something to work with.
fn _help(txt: &[u8]) {
	versioner();

	let writer = std::io::stdout();
	let mut handle = writer.lock();

	handle.write_all(env!("CARGO_PKG_DESCRIPTION").as_bytes()).unwrap();
	handle.write_all(b"\n\n").unwrap();
	handle.write_all(txt).unwrap();
	handle.write_all(b"\n").unwrap();

	handle.flush().unwrap();
}
