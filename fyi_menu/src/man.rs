/*!
# FYI Menu: Man
*/

use std::{
	fmt,
	path::Path,
};



#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// # Manual Section Kind.
///
/// This is just a simple internal enum to help us differentiate between outer
/// text sections and inner list sections.
enum ManSectionKind {
	/// Outer text section.
	Text,
	/// Inner list section.
	List,
}

impl ManSectionKind {
	#[must_use]
	/// # Heading Code.
	const fn code(self) -> &'static str {
		match self {
			Self::Text => ".SH",
			Self::List => ".SS",
		}
	}
}



#[derive(Debug, Clone)]
/// # Manual Section Item.
///
/// This struct represents any kind of section item. For lists, this is
/// typically some combination of option/flag usage. For text sections, only
/// the `help` field will be populated, and will represent a sentence.
pub struct ManSectionItem {
	/// Keys.
	///
	/// These are uesd for flag/option-type items and would look like "-h" or
	/// "--help".
	keys: Vec<String>,
	/// Value.
	///
	/// This holds the value portion for an option requiring a value, e.g.
	/// "<FILE>".
	value: String,
	/// Description.
	///
	/// This is either a description of the flag/option, or a sentence that
	/// prints directly under the text section header.
	help: String,
}

impl ManSectionItem {
	/// # New!
	///
	/// Create a new item instance. Description is required, and might be all
	/// there is (for e.g. text sections).
	pub fn new<S>(help: S) -> Self
	where S: Into<String> {
		Self {
			keys: Vec::new(),
			help: help.into(),
			value: String::new(),
		}
	}

	#[must_use]
	/// # With Key.
	///
	/// This builder pattern method adds a key to the item, e.g. "--help".
	pub fn with_key<S>(mut self, tag: S) -> Self
	where S: Into<String> {
		self.keys.push(tag.into());
		self
	}

	#[must_use]
	/// # With Value.
	///
	/// This builder pattern method adds a value to the item, e.g. "<FILE>".
	pub fn with_value<S>(mut self, value: S) -> Self
	where S: Into<String> {
		self.value = value.into();
		self
	}

	#[must_use]
	/// # Description Only.
	///
	/// Return the description as an owned string.
	pub fn description(&self) -> String {
		self.help.clone()
	}

	#[must_use]
	/// # As String.
	///
	/// Format the item entry, dealing with keys and values and descriptions
	/// according to what data is present.
	pub fn print(&self) -> String {
		let tagline = self.tagline();

		// If there is no tagline portion, we can just return the description.
		if tagline.is_empty() {
			format!(".TP\n{}", &self.help)
		}
		// Otherwise we should glue the tagline and description together.
		else {
			format!(".TP\n{}\n{}", tagline, &self.help)
		}
	}

	/// # Format Tagline.
	///
	/// This formats the keys and values, if any, into a coherent line.
	fn tagline(&self) -> String {
		let mut out: String = self.keys.iter()
			.map(|x| format!(r"\fB{}\fR", x))
			.collect::<Vec<String>>()
			.join(", ");

		if ! self.value.is_empty() {
			out.push(' ');
			out.push_str(&self.value);
		}

		out.trim().to_string()
	}
}



#[derive(Debug, Clone)]
/// # Manual Section.
///
/// This represents an arbitrary manual section like "DESCRIPTION" or "FLAGS".
///
/// This can represent either an outer text section or an inner list section.
/// See [`ManSection::text`] and [`ManSection::list`] respectively.
pub struct ManSection {
	name: String,
	kind: ManSectionKind,
	items: Vec<ManSectionItem>,
}

impl fmt::Display for ManSection {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.print())
	}
}

impl ManSection {
	/// # New List Section.
	pub fn list<S>(name: S) -> Self
	where S: Into<String> {
		Self {
			name: name.into(),
			kind: ManSectionKind::List,
			items: Vec::new(),
		}
	}

	/// # New Text Section.
	pub fn text<S>(name: S) -> Self
	where S: Into<String> {
		Self {
			name: name.into(),
			kind: ManSectionKind::Text,
			items: Vec::new(),
		}
	}

