/*!
# FYI: Build
*/

#![allow(unused_mut, reason = "It is conditionally used.")]

use fyi_ansi::ansi;
use std::{
	fmt,
	fs::File,
	io::Write,
	path::PathBuf,
};



#[cfg(feature = "bin_kinds")]
/// # Total Kinds.
const NUM_KINDS: usize = 17;

#[cfg(not(feature = "bin_kinds"))]
/// # Total Kinds.
const NUM_KINDS: usize = 15;

/// # Message Kinds.
static KINDS: [(&str, &str); NUM_KINDS] = [
	("None", ""),
	("Aborted",  concat!(ansi!((bold, light_red)     "Aborted:"),  " ")),
	("Confirm",  concat!(ansi!((bold, dark_orange)   "Confirm:"),  " ")),
	("Crunched", concat!(ansi!((bold, light_green)   "Crunched:"), " ")),
	("Debug",    concat!(ansi!((bold, light_cyan)    "Debug:"),    " ")),
	("Done",     concat!(ansi!((bold, light_green)   "Done:"),     " ")),
	("Error",    concat!(ansi!((bold, light_red)     "Error:"),    " ")),
	("Found",    concat!(ansi!((bold, light_green)   "Found:"),    " ")),
	("Info",     concat!(ansi!((bold, light_magenta) "Info:"),     " ")),
	("Notice",   concat!(ansi!((bold, light_magenta) "Notice:"),   " ")),
	("Review",   concat!(ansi!((bold, light_cyan)    "Review:"),   " ")),
	("Skipped",  concat!(ansi!((bold, light_yellow)  "Skipped:"),  " ")),
	("Success",  concat!(ansi!((bold, light_green)   "Success:"),  " ")),
	("Task",     concat!(ansi!((bold, 199)           "Task:"),     " ")),
	("Warning",  concat!(ansi!((bold, light_yellow)  "Warning:"),  " ")),
	#[cfg(feature = "bin_kinds")] ("Blank", ""),
	#[cfg(feature = "bin_kinds")] ("Custom", ""),
];

// COLORS: [(name, hex)].
include!("skel/ansi256.rs");



/// # Build stuff!
fn main() {
	println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
	build_ansi_color();
	build_msg_kinds();
}

