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
	build_macro();
	build_table();
}

/// # Build Code Macros.
fn build_macro() {
	use std::fmt::Write;

	let mut out = String::with_capacity(65_536);

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
	out.push_str("\t}\n");

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
	out.insert(Cow::Borrowed("!underline"), Cow::Borrowed("24"));

	// Enable bold, dim, underline.
	out.insert(Cow::Borrowed("bold"), Cow::Borrowed("1"));
	out.insert(Cow::Borrowed("dim"), Cow::Borrowed("2"));
	out.insert(Cow::Borrowed("underline"), Cow::Borrowed("4"));

	// Now the colors!
	for (k, v) in (u8::MIN..=u8::MAX).zip(COLORS) {
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
		for (k, color) in HEX.iter().enumerate() {
			buf.truncate(0);
			if COLORS[k].starts_with("Misc") {
				write!(&mut buf, "{k}").unwrap();
			}
			else {
				write!(&mut buf, "{k} or {}", SnakeCase(COLORS[k])).unwrap();
			}

			write!(
				f,
				r##"<div style="width: 40px; height: 40px; margin: 0 1px 1px 0; background: {color}" title="{}">{k:03}</div>"##,
				buf,
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
