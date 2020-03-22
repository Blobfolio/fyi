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
use fyi_core::{Msg, Prefix, NO_COLOR, TIMESTAMP};
use std::process::exit;

mod menu;



fn main() {
	// Command line arguments.
	let opts: ArgMatches = menu::menu()
		.get_matches();

	// Make the message.
	if let Some(name) = opts.subcommand_name() {
		if let Some(opts2) = opts.subcommand_matches(&name) {
			// Blank lines are easy.
			if "blank" == name {
				let mut count: u8 = parse_cli_u8(opts2.value_of("count").unwrap_or("1"));
				if 1 > count {
					count = 1;
				}

				for _ in 0..count {
					match opts2.is_present("stderr") {
						true => eprintln!(""),
						false => println!(""),
					}
				}

				exit(0);
			}

			// Convert the CLI subcommand into an appropriate prefix.
			let prefix: Prefix = match name {
				"debug" => Prefix::Debug,
				"error" => Prefix::Error,
				"info" => Prefix::Info,
				"notice" => Prefix::Notice,
				"prompt" => Prefix::None,
				"success" => Prefix::Success,
				"warning" => Prefix::Warning,
				_ => {
					let color: u8 = parse_cli_u8(opts2.value_of("prefix_color").unwrap_or("199"));

					match opts2.value_of("prefix") {
						Some(p) => Prefix::Custom(p, color),
						_ => Prefix::None,
					}
				},
			};

			// Calculate flags.
			let mut flags: u8 = 0;
			if opts2.is_present("no_color") {
				flags |= NO_COLOR;
			}
			if opts2.is_present("time") {
				flags |= TIMESTAMP;
			}

			// Build and print!
			let msg: Msg = Msg::new(opts2.value_of("msg").unwrap_or(""))
				.with_prefix(prefix)
				.with_flags(flags)
				.with_indent(parse_cli_u8(opts2.value_of("indent").unwrap_or("0")));

			// Prompt.
			if "prompt" == name {
				match msg.prompt() {
					true => exit(0),
					false => exit(1),
				};
			}
			// Echo.
			else {
				msg.print();
			}
		}
	}

	// We're done!
	exit(parse_cli_u8(opts.value_of("exit").unwrap_or("0")) as i32);
}

/// Validate CLI numeric inputs.
fn parse_cli_u8<S> (val: S) -> u8
where S: Into<String> {
	match val.into().parse::<u8>() {
		Ok(x) => x,
		_ => 0,
	}
}
