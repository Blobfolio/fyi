/*!
# FYI: Build
*/

#![allow(unused_mut, reason = "It is conditionally used.")]

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
	("Aborted", "\x1b[1;91mAborted:\x1b[0m "),
	("Confirm", "\x1b[1;38;5;208mConfirm:\x1b[0m "),
	("Crunched", "\x1b[1;92mCrunched:\x1b[0m "),
	("Debug", "\x1b[1;96mDebug:\x1b[0m "),
	("Done", "\x1b[1;92mDone:\x1b[0m "),
	("Error", "\x1b[1;91mError:\x1b[0m "),
	("Found", "\x1b[1;92mFound:\x1b[0m "),
	("Info", "\x1b[1;95mInfo:\x1b[0m "),
	("Notice", "\x1b[1;95mNotice:\x1b[0m "),
	("Review", "\x1b[1;96mReview:\x1b[0m "),
	("Skipped", "\x1b[1;93mSkipped:\x1b[0m "),
	("Success", "\x1b[1;92mSuccess:\x1b[0m "),
	("Task", "\x1b[1;38;5;199mTask:\x1b[0m "),
	("Warning", "\x1b[1;93mWarning:\x1b[0m "),
	#[cfg(feature = "bin_kinds")] ("Blank", ""),
	#[cfg(feature = "bin_kinds")] ("Custom", ""),
];

// Build up a list of 256 ANSI colors, naming the ones we can.
static COLORS: [&str; 256] = [
	"Black",
	"Red",
	"Green",
	"Yellow",
	"Blue",
	"Magenta",
	"Cyan",
	"LightGrey",
	"DarkGrey",
	"LightRed",
	"LightGreen",
	"LightYellow",
	"LightBlue",
	"LightMagenta",
	"LightCyan",
	"White",
	"Misc016",
	"NavyBlue",
	"DarkBlue",
	"Misc019", "Misc020", "Misc021",
	"DarkGreen",
	"Misc023", "Misc024", "Misc025", "Misc026", "Misc027", "Misc028", "Misc029",
	"Misc030", "Misc031", "Misc032", "Misc033", "Misc034", "Misc035",
	"DarkCyan",
	"LightSeaGreen",
	"Misc038", "Misc039", "Misc040", "Misc041", "Misc042", "Misc043",
	"DarkTurquoise",
	"Misc045", "Misc046", "Misc047", "Misc048",
	"MediumSpringGreen",
	"Misc050", "Misc051", "Misc052", "Misc053", "Misc054", "Misc055", "Misc056",
	"BlueViolet",
	"Misc058", "Misc059", "Misc060", "Misc061", "Misc062", "Misc063", "Misc064",
	"Misc065", "Misc066",
	"SteelBlue",
	"Misc068",
	"CornflowerBlue",
	"Misc070", "Misc071", "Misc072", "Misc073", "Misc074", "Misc075", "Misc076",
	"Misc077", "Misc078", "Misc079",
	"MediumTurquoise",
	"Misc081", "Misc082", "Misc083", "Misc084", "Misc085", "Misc086", "Misc087",
	"Misc088", "Misc089", "Misc090", "Misc091", "Misc092", "Misc093", "Misc094",
	"Misc095", "Misc096", "Misc097", "Misc098", "Misc099", "Misc100", "Misc101",
	"Misc102",
	"LightSlateGrey",
	"MediumPurple",
	"LightSlateBlue",
	"Misc106", "Misc107",
	"DarkSeaGreen",
	"Misc109", "Misc110", "Misc111", "Misc112", "Misc113", "Misc114", "Misc115",
	"Misc116", "Misc117", "Misc118", "Misc119", "Misc120", "Misc121", "Misc122",
	"Misc123", "Misc124", "Misc125",
	"MediumVioletRed",
	"Misc127", "Misc128", "Misc129", "Misc130", "Misc131", "Misc132", "Misc133",
	"Misc134", "Misc135",
	"DarkGoldenrod",
	"Misc137",
	"RosyBrown",
	"Misc139", "Misc140", "Misc141", "Misc142",
	"DarkKhaki",
	"Misc144", "Misc145", "Misc146",
	"LightSteelBlue",
	"Misc148", "Misc149", "Misc150", "Misc151", "Misc152", "Misc153",
	"GreenYellow",
	"Misc155", "Misc156", "Misc157", "Misc158", "Misc159", "Misc160", "Misc161",
	"Misc162", "Misc163", "Misc164", "Misc165", "Misc166", "Misc167", "Misc168",
	"Misc169",
	"Orchid",
	"Misc171", "Misc172", "Misc173", "Misc174", "Misc175", "Misc176",
	"Violet",
	"Misc178", "Misc179",
	"Tan",
	"Misc181", "Misc182", "Misc183", "Misc184", "Misc185", "Misc186", "Misc187",
	"Misc188", "Misc189", "Misc190", "Misc191", "Misc192", "Misc193", "Misc194",
	"Misc195", "Misc196", "Misc197", "Misc198", "Misc199", "Misc200", "Misc201",
	"Misc202", "Misc203", "Misc204", "Misc205", "Misc206", "Misc207",
	"DarkOrange",
	"Misc209",
	"LightCoral",
	"Misc211", "Misc212", "Misc213", "Misc214",
	"SandyBrown",
	"Misc216", "Misc217", "Misc218", "Misc219", "Misc220", "Misc221", "Misc222",
	"Misc223", "Misc224", "Misc225", "Misc226", "Misc227", "Misc228", "Misc229",
	"Misc230", "Misc231", "Misc232", "Misc233", "Misc234", "Misc235", "Misc236",
	"Misc237", "Misc238", "Misc239", "Misc240", "Misc241", "Misc242", "Misc243",
	"Misc244", "Misc245", "Misc246", "Misc247", "Misc248", "Misc249", "Misc250",
	"Misc251", "Misc252", "Misc253", "Misc254", "Misc255",
];

