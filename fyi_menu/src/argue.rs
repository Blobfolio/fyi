/*!
# FYI Menu: Argue
*/

use crate::{
	KeyKind,
	utility,
};
use fyi_msg::Msg;
use std::{
	cell::Cell,
	env,
	iter::FromIterator,
	ops::Deref,
	process::exit,
};



/// # Flag: Argument(s) Required.
///
/// If a program is called with zero arguments, an error will be printed and
/// the thread will exit with status code `1`.
pub const FLAG_REQUIRED: u8 =   0b0001;

/// # Flag: Merge Separator Args.
///
/// If the program is called with `--` followed by additional arguments, those
/// arguments are glued back together into a single entry, replacing that `--`.
pub const FLAG_SEPARATOR: u8 =  0b0010;

/// # Flag: Expect Subcommand.
///
/// Set this flag to treat the first value as a subcommand rather than a
/// trailing argument. (This fixes the edge case where the command has zero
/// keys.)
pub const FLAG_SUBCOMMAND: u8 = 0b0100;

/// # Flag: Has Help.
///
/// This flag is set if either `-h` or `--help` switches are present. It has
/// no effect unless [`Argue::with_help`] is called.
const FLAG_HAS_HELP: u8 =       0b1000;

/// # Flag: Has Version.
///
/// This flag is set if either `-V` or `--version` switches are present. It has
/// no effect unless [`Argue::with_version`] is called.
const FLAG_HAS_VERSION: u8 =  0b1_0000;



/// # The size of our keys array.
const KEY_SIZE: usize = 16;
/// # The index noting total key length.
const KEY_LEN: usize = 15;



#[derive(Debug)]
/// `Argue` is a minimalistic, agnostic CLI argument parser. It does not hold
/// information about all the expected or required arguments an application
/// might have; instead it merely parses the raw [`std::env::args`] output into a
/// consistent state and provides various methods of querying the set
/// afterwards.
///
/// ## Assumptions
///
/// `Argue` is built for speed and simplicity, and as such, contains a number
/// of assumptions and limitations that might make it unsuitable for use.
///
/// ### Keys
///
/// A "key" is an argument entry beginning with one or two dashes `-` and an
/// ASCII letter. Entries with one dash are "short", and can only consist of
/// two bytes. Entries with two dashes are "long" and can be however long they
/// want to be.
///
/// If a short key entry is longer than two bytes, everything in range `2..` is
/// assumed to be a value and is split off into its own entry. For example,
/// `-kVal` is equivalent to `-k Val`.
///
/// If a long key contains an `=`, it is likewise assumed to be a key/value
/// pair, and will be split into two at that index. For example, `--key=Val` is
/// equivalent to `--key Val`.
///
/// A key without a value is called a "switch". It is `true` if present,
/// `false` if not.
///
/// A key with one value is called an "option". Multi-value options are *not*
/// supported.
///
/// ### Trailing Arguments
///
/// All values beginning after the last known switch or option value are
/// considered to be trailing arguments. Any number (including zero) of
/// trailing arguments can be provided.
///
/// ### Restrictions
///
/// 1. Keys are not checked for uniqueness, but only the first occurrence of a given key will match.
/// 2. A given argument set may only include up to **15** keys. If that number is exceeded, `Argue` will print an error and terminate the thread with a status code of `1`.
///
/// ## Examples
///
/// `Argue` follows a builder pattern for construction, with a few odds and
/// ends tucked away as flags.
///
/// ```no_run
/// use fyi_menu::{Argue, FLAG_REQUIRED};
///
/// // Parse the env arguments, aborting if the set is empty.
/// let mut args = Argue::new(FLAG_REQUIRED);
///
/// // Check to see what's there.
/// let switch: bool = args.switch("-s");
/// let option: Option<&str> = args.option("--my-opt");
/// let extras: &[String] = args.args();
/// ```
///
/// If you just want a clean set to iterate over, `Argue` can be dereferenced
/// to a string slice:
///
/// ```no_run
/// let arg_slice: &[String] = *args;
/// ```
///
/// Or it can be converted into an owned string Vector:
/// ```no_run
/// let args: Vec<String> = args.take();
/// ```
pub struct Argue {
	/// Parsed arguments.
	args: Vec<String>,
	/// Keys.
	///
	/// This array holds the indexes (in args) of any keys found so checks can
	/// iterate over the relevant subset (skipping values, etc.).
	///
	/// The last slot holds the number of keys.
	keys: [usize; KEY_SIZE],
	/// Highest non-arg index.
	last: Cell<usize>,
	/// Flags.
	flags: u8,
}

