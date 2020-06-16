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
use fyi_msg::{
	Msg,
	utility::str_to_u8,
};
use std::{
	io::{
		self,
		Write
	},
	process
};

mod menu;



fn main() {
	// Make the message.
	match menu::menu().get_matches().subcommand() {
		("blank", Some(o)) => do_blank(o),
		("confirm", Some(o)) => do_confirm(o),
		(name, Some(o)) => do_msg(name, o),
		_ => {},
	}
}

/// Shoot blanks.
fn do_blank(opts: &ArgMatches) {
	let count: usize = usize::max(
		1,
		opts.value_of("count").unwrap_or("1").parse::<usize>().unwrap_or_default()
	);

	if opts.is_present("stderr") {
		io::stderr().write_all(&[10].repeat(count)).unwrap();
	}
	else {
		io::stdout().write_all(&[10].repeat(count)).unwrap();
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
	let msg_str: &str = &[opts.value_of("msg").unwrap_or_default(), "\n"].concat();
	let mut msg: Msg = match name {
		"crunched" => Msg::crunched(msg_str),
		"debug" => Msg::debug(msg_str),
		"done" => Msg::done(msg_str),
		"error" => Msg::error(msg_str),
		"info" => Msg::info(msg_str),
		"notice" => Msg::notice(msg_str),
		"success" => Msg::success(msg_str),
		"task" => Msg::task(msg_str),
		"warning" => Msg::warning(msg_str),
		_ => match opts.value_of("prefix") {
			Some(p) => Msg::new(
				p,
				str_to_u8(opts.value_of("prefix_color").unwrap_or("199")),
				msg_str
			),
			None => Msg::new("", 0, msg_str),
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
		io::stderr().write_all(&msg).unwrap();
	}
	// Print it to `Stdout`.
	else {
		io::stdout().write_all(&msg).unwrap();
	}

	// We might have a custom exit code.
	let exit: u8 = str_to_u8(opts.value_of("exit").unwrap_or("0"));
	if 0 != exit {
		process::exit(i32::from(exit));
	}
}
