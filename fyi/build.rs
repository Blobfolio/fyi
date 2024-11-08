/*!
# FYI: Build
*/

use argyle::KeyWordsBuilder;
use fyi_msg::{
	Msg,
	MsgKind,
};
use std::path::{
	Path,
	PathBuf,
};



/// # Build Arguments and Help Screens
///
/// This generates text for the various help screens to avoid having to do that
/// at runtime. The binary actually ends up slightly smaller this way, too.
///
/// It also generates the keywords used for CLI parsing, again to save some
/// runtime overhead.
pub fn main() {
	println!("cargo:rerun-if-changed=help");
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

	// Build the CLI arguments.
	write_cli();

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
	for &(name, kind) in &[
		("Crunched", MsgKind::Crunched),
		("Debug", MsgKind::Debug),
		("Done", MsgKind::Done),
		("Error", MsgKind::Error),
		("Info", MsgKind::Info),
		("Notice", MsgKind::Notice),
		("Review", MsgKind::Review),
		("Skipped", MsgKind::Skipped),
		("Success", MsgKind::Success),
		("Task", MsgKind::Task),
		("Warning", MsgKind::Warning),
	] {
		write_help(
			help_path(&name.to_lowercase()),
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
	let mut src = std::fs::canonicalize(env!("CARGO_MANIFEST_DIR")).expect("Missing Cargo Dir.");
	src.push(format!("help/{name}.txt"));
	write_help(help_path(name), &std::fs::read(src).expect("Failed to open file."));
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
	builder.push_commands([
		"blank",
		"confirm",
		"crunched",
		"debug",
		"done",
		"error",
		"info",
		"notice",
		"print",
		"review",
		"skipped",
		"success",
		"task",
		"warning",
	]);
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

/// # Write file.
fn write_help<P>(path: P, data: &[u8])
where P: AsRef<Path> {
	use std::io::Write;

	let mut file = std::fs::File::create(path).expect("Unable to create file.");
	file.write_all(data).expect("Write failed.");
	file.write_all(b"\n").expect("Write failed.");
	file.flush().expect("Flush failed.");
}