impl Default for Argue {
	#[inline]
	fn default() -> Self {
		Self {
			args: Vec::with_capacity(16),
			keys: [0_usize; KEY_SIZE],
			last: Cell::new(0),
			flags: 0,
		}
	}
}

impl Deref for Argue {
	type Target = [String];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.args }
}

impl<I> From<I> for Argue
where I: Iterator<Item=String> {
	fn from(src: I) -> Self
	where I: Iterator<Item=String> {
		src.skip_while(|x|
			x.is_empty() ||
			x.as_bytes().iter().all(u8::is_ascii_whitespace)
		)
			.fold(
				Self::default(),
				Self::push
			)
	}
}

impl FromIterator<String> for Argue {
	#[inline]
	fn from_iter<I: IntoIterator<Item=String>>(src: I) -> Self {
		Self::from(src.into_iter())
	}
}

/// ## Instantiation and Builder Patterns.
impl Argue {
	#[must_use]
	#[inline]
	/// # New Instance.
	///
	/// Populate arguments from `std::env::args()`. The first (command path)
	/// part is automatically excluded.
	///
	/// To construct an `Argue` from arbitrary raw values, use the
	/// `Argue::from_iter()` method (via the [`std::iter::FromIterator`] trait).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0);
	/// ```
	pub fn new(flags: u8) -> Self {
		env::args().skip(1)
			.skip_while(|x|
				x.is_empty() ||
				x.as_bytes().iter().all(u8::is_ascii_whitespace)
			)
			.fold(
				Self {
					flags,
					..Self::default()
				},
				Self::push
			)
	}

	#[must_use]
	/// # With Flags.
	///
	/// This method can be used to set additional parsing options in cases
	/// where the struct was initialized without calling [`Argue::new`].
	pub fn with_flags(mut self, flags: u8) -> Self {
		self.flags |= flags;

		// Required?
		if 0 != flags & FLAG_REQUIRED && self.args.is_empty() {
			Msg::error("Missing options, flags, arguments, and/or ketchup.").die(1);
		}

		// Handle separator.
		if 0 != flags & FLAG_SEPARATOR {
			self.parse_separator();
		}

		self
	}

	#[must_use]
	/// # Help Helper.
	///
	/// Chain this method to `new()` to run the specified callback if the first
	/// entry is "help" (a subcommand of sorts) or either "-h" or "--help"
	/// switches are present. The thread will automatically terminate with an
	/// exit code of `0` after running the callback.
	///
	/// Note: the callback can technically do whatever it wants, but usually it
	/// would be expected to print some output to the screen.
	///
	/// In cases where these flags appear in the middle and the first entry in
	/// the set is not a key (e.g. it is likely a subcommand), that first value
	/// will be passed to the callback for reference, allowing it to
	/// potentially adapt the output based on that condition.
	///
	/// If no help flags are found, this just transparently returns `self`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0).with_help(|_: Option<&str>| {
	///     println!("Help-o world!");
	/// });
	/// ```
	pub fn with_help<F>(self, cb: F) -> Self
	where F: Fn(Option<&str>) {
		// There has to be a first entry...
		if ! self.args.is_empty() {
			// If that entry is "help", we're done!
			if self.args[0] == "help" {
				cb(None);
				exit(0);
			}
			// Otherwise we need to check for the flags.
			else if 0 != self.flags & FLAG_HAS_HELP {
				if self.keys[0] == 0 && self.keys[KEY_LEN] != 0 { cb(None); }
				else { cb(Some(&self.args[0])); }
				exit(0);
			}
		}

		self
	}

