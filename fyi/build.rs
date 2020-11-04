#[cfg(not(feature = "man"))]
/// # Do Nothing.
///
/// We only need to rebuild stuff for new releases. The "man" feature is
/// basically used to figure that out.
fn main() {}



#[cfg(feature = "man")]
/// # Build MAN Page.
fn main() {
	use fyi_menu::{
		Agree,
		AgreeSection,
		AgreeKind,
		FLAG_MAN_NAME,
		FLAG_MAN_DESCRIPTION,
	};
	use std::{
		env,
		path::PathBuf,
	};

	let app: Agree = Agree::new(
		"FYI",
		env!("CARGO_PKG_NAME"),
		env!("CARGO_PKG_VERSION"),
		env!("CARGO_PKG_DESCRIPTION"),
	)
		.with_flags(FLAG_MAN_NAME | FLAG_MAN_DESCRIPTION)
		.with_section(
			AgreeSection::new("USAGE:", true)
				.with_item(
					AgreeKind::paragraph("fyi <SUBCOMMAND> [FLAGS]")
						.with_line("fyi <SUBCOMMAND> --help")
				)
		)
		.with_section(
			AgreeSection::new("FLAGS:", true)
				.with_item(
					AgreeKind::switch("Print help information.")
						.with_short("-h")
						.with_long("--help")
				)
				.with_item(
					AgreeKind::switch("Print program version.")
						.with_short("-V")
						.with_long("--version")
				)
		)
		.with_section(
			AgreeSection::new("SUBCOMMANDS:", true)
				.with_item(AgreeKind::item("help", "Print help information."))
				.with_item(AgreeKind::item("blank", "Print blank line(s)."))
				.with_item(AgreeKind::item(
					"confirm",
					"Ask a Yes/No question, exiting 0 or 1 respectively."
				))
				.with_item(AgreeKind::item(
					"print",
					"Print a message without a prefix (or with a custom one)."
				))
				.with_item(AgreeKind::item("crunched", "Crunched: Hello World"))
				.with_item(AgreeKind::item("debug", "Debug: Hello World"))
				.with_item(AgreeKind::item("done", "Done: Hello World"))
				.with_item(AgreeKind::item("error", "Error: Hello World"))
				.with_item(AgreeKind::item("info", "Info: Hello World"))
				.with_item(AgreeKind::item("notice", "Notice: Hello World"))
				.with_item(AgreeKind::item("success", "Success: Hello World"))
				.with_item(AgreeKind::item("task", "Task: Hello World"))
				.with_item(AgreeKind::item("warning", "Warning: Hello World"))
		);

	// Chuck this in ../man.
	let mut dir: PathBuf = env::var("CARGO_MANIFEST_DIR")
		.ok()
		.and_then(|x| std::fs::canonicalize(x).ok())
		.expect("Missing base directory.");

	dir.push("man");

	// Write it!
	app.write_man(&dir)
		.unwrap_or_else(|_| panic!("Unable to write MAN script: {:?}", dir));
}
