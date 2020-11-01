/*!
# FYI Menu: Basher
*/

use std::fmt;
use std::path::Path;



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Basher Key.
///
/// This is a simple struct used internally by [`Basher`] to hold keys. It
/// holds both short and long variants, allowing us to avoid suggesting either
/// if either are already present.
///
/// The `value` boolean is used to differentiate between switches and options.
/// The only functional difference between the two is that if the cursor is
/// placed right after an option, the local file/dir list will be suggested.
struct BasherKey {
	short: Option<String>,
	long: Option<String>,
	value: bool,
}

impl BasherKey {
	/// # Completion Condition.
	///
	/// This returns a BASH-formatted IF statement that appends the short
	/// and/or long values if neither have been used already.
	fn completion_cond(&self) -> String {
		match (self.short.as_deref(), self.long.as_deref()) {
			(Some(s), Some(l)) => include_str!("../skel/basher.cond2.txt")
				.replace("%SHORT%", s)
				.replace("%LONG%", l),
			(None, Some(s)) | (Some(s), None) => include_str!("../skel/basher.cond1.txt")
				.replace("%KEY%", s),
			_ => String::new(),
		}
	}

	/// # Short Option.
	fn short_option(&self) -> Option<&str> {
		if self.value { self.short.as_deref() }
		else { None }
	}

	/// # Long Option.
	fn long_option(&self) -> Option<&str> {
		if self.value { self.long.as_deref() }
		else { None }
	}
}



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// # Basher.
///
/// This is a very crude BASH completion generator that apps can use via
/// `build.rs` so that switches and options can be produced when users `TAB`
/// in the CLI.
///
/// It follows a builder pattern, with [`Basher::with_switch`] and [`Basher::with_option`]
/// being the two main methods. The only functional difference between the two
/// is that if the cursor is placed right after an option, the local file/dir
/// list will be suggested.
///
/// There is not currently support for subcommands or hierarchical options, so
/// ironically the app needs to be simpler than [`fyi`]. Our other apps are,
/// though, so maybe yours will be too. :)
///
/// The BASH completion code can be obtained as a string via [`Basher::to_string`]
/// or [`Basher::completions`].
///
/// It can also be written directly to a file (e.g. `/some/path.bash`) using
/// [`Basher::write`].
///
/// ## Example
///
/// ```no_run
/// use fyi_menu::Basher;
///
/// let b = Basher::new("my_bin_name")
///     .with_switch(Some("-h"), Some("--help"));
///
/// let res = b.write("/path/to/my_bin_name.bash").is_ok();
/// ```
pub struct Basher {
	bin: String,
	keys: Vec<BasherKey>,
}

impl fmt::Display for Basher {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.completions())
	}
}

impl Basher {
	#[must_use]
	/// # New Instance.
	///
	/// Note: this expects the name of the *binary* (e.g. `my_app`), not the
	/// proper program name (e.g. `My App`).
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_menu::Basher;
	///
	/// let b = Basher::new("my_bin_name")
	///     .with_switch(Some("-h"), Some("--help"));
	/// ```
	pub fn new<S>(bin: S) -> Self
	where S: Into<String> {
		Self {
			bin: bin.into()
				.trim()
				.chars()
				.filter(|&x| x.is_alphanumeric() || x == '-' || x == '_')
				.collect(),
			keys: Vec::new(),
		}
	}

	#[must_use]
	/// # With Switch.
	///
	/// Define a short and/or long key that does not accept any value.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_menu::Basher;
	///
	/// let b = Basher::new("my_bin_name")
	///     .with_switch(Some("-h"), Some("--help"));
	/// ```
	pub fn with_switch<S>(mut self, short: Option<S>, long: Option<S>) -> Self
	where S: Into<String> {
		if short.is_some() || long.is_some() {
			self.keys.push(BasherKey {
				short: short.map(std::convert::Into::into),
				long: long.map(std::convert::Into::into),
				value: false,
			});
		}

		self
	}

	#[must_use]
	/// # With Option.
	///
	/// Define a short and/or long key that accepts a value. Completion
	/// suggestions will include the local file/dir list following this key.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_menu::Basher;
	///
	/// let b = Basher::new("my_bin_name")
	///     .with_option(Some("-i"), Some("--input"));
	/// ```
	pub fn with_option<S>(mut self, short: Option<S>, long: Option<S>) -> Self
	where S: Into<String> {
		if short.is_some() || long.is_some() {
			self.keys.push(BasherKey {
				short: short.map(std::convert::Into::into),
				long: long.map(std::convert::Into::into),
				value: true,
			});
		}

		self
	}

	#[must_use]
	/// # Completions.
	///
	/// Return the BASH completion script as a string.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_menu::Basher;
	///
	/// let b = Basher::new("my_bin_name")
	///     .with_switch(Some("-h"), Some("--help"));
	///
	/// // Print the script.
	/// println!("{}", b.completions());
	///
	/// // You can also print this way:
	/// println!("{}", b);
	/// ```
	pub fn completions(&self) -> String {
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
				&self.keys.iter()
					.map(BasherKey::completion_cond)
					.collect::<Vec<String>>()
					.join("")
			)
			.replace("%PATHS%", &self.completion_paths())
	}

	/// # Write!
	///
	/// This is a convenience method to write the BASH completion script to a
	/// file.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_menu::Basher;
	///
	/// let b = Basher::new("my_bin_name")
	///     .with_switch(Some("-h"), Some("--help"));
	///
	/// // Traditionally, completion scripts end with a `.bash`.
	/// b.write("/path/to/my_app.bash");
	/// ```
	pub fn write<P>(&mut self, out: P) -> Result<(), ()>
	where P: AsRef<Path> {
		use std::io::Write;

		let mut out = std::fs::File::create(out.as_ref()).map_err(|_| ())?;
		out.write_all(self.completions().as_bytes()).map_err(|_| ())?;
		out.flush().map_err(|_| ())?;
		Ok(())
	}

	/// # Completion Paths.
	///
	/// This is an internal helper that generates the file/dir list suggestions
	/// for value-taking keys.
	///
	/// If there are none, an empty string is returned.
	fn completion_paths(&self) -> String {
		let keys: Vec<&str> = self.keys.iter()
			.filter_map(BasherKey::short_option)
			.chain(self.keys.iter().filter_map(BasherKey::long_option))
			.collect();

		if keys.is_empty() { String::new() }
		else {
			include_str!("../skel/basher.paths.txt")
				.replace("%KEYS%", &keys.join("|"))
		}
	}
}