	#[must_use]
	/// # Add Arguments From a Text File.
	///
	/// When chained to `new()`, if either "-l" or "--list" options are found,
	/// the subsequent value (if any) is read as a text file, and each non-
	/// empty line within is appended to the set as additional arguments,
	/// exactly as if they were provided directly.
	///
	/// No judgments are passed on the contents of the file. If a line has
	/// length, it is appended.
	///
	/// Note: if using this approach to seed a command with file paths, make
	/// sure those paths are absolute as their relativity will likely be lost
	/// in translation.
	///
	/// This method always transparently returns `self`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0).with_list();
	/// ```
	pub fn with_list(mut self) -> Self {
		use std::{
			fs::File,
			io::{
				BufRead,
				BufReader,
			},
		};

		if let Some(file) = self.option2("-l", "--list").and_then(|p| File::open(p).ok()) {
			BufReader::new(file).lines()
				.filter_map(std::result::Result::ok)
				.for_each(|line| {
					let trim = line.trim();
					if ! trim.is_empty() {
						if trim == line { self.args.push(line); }
						else { self.args.push(trim.into()); }
					}
				});
		}

		self
	}

	#[must_use]
	/// # Print Name/Version.
	///
	/// Similar to `with_help()`, this method can be chained to `new()` to
	/// print the program name and version or whatever, then exit with a status
	/// code of `0`, if either "-V" or "--version" flags are present.
	///
	/// If no version flags are found, `self` is transparently passed through.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0)
	///     .with_version("My App", env!("CARGO_PKG_VERSION"));
	/// ```
	pub fn with_version<S>(self, name: S, version: S) -> Self
	where S: AsRef<str> {
		if 0 != self.flags & FLAG_HAS_VERSION {
			Msg::plain(format!("{} v{}\n", name.as_ref(), version.as_ref()))
				.print();
			exit(0);
		}

		self
	}
}

/// ## Casting.
///
/// These methods convert `Argue` into different data structures.
impl Argue {
	#[allow(clippy::missing_const_for_fn)] // Doesn't work!
	#[must_use]
	#[inline]
	/// # Into Owned Vec.
	///
	/// Use this method to consume the struct and return the parsed arguments
	/// as a `Vec<String>`.
	///
	/// Unless `take_arg()` was called previously, the vector should represent
	/// all of the original arguments (albeit reformatted).
	///
	/// If you merely want something to iterate over, you can alternatively
	/// dereference the struct to a string slice.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let args: Vec<String> = Argue::new(0).take();
	/// ```
	pub fn take(self) -> Vec<String> { self.args }
}

/// ## Queries.
///
/// These methods allow data to be questioned and extracted.
impl Argue {
	#[must_use]
	#[inline]
	/// # First Entry.
	///
	/// Borrow the first entry, if any.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0);
	///
	/// if let Some("happy") = args.peek() { ... }
	/// ```
	pub fn peek(&self) -> Option<&str> { self.args.get(0).map(String::as_str) }

	#[must_use]
	#[inline]
	/// # First Entry.
	///
	/// Borrow the first entry without first checking for its existence.
	///
	/// ## Safety
	///
	/// This assumes a first argument exists; it will panic if the set is
	/// empty.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::{Argue, FLAG_REQUIRED};
	///
	/// let mut args = Argue::new(FLAG_REQUIRED);
	///
	/// // This is actually safe because FLAG_REQUIRED would have errored out
	/// // if nothing were present.
	/// let first: &str = unsafe { args.peek_unchecked() };
	/// ```
	pub unsafe fn peek_unchecked(&self) -> &str { &self.args[0] }

	#[must_use]
	#[inline]
	/// # Switch.
	///
	/// Returns `true` if the switch is present, `false` if not.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0);
	/// let switch: bool = args.switch("--my-switch");
	/// ```
	pub fn switch(&self, key: &str) -> bool {
		self.keys.iter()
			.take(self.keys[KEY_LEN])
			.map(|x| &self.args[*x])
			.any(|x| x == key)
	}

	#[must_use]
	#[inline]
	/// # Switch x2.
	///
	/// This is a convenience method that checks for the existence of two
	/// switches at once, returning `true` if either are present.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0);
	/// let switch: bool = args.switch2("-s", "--my-switch");
	/// ```
	pub fn switch2(&self, short: &str, long: &str) -> bool {
		self.keys.iter()
			.take(self.keys[KEY_LEN])
			.map(|x| &self.args[*x])
			.any(|x| x == short || x == long)
	}

