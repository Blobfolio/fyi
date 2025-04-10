/*!
# FYI: Build
*/

use argyle::{
	FlagsBuilder,
	KeyWordsBuilder,
};
use fyi_ansi::{
	ansi,
	csi,
};
use fyi_msg::{
	Msg,
	MsgKind,
};
use std::{
	borrow::Cow,
	collections::{
		BTreeMap,
		BTreeSet,
	},
	fmt::Write as _,
	fs::File,
	io::Write,
	path::PathBuf,
};



/// # Manifest.
const MANIFEST: &str = include_str!(env!("CARGO_MANIFEST_PATH"));



/// # Build Arguments and Help Screens
///
/// This generates text for the various help screens to avoid having to do that
/// at runtime. The binary actually ends up slightly smaller this way, too.
///
/// It also generates the keywords used for CLI parsing, again to save some
/// runtime overhead.
fn main() {
	println!("cargo:rerun-if-changed=help");
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

	// Build the CLI arguments.
	write_cli();

	// Build the flags.
	write_flags();

	// Build the help.
	write_help();
}

/// # Output Path.
///
/// Append the sub-path to OUT_DIR and return it.
fn out_path(stub: &str) -> PathBuf {
	std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join(stub)
}

/// # Generate CLI arguments.
fn write_cli() {
	// Main arguments first.
	let mut builder = KeyWordsBuilder::default();
	builder.push_commands(
		MsgKind::ALL.into_iter().filter_map(|k| {
			let cmd = k.command();
			if cmd.is_empty() { None }
			else { Some(cmd) }
		})
	);
	builder.push_keys([
		"-h", "--help",
		"-V", "--version",
	]);
	builder.save(out_path("argyle-kind.rs"));

	// Blank arguments.
	builder = KeyWordsBuilder::default();
	builder.push_keys([
		"-h", "--help",
		"--stderr",
	]);
	builder.push_keys_with_values(["-c", "--count"]);
	builder.save(out_path("argyle-blank.rs"));

	// Message arguments.
	builder = KeyWordsBuilder::default();
	builder.push_keys([
		"-h", "--help",
		"-i", "--indent",
		"--stderr",
		"-t", "--timestamp",
		"-y", "--yes",
	]);
	builder.push_keys_with_values([
		"-c", "--prefix-color",
		"-e", "--exit",
		"-p", "--prefix",
	]);
	builder.save(out_path("argyle-msg.rs"));
}

/// # Write flags (program settings bools).
fn write_flags() {
	FlagsBuilder::new("Flags")
		.with_flag("Indent", Some("# Indent Message."))
		.with_flag("Stderr", Some("# Print to STDERR."))
		.with_flag("Timestamp", Some("# Timestamp Message."))
		.with_flag("Yes", Some("# Assume Yes (No Prompt)."))
		.save(out_path("flags.rs"));
}

/// # Write Help.
fn write_help() {
	let mut help_text = BTreeMap::new();
	help_text.insert("Blank", Cow::Borrowed(include_str!("help/blank.txt")));
	help_text.insert("Custom", Cow::Borrowed(include_str!("help/print.txt")));
	help_text.insert("Confirm", Cow::Borrowed(include_str!("help/confirm.txt")));

	let mut help_subcommands = BTreeSet::new();

	// The rest get manually built.
	for kind in MsgKind::ALL {
		// Before we get to work, let's make sure we remembered to add
		// manual and subcommand entries for it in the Cargo.toml manifest.
		let cmd = kind.command();
		if ! cmd.is_empty() {
			assert!(
				MANIFEST.contains(&format!(r#"cmd = "{cmd}""#)),
				"Manifest missing subcommand entry for {cmd}.",
			);
			assert!(
				MANIFEST.contains(&format!(r#""../release/man/fyi-{cmd}.1.gz""#)),
				"Manifest missing manual entry for {cmd}.",
			);
		}

		// Some of the kinds are already accounted for and can be skipped.
		if kind.is_empty() || matches!(kind, MsgKind::Confirm) { continue; }
		let name = kind.as_str();

		// SUBCOMMANDS in the main help.
		help_subcommands.insert(format!("    {cmd:<12}{name}: Hello World"));

		assert!(
			help_text.insert(kind.as_str(), Cow::Owned(format!(
				include_str!("./help/generic.txt"),
				name,
				Msg::new(kind, "Hello World").as_str(),
				cmd,
			))).is_none(),
			"BUG: duplicate kind help: {kind}",
		);
	}

	// Finish the main help, and add it to the list.
	let mut main = include_str!("./help/help.txt").trim().to_owned();
	main.push('\n');
	for line in help_subcommands {
		main.push_str(&line);
		main.push('\n');
	}
	help_text.insert("None", Cow::Owned(main));

	// Finally, make some code!
	let mut out = format!(
		"#[cold]
/// # Help Page.
///
/// Print the appropriate help screen given the CLI arguments used.
fn helper(cmd: MsgKind) {{
	use std::io::Write;

	let mut handle = std::io::stdout().lock();

	// The top is always the same.
	handle.write_all({help_top:?}.as_bytes()).unwrap();

	// The middle varies by command.
	handle.write_all(match cmd {{",
		help_top=concat!(
			r#"
                      ;\
                     |' \
  _                  ; : ;
 / `-.              /: : |
|  ,-.`-.          ,': : |
\  :  `. `.       ,'-. : |
 \ ;    ;  `-.__,'    `-.|         "#, csi!(199), "FYI", ansi!((cornflower_blue) " v", env!("CARGO_PKG_VERSION")), r#"
  \ ;   ;  :::  ,::'`:.  `.        Simple CLI status messages.
   \ `-. :  `    :.    `.  \
    \   \    ,   ;   ,:    (\
     \   :., :.    ,'o)): ` `-.
    ,/,' ;' ,::"'`.`---'   `.  `-._
  ,/  :  ; '"      `;'          ,--`.
 ;/   :; ;             ,:'     (   ,:)
   ,.,:.    ; ,:.,  ,-._ `.     \""'/
   '::'     `:'`  ,'(  \`._____.-'"'
      ;,   ;  `.  `. `._`-.  \\
      ;:.  ;:       `-._`-.\  \`.
       '`:. :        |' `. `\  ) \
          ` ;:       |    `--\__,'
            '`      ,'
                 ,-'

"#),
	);
	for (kind, help) in help_text {
		writeln!(&mut out, "\t\tMsgKind::{kind} => {help:?},").unwrap();
	}
	writeln!(
		&mut out,
		"\t}}.as_bytes()).unwrap();

	// The bottom is _usually_ the same.
	if ! matches!(cmd, MsgKind::Blank | MsgKind::Confirm | MsgKind::Custom | MsgKind::None) {{
		handle.write_all({help_bottom:?}.as_bytes()).unwrap();
	}}

	let _res = handle.flush();
}}",
		help_bottom=include_str!("help/generic-bottom.txt"),
	).unwrap();

	// Save it.
	File::create(out_path("help.rs"))
		.and_then(|mut f| f.write_all(out.as_bytes()).and_then(|()| f.flush()))
		.expect("Unable to save help.rs.");
}
