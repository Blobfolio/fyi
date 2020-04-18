/*!
# FYI

FYI is a dead-simple status message printer for CLI use applications.
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

extern crate clap;
extern crate fyi_core;

use clap::ArgMatches;
use fyi_core::{
	Msg,
	MSG_TIMESTAMP,
	Prefix,
	PRINT_NEWLINE,
	PRINT_NO_COLOR,
	PRINT_STDERR,
	util::cli,
};
use std::{
	borrow::Cow,
	process,
};

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

	let flags: u8 = match opts.is_present("stderr") {
		true => PRINT_STDERR | PRINT_NEWLINE,
		false => PRINT_NEWLINE,
	};

	for _ in 0..count {
		cli::print("", flags);
	}
}

/// Print message.
fn do_msg(name: &str, opts: &ArgMatches) {
	// Build and print!
	let msg: Msg = Msg::new(opts.value_of("msg").unwrap_or(""))
		.with_prefix(match name {
			"debug" => Prefix::Debug,
			"error" => Prefix::Error,
			"info" => Prefix::Info,
			"notice" => Prefix::Notice,
			"prompt" => Prefix::None,
			"success" => Prefix::Success,
			"warning" => Prefix::Warning,
			_ => {
				match opts.value_of("prefix") {
					Some(p) => Prefix::Custom(
						Cow::Borrowed(p),
						parse_cli_u8(opts.value_of("prefix_color").unwrap_or("199"))
					),
					_ => Prefix::None,
				}
			},
		})
		.with_flags({
			let mut flags: u8 = 0;
			if opts.is_present("no_color") {
				flags |= PRINT_NO_COLOR;
			}
			if opts.is_present("time") {
				flags |= MSG_TIMESTAMP;
			}
			flags
		})
		.with_indent(parse_cli_u8(opts.value_of("indent").unwrap_or("0")));

	// Prompt.
	if "prompt" == name {
		match msg.prompt() {
			true => process::exit(0),
			false => process::exit(1),
		};
	}

	msg.print();

	// We might have a custom exit code.
	let exit: u8 = parse_cli_u8(opts.value_of("exit").unwrap_or("0"));
	if 0 != exit {
		process::exit(exit as i32);
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
