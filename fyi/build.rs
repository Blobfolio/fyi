/*!
# Build
*/

extern crate clap;
extern crate regex;

include!("src/menu.rs");



/// Build tasks.
fn main() {
	completions();
	man();
}

/// Bash and Zsh completions.
fn completions() {
	use clap::Shell;
	use std::path::PathBuf;

	// Store the completions here.
	let outdir: PathBuf = PathBuf::from("../release/completions");
	if false == outdir.is_dir() {
		std::fs::create_dir(&outdir).expect("Unable to create temporary completion directory.");
	}

	// Complete it!
	menu().gen_completions(
		"fyi",
		Shell::Bash,
		outdir
	);
}

/// Man page.
///
/// This is a shitty help2man conversion; some day Clap will include
/// this natively and life will be better.
fn man() {
	use regex::Regex;
	use std::{
		fs::File,
		path::PathBuf,
		process::{
			Command,
			Stdio,
		},
		io::Write,
	};

	// The app we just built.
	let app: PathBuf = PathBuf::from("../target/release/fyi")
		.canonicalize()
		.expect("Missing compiled binary.");

	// Run Help2Man.
	let output = Command::new("help2man")
		.args(&[
			"-N",
			app.to_str().unwrap(),
		])
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.output()
		.expect("Help2Man failed.");

	// We need to Regex some badness away before saving it.
	let raw: String = String::from_utf8(output.stdout)
		.expect("Missing manual.");
	let re: Regex = Regex::new(r"FYI [0-9.]+\nBlobfolio, LLC. <hello@blobfolio.com>\n")
		.unwrap();

	// Save the output.
	let mut file = File::create(PathBuf::from("../release/man/fyi.1"))
		.expect("Missing manual.");
	file.write_all(re.replace_all(&raw, "").as_bytes()).unwrap();
	file.flush().unwrap();
	drop(file);
}
