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

use clap::{App, AppSettings, ArgMatches, SubCommand};
use fyi_core::Msg;
use std::process::exit;



fn main() {
	// Command line arguments.
	let opts: ArgMatches = menu()
		.get_matches();

	// Make the message.
	if let Some(name) = opts.subcommand_name() {
		if let Some(opts2) = opts.subcommand_matches(&name) {
			let msg: String = opts2.value_of("msg").unwrap_or("").to_string();

			let out: Msg = match name {
				"print" => {
					let color: u8 = parse_cli_u64(opts2.value_of("prefix_color").unwrap_or("199")) as u8;

					match opts2.value_of("prefix") {
						Some(p) => Msg::Custom(p.to_string(), msg, color),
						_ => Msg::Custom("".to_string(), msg, color),
					}
				},
				"debug" => Msg::Debug(msg),
				"error" => Msg::Error(msg),
				"notice" => Msg::Notice(msg),
				"success" => Msg::Success(msg),
				"warning" => Msg::Warning(msg),
				_ => unreachable!(),
			};

			out.print_special(
				! opts2.is_present("no_color"),
				parse_cli_u64(opts2.value_of("indent").unwrap_or("0")),
				opts2.is_present("timestamp")
			);
		}
	}

	// We're done!
	exit(parse_cli_u64(opts.value_of("exit").unwrap_or("0")) as i32);
}

/// CLI Menu.
fn menu() -> App<'static, 'static> {
	App::new("FYI")
		.version(env!("CARGO_PKG_VERSION"))
		.author("Blobfolio, LLC. <hello@blobfolio.com>")
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.settings(&[
			AppSettings::VersionlessSubcommands,
			AppSettings::SubcommandRequiredElseHelp,
		])
		.subcommand(
			SubCommand::with_name("print")
				.about("Print a message with a custom prefix (or no prefix).")
				.arg(clap::Arg::with_name("prefix")
					.long("prefix")
					.takes_value(true)
					.default_value("")
					.help("Set a custom prefix.")
				)
				.arg(clap::Arg::with_name("prefix_color")
					.long("prefix-color")
					.takes_value(true)
					.default_value("199")
					.validator(validate_cli_u64)
					.help("Use this color for the prefix.")
				)
				.arg(clap::Arg::with_name("msg")
					//.index(1)
					.help("The message!")
					.multiple(false)
					.value_name("MSG")
					.use_delimiter(false)
				)
		)
		.subcommand(
			SubCommand::with_name("debug")
				.about("Print a debug message.")
				.arg(clap::Arg::with_name("msg")
					//.index(1)
					.help("The message!")
					.multiple(false)
					.value_name("MSG")
					.use_delimiter(false)
				)
		)
		.subcommand(
			SubCommand::with_name("error")
				.about("Print an error message.")
				.arg(clap::Arg::with_name("exit")
					.long("exit")
					.takes_value(true)
					.default_value("0")
					.help("Exit with this status code after printing.")
					.validator(validate_cli_u64)
				)
				.arg(clap::Arg::with_name("msg")
					//.index(1)
					.help("The message!")
					.multiple(false)
					.value_name("MSG")
					.use_delimiter(false)
				)
		)
		.subcommand(
			SubCommand::with_name("notice")
				.about("Print a notice.")
				.arg(clap::Arg::with_name("msg")
					//.index(1)
					.help("The message!")
					.multiple(false)
					.value_name("MSG")
					.use_delimiter(false)
				)
		)
		.subcommand(
			SubCommand::with_name("success")
				.about("Print a success message.")
				.arg(clap::Arg::with_name("msg")
					//.index(1)
					.help("The message!")
					.multiple(false)
					.value_name("MSG")
					.use_delimiter(false)
				)
		)
		.subcommand(
			SubCommand::with_name("warning")
				.about("Print a warning message.")
				.arg(clap::Arg::with_name("msg")
					//.index(1)
					.help("The message!")
					.multiple(false)
					.value_name("MSG")
					.use_delimiter(false)
				)
		)
		.arg(clap::Arg::with_name("indent")
			.long("indent")
			.takes_value(true)
			.default_value("0")
			.help("Number of indentations.")
			.validator(validate_cli_u64)
			.global(true)
		)
		.arg(clap::Arg::with_name("no_color")
			.long("no-color")
			.takes_value(false)
			.help("Print without any fancy formatting.")
			.global(true)
		)
		.arg(clap::Arg::with_name("timestamp")
			.long("timestamp")
			.takes_value(false)
			.help("Include a timestamp.")
			.global(true)
		)
}

/// Validate CLI numeric inputs.
fn parse_cli_u64<S> (val: S) -> u64
where S: Into<String> {
	match val.into().parse::<u64>() {
		Ok(x) => x,
		_ => 0,
	}
}

/// Validate CLI numeric inputs.
fn validate_cli_u64(val: String) -> Result<(), String> {
	match val.parse::<u64>().is_ok() {
		true => Ok(()),
		false => Err("Value must be at least 0.".to_string()),
	}
}