/// # Build ANSI Color Enum.
///
/// This enum has 256 variants; it's a lot less painful to generate the more
/// verbose routines programmatically. Haha.
fn build_ansi_color() {
	use fmt::Write;

	// This'll be kinda big, so let's start with a decent buffer.
	let mut out = String::with_capacity(65_536);

	// Start with the definition.
	writeln!(
		&mut out,
		"#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
/// # ANSI Colors (8/16/256).
///
/// This enum is used to print colored text to ANSI-capable[^note] terminals.
///
/// Owing to deviations from _proper_ standards, only some variants have nice
/// names; the rest are simply designated by code (e.g. [`AnsiColor::Misc217`]
/// for `#217`, a sort of mortician's blush).
///
/// [^note]: Most modern software will happily render the full range, but if
/// (ancient) backward compatibility is a concern, stick with the first sixteen
/// choices, as they've been around _forever_.
///
/// ## Reference
///
/// The actual color rendered will vary by software, but should look something
/// like the following:
///
/// {}
pub enum AnsiColor {{",
		AnsiTable
	).unwrap();
	for (k, (v, h)) in COLORS.iter().enumerate() {
		// Give it a title.
		if v.starts_with("Misc") {
			writeln!(&mut out, "\t/// # Color #{k}.").unwrap();
		}
		else {
			writeln!(
				&mut out,
				"\t/// # {}{}.",
				TitleCase(v),
				if k < 8 { " (8)" } else if k < 16 { " (16)" } else { "" },
			).unwrap();
		}

		// And because the color names suck, print the corresponding hex value.
		writeln!(&mut out, "\t///\n\t/// This is roughly equivalent to the RGB `{h}`.")
			.unwrap();

		// And the arm.
		writeln!(&mut out, "\t{v} = {k}_u8,").unwrap();

		// Add an extra line.
		if k != usize::from(u8::MAX) { out.push('\n'); }
	}
	out.push_str("}\n");

	// Open an impl block for it.
	out.push_str("#[expect(clippy::too_many_lines, reason = \"There are 256 colors to deal with.\")]
impl AnsiColor {\n");

	// Let's do the from_u8 method first.
	out.push_str("\t#[must_use]
	/// # From `u8`.
	///
	/// Convert a `u8` to the corresponding `AnsiColor`.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::AnsiColor;
	///
	/// assert_eq!(
	///     AnsiColor::from_u8(5),
	///     AnsiColor::Magenta,
	/// );
	/// ```
	pub const fn from_u8(num: u8) -> Self {
		match num {\n");
	for (k, v) in COLORS.iter().map(|(v, _)| v).enumerate() {
		writeln!(&mut out, "\t\t\t{k} => Self::{v},").unwrap();
	}
	out.push_str("\t\t}\n\t}\n");

	// Now let's handle the as_str method.
	out.push_str("\t#[must_use]
	/// # As String Slice.
	///
	/// Return the full ANSI sequence for the selected color as a string
	/// slice.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::AnsiColor;
	///
	/// assert_eq!(
	///     AnsiColor::DarkOrange.as_str(),
	///     \"\\x1b[38;5;208m\",
	/// );
	/// ```
	pub const fn as_str(self) -> &'static str {
		match self {\n");
	for (k, v) in COLORS.iter().map(|(v, _)| v).enumerate() {
		writeln!(&mut out, "\t\t\tSelf::{v} => fyi_ansi::csi!({k}),").unwrap();
	}
	out.push_str("\t\t}\n\t}\n");

	// Now let's handle the as_str_bold method.
	out.push_str("\t#[must_use]
	/// # As String Slice (Bold).
	///
	/// Return the full ANSI sequence for the selected color as a string
	/// slice, bolded.
	pub(crate) const fn as_str_bold(self) -> &'static str {
		match self {\n");
	for (k, v) in COLORS.iter().map(|(v, _)| v).enumerate() {
		writeln!(&mut out, "\t\t\tSelf::{v} => fyi_ansi::csi!(bold, {k}),").unwrap();
	}
	out.push_str("\t\t}\n\t}\n");

	// Close the block.
	out.push_str("}\n");

	// Save it!
	File::create(out_path("ansi-color.rs"))
		.and_then(|mut f| f.write_all(out.as_bytes()).and_then(|()| f.flush()))
		.expect("Unable to save ansi-color.rs");
}

/// # Build/Save `MsgKind`.
///
/// The `MsgKind` enum doesn't feel all that complicated, but there are a lot
/// of little ways inconsistencies can creep in. Building the trickier pieces
/// programmatically helps ensure nothing is missed.
///
/// This generates code for the definition, an `ALL` constant,
/// `MsgKind::as_str_prefix`, and the `Msg::kind` helpers.
fn build_msg_kinds() {
	use std::fmt::Write;

	/// # Hidden Kinds.
	const HIDDEN: [&str; 2] = ["Blank", "Custom"];

	let mut out = String::with_capacity(8192);
	out.push_str(r#"#[expect(missing_docs, reason = "Redudant.")]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
/// # Message Kind.
///
/// This enum contains built-in prefixes for [`Msg`](crate::Msg). These are
/// generally only used to initiate a new message with this prefix, like:
///
/// ## Examples
///
/// ```
/// use fyi_msg::{Msg, MsgKind};
///
/// // Error: Oh no!
/// assert_eq!(
///     Msg::new(MsgKind::Error, "Oh no!"),
///     MsgKind::Error.into_msg("Oh no!"),
/// );
/// ```
///
/// Most kinds have their own dedicated [`Msg`] helper method which, unlike the
/// previous examples, comes with a line break at the end.
///
/// ```
/// use fyi_msg::{Msg, MsgKind};
///
/// // Error: Oh no!\n
/// assert_eq!(
///     Msg::error("Oh no!"),
///     Msg::new(MsgKind::Error, "Oh no!").with_newline(true),
/// );
/// ```
pub enum MsgKind {"#);
	for (kind, _) in KINDS {
		if kind != "None" { out.push('\n'); }
		if HIDDEN.contains(&kind) { out.push_str("\t#[doc(hidden)]\n"); }
		writeln!(&mut out, "\t{kind},").unwrap();
	}
	out.push_str("}\n");

	// Add a constant containing all kinds to make iteration easier.
	writeln!(
		&mut out,
		"impl MsgKind {{
	/// # All Variants.
	///
	/// This array can be used to cheaply iterate through all message kinds.
	pub const ALL: [Self; {NUM_KINDS}] = ["
	).unwrap();
	for chunk in KINDS.chunks(8) {
		out.push_str("\t\t");
		for (kind, _) in chunk { write!(&mut out, "Self::{kind}, ").unwrap(); }
		out.push('\n');
	}
	out.push_str("\t];\n");

	// And a crate-wide method to expose the preformatted prefix string.
	#[cfg(feature = "bin_kinds")]      let wild = "_";
	#[cfg(not(feature = "bin_kinds"))] let wild = "Self::None";

	out.push_str("\t#[inline]
	#[must_use]
	/// # As String Slice (Prefix).
	///
	/// Return the kind as a string slice, formatted and with a trailing `\": \"`,
	/// same as [`Msg`] uses for prefixes.
	pub(crate) const fn as_str_prefix(self) -> &'static str {
		match self {\n");
	for (kind, prefix) in KINDS {
		// Skip empties.
		if prefix.is_empty() {continue; }

		// While we're here, check the predictable parts were typed correctly.
		assert!(
			prefix.starts_with("\x1b[1;") &&
			prefix.ends_with(&format!("m{kind}:\x1b[0m ")),
			"BUG: {kind}::as_str_prefix is wrong!",
		);

		writeln!(&mut out, "\t\t\tSelf::{kind} => {prefix:?},").unwrap();
	}
	writeln!(
		&mut out,
		"\t\t\t{wild} => \"\",
		}}
	}}
}}").unwrap();

	// Generate helper methods for (most) of the kinds. (Might as well do this
	// here.)
	out.push_str("/// ## [`MsgKind`] One-Shots.
impl Msg {\n");
	for (kind, prefix) in KINDS {
		// Skip the empties and "Confirm" (since it has a macro).
		if prefix.is_empty() || kind == "Confirm" { continue; }

		let prefix_len = prefix.len();
		writeln!(
			&mut out,
			"\t#[must_use]
	/// # New {kind}.
	///
	/// Create a new [`Msg`] with a built-in [`MsgKind::{kind}`] prefix _and_ trailing line break.
	///
	/// ## Examples.
	///
	/// ```
	/// use fyi_msg::{{Msg, MsgKind}};
	///
	/// assert_eq!(
	///     Msg::{kind_low}(\"Hello World\"),
	///     Msg::new(MsgKind::{kind}, \"Hello World\").with_newline(true),
	/// );
	/// ```
	pub fn {kind_low}<S: AsRef<str>>(msg: S) -> Self {{
			// Glue it all together.
			let msg = msg.as_ref();
			let m_end = {prefix_len} + msg.len();
			let mut inner = String::with_capacity(m_end + 1);
			inner.push_str({prefix:?});
			inner.push_str(msg);
			inner.push('\\n');

			// Done!
			Self {{
				inner,
				toc: super::toc!({prefix_len}, m_end, true),
			}}
	}}",
			kind=kind,
			kind_low=kind.to_ascii_lowercase(),
		).unwrap();
	}
	out.push_str("}\n");

	File::create(out_path("msg-kinds.rs"))
		.and_then(|mut f| f.write_all(out.as_bytes()).and_then(|()| f.flush()))
		.expect("Unable to save msg-kinds.rs");
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
		f.write_str(r##"<div style="display: flex; flex-wrap: wrap; justify-content: flex-start; align-items: stretch; color: #fff; font-size: 13px; font-weight: bold; text-align: center; line-height: 40px;">"##)?;
		for (k, (name, hex)) in COLORS.iter().enumerate() {
			write!(
				f,
				r##"<div style="width: 40px; height: 40px; margin: 0 1px 1px 0; background: {hex}" title="{name} ({k:03})">{k:03}</div>"##,
			).unwrap();
		}
		f.write_str("</div>")
	}
}



/// # Title Case Formatting.
///
/// This is used to convert the PascalCase enum variant names into title case
/// for the documentation. (It just adds a space between lower/UPPER.)
struct TitleCase<'a>(&'a str);

impl fmt::Display for TitleCase<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use fmt::Write;

		let mut last = '?';
		for c in self.0.chars() {
			if c.is_ascii_uppercase() && last.is_ascii_lowercase() {
				f.write_char(' ')?;
			}
			f.write_char(c)?;
			last = c;
		}
		Ok(())
	}
}