	#[must_use]
	/// # With Item.
	///
	/// This builder pattern method adds an item to the section.
	pub fn with_item(mut self, item: ManSectionItem) -> Self {
		self.items.push(item);
		self
	}

	#[must_use]
	/// # With Line.
	///
	/// This builder pattern method adds a simple text item to the section.
	pub fn with_line<S>(self, line: S) -> Self
	where S: Into<String> {
		self.with_item(ManSectionItem::new(line))
	}

	#[must_use]
	/// # As String.
	///
	/// Return the entire section formatted as an owned string.
	pub fn print(&self) -> String {
		// Start with the header.
		let mut out: String = format!("{} {}", self.kind.code(), &self.name);

		// Text sections only need descriptions.
		if self.kind == ManSectionKind::Text {
			self.items.iter().for_each(|x| {
				out.push('\n');
				out.push_str(&x.description());
			});
		}
		// List sections are more complicated.
		else {
			self.items.iter().for_each(|x| {
				out.push('\n');
				out.push_str(&x.print());
			});
		}

		out
	}
}



#[derive(Debug, Clone)]
/// # The Manual.
///
/// This is a very crude build helper to generate a properly formatted MAN page
/// for your application. Throw it in `build.rs` and you won't need `help2man`
/// anymore!
///
/// It follows a basic builder pattern for setup, after which you can obtain
/// the final string using the formatter or [`Man::print`] method.
///
/// Take a look at `man.rs` in the examples folder for basic usage information.
pub struct Man {
	name: String,
	bin: String,
	version: String,
	sections: Vec<ManSection>,
}

impl fmt::Display for Man {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.print())
	}
}

impl Man {
	/// # New.
	///
	/// Create a new instance by providing a nice name, the binary name, and
	/// the app version.
	///
	/// A default `NAME` section is always created, but it is up to you to add
	/// other sections like `DESCRIPTION` or `USAGE`.
	pub fn new<S>(name: S, bin: S, version: S) -> Self
	where S: Into<String> {
		let name: String = name.into();
		let bin: String = bin.into();
		let version: String = version.into();

		Self {
			name: name.clone(),
			bin: bin.clone(),
			version: version.clone(),
			sections: vec![
				ManSection::text("NAME")
					.with_line(format!(
						"{} - Manual page for {} v{}.",
						&name,
						&bin,
						&version
					))
			],
		}
	}

	#[must_use]
	/// # With Section.
	///
	/// Add an already-compiled section to the manual.
	pub fn with_section(mut self, section: ManSection) -> Self {
		self.sections.push(section);
		self
	}

	/// # With Text Section.
	///
	/// This is a convenience method to add a new text section.
	pub fn with_text<S>(self, name: S, help: S, indent: bool) -> Self
	where S: Into<String> {
		if indent {
			self.with_section(
				ManSection::list(name)
					.with_line(help)
			)
		}
		else {
			self.with_section(
				ManSection::text(name)
					.with_line(help)
			)
		}
	}

	#[must_use]
	/// # As String.
	///
	/// Format and return the whole manual as an owned string.
	pub fn print(&self) -> String {
		// Start with the header.
		let mut out: String = format!(
			r#".TH {} "1" "{}" "{} v{}" "User Commands""#,
			self.name.to_uppercase(),
			chrono::Local::now().format("%B %Y"),
			&self.name,
			&self.version,
		);

		// Add each section.
		self.sections.iter().for_each(|x| {
			out.push_str(&format!("\n{}", x));
		});

		out.push('\n');
		out.replace('-', r"\-")
	}

	/// # Write!
	///
	/// This is a convenience method to write the manual directly to a file of
	/// your choosing, though conventions dictate it should be called `myapp.1`.
	pub fn write<P>(&self, out: P) -> Result<(), ()>
	where P: AsRef<Path> {
		use std::io::Write;

		let mut out = std::fs::File::create(out.as_ref()).map_err(|_| ())?;
		out.write_all(self.print().as_bytes()).map_err(|_| ())?;
		out.flush().map_err(|_| ())?;
		Ok(())
	}
}
