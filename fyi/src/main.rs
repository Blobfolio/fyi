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
#![warn(clippy::pedantic)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use clap::ArgMatches;
use fyi_msg::{
	PrintFlags,
	PrinterKind,
	Msg,
};
use std::process;

mod menu;



fn main() {
	// Command line arguments.
	let opts: ArgMatches = menu::menu()
		.get_matches();

	// Make the message.
	match opts.subcommand() {
		("blank", Some(o)) => do_blank(o),
		(name, Some(o)) => do_msg(name, o),
		_ => {},
	}

	process::exit(0);
}

/// Shoot blanks.
fn do_blank(opts: &ArgMatches) {
	let mut count: u8 = parse_cli_u8(opts.value_of("count").unwrap_or("1"));
	if 0 == count {
		count = 1;
	}

	if opts.is_present("stderr") {
		for _ in 0..count { eprintln!(); }
	}
	else {
		for _ in 0..count { println!(); }
	}
}

/// Print message.
fn do_msg(name: &str, opts: &ArgMatches) {
	let mut msg: Msg = match name {
		"confirm" => Msg::confirm(opts.value_of("msg").unwrap_or("")),
		"debug" => Msg::debug(opts.value_of("msg").unwrap_or("")),
		"error" => Msg::error(opts.value_of("msg").unwrap_or("")),
		"info" => Msg::info(opts.value_of("msg").unwrap_or("")),
		"notice" => Msg::notice(opts.value_of("msg").unwrap_or("")),
		"success" => Msg::success(opts.value_of("msg").unwrap_or("")),
		"warning" => Msg::warning(opts.value_of("msg").unwrap_or("")),
		_ => match opts.value_of("prefix") {
			Some(p) => Msg::new(
				p,
				parse_cli_u8(opts.value_of("prefix_color").unwrap_or("199")),
				opts.value_of("msg").unwrap_or("")
			),
			_ => Msg::plain(opts.value_of("msg").unwrap_or("")),
		},
	};

	// Build and print!
	if opts.is_present("indent") {
		msg.indent();
	}

	// Prompt.
	if "confirm" == name {
		if msg.prompt() {
			return;
		}
		else {
			process::exit(1);
		}
	}

	if opts.is_present("stderr") {
		msg.set_printer(PrinterKind::Stderr);
	}

	if opts.is_present("time") {
		msg.timestamp();
	}

	msg.print(PrintFlags::NONE);

	// We might have a custom exit code.
	let exit: u8 = parse_cli_u8(opts.value_of("exit").unwrap_or("0"));
	if 0 != exit {
		process::exit(i32::from(exit));
	}
}

/// Validate CLI numeric inputs.
fn parse_cli_u8<S> (val: S) -> u8
where S: Into<String> {
	match val.into().parse::<u8>() {
		Ok(x) => x,
		_ => 0,
	}
}
