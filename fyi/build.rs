/*!
# FYI: Build
*/

use argyle::{
	FlagsBuilder,
	KeyWordsBuilder,
};
use fyi_msg::{
	Msg,
	MsgKind,
};
use std::path::{
	Path,
	PathBuf,
};



/// # Generic Help.
const HELP: &str = include_str!("help/help.txt");

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

	// Handle the top.
	write_help(
		help_path("top"),
		format!(
			r#"
                      ;\
                     |' \
  _                  ; : ;
 / `-.              /: : |
|  ,-.`-.          ,': : |
\  :  `. `.       ,'-. : |
 \ ;    ;  `-.__,'    `-.|         {}{}{}
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
"#,
			"\x1b[38;5;199mFYI\x1b[0;38;5;69m v",
			env!("CARGO_PKG_VERSION"),
			"\x1b[0m"
		).as_bytes()
	);

	// A few files are already static; let's just copy them to the "generated"
	// directory for consistency.
	copy_path("blank");
	copy_path("confirm");
	copy_path("help");
	copy_path("print");
	copy_path("generic-bottom");

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

		// Let's double-check there's a mention in the top-level help's
		// SUBCOMMANDS section for this kind.
		assert!(
			HELP.contains(&format!("{name}: Hello World")),
			"Top-level help is missing subcommand entry for {name}.",
		);

		// And generate the help!
		write_help(
			help_path(&name.to_ascii_lowercase()),
			format!(
				include_str!("./help/generic.txt"),
				name,
				Msg::new(kind, "Hello World").as_str(),
				name.to_lowercase(),
			).as_bytes()
		);
	}
}

/// # Out path.
fn copy_path(name: &str) {
	write_help(help_path(name), &std::fs::read(format!("help/{name}.txt")).expect("Failed to open file."));
}

/// # Output path (help).
fn help_path(name: &str) -> PathBuf {
	PathBuf::from(format!(
		"{}/help-{}.txt",
		std::env::var("OUT_DIR").expect("Missing OUT_DIR"),
		name
	))
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

/// # Write file.
fn write_help<P>(path: P, data: &[u8])
where P: AsRef<Path> {
	use std::io::Write;

	let mut file = std::fs::File::create(path).expect("Unable to create file.");
	file.write_all(data).expect("Write failed.");
	file.write_all(b"\n").expect("Write failed.");
	file.flush().expect("Flush failed.");
}