	/// # Option.
	///
	/// Return the value corresponding to `key`, if present. "Value" in this
	/// case means the entry immediately following the key. Multi-value
	/// options are not supported.
	///
	/// Note: this method requires mutable access to `self` because it can
	/// potentially nudge the option/argument boundary +1 to the right. Because
	/// of this, you should be sure to query any expected options before
	/// calling `args()`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0);
	/// let opt: Option<&str> = args.option("--my-opt");
	/// ```
	pub fn option(&self, key: &str) -> Option<&str> {
		self.keys.iter()
			.take(self.keys[KEY_LEN])
			.position(|&x| self.args.get(x).map_or(false, |x| x == key))
			.and_then(|idx| {
				let idx = self.keys[idx] + 1;
				self.args.get(idx).map(|x| {
					if idx > self.last.get() { self.last.set(idx); }
					x.as_str()
				})
			})
	}

	/// # Option x2.
	///
	/// This is a convenience method that checks for the existence of two
	/// options at once, returning the first found value, if any.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0);
	/// let opt: Option<&str> = args.option2("-o", "--my-opt");
	/// ```
	pub fn option2(&self, short: &str, long: &str) -> Option<&str> {
		self.keys.iter()
			.take(self.keys[KEY_LEN])
			.position(|&x| self.args.get(x).map_or(false, |x| x == short || x == long))
			.and_then(|idx| {
				let idx = self.keys[idx] + 1;
				self.args.get(idx).map(|x| {
					if idx > self.last.get() { self.last.set(idx); }
					x.as_str()
				})
			})
	}

	#[must_use]
	/// # Trailing Arguments.
	///
	/// This returns a slice from the end of the result set assumed to
	/// represent unnamed arguments. The boundary for the split is determined
	/// by the position of the last known key (or key value).
	///
	/// It is important to query any expected options prior to calling this
	/// method, as the existence of those options might shift the boundary.
	///
	/// If there are no arguments, an empty slice is returned.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0);
	/// let extras: &[String] = args.args();
	/// ```
	pub fn args(&self) -> &[String] { &self.args[self.arg_idx()..] }

	#[must_use]
	/// # Arg at Index
	///
	/// Pluck the arg at a given index, if any, zero being the first trailing
	/// argument.
	pub fn arg(&self, idx: usize) -> Option<&str> {
		let start_idx = self.arg_idx();
		if start_idx + idx < self.args.len() {
			Some(self.args[start_idx + idx].as_str())
		}
		else { None }
	}

	#[must_use]
	/// # Take Next Trailing Argument.
	///
	/// Return an owned copy of the first available argument — removing it from
	/// the collection — or print an error message and exit.
	///
	/// This method is intended for use in cases where exactly one argument is
	/// expected and required. All other cases should probably just call
	/// `args()`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new(0);
	/// let opt: String = args.take_arg();
	/// ```
	pub fn take_arg(&mut self) -> String {
		let idx = self.arg_idx();
		if idx >= self.args.len() {
			Msg::error("Missing required argument.").die(1);
		}

		self.args.remove(idx)
	}
}

/// ## Internal Helpers.
impl Argue {
	/// # Arg Index.
	///
	/// This is an internal method that returns the index at which the first
	/// unnamed argument may be found.
	///
	/// Note: the index may be out of range, but won't be used in that case.
	fn arg_idx(&self) -> usize {
		if self.keys[KEY_LEN] == 0 && 0 == self.flags & FLAG_SUBCOMMAND { 0 }
		else { self.last.get() + 1 }
	}

	/// # Insert Key.
	///
	/// This will record the key index, unless the maximum number of keys
	/// has been reached, in which case it will print an error and exit with a
	/// status code of `1` instead.
	fn insert_key(&mut self, idx: usize) {
		if self.keys[KEY_LEN] == KEY_LEN {
			Msg::error("Too many options.").die(1);
		}

		self.keys[self.keys[KEY_LEN]] = idx;
		self.keys[KEY_LEN] += 1;
	}

