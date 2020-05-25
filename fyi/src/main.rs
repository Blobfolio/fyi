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
use fyi_msg::Msg;
use std::{
	io::{
		self,
		Write
	},
	process
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
}

/// Shoot blanks.
fn do_blank(opts: &ArgMatches) {
	let mut count: u8 = parse_cli_u8(opts.value_of("count").unwrap_or("1"));
	if 0 == count {
		count = 1;
	}

	if opts.is_present("stderr") {
		io::stderr().write_all(&[10].repeat(count as usize)).unwrap();
	}
	else {
		io::stdout().write_all(&[10].repeat(count as usize)).unwrap();
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
			_ => Msg::new("", 0, opts.value_of("msg").unwrap_or("")),
		},
	};

	// Build and print!
	if opts.is_present("indent") {
		msg.set_indent(1);
	}

	// Prompt.
	if "confirm" == name {
		if casual::confirm(msg) {
			return;
		}
		else {
			process::exit(1);
		}
	}

	if opts.is_present("time") {
		msg.set_timestamp(false);
	}

	// Print it to `Stderr`.
	if opts.is_present("stderr") {
		io::stderr().write_all(
			&msg.iter()
				.chain(&[10])
				.copied()
				.collect::<Vec<u8>>()
		).unwrap();
	}
	// Print it to `Stdout`.
	else {
		io::stdout().write_all(
			&msg.iter()
				.chain(&[10])
				.copied()
				.collect::<Vec<u8>>()
		).unwrap();
	}

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
