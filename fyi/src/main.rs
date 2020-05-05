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

extern crate clap;
extern crate fyi_msg;

use clap::ArgMatches;
use fyi_msg::{
	Flags,
	Msg,
	traits::Printable,
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

	let flags = if opts.is_present("stderr") {
		Flags::TO_STDERR
	}
	else {
		Flags::NONE
	};

	for _ in 0..count {
		fyi_msg::print(b"", 0, flags);
	}
}

/// Print message.
fn do_msg(name: &str, opts: &ArgMatches) {
	// Build and print!
	let indent: u8 = parse_cli_u8(opts.value_of("indent").unwrap_or("0"));

	let mut flags: Flags = Flags::NONE;
	if opts.is_present("no_color") {
		flags.insert(Flags::NO_ANSI);
	}
	if opts.is_present("time") {
		flags.insert(Flags::TIMESTAMPED);
	}

	let msg: Msg = match name {
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

	// Prompt.
	if "confirm" == name {
		if msg.prompt(indent, flags) {
			return;
		}
		else {
			process::exit(1);
		}
	}

	msg.print(indent, flags);

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
