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
	Msg,
	MsgKind,
	FLAG_INDENT,
	FLAG_TIMESTAMP,
};



/// Main.
fn main() {
	// Parse CLI arguments.
	let mut args = Argue::new()
		.with_any()
		.with_version(b"FYI", env!("CARGO_PKG_VERSION").as_bytes())
		.with_help(helper)
		.with_subcommand();

	// Where we going?
	match unsafe { args.peek_unchecked() } {
		"blank" => blank(&mut args),
		"print" => message(
			{
				let color = args.option2("-c", "--prefix-color")
					.map_or(199_u8, |x| x.parse::<u8>().unwrap_or(199));
				let prefix = args.option2("-p", "--prefix").unwrap_or_default();
				MsgKind::new(prefix, color)
			},
			&mut args
		),
		x => match MsgKind::from(x) {
			MsgKind::None => {
				die(b"Missing subcommand.");
				unreachable!();
			},
			x => message(x, &mut args),
		},
	}
}

/// Shoot Blanks.
///
/// Print one or more blank lines to `Stdout` or `Stderr`.
fn blank(args: &mut Argue) {
	use std::iter::FromIterator;

	// How many lines should we print?
	let msg = Msg::from_iter([10_u8].repeat(
		1_usize.max(
			args.option2("-c", "--count")
				.map_or(1, |c| c.parse::<usize>().unwrap_or(1))
		)
	));

	// Print it to `Stderr`.
	if args.switch("--stderr") {
		msg.eprint();
	}
	// Print it to `Stdout`.
	else {
		msg.print();
	}
}

#[cfg(not(feature = "man"))]
/// Help Page.
///
/// Print the appropriate help screen given the call details. Most of the sub-
/// commands work the same way, but a few have their own distinct messages.
fn helper(cmd: Option<&str>) {
	Msg::from(
		format!(
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

{}"#,
			"\x1b[38;5;199mFYI\x1b[0;38;5;69m v",
			env!("CARGO_PKG_VERSION"),
			"\x1b[0m",
			match cmd {
				Some("blank") => include_str!("../help/blank.txt").to_string(),
				Some("print") => include_str!("../help/print.txt").to_string(),
				Some("confirm") | Some("prompt") => include_str!("../help/confirm.txt").to_string(),
				Some(x) if MsgKind::from(x) != MsgKind::None => format!(
					include_str!("../help/generic.txt"),
					x,
					MsgKind::from(x).into_msg("Hello World").as_str(),
					x.to_lowercase(),
				),
				_ => include_str!("../help/help.txt").to_string(),
			}
		)
	).println();
}

#[cfg(feature = "man")]
#[cold]
/// Help Page.
///
/// This is a boring version of the help screen used during build to make it
/// easier for `help2man` to parse.
fn helper(_: Option<&str>) {
	Msg::from([
		b"FYI ",
		env!("CARGO_PKG_VERSION").as_bytes(),
		b"\n",
		env!("CARGO_PKG_DESCRIPTION").as_bytes(),
		b"\n\n",
		include_bytes!("../help/generic.txt"),
	].concat())
		.println();
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
			std::process::exit(1);
		}

		return;
	}
	// Print to `Stderr`.
	else if args.switch("--stderr") { msg.eprintln(); }
	// Print to `Stdout`.
	else { msg.println(); }

	// Special exit?
	if 0 != exit {
		std::process::exit(exit);
	}
}
