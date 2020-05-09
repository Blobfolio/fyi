use clap::{
	App,
	AppSettings,
	SubCommand,
};



/// Most of these have exactly the same options. Haha.
macro_rules! clap_subcommand {
	($name:literal, $desc:literal) => {
		SubCommand::with_name($name)
			.about($desc)
			.arg(clap::Arg::with_name("indent")
				.short("i")
				.long("indent")
				.takes_value(true)
				.default_value("0")
				.help("Number of indentations.")
				.validator(validate_cli_u8)
			)
			.arg(clap::Arg::with_name("no_color")
				.long("no-color")
				.takes_value(false)
				.help("Print without any fancy formatting.")
			)
			.arg(clap::Arg::with_name("stderr")
				.long("stderr")
				.takes_value(false)
				.help("Print to STDERR instead of STDOUT.")
			)
			.arg(clap::Arg::with_name("time")
				.short("t")
				.long("add-timestamp")
				.alias("time")
				.alias("timestamp")
				.takes_value(false)
				.help("Include a timestamp.")
			)
			.arg(clap::Arg::with_name("msg")
				.multiple(false)
				.required(true)
				.use_delimiter(false)
				.value_name("MSG")
				.help("The message!")
			)
	};
}



#[allow(clippy::too_many_lines)]
/// CLI Menu.
pub fn menu() -> App<'static, 'static> {
	App::new("FYI")
		.version(env!("CARGO_PKG_VERSION"))
		.author("Blobfolio, LLC. <hello@blobfolio.com>")
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.settings(&[
			AppSettings::SubcommandRequiredElseHelp,
		])
		.global_settings(&[
			AppSettings::VersionlessSubcommands,
		])
		.subcommand(
			SubCommand::with_name("blank")
				.about("Print an empty line.")
				.arg(clap::Arg::with_name("count")
					.short("c")
					.long("count")
					.takes_value(true)
					.default_value("1")
					.help("Number of empty lines to print.")
					.validator(validate_cli_u8)
				)
				.arg(clap::Arg::with_name("stderr")
					.short("e")
					.long("stderr")
					.takes_value(false)
					.help("Print to STDERR instead of STDOUT.")
				)
		)
		.subcommand(
			clap_subcommand!("print", "Print a message without a prefix (or with a custom one).")
				.arg(clap::Arg::with_name("prefix")
					.short("p")
					.long("prefix")
					.takes_value(true)
					.default_value("")
					.help("Set a custom prefix.")
				)
				.arg(clap::Arg::with_name("prefix_color")
					.short("c")
					.long("prefix-color")
					.takes_value(true)
					.default_value("199")
					.validator(validate_cli_u8)
					.help("Use this color for the prefix.")
				)
		)
		.subcommand(
			SubCommand::with_name("confirm")
				.alias("prompt")
				.about("Ask a Yes/No question. An exit code of 0 indicates acceptance.")
				.arg(clap::Arg::with_name("no_color")
					.long("no-color")
					.takes_value(false)
					.help("Print without any fancy formatting.")
				)
				.arg(clap::Arg::with_name("msg")
					.help("The question!")
					.multiple(false)
					.required(true)
					.value_name("QUESTION")
					.use_delimiter(false)
				)
		)
		.subcommand(clap_subcommand!("debug", "Print a debug message."))
		.subcommand(clap_subcommand!("done", "Print a finished message."))
		.subcommand(
			clap_subcommand!("error", "Print an error message.")
				.arg(clap::Arg::with_name("exit")
					.short("e")
					.long("exit")
					.takes_value(true)
					.default_value("0")
					.help("Exit with this status code after printing.")
					.validator(validate_cli_u8)
				)
		)
		.subcommand(clap_subcommand!("info", "Print an info message."))
		.subcommand(clap_subcommand!("notice", "Print a notice."))
		.subcommand(clap_subcommand!("success", "Print a success message."))
		.subcommand(clap_subcommand!("task", "Print a task message."))
		.subcommand(clap_subcommand!("warning", "Print a warning message."))
}



#[allow(clippy::needless_pass_by_value)]
/// Validate CLI numeric inputs.
fn validate_cli_u8(val: String) -> Result<(), String> {
	if val.parse::<u8>().is_ok() {
		Ok(())
	}
	else {
		Err("Value must be at least 0.".to_string())
	}
}