/// # ANSI-to-Web Conversion Table.
static HEX: [&str; 256] = [
	"#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#c0c0c0", "#808080", "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
	"#000000", "#00005f", "#000087", "#0000af", "#0000d7", "#0000ff", "#005f00", "#005f5f", "#005f87", "#005faf", "#005fd7", "#005fff", "#008700", "#00875f", "#008787", "#0087af",
	"#0087d7", "#0087ff", "#00af00", "#00af5f", "#00af87", "#00afaf", "#00afd7", "#00afff", "#00d700", "#00d75f", "#00d787", "#00d7af", "#00d7d7", "#00d7ff", "#00ff00", "#00ff5f",
	"#00ff87", "#00ffaf", "#00ffd7", "#00ffff", "#5f0000", "#5f005f", "#5f0087", "#5f00af", "#5f00d7", "#5f00ff", "#5f5f00", "#5f5f5f", "#5f5f87", "#5f5faf", "#5f5fd7", "#5f5fff",
	"#5f8700", "#5f875f", "#5f8787", "#5f87af", "#5f87d7", "#5f87ff", "#5faf00", "#5faf5f", "#5faf87", "#5fafaf", "#5fafd7", "#5fafff", "#5fd700", "#5fd75f", "#5fd787", "#5fd7af",
	"#5fd7d7", "#5fd7ff", "#5fff00", "#5fff5f", "#5fff87", "#5fffaf", "#5fffd7", "#5fffff", "#870000", "#87005f", "#870087", "#8700af", "#8700d7", "#8700ff", "#875f00", "#875f5f",
	"#875f87", "#875faf", "#875fd7", "#875fff", "#878700", "#87875f", "#878787", "#8787af", "#8787d7", "#8787ff", "#87af00", "#87af5f", "#87af87", "#87afaf", "#87afd7", "#87afff",
	"#87d700", "#87d75f", "#87d787", "#87d7af", "#87d7d7", "#87d7ff", "#87ff00", "#87ff5f", "#87ff87", "#87ffaf", "#87ffd7", "#87ffff", "#af0000", "#af005f", "#af0087", "#af00af",
	"#af00d7", "#af00ff", "#af5f00", "#af5f5f", "#af5f87", "#af5faf", "#af5fd7", "#af5fff", "#af8700", "#af875f", "#af8787", "#af87af", "#af87d7", "#af87ff", "#afaf00", "#afaf5f",
	"#afaf87", "#afafaf", "#afafd7", "#afafff", "#afd700", "#afd75f", "#afd787", "#afd7af", "#afd7d7", "#afd7ff", "#afff00", "#afff5f", "#afff87", "#afffaf", "#afffd7", "#afffff",
	"#d70000", "#d7005f", "#d70087", "#d700af", "#d700d7", "#d700ff", "#d75f00", "#d75f5f", "#d75f87", "#d75faf", "#d75fd7", "#d75fff", "#d78700", "#d7875f", "#d78787", "#d787af",
	"#d787d7", "#d787ff", "#d7af00", "#d7af5f", "#d7af87", "#d7afaf", "#d7afd7", "#d7afff", "#d7d700", "#d7d75f", "#d7d787", "#d7d7af", "#d7d7d7", "#d7d7ff", "#d7ff00", "#d7ff5f",
	"#d7ff87", "#d7ffaf", "#d7ffd7", "#d7ffff", "#ff0000", "#ff005f", "#ff0087", "#ff00af", "#ff00d7", "#ff00ff", "#ff5f00", "#ff5f5f", "#ff5f87", "#ff5faf", "#ff5fd7", "#ff5fff",
	"#ff8700", "#ff875f", "#ff8787", "#ff87af", "#ff87d7", "#ff87ff", "#ffaf00", "#ffaf5f", "#ffaf87", "#ffafaf", "#ffafd7", "#ffafff", "#ffd700", "#ffd75f", "#ffd787", "#ffd7af",
	"#ffd7d7", "#ffd7ff", "#ffff00", "#ffff5f", "#ffff87", "#ffffaf", "#ffffd7", "#ffffff", "#080808", "#121212", "#1c1c1c", "#262626", "#303030", "#3a3a3a", "#444444", "#4e4e4e",
	"#585858", "#626262", "#6c6c6c", "#767676", "#808080", "#8a8a8a", "#949494", "#9e9e9e", "#a8a8a8", "#b2b2b2", "#bcbcbc", "#c6c6c6", "#d0d0d0", "#dadada", "#e4e4e4", "#eeeeee",
];



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
	for (k, (v, h)) in COLORS.iter().zip(HEX.iter()).enumerate() {
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
	for (k, v) in COLORS.iter().enumerate() {
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
	for (k, v) in COLORS.iter().enumerate() {
		if k < 8 {
			writeln!(&mut out, "\t\t\tSelf::{v} => \"\\x1b[3{k}m\",").unwrap();
		}
		else if k < 16 {
			writeln!(&mut out, "\t\t\tSelf::{v} => \"\\x1b[9{}m\",", k - 8).unwrap();
		}
		else {
			writeln!(&mut out, "\t\t\tSelf::{v} => \"\\x1b[38;5;{k}m\",").unwrap();
		}
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
	for (k, v) in COLORS.iter().enumerate() {
		if k < 8 {
			writeln!(&mut out, "\t\t\tSelf::{v} => \"\\x1b[1;3{k}m\",").unwrap();
		}
		else if k < 16 {
			writeln!(&mut out, "\t\t\tSelf::{v} => \"\\x1b[1;9{}m\",", k - 8).unwrap();
		}
		else {
			writeln!(&mut out, "\t\t\tSelf::{v} => \"\\x1b[1;38;5;{k}m\",").unwrap();
		}
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

	let mut out = String::with_capacity(2048);
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
		for (k, color) in HEX.iter().enumerate() {
			write!(
				f,
				r##"<div style="width: 40px; height: 40px; margin: 0 1px 1px 0; background: {color}" title="{} ({k:03})">{k:03}</div>"##,
				COLORS[k],
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
