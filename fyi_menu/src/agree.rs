/*!
# FYI Menu: Agree
*/

use std::path::Path;
use std::path::PathBuf;



/// # Man Flag: Generate All Auto Sections.
pub const FLAG_MAN_ALL: u8 =         0b0011_1111;

/// # Man Flag: Generate DESCRIPTION Section.
pub const FLAG_MAN_DESCRIPTION: u8 = 0b0000_0001;

/// # Man Flag: Generate NAME Section.
pub const FLAG_MAN_NAME: u8 =        0b0000_0010;

/// # Man Flag: Generate USAGE Section.
pub const FLAG_MAN_USAGE: u8 =       0b0000_0100;

/// # Man Flag: Generate FLAGS Section.
pub const FLAG_MAN_FLAGS: u8 =       0b0000_1000;

/// # Man Flag: Generate OPTIONS Section.
pub const FLAG_MAN_OPTIONS: u8 =     0b0001_0000;

/// # Man Flag: Generate ARGS Section.
pub const FLAG_MAN_ARGS: u8 =        0b0010_0000;




#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Agreement Kind.
pub enum AgreeKind {
	/// # Switch.
	///
	/// This is a flag (true/false) with a short and/or long key and
	/// description.
	Switch(AgreeSwitch),

	/// # Option.
	///
	/// This is an option (that takes a value) with a short and/or long key and
	/// description. The value can be open-ended or path-based.
	Option(AgreeOption),

	/// # Argument.
	///
	/// A trailing argument with a label and description.
	Arg(AgreeItem),

	/// # Subcommand.
	///
	/// Note: This is not yet supported.
	SubCommand(Agree),

	/// # Miscellaneous K/V Item.
	///
	/// This is a miscellaneous key/value pair that can be used for MAN lists
	/// of non-usage topics.
	Item(AgreeItem),

	/// # Paragraph.
	///
	/// This is a text block with one or more lines.
	Paragraph(AgreeParagraph),
}

impl AgreeKind {
	/// # New Switch.
	///
	/// This is a convenience method to register a new [`AgreeKind::Switch`].
	pub fn switch<S>(description: S) -> Self
	where S: Into<String> {
		Self::Switch(AgreeSwitch::new(description))
	}

	/// # New Option.
	///
	/// This is a convenience method to register a new [`AgreeKind::Option`].
	pub fn option<S>(value: S, description: S, path: bool) -> Self
	where S: Into<String> {
		Self::Option(AgreeOption::new(value, description, path))
	}

	/// # New Argument.
	///
	/// This is a convenience method to register a new [`AgreeKind::Arg`].
	pub fn arg<S>(name: S, description: S) -> Self
	where S: Into<String> {
		Self::Arg(AgreeItem::new(name, description))
	}

	/// # New K/V Item.
	///
	/// This is a convenience method to register a new [`AgreeKind::Item`].
	pub fn item<S>(name: S, description: S) -> Self
	where S: Into<String> {
		Self::Item(AgreeItem::new(name, description))
	}

	/// # New Argument.
	///
	/// This is a convenience method to register a new [`AgreeKind::Paragraph`].
	pub fn paragraph<S>(line: S) -> Self
	where S: Into<String> {
		Self::Paragraph(AgreeParagraph::new(line))
	}

	/// # With Line.
	///
	/// This is a convenience method that passes through to the underlying
	/// data's `with_line()` method, if any.
	pub fn with_line<S>(self, line: S) -> Self
	where S: Into<String> {
		match self {
			Self::Paragraph(s) => Self::Paragraph(s.with_line(line)),
			_ => self,
		}
	}

	/// # With Long.
	///
	/// This is a convenience method that passes through to the underlying
	/// data's `with_long()` method, if any.
	pub fn with_long<S>(self, key: S) -> Self
	where S: Into<String> {
		match self {
			Self::Switch(s) => Self::Switch(s.with_long(key)),
			Self::Option(s) => Self::Option(s.with_long(key)),
			_ => self,
		}
	}

	/// # With Short.
	///
	/// This is a convenience method that passes through to the underlying
	/// data's `with_short()` method, if any.
	pub fn with_short<S>(self, key: S) -> Self
	where S: Into<String> {
		match self {
			Self::Switch(s) => Self::Switch(s.with_short(key)),
			Self::Option(s) => Self::Option(s.with_short(key)),
			_ => self,
		}
	}

