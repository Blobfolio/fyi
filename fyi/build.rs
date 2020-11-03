#[cfg(feature = "man")]
/// # Build BASH Completions.
///
/// We can do this in the same run we use for building the MAN pages.
fn main() {
	use fyi_menu::{
		Man,
		ManSection,
		ManSectionItem,
	};
	use std::{
		env,
		path::PathBuf,
	};

	let m: Man = Man::new("FYI", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
		.with_text(
			"DESCRIPTION",
			env!("CARGO_PKG_DESCRIPTION"),
			false
		)
		.with_text(
			"USAGE:",
			"fyi <SUBCOMMAND>",
			true
		)
		.with_section(
			ManSection::list("FLAGS:")
				.with_item(
					ManSectionItem::new("Print help information.")
						.with_key("-h")
						.with_key("--help")
				)
				.with_item(
					ManSectionItem::new("Print version information.")
						.with_key("-V")
						.with_key("--version")
				)
		)
		.with_section(
			ManSection::list("SUBCOMMANDS:")
				.with_item(
					ManSectionItem::new("Print this screen.")
						.with_key("help")
				)
				.with_item(
					ManSectionItem::new("Print blank line(s).")
						.with_key("blank")
				)
				.with_item(
					ManSectionItem::new("Ask a Yes/No question, exiting 0 or 1 respectively.")
						.with_key("confirm")
				)
				.with_item(
					ManSectionItem::new("Print a message without a prefix (or with a custom one).")
						.with_key("print")
				)
				.with_item(
					ManSectionItem::new("Crunched: Hello World")
						.with_key("crunched")
				)
				.with_item(
					ManSectionItem::new("Debug: Hello World")
						.with_key("debug")
				)
				.with_item(
					ManSectionItem::new("Done: Hello World")
						.with_key("done")
				)
				.with_item(
					ManSectionItem::new("Error: Hello World")
						.with_key("error")
				)
				.with_item(
					ManSectionItem::new("Info: Hello World")
						.with_key("info")
				)
				.with_item(
					ManSectionItem::new("Notice: Hello World")
						.with_key("notice")
				)
				.with_item(
					ManSectionItem::new("Success: Hello World")
						.with_key("success")
				)
				.with_item(
					ManSectionItem::new("Task: Hello World")
						.with_key("task")
				)
				.with_item(
					ManSectionItem::new("Warning: Hello World")
						.with_key("warning")
				)
		);

	// We're going to shove this in "fyi/help/fyi.1". If we used
	// `OUT_DIR` like Cargo suggests, we'd never be able to find it to shove
	// it into the `.deb` package.
	let mut path: PathBuf = env::var("CARGO_MANIFEST_DIR")
		.ok()
		.and_then(|x| std::fs::canonicalize(x).ok())
		.expect("Missing MAN directory.");

	path.push("man");
	path.push("fyi.1");

	// Write it!
	m.write(&path)
		.unwrap_or_else(|_| panic!("Unable to write MAN script: {:?}", path));
}

#[cfg(not(feature = "man"))]
/// # Do Nothing.
fn main() {}
