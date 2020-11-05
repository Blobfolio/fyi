#[cfg(not(feature = "man"))]
/// # Do Nothing.
///
/// We only need to rebuild stuff for new releases. The "man" feature is
/// basically used to figure that out.
fn main() {}



#[cfg(feature = "man")]
/// # Build BASH completions and MAN pages.
fn main() {
	use fyi_menu::{
		Agree,
		AgreeKind,
		FLAG_MAN_ALL,
	};
	use std::{
		env,
		path::PathBuf,
	};

	fn exit_arg() -> AgreeKind {
		AgreeKind::option("<NUM>", "Exit with this status code after printing. [default: 0]", false)
			.with_short("-e")
			.with_long("--exit")
	}

	fn help_arg() -> AgreeKind {
		AgreeKind::switch("Print help information.")
			.with_short("-h")
			.with_long("--help")
	}

	fn indent_arg() -> AgreeKind {
		AgreeKind::switch("Indent the line.")
			.with_short("-i")
			.with_long("--indent")
	}

	fn msg_arg() -> AgreeKind {
		AgreeKind::arg("<MSG>", "The message!")
	}

	fn stderr_arg() -> AgreeKind {
		AgreeKind::switch("Print to STDERR instead of STDOUT.")
			.with_long("--stderr")
	}

	fn timestamp_arg() -> AgreeKind {
		AgreeKind::switch("Include a timestamp.")
			.with_short("-t")
			.with_long("--timestamp")
	}

	fn builtin(cmd: &str, prefix: &str) -> AgreeKind {
		AgreeKind::SubCommand(
			Agree::new(
				cmd,
				cmd,
				"1.2.3",
				&format!("{}: Hello World", prefix),
			)
				.with_flags(FLAG_MAN_ALL)
				.with_arg(help_arg())
				.with_arg(indent_arg())
				.with_arg(stderr_arg())
				.with_arg(timestamp_arg())
				.with_arg(exit_arg())
				.with_arg(msg_arg())
		)
	}

	let app: Agree = Agree::new(
		"FYI",
		env!("CARGO_PKG_NAME"),
		env!("CARGO_PKG_VERSION"),
		env!("CARGO_PKG_DESCRIPTION"),
	)
		.with_flags(FLAG_MAN_ALL)
		.with_arg(help_arg())
		.with_arg(
			AgreeKind::switch("Print program version.")
				.with_short("-V")
				.with_long("--version")
		)
		.with_arg(
			AgreeKind::SubCommand(
				Agree::new(
					"help",
					"help",
					"1.2.3",
					"Print this screen.",
				)
					.with_flags(FLAG_MAN_ALL)
			)
		)
		.with_arg(
			AgreeKind::SubCommand(
				Agree::new(
					"blank",
					"blank",
					"1.2.3",
					"Print blank line(s).",
				)
					.with_flags(FLAG_MAN_ALL)
					.with_arg(help_arg())
					.with_arg(stderr_arg())
					.with_arg(
						AgreeKind::option("<NUM>", "Number of empty lines to print. [default: 1]", false)
							.with_short("-c")
							.with_long("--count")
					)
			)
		)
		.with_arg(
			AgreeKind::SubCommand(
				Agree::new(
					"confirm",
					"confirm",
					"1.2.3",
					"Ask a Yes/No question using the built-in prefix \"confirm\".",
				)
					.with_flags(FLAG_MAN_ALL)
					.with_arg(help_arg())
					.with_arg(indent_arg())
					.with_arg(timestamp_arg())
					.with_arg(msg_arg())
			)
		)
		.with_arg(
			AgreeKind::SubCommand(
				Agree::new(
					"print",
					"print",
					"1.2.3",
					"Print a message without a prefix (or with a custom one).",
				)
					.with_flags(FLAG_MAN_ALL)
					.with_arg(help_arg())
					.with_arg(indent_arg())
					.with_arg(stderr_arg())
					.with_arg(timestamp_arg())
					.with_arg(exit_arg())
					.with_arg(
						AgreeKind::option("<NUM>", "Use this color for the prefix. [default: 199]", false)
							.with_short("-c")
							.with_long("--prefix-color")
					)
					.with_arg(
						AgreeKind::option("<PREFIX>", "Set a custom prefix. [default: ]", false)
							.with_short("-p")
							.with_long("--prefix")
					)
					.with_arg(msg_arg())
			)
		)
		.with_arg(builtin("crunched", "Crunched"))
		.with_arg(builtin("debug", "Debug"))
		.with_arg(builtin("done", "Done"))
		.with_arg(builtin("error", "Error"))
		.with_arg(builtin("info", "Info"))
		.with_arg(builtin("notice", "Notice"))
		.with_arg(builtin("success", "Success"))
		.with_arg(builtin("task", "Task"))
		.with_arg(builtin("warning", "Warning"));

	// Our base directory.
	let mut dir: PathBuf = env::var("CARGO_MANIFEST_DIR")
		.ok()
		.and_then(|x| std::fs::canonicalize(x).ok())
		.expect("Missing base directory.");


	// Write MAN pages!
	dir.push("man");
	app.write_man(&dir)
		.unwrap_or_else(|_| panic!("Unable to write MAN script: {:?}", dir));

	// Write BASH completions!
	dir.pop();
	dir.push("completions");
	app.write_bash(&dir)
		.unwrap_or_else(|_| panic!("Unable to write MAN script: {:?}", dir));
}