	/// # BASH Helper.
	///
	/// This formats the BASH flag/option conditions, if any, for the
	/// completion script. This partial value is incorporated into the full
	/// output produced by [`Agree::bash`].
	fn bash(&self) -> String {
		match self {
			Self::Switch(s) => match (s.short.as_deref(), s.long.as_deref()) {
				(Some(s), Some(l)) => include_str!("../skel/basher.cond2.txt")
					.replace("%SHORT%", s)
					.replace("%LONG%", l),
				(None, Some(k)) | (Some(k), None) => include_str!("../skel/basher.cond1.txt")
					.replace("%KEY%", k),
				(None, None) => String::new(),
			},
			Self::Option(o) => match (o.short.as_deref(), o.long.as_deref()) {
				(Some(s), Some(l)) => include_str!("../skel/basher.cond2.txt")
					.replace("%SHORT%", s)
					.replace("%LONG%", l),
				(None, Some(k)) | (Some(k), None) => include_str!("../skel/basher.cond1.txt")
					.replace("%KEY%", k),
				(None, None) => String::new(),
			},
			// TODO: Handle Subcommands.
			_ => String::new(),
		}
	}

	/// # MAN Helper.
	///
	/// This formats the MAN line(s) for the underlying data. This partial
	/// value is incorporated into the full output produced by [`Agree::man`].
	fn man(&self, indent: bool) -> String {
		match self {
			Self::Switch(i) => {
				let mut out: String = self.man_tagline();
				if out.is_empty() {
					format!(".TP\n{}", &i.description)
				}
				else {
					out.push('\n');
					out.push_str(&i.description);
					out
				}
			},
			Self::Option(i) => {
				let mut out: String = self.man_tagline();
				if out.is_empty() {
					format!(".TP\n{}", &i.description)
				}
				else {
					out.push('\n');
					out.push_str(&i.description);
					out
				}
			},
			Self::Arg(i) | Self::Item(i) => {
				let mut out: String = self.man_tagline();
				if out.is_empty() {
					format!(".TP\n{}", &i.description)
				}
				else {
					out.push('\n');
					out.push_str(&i.description);
					out
				}
			},
			Self::Paragraph(i) => {
				if indent {
					format!(".TP\n{}", i.p.join("\n.RE\n"))
				}
				else {
					i.p.join("\n.RE\n")
				}
			},
			_ => String::new(),
		}
	}

	/// # MAN Helper (Tagline).
	///
	/// This formats the key/value line for the MAN output. This gets
	/// incorporated into [`AgreeKind::man`], which gets incorporated into
	/// [`Agree::man`] to produce the full output.
	fn man_tagline(&self) -> String {
		match self {
			Self::Switch(s) => man_tagline(s.short.as_deref(), s.long.as_deref(), None),
			Self::Option(o) => man_tagline(o.short.as_deref(), o.long.as_deref(), Some(&o.value)),
			Self::Arg(k) | Self::Item(k) => man_tagline(None, None, Some(&k.name)),
			// TODO: Handle Subcommands.
			_ => String::new(),
		}
	}

	/// # Get Long (Path) Option.
	///
	/// This is a convenience filter applied during BASH completion
	/// construction to resolve the long option key *if* the option accepts
	/// paths for its value.
	fn long_path_option(&self) -> Option<&str> {
		match self {
			Self::Option(s) if s.path => s.long.as_deref(),
			_ => None,
		}
	}

	/// # Name.
	///
	/// This returns the underlying data's name, if any.
	fn name(&self) -> Option<&str> {
		match self {
			Self::Arg(k) | Self::Item(k) => Some(&k.name),
			_ => None,
		}
	}

