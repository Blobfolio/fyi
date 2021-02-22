/*!
# FYI: Build
*/

use fyi_msg::{
	Msg,
	MsgKind,
};
use std::path::{
	Path,
	PathBuf,
};



/// # Build Help Screens
///
/// This generates text for the various help screens to avoid having to do that
/// at runtime. The binary actually ends up slightly smaller this way, too.
pub fn main() {
	println!("cargo:rerun-if-changed=help/stock");
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

	// Handle the top.
	write_help(
		out_path("top"),
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
		("Success", MsgKind::Success),
		("Task", MsgKind::Task),
		("Warning", MsgKind::Warning),
	] {
		write_help(
			out_path(&name.to_lowercase()),
			format!(
				include_str!("./help/stock/generic.txt"),
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
	src.push("help/stock");
	src.push(format!("{}.txt", name));
	write_help(out_path(name), &std::fs::read(src).expect("Failed to open file."));
}

/// # Out path.
fn out_path(name: &str) -> PathBuf {
	let mut out = std::fs::canonicalize(env!("CARGO_MANIFEST_DIR")).expect("Missing Cargo Dir.");
	out.push("help/generated");
	out.push(format!("{}.txt", name));
	out
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
