/*!
# FYI: Build
*/

#![allow(unused_mut, reason = "It is conditionally used.")]

use std::{
	borrow::Cow,
	collections::BTreeMap,
	fmt,
	fs::File,
	io::Write,
	path::PathBuf,
};



// COLORS: [(name, hex)].
include!("skel/ansi256.rs");



/// # Build stuff!
fn main() {
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
	build_macro();
	build_table();
}

/// # Build Code Macros.
fn build_macro() {
	use std::fmt::Write;

	let mut out = String::with_capacity(8192);

	out.push_str("\t#[macro_export(local_inner_macros)]
	#[doc(hidden)]
	/// # ANSI CSI Code.
	///
	/// This macro converts a keyword like \"bold\" into an ANSI CSI code
	/// fragment like \"1\".
	macro_rules! csi_code {\n");
	for (tag, code) in macro_codes() {
		writeln!(&mut out, "\t\t({tag}) => ( \"{code}\" );").unwrap();
	}
	out.push_str("\t\t($other:tt) => (
			::std::compile_error!(::std::concat!(
				\"Unrecognized CSI code: \", ::std::stringify!($other)
			))
		);
	}
");

	File::create(out_path("codes.rs"))
		.and_then(|mut f| f.write_all(out.as_bytes()).and_then(|()| f.flush()))
		.expect("Unable to save codes.rs.");
}

/// # Build Color Table.
fn build_table() {
	let out = AnsiTable.to_string();
	File::create(out_path("color-table.rs"))
		.and_then(|mut f| f.write_all(out.as_bytes()).and_then(|()| f.flush()))
		.expect("Unable to save color-table.rs.");
}

/// # Macro Codes.
///
/// Compile and return a list of all the individual ANSI CSI code fragments
/// by keyword.
fn macro_codes() -> BTreeMap<Cow<'static, str>, Cow<'static, str>> {
	let mut out = BTreeMap::new();

	// Special resets.
	out.insert(Cow::Borrowed("reset"), Cow::Borrowed("0"));
	out.insert(Cow::Borrowed("!reset"), Cow::Borrowed("0"));
	out.insert(Cow::Borrowed("!fg"), Cow::Borrowed("39"));

	// Reset bold, dim, underline.
	out.insert(Cow::Borrowed("!bold"), Cow::Borrowed("21"));
	out.insert(Cow::Borrowed("!dim"), Cow::Borrowed("22"));
	out.insert(Cow::Borrowed("!italic"), Cow::Borrowed("23"));
	out.insert(Cow::Borrowed("!underline"), Cow::Borrowed("24"));
	out.insert(Cow::Borrowed("!blink"), Cow::Borrowed("25"));
	out.insert(Cow::Borrowed("!strike"), Cow::Borrowed("29"));

	// Enable bold, dim, underline.
	out.insert(Cow::Borrowed("bold"), Cow::Borrowed("1"));
	out.insert(Cow::Borrowed("dim"), Cow::Borrowed("2"));
	out.insert(Cow::Borrowed("italic"), Cow::Borrowed("3"));
	out.insert(Cow::Borrowed("underline"), Cow::Borrowed("4"));
	out.insert(Cow::Borrowed("blink"), Cow::Borrowed("5"));
	out.insert(Cow::Borrowed("strike"), Cow::Borrowed("9"));

	// Now the colors!
	for (k, v) in (u8::MIN..=u8::MAX).zip(COLORS.iter().map(|(v, _)| v)) {
		let code = ColorCode(k).to_string();

		// Only some colors get named entries.
		if ! v.starts_with("Misc") {
			let v2 = SnakeCase(v).to_string();

			// Reset/enable by name.
			out.insert(Cow::Owned(format!("!{v2}")), Cow::Borrowed("39"));
			out.insert(Cow::Owned(v2), Cow::Owned(code.clone()));
		}

		// Reset/enable by number.
		out.insert(Cow::Owned(format!("!{k}")), Cow::Borrowed("39"));
		out.insert(Cow::Owned(k.to_string()), Cow::Owned(code));
	}

	out
}

/// # Output Path.
///
/// Append the sub-path to OUT_DIR and return it.
fn out_path(stub: &str) -> PathBuf {
	std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join(stub)
}



/// # Print a Pretty ANSI Table.
///
/// This is used to output an HTML "table" showing the approximate ANSI colors
/// and codes.
struct AnsiTable;

impl fmt::Display for AnsiTable {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use std::fmt::Write;

		f.write_str(r##"<div style="display: flex; flex-wrap: wrap; justify-content: flex-start; align-items: stretch; color: #fff; font-size: 13px; font-weight: bold; text-align: center; line-height: 40px;">"##)?;

		let mut buf = String::new();
		for (k, (name, hex)) in COLORS.iter().enumerate() {
			buf.truncate(0);
			if name.starts_with("Misc") {
				write!(&mut buf, "{k}").unwrap();
			}
			else {
				write!(&mut buf, "{k} or {}", SnakeCase(name)).unwrap();
			}

			write!(
				f,
				r##"<div style="width: 40px; height: 40px; margin: 0 1px 1px 0; background: {hex}" title="{buf}">{k:03}</div>"##,
			).unwrap();
		}

		f.write_str("</div>")
	}
}

/// # Color Code.
struct ColorCode(u8);

impl fmt::Display for ColorCode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let num = self.0;
		if num < 8 { write!(f, "3{num}") }
		else if num < 16 { write!(f, "9{}", num - 8) }
		else { write!(f, "38;5;{num}") }
	}
}

/// # Snake Case.
///
/// Convert `PascalCase` into `snake_case`.
struct SnakeCase<'a>(&'a str);

impl fmt::Display for SnakeCase<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use fmt::Write;

		let mut last = '?';
		for c in self.0.chars() {
			if c.is_ascii_uppercase() && last.is_ascii_lowercase() {
				f.write_char('_')?;
			}
			f.write_char(c.to_ascii_lowercase())?;
			last = c;
		}
		Ok(())
	}
}