	/// # Get Short (Path) Option.
	///
	/// This is a convenience filter applied during BASH completion
	/// construction to resolve the short option key *if* the option accepts
	/// paths for its value.
	fn short_path_option(&self) -> Option<&str> {
		match self {
			Self::Option(s) if s.path => s.short.as_deref(),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Switch.
///
/// This holds the internal data for [`AgreeKind::Switch`]. It is public
/// because [`AgreeKind`] is public, but is not meant to be interacted with
/// directly. You should be using the passthrough methods provided by
/// [`AgreeKind`] instead.
pub struct AgreeSwitch {
	short: Option<String>,
	long: Option<String>,
	description: String,
}

impl AgreeSwitch {
	/// # New.
	pub fn new<S>(description: S) -> Self
	where S: Into<String> {
		Self {
			short: None,
			long: None,
			description: description.into(),
		}
	}

	/// # With Long.
	pub fn with_long<S>(mut self, key: S) -> Self
	where S: Into<String> {
		self.long = Some(key.into());
		self
	}

	/// # With Short.
	pub fn with_short<S>(mut self, key: S) -> Self
	where S: Into<String> {
		self.short = Some(key.into());
		self
	}
}



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Option.
///
/// This holds the internal data for [`AgreeKind::Option`]. It is public
/// because [`AgreeKind`] is public, but is not meant to be interacted with
/// directly. You should be using the passthrough methods provided by
/// [`AgreeKind`] instead.
pub struct AgreeOption {
	short: Option<String>,
	long: Option<String>,
	value: String,
	path: bool,
	description: String,
}

impl AgreeOption {
	/// # New.
	pub fn new<S>(value: S, description: S, path: bool) -> Self
	where S: Into<String> {
		Self {
			short: None,
			long: None,
			value: value.into(),
			path,
			description: description.into(),
		}
	}

	#[must_use]
	/// # With Long.
	pub fn with_long<S>(mut self, key: S) -> Self
	where S: Into<String> {
		self.long = Some(key.into());
		self
	}

	#[must_use]
	/// # With Short.
	pub fn with_short<S>(mut self, key: S) -> Self
	where S: Into<String> {
		self.short = Some(key.into());
		self
	}
}



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Item.
///
/// This holds the internal data for [`AgreeKind::Arg`] and [`AgreeKind::Item`].
/// It is public because [`AgreeKind`] is public, but is not meant to be
/// interacted with directly. You should be using the passthrough methods
/// provided by [`AgreeKind`] instead.
pub struct AgreeItem {
	name: String,
	description: String,
}

impl AgreeItem {
	/// # New.
	pub fn new<S>(name: S, description: S) -> Self
	where S: Into<String> {
		Self {
			name: name.into(),
			description: description.into(),
		}
	}
}



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Paragraph.
///
/// This holds the internal data for [`AgreeKind::Paragraph`]. It is public
/// because [`AgreeKind`] is public, but is not meant to be interacted with
/// directly. You should be using the passthrough methods provided by
/// [`AgreeKind`] instead.
pub struct AgreeParagraph {
	p: Vec<String>,
}

impl AgreeParagraph {
	/// # New.
	pub fn new<S>(line: S) -> Self
	where S: Into<String> {
		Self {
			p: vec![line.into()],
		}
	}

	/// # With Line.
	pub fn with_line<S>(mut self, line: S) -> Self
	where S: Into<String> {
		self.p.push(line.into());
		self
	}
}



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Section.
///
/// This struct represents a section of the MAN page. If not using any of the
/// auto-section flags with [`Agree`], you can use it to make your own.
pub struct AgreeSection {
	name: String,
	indent: bool,
	items: Vec<AgreeKind>
}

impl AgreeSection {
	/// # New.
	pub fn new<S>(name: S, indent: bool) -> Self
	where S: Into<String> {
		Self {
			name: name.into(),
			indent,
			items: Vec::new(),
		}
	}

	#[must_use]
	/// # With Item.
	pub fn with_item(mut self, item: AgreeKind) -> Self {
		self.items.push(item);
		self
	}

	#[must_use]
	/// # Is Empty.
	///
	/// This returns `true` if no items are attached to the section.
	pub fn is_empty(&self) -> bool { self.items.is_empty() }

	#[must_use]
	/// # Length.
	///
	/// This returns the number of items attached to the section.
	pub fn len(&self) -> usize { self.items.len() }

	/// # MAN Helper.
	///
	/// This generates the MAN code for the section, which is incorporated by
	/// [`Agree::man`] to produce the full document.
	fn man(&self) -> String {
		// Start with the header.
		let mut out: String = format!(
			"{} {}",
			if self.indent { ".SS" } else { ".SH" },
			&self.name,
		);

		// Add the items one at a time.
		self.items.iter()
			.map(|i| i.man(self.indent))
			.for_each(|s| {
				out.push('\n');
				out.push_str(&s);
			});

		// Done!
		out
	}
}



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # App Details.
///
/// [`Agree`] is a very crude, simple library to generate BASH completions
/// and/or MAN pages for apps.
///
/// The main idea is to toss a call to this in `build.rs`, keeping the
/// overhead out of the runtime application entirely.
///
/// Subcommands are not currently supported, but will be coming eventually so
/// FYI itself can make use of this. Haha.
pub struct Agree {
	name: String,
	bin: String,
	version: String,
	description: String,
	flags: u8,
	args: Vec<AgreeKind>,
	other: Vec<AgreeSection>,
}

impl Agree {
	/// # New.
	pub fn new<S>(name: S, bin: S, version: S, description: S) -> Self
	where S: Into<String> {
		Self {
			name: name.into(),
			bin: bin.into(),
			version: version.into(),
			description: description.into(),
			flags: 0,
			args: Vec::new(),
			other: Vec::new(),
		}
	}

	#[must_use]
	/// # With Flags.
	///
	/// Flags control which (if any) common sections to automatically generate
	/// for you. See the module-level documentation for more information.
	pub const fn with_flags(mut self, flags: u8) -> Self {
		self.flags |= flags;
		self
	}

	#[must_use]
	/// # With Arg.
	///
	/// Use this builder pattern method to attach every flag, option, and
	/// trailing arg supported by your program.
	///
	/// If you set the corresponding flags, `Agree` will generate `FLAGS:`,
	/// `OPTIONS:`, and `<ARGS>:` sections using this data automatically.
	pub fn with_arg(mut self, arg: AgreeKind) -> Self {
		self.args.push(arg);
		self
	}

	#[must_use]
	/// # With Section.
	///
	/// Use this builder pattern method to attach arbitrary MAN sections to
	/// the app.
	pub fn with_section(mut self, section: AgreeSection) -> Self {
		self.other.push(section);
		self
	}

	#[must_use]
	/// # BASH Completions.
	///
	/// Generate and return the code for a BASH completion script as a string.
	/// You can alternatively use [`Agree::write_bash`] to save this to a file
	/// instead.
	pub fn bash(&self) -> String {
		// Hold the string we're building.
		include_str!("../skel/basher.txt")
			.replace("%BNAME%", &self.bin)
			.replace(
				"%FNAME%",
				&"_basher__".chars()
					.chain(self.bin.trim().replace('-', "_").to_lowercase().chars())
					.filter(|&x| x.is_alphanumeric() || x == '_')
					.collect::<String>()
			)
			.replace(
				"%CONDS%",
				&self.args.iter()
					.filter_map(|x| {
						let txt: String = x.bash();
						if txt.is_empty() { None }
						else { Some(txt) }
					})
					.collect::<Vec<String>>()
					.join("")
			)
			.replace("%PATHS%", &self.bash_paths())
	}

	#[must_use]
	/// # MAN Page.
	///
	/// Generate and return the code for a MAN page as a string. You can
	/// alternatively use [`Agree::write_man`] to save this to a file instead.
	pub fn man(&self) -> String {
		// Start with the header.
		let mut out: String = format!(
			r#".TH {} "1" "{}" "{} v{}" "User Commands""#,
			self.name.to_uppercase(),
			chrono::Local::now().format("%B %Y"),
			&self.name,
			&self.version,
		);

		// Add each section.
		let mut pre: Vec<AgreeSection> = Vec::new();

		// Generated Name Section.
		if 0 != self.flags & FLAG_MAN_NAME {
			pre.push(
				AgreeSection::new("NAME", false)
					.with_item(AgreeKind::paragraph(format!(
						"{} - Manual page for {} v{}.",
						&self.name,
						&self.bin,
						&self.version
					)))
			);
		}

		// Generated Description Section.
		if 0 != self.flags & FLAG_MAN_DESCRIPTION {
			pre.push(
				AgreeSection::new("DESCRIPTION", false)
					.with_item(AgreeKind::paragraph(&self.description))
			);
		}

		// Generated Usage Section.
		if 0 != self.flags & FLAG_MAN_USAGE {
			pre.push(
				AgreeSection::new("USAGE:", true)
					.with_item(AgreeKind::paragraph(self.man_usage()))
			);
		}

		// Generated FLAGS Section.
		if 0 != self.flags & FLAG_MAN_FLAGS {
			let section = self.args.iter()
				.filter(|s| matches!(s, AgreeKind::Switch(_)))
				.cloned()
				.fold(
					AgreeSection::new("FLAGS:", true),
					AgreeSection::with_item
				);
			if ! section.is_empty() {
				pre.push(section);
			}
		}

		// Generated OPTIONS Section.
		if 0 != self.flags & FLAG_MAN_OPTIONS {
			let section = self.args.iter()
				.filter(|s| matches!(s, AgreeKind::Option(_)))
				.cloned()
				.fold(
					AgreeSection::new("OPTIONS:", true),
					AgreeSection::with_item
				);
			if ! section.is_empty() {
				pre.push(section);
			}
		}

		// Generated ARGUMENTS Section.
		if 0 != self.flags & FLAG_MAN_ARGS {
			self.args.iter()
				.filter_map(|s| match s {
					AgreeKind::Arg(s) => Some(s.clone()),
					_ => None,
				})
				.for_each(|s| {
					pre.push(
						AgreeSection::new(&format!("{}:", s.name), true)
							.with_item(AgreeKind::paragraph(&s.description))
					);
				});
		}

		pre.iter()
			.chain(self.other.iter())
			.for_each(|x| {
				out.push('\n');
				out.push_str(&x.man());
			});

		out.push('\n');
		out.replace('-', r"\-")
	}

	/// # Write BASH Completions!
	///
	/// This will write the BASH completion script to the directory of your
	/// choosing. The filename will simply be `your-bin.bash`.
	pub fn write_bash<P>(&self, dir: P) -> Result<(), String>
	where P: AsRef<Path> {
		let mut path = std::fs::canonicalize(dir.as_ref()).map_err(|_| format!(
			"Missing BASH completion directory: {:?}",
			dir.as_ref()
		))?;

		if path.is_dir() {
			path.push(&format!("{}.bash", &self.bin));
			write_to(&path, self.bash().as_bytes())
				.map_err(|_| format!(
					"Unable to write BASH completions: {:?}",
					path
				))
		}
		else {
			Err(format!("Invalid BASH completion directory: {:?}", dir.as_ref()))
		}
	}

	/// # Write MAN Page!
	///
	/// This will write the MAN page to the directory of your choosing. The
	/// filename will simply be `your-bin.1`.
	pub fn write_man<P>(&self, dir: P) -> Result<(), String>
	where P: AsRef<Path> {
		let mut path = std::fs::canonicalize(dir.as_ref()).map_err(|_| format!(
			"Missing MAN directory: {:?}",
			dir.as_ref()
		))?;

		if path.is_dir() {
			path.push(&format!("{}.1", &self.bin));
			write_to(&path, self.man().as_bytes())
				.map_err(|_| format!(
					"Unable to write MAN page: {:?}",
					path
				))
		}
		else {
			Err(format!("Invalid MAN directory: {:?}", dir.as_ref()))
		}
	}

	/// # BASH Helper (Path Options).
	///
	/// This produces the file/directory-listing portion of the BASH completion
	/// script for cases where the last option entered expects a path. It is
	/// integrated into the main [`Agree::bash`] output.
	fn bash_paths(&self) -> String {
		let keys: Vec<&str> = self.args.iter()
			.filter_map(AgreeKind::short_path_option)
			.chain(self.args.iter().filter_map(AgreeKind::long_path_option))
			.collect();

		if keys.is_empty() { String::new() }
		else {
			include_str!("../skel/basher.paths.txt")
				.replace("%KEYS%", &keys.join("|"))
		}
	}

	/// # MAN Helper (Usage).
	///
	/// This generates an example command for the usage section, if any.
	fn man_usage(&self) -> String {
		let mut out: String = self.bin.clone();

		if self.args.iter().any(|x| matches!(x, AgreeKind::Switch(_))) {
			out.push_str(" [FLAGS]");
		}

		if self.args.iter().any(|x| matches!(x, AgreeKind::Option(_))) {
			out.push_str(" [OPTIONS]");
		}

		if let Some(idx) = self.args.iter().position(|x| matches!(x, AgreeKind::Arg(_))) {
			out.push(' ');
			out.push_str(self.args[idx].name().unwrap());
		}

		out
	}
}



/// # Man Tagline.
fn man_tagline(short: Option<&str>, long: Option<&str>, value: Option<&str>) -> String {
	match (short, long, value) {
		// Option: long and short.
		(Some(s), Some(l), Some(v)) => format!(
			".TP\n\\fB{}\\fR, \\fB{}\\fR {}",
			s, l, v
		),
		// Option: long or short.
		(Some(k), None, Some(v)) | (None, Some(k), Some(v)) => format!(
			".TP\n\\fB{}\\fR {}",
			k, v
		),
		// Switch: long and short.
		(Some(s), Some(l), None) => format!(
			".TP\n\\fB{}\\fR, \\fB{}\\fR",
			s, l
		),
		// Switch: long or short.
		// Key/Value.
		(Some(k), None, None) | (None, Some(k), None) | (None, None, Some(k)) => format!(
			".TP\n\\fB{}\\fR",
			k
		),
		_ => String::new(),
	}
}

/// # Write File.
fn write_to(file: &PathBuf, data: &[u8]) -> Result<(), ()> {
	use std::io::Write;

	let mut out = std::fs::File::create(file).map_err(|_| ())?;
	out.write_all(data).map_err(|_| ())?;
	out.flush().map_err(|_| ())?;
	Ok(())
}
