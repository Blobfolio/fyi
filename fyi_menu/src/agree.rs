/*!
# FYI Menu: Agree
*/

use std::path::{
	Path,
	PathBuf,
};



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Agreement Kind.
///
/// This enum provides a more or less consistent interface for dealing with the
/// disparate argument/item types making up an application.
///
/// With the exception of [`AgreeKind::SubCommand`], each type has a
/// corresponding initialization method. The intention is you should never have
/// to import the underlying struct directly.
///
/// For example, to register a new [`AgreeKind::Switch`], just call
/// [`AgreeKind::switch`].
///
/// The enum additionally passes through the underyling structs' builder
/// methods. For example, you can add keys to options and switches using
/// [`AgreeKind::with_short`] or [`AgreeKind::with_long`], and for paragraphs,
/// you can add additional lines using [`AgreeKind::with_line`].
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
	/// This is a recursive [`Agree`], complete with its own description,
	/// flags, etc.
	///
	/// When calling [`Agree::write_man`], separate manuals will be written for
	/// each subcommand, following a "{bin}-{subcommand}.1" naming scheme.
	///
	/// Take a look at the `man` example in this crate, and also the `fyi`
	/// bin's own `build.rs` for sample construction.
	///
	/// ## Safety
	///
	/// There is support for ONE LEVEL of subcommands. That is, the main
	/// [`Agree`] struct can have any number of subcommands among its
	/// arguments, however those subcommands CANNOT have their own
	/// sub-subcommands. Undefined things will happen if 2+ levels are
	/// included.
	SubCommand(Agree),

	/// # Miscellaneous K/V Item.
	///
	/// This is a miscellaneous key/value pair that can be used for custom MAN
	/// sections. See also [`AgreeSection`].
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
	///
	/// This can be used to force a line break between bits of text. Otherwise
	/// if you jam everything into one "line", it will just wrap as needed.
	///
	/// This has no effect unless the type is [`AgreeKind::Paragraph`].
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
	///
	/// Specify a long key, e.g. `--help`.
	///
	/// This has no effect unless the type is [`AgreeKind::Switch`] or
	/// [`AgreeKind::Option`].
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
	///
	/// Specify a short key, e.g. `-h`.
	///
	/// This has no effect unless the type is [`AgreeKind::Switch`] or
	/// [`AgreeKind::Option`].
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
			Self::SubCommand(s) => format!("\topts+=(\"{}\")\n", &s.bin),
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
			Self::SubCommand(s) => format!(
				"{}\n{}",
				self.man_tagline(),
				&s.description,
			),
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
			Self::SubCommand(s) => man_tagline(None, None, Some(&s.bin)),
			_ => String::new(),
		}
	}

	/// # Name.
	///
	/// This returns the underlying data's name, if any.
	///
	/// This has no effect unless the type is [`AgreeKind::Arg`] or
	/// [`AgreeKind::Item`].
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
	///
	/// Specify a long key, e.g. `--help`.
	pub fn with_long<S>(mut self, key: S) -> Self
	where S: Into<String> {
		self.long = Some(key.into());
		self
	}

	/// # With Short.
	///
	/// Specify a short key, e.g. `-h`.
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
	///
	/// The `path` flag indicates whether or not this option expects some sort
	/// of file system path for its value. If `true`, the BASH completion will
	/// reveal files and folders in the current directory.
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
	///
	/// Specify a long key, e.g. `--help`.
	pub fn with_long<S>(mut self, key: S) -> Self
	where S: Into<String> {
		self.long = Some(key.into());
		self
	}

	#[must_use]
	/// # With Short.
	///
	/// Specify a short key, e.g. `-h`.
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
	///
	/// This can be used to force a line break between bits of text. Otherwise
	/// if you jam everything into one "line", it will just wrap as needed.
	pub fn with_line<S>(mut self, line: S) -> Self
	where S: Into<String> {
		self.p.push(line.into());
		self
	}
}



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Section.
///
/// This struct represents a section of the MAN page. It can be used to add any
/// arbitrary content you want (on top of the default NAME/DESCRIPTION/USAGE
/// bits.)
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
	///
	/// Attach any sort of [`AgreeKind`] data to the list. Mixing and matching
	/// might look weird in a single section, but any type will do.
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
/// It is constructed using builder patterns ([`Agree::with_arg`],
/// [`Agree::with_section`], etc.). Once set up, you can either obtain the
/// BASH completions and MAN page as a string ([`Agree::bash`] and
/// [`Agree::man`] respectively), or write them straight to files ([`Agree::write_bash`]
/// and [`Agree::write_man`] respectively).
///
/// The write methods are probably what you want.
///
/// Take a look at the crate examples or FYI's own `build.rs` for construction
/// and usage samples.
///
/// ## Safety
///
/// There is support for ONE LEVEL of subcommands. That is, the main [`Agree`]
/// struct can have any number of subcommands among its arguments, however
/// those subcommands CANNOT have their own sub-subcommands. Undefined things
/// will happen if 2+ levels are included.
pub struct Agree {
	name: String,
	bin: String,
	version: String,
	description: String,
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
			args: Vec::new(),
			other: Vec::new(),
		}
	}

	#[must_use]
	/// # With Arg.
	///
	/// Use this builder pattern method to attach every flag, option,
	/// trailing arg, and subcommand supported by your program.
	///
	/// When building manuals, these will automatically be separated out into
	/// appropriate sections for you.
	pub fn with_arg(mut self, arg: AgreeKind) -> Self {
		self.args.push(arg);
		self
	}

	#[must_use]
	/// # With Section.
	///
	/// Use this builder pattern method to attach arbitrary MAN sections to
	/// the app. Any sections you add here will appear after the default ones.
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
	///
	/// The completions are set up such that suggestions will only appear once.
	/// That is, if you have a help flag and the line already includes `-h`, it
	/// will not suggest you add `--help`.
	///
	/// Completions are subcommand-aware. You can have different options for
	/// different subcommands, and/or options available only to the top-level
	/// bin.
	pub fn bash(&self) -> String {
		// Start by building all the subcommand code. We'll handle things
		// differently depending on whether or not the resulting string is
		// empty.
		let mut out: String = self.args.iter()
			.filter_map(|x| match x {
				AgreeKind::SubCommand(y) => {
					let tmp = y.bash_completions(&self.bin);
					if tmp.is_empty() { None }
					else { Some(tmp) }
				},
				_ => None,
			})
			.collect();

		// If this is empty, just add our app and call it quits.
		if out.is_empty() {
			return [
				self.bash_completions(""),
				include_str!("../skel/basher.end.txt")
					.replace("%BNAME%", &self.bin)
					.replace("%FNAME%", &self.bash_fname("")),
			].concat();
		}

		// Add the app method.
		out.push_str(&self.bash_completions(""));

		// Add the function chooser.
		out.push_str(&self.bash_subcommands());

		// Done!
		out
	}

	#[must_use]
	/// # MAN Page.
	///
	/// Generate and return the code for a MAN page as a string. You can
	/// alternatively use [`Agree::write_man`] to save this to a file instead.
	///
	/// This automatically generates sections for `NAME`, `DESCRIPTION`, and
	/// `USAGE`, and if applicable, `FLAGS`, `OPTIONS`, trailing args, and
	/// `SUBCOMMANDS`.
	///
	/// If custom sections have been added, those will be printed after the
	/// above default sections.
	///
	/// Note: this will only return the manual for the top-level app. If there
	/// are subcommands, those pages will be ignored. To obtain those, call
	/// [`Agree::write_man`] instead.
	pub fn man(&self) -> String {
		self.subman("")
	}

	/// # Write BASH Completions!
	///
	/// This will write the BASH completion script to the directory of your
	/// choosing, using the file name "{bin}.bash".
	pub fn write_bash<P>(&self, dir: P) -> Result<(), String>
	where P: AsRef<Path> {
		let mut path = std::fs::canonicalize(dir.as_ref()).map_err(|_| format!(
			"Missing BASH completion directory: {:?}",
			dir.as_ref()
		))?;

		if path.is_dir() {
			path.push(&format!("{}.bash", &self.bin));
			write_to(&path, self.bash().as_bytes(), false)
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
	/// This will write the MAN page(s) to the directory of your choosing,
	/// using the file name "{bin}.1" for the top-level app, and
	/// "{bin}-{subcommand}.1" for any specified subcommands.
	///
	/// This method will also write Gzipped copies of any manuals produced in
	/// case you want to use them for distribution (reducing the file size a
	/// bit).
	///
	/// You should only push one copy of each manual to `/usr/share/man/man1`,
	/// either the "{bin}.1" or "{bin}.1.gz" version, not both. ;)
	pub fn write_man<P>(&self, dir: P) -> Result<(), String>
	where P: AsRef<Path> {
		let mut path = std::fs::canonicalize(dir.as_ref()).map_err(|_| format!(
			"Missing MAN directory: {:?}",
			dir.as_ref()
		))?;

		// The main file.
		if path.is_dir() {
			path.push(&format!("{}.1", &self.bin));
			write_to(&path, self.man().as_bytes(), true)
				.map_err(|_| format!(
					"Unable to write MAN page: {:?}",
					path
				))?;
		}
		else {
			return Err(format!("Invalid MAN directory: {:?}", dir.as_ref()))
		}

		// Write subcommand pages.
		for (bin, man) in self.args.iter()
			.filter_map(|x| match x {
				AgreeKind::SubCommand(s) => Some((s.bin.clone(), s.subman(&self.bin))),
				_ => None,
			})
		{
			path.pop();
			path.push(&format!("{}-{}.1", &self.bin, bin));
			write_to(&path, man.as_bytes(), true)
				.map_err(|_| format!(
					"Unable to write MAN page: {:?}",
					path
				))?;
		}

		Ok(())
	}

	/// # BASH Helper (Function Name).
	///
	/// This generates a unique-ish function name for use in the BASH
	/// completion script.
	fn bash_fname(&self, parent: &str) -> String {
		[
			"_basher__",
			parent,
			"_",
			&self.bin,
		].concat()
			.replace('-', "_")
			.to_lowercase()
			.chars()
			.filter(|&x| x.is_alphanumeric() || x == '_')
			.collect::<String>()
	}

	/// # BASH Helper (Completions).
	///
	/// This generates the completions for a given app or subcommand. The
	/// output is combined with other code to produce the final script returned
	/// by the main [`Agree::bash`] method.
	fn bash_completions(&self, parent: &str) -> String {
		// Hold the string we're building.
		include_str!("../skel/basher.txt")
			.replace("%FNAME%", &self.bash_fname(parent))
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

	/// # BASH Helper (Subcommand Chooser).
	///
	/// This generates an additional method for applications with subcommands
	/// to allow per-command suggestions. The output is incorporated into the
	/// value returned by [`Agree::bash`].
	fn bash_subcommands(&self) -> String {
		let (cmd, chooser): (String, String) = std::iter::once((self.bin.clone(), self.bash_fname("")))
			.chain(
				self.args.iter()
					.filter_map(|x| match x {
						AgreeKind::SubCommand(y) => Some((y.bin.clone(), y.bash_fname(&self.bin))),
						_ => None,
					})
			)
			.fold(
				(String::new(), String::new()),
				|(mut a, mut b), (c, d)| {
					a.push_str(
						&include_str!("../skel/basher.subcmd.1.txt")
							.replace("%BNAME%", &c)
					);
					b.push_str(
						&include_str!("../skel/basher.subcmd.2.txt")
							.replace("%BNAME%", &c)
							.replace("%FNAME%", &d)
					);

					(a, b)
				}
			);

		include_str!("../skel/basher.subcmd.txt")
			.replace("%BNAME%", &self.bin)
			.replace("%FNAME%", &self.bash_fname(""))
			.replace("%SUBCMD1%", &cmd)
			.replace("%SUBCMD2%", &chooser)
	}

	/// # MAN Helper (Usage).
	///
	/// This generates an example command for the `USAGE` section, if any.
	fn man_usage(&self, parent: &str) -> String {
		let mut out: String = format!("{} {}", parent, &self.bin)
			.trim()
			.to_string();

		if self.args.iter().any(|x| matches!(x, AgreeKind::SubCommand(_))) {
			out.push_str(" [SUBCOMMAND]");
		}

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

	/// # MAN Helper (Subcommands)
	///
	/// This produces the main manual content, varying based on whether or not
	/// this is for a subcommand. Its output is incorporated into the main
	/// [`Agree::man`] result.
	fn subman(&self, parent: &str) -> String {
		// Start with the header.
		let mut out: String = format!(
			r#".TH "{}" "1" "{}" "{} v{}" "User Commands""#,
			format!("{} {}", parent.to_uppercase(), self.name.to_uppercase()).trim(),
			chrono::Local::now().format("%B %Y"),
			&self.name,
			&self.version,
		);

		// Add each section.
		let mut pre: Vec<AgreeSection> = vec![
			AgreeSection::new("NAME", false)
				.with_item(AgreeKind::paragraph(format!(
					"{} - Manual page for {} v{}.",
					&self.name,
					&self.bin,
					&self.version
				))),
				AgreeSection::new("DESCRIPTION", false)
					.with_item(AgreeKind::paragraph(&self.description)),
				AgreeSection::new("USAGE:", true)
					.with_item(AgreeKind::paragraph(self.man_usage(parent))),
		];

		// Generated FLAGS Section.
		{
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
		{
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
		{
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

		// Generated SUBCOMMANDS Section.
		{
			let section = self.args.iter()
				.filter(|s| matches!(s, AgreeKind::SubCommand(_)))
				.cloned()
				.fold(
					AgreeSection::new("SUBCOMMANDS:", true),
					AgreeSection::with_item
				);
			if ! section.is_empty() {
				pre.push(section);
			}
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
}



/// # Man Tagline.
///
/// This helper method generates an appropriate key/value line given what sorts
/// of keys and values exist for the given [`AgreeKind`] type.
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

#[allow(trivial_casts)] // Triviality is required.
/// # Write File.
///
/// This writes data to a file, optionally recursing to save a `GZipped`
/// version (for MAN pages).
fn write_to(file: &PathBuf, data: &[u8], compress: bool) -> Result<(), ()> {
	use libdeflater::{
		CompressionLvl,
		Compressor,
	};
	use std::{
		ffi::OsStr,
		os::unix::ffi::OsStrExt,
		io::Write,
	};

	let mut out = std::fs::File::create(file).map_err(|_| ())?;
	out.write_all(data).map_err(|_| ())?;
	out.flush().map_err(|_| ())?;

	// Save a compressed copy?
	if compress {
		let mut writer = Compressor::new(CompressionLvl::best());
		let mut buf: Vec<u8> = Vec::with_capacity(data.len());
		buf.resize(writer.gzip_compress_bound(data.len()), 0);

		if let Ok(len) = writer.gzip_compress(data, &mut buf) {
			// Trim any excess now that we know the final length.
			buf.truncate(len);

			// Toss ".gz" onto the original file path.
			let filegz: PathBuf = PathBuf::from(OsStr::from_bytes(&[
				unsafe { &*(file.as_os_str() as *const OsStr as *const [u8]) },
				b".gz",
			].concat()));

			// Recurse to write it!
			return write_to(&filegz, &buf, false);
		}
	}

	Ok(())
}
