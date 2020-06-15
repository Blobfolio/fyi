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
		("confirm", Some(o)) => do_confirm(o),
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

/// Confirmation prompt.
fn do_confirm(opts: &ArgMatches) {
	let mut msg: Msg = Msg::confirm(opts.value_of("msg").unwrap_or_default());

	// Indent it?
	if opts.is_present("indent") {
		msg.set_indent(1);
	}

	if ! casual::confirm(msg) {
		process::exit(1);
	}
}

/// Print message.
fn do_msg(name: &str, opts: &ArgMatches) {
	let mut msg: Msg = match name {
		"crunched" => Msg::crunched(opts.value_of("msg").unwrap_or_default()),
		"debug" => Msg::debug(opts.value_of("msg").unwrap_or_default()),
		"done" => Msg::done(opts.value_of("msg").unwrap_or_default()),
		"error" => Msg::error(opts.value_of("msg").unwrap_or_default()),
		"info" => Msg::info(opts.value_of("msg").unwrap_or_default()),
		"notice" => Msg::notice(opts.value_of("msg").unwrap_or_default()),
		"success" => Msg::success(opts.value_of("msg").unwrap_or_default()),
		"task" => Msg::task(opts.value_of("msg").unwrap_or_default()),
		"warning" => Msg::warning(opts.value_of("msg").unwrap_or_default()),
		_ => match opts.value_of("prefix") {
			Some(p) => Msg::new(
				p,
				parse_cli_u8(opts.value_of("prefix_color").unwrap_or("199")),
				opts.value_of("msg").unwrap_or_default()
			),
			None => Msg::new("", 0, opts.value_of("msg").unwrap_or_default()),
		},
	};

	// Indent it?
	if opts.is_present("indent") {
		msg.set_indent(1);
	}

	// Add a timestamp?
	if opts.is_present("time") {
		msg.set_timestamp();
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
	if let Ok(x) = val.into().parse::<u8>() { x }
	else { 0 }
}