	/// # Parse Keys.
	///
	/// This indexes the keys and parses out dangling values in cases like
	/// `-aVal` or `--a=Val`.
	fn push(mut self, mut val: String) -> Self {
		match KeyKind::from(val.as_bytes()) {
			// Passthrough.
			KeyKind::None => {
				self.args.push(val);
			},
			// Record the key and passthrough.
			KeyKind::Short => {
				let idx: usize = self.args.len();
				if val == "-V" { self.flags |= FLAG_HAS_VERSION; }
				else if val == "-h" { self.flags |= FLAG_HAS_HELP; }

				self.args.push(val);
				self.insert_key(idx);
				self.last.set(idx);
			},
			// Record the key and passthrough.
			KeyKind::Long => {
				let idx: usize = self.args.len();
				if val == "--version" { self.flags |= FLAG_HAS_VERSION; }
				else if val == "--help" { self.flags |= FLAG_HAS_HELP; }

				self.args.push(val);
				self.insert_key(idx);
				self.last.set(idx);
			},
			// Split a short key/value pair.
			KeyKind::ShortV => {
				let idx: usize = self.args.len();
				let tmp: String = val.split_off(2);

				self.args.push(val);
				self.args.push(tmp);
				self.insert_key(idx);
				self.last.set(idx + 1);
			},
			// Split a long key/value pair.
			KeyKind::LongV(x) => {
				let idx: usize = self.args.len();
				let tmp: String =
					if x + 1 < val.len() { val.split_off(x + 1) }
					else { String::new() };

				// Chop off the "=" sign.
				val.truncate(x);

				self.args.push(val);
				self.args.push(tmp);
				self.insert_key(idx);
				self.last.set(idx + 1);
			},
		}

		self
	}

	/// # Parse Separator.
	///
	/// This concatenates all arguments trailing a "--" entry into a single
	/// value, replacing the "--".
	fn parse_separator(&mut self) {
		if let Some(idx) = self.args.iter().position(|x| x == "--") {
			if idx + 1 < self.args.len() {
				self.args[idx] = self.args.drain(idx + 1..)
					.map(utility::esc_arg)
					.collect::<Vec<String>>()
					.join(" ");
			}
			else {
				self.args.truncate(idx);
			}
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use criterion as _;

	#[test]
	fn t_parse_args() {
		let mut base: Vec<String> = vec![
			String::from(""),
			String::from("hey"),
			String::from("-kVal"),
			String::from("--empty="),
			String::from("--key=Val"),
			String::from("--"),
			String::from("stuff"),
			String::from("and things")
		];

		let mut args = Argue::from_iter(base.clone()).with_flags(FLAG_SEPARATOR);
		assert_eq!(
			*args,
			[
				String::from("hey"),
				String::from("-k"),
				String::from("Val"),
				String::from("--empty"),
				String::new(),
				String::from("--key"),
				String::from("Val"),
				String::from("stuff 'and things'"),
			]
		);

		assert_eq!(args.peek(), Some("hey"));
		assert!(args.switch("-k"));
		assert!(args.switch("--key"));
		assert!(args.switch2("-k", "--key"));
		assert_eq!(args.option("--key"), Some("Val"));
		assert_eq!(args.option2("-k", "--key"), Some("Val"));
		assert_eq!(args.args(), &[String::from("stuff 'and things'")]);

		// Let's make sure first-position keys are OK.
		base[0] = String::from("--prefix");
		args = Argue::from_iter(base.clone());
		assert_eq!(
			*args,
			[
				String::from("--prefix"),
				String::from("hey"),
				String::from("-k"),
				String::from("Val"),
				String::from("--empty"),
				String::new(),
				String::from("--key"),
				String::from("Val"),
				String::from("--"), // We didn't reglue these this time.
				String::from("stuff"),
				String::from("and things")
			]
		);

		assert_eq!(args.peek(), Some("--prefix"));
		assert!(args.switch("--prefix"));
		assert_eq!(args.option("--prefix"), Some("hey"));

		args = Argue::from_iter(vec!["-k".to_string()]);
		assert_eq!(args.args(), Vec::<String>::new().as_slice());

		args = Argue::from_iter(vec![]);
		assert!(! args.switch("--prefix"));
		assert_eq!(args.option("--prefix"), None);
		assert_eq!(args.args(), Vec::<String>::new().as_slice());
	}
}
