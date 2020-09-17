/*!
# FYI Menu: Argue
*/

use crate::{
	die,
	KeyKind,
	KeyMaster,
	utility,
};
use fyi_msg::Msg;
use std::{
	env,
	iter::FromIterator,
	ops::Deref,
	process::exit,
};



#[derive(Debug)]
/// `Argue` is a minimalistic, argnostic CLI argument parser. It does not hold
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
/// All values beginning after the last known switch (or option value) are
/// considered to be trailing arguments. Any number (including zero) of
/// trailing arguments can be provided.
///
/// ### Restrictions
///
/// 1. Keys must be unique. If the same key appears twice, `Argue` will print an error and terminate the thread with a status code of `1`.
/// 2. A given argument set may only include up to **8** keys. If that number is exceeded, `Argue` will print an error and terminate the thread with a status code of `1`.
///
/// ## Examples
///
/// `Argue` follows a builder pattern for construction.
///
/// ```no_run
/// use fyi_menu::Argue;
///
/// // Parse the env arguments, aborting if the set is empty.
/// let mut args = Argue::new().with_any();
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
	/// A pseudo hash-map of key/idx pairs.
	keys: KeyMaster,
	/// The last known key/value index.
	///
	/// Because `Argue` is agnostic, it needs to keep track of the position of
	/// the last known key (or option value) so that it can make an educated
	/// guess about where the unnamed arguments begin.
	///
	/// This is an inclusive value; the first argument (if any) would exist at
	/// `last + 1`, except in cases where no keys were passed, in which case it
	/// is assumed the arguments begin at `0`.
	last: usize,
	/// Last Offset.
	///
	/// For applications expecting a subcommand, this value ensures that if no
	/// keys were passed, the argument index begins at `1` instead of `0` (as
	/// the first slot would be the subcommand).
	last_offset: bool,
}

impl Default for Argue {
	#[inline]
	fn default() -> Self {
		Self {
			args: Vec::with_capacity(16),
			keys: KeyMaster::default(),
			last: 0,
			last_offset: false,
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
			.fold(Self::default(), Self::fold_entry)
	}
}

impl FromIterator<String> for Argue {
	#[inline]
	fn from_iter<I: IntoIterator<Item=String>>(src: I) -> Self {
		Self::from(src.into_iter())
	}
}

impl Argue {
	// ------------------------------------------------------------------------
	// Builder
	// ------------------------------------------------------------------------

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
	/// let mut args = Argue::new();
	/// ```
	pub fn new() -> Self {
		Self::from(env::args().skip(1))
	}

	/// # With Entry.
	///
	/// This is an internal method used to fold entries into a collection from
	/// an iterator.
	///
	/// While it is slightly more efficient to split the collection and parsing
	/// into two loops, this approach is significantly cleaner and more
	/// readable.
	fn fold_entry(mut self, mut e: String) -> Self {
		match KeyKind::from(e.as_bytes()) {
			// Passthrough.
			KeyKind::None => { self.args.push(e); },
			// Record the keys and passthrough.
			KeyKind::Short | KeyKind::Long => {
				let idx: usize = self.args.len();
				self.keys.insert_unique(&e, idx);
				self.args.push(e);
				self.last = idx;
			},
			// Split a short key/value pair.
			KeyKind::ShortV => {
				let idx: usize = self.args.len();
				let tmp: String = e.split_off(2);
				self.keys.insert_unique(&e, idx);
				self.args.push(e);
				self.args.push(tmp);
				self.last = idx + 1;
			},
			// Split a long key/value pair.
			KeyKind::LongV(x) => {
				let idx: usize = self.args.len();
				let tmp: String =
					if x + 1 < e.len() { e.split_off(x + 1) }
					else { String::new() };

				// Chop off the "=" sign.
				e.truncate(x);

				self.keys.insert_unique(&e, idx);
				self.args.push(e);
				self.args.push(tmp);
				self.last = idx + 1;
			},
		}

		self
	}

	#[must_use]
	/// # Assert Non-Empty.
	///
	/// Chain this method to `new()` to print an error and exit with a status
	/// code of `1` in cases where no CLI arguments were present.
	///
	/// If arguments are found, this just transparently returns `self`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new().with_any();
	/// ```
	pub fn with_any(self) -> Self {
		if self.args.is_empty() {
			die(b"Missing options, flags, arguments, and/or ketchup.");
			unreachable!();
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
	/// let mut args = Argue::new().with_help(|_: Option<&str>| {
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
			else if self.keys.contains2("-h", "--help") {
				if self.args[0].as_bytes()[0] == b'-' { cb(None); }
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
	/// let mut args = Argue::new().with_list();
	/// ```
	pub fn with_list(mut self) -> Self {
		if let Some(path) = self.option2("-l", "--list") {
			use std::{
				fs::File,
				io::{
					BufRead,
					BufReader,
				},
			};

			if let Ok(file) = File::open(path) {
				BufReader::new(file).lines()
					.filter_map(|line| match line.unwrap_or_default().trim() {
						"" => None,
						x => Some(String::from(x)),
					})
					.for_each(|x| self.args.push(x));
			}
		}

		self
	}

	#[must_use]
	/// # With Separator.
	///
	/// Chain this method to `new()` if you would like any "separator"
	/// arguments glued back together as a single argument (replacing the "--"
	/// entry).
	///
	/// See the documentation for [`utility::esc_arg`] for a list of all the
	/// assumptions this method makes. (It may not do what you want.)
	///
	/// This method always transparently returns `self`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new().with_separator();
	/// ```
	pub fn with_separator(mut self) -> Self {
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

		self
	}

	#[must_use]
	/// # With Subcommand.
	///
	/// Chain this method to `new()` if your program might be executed with a
	/// subcommand. In such cases, this helps ensure that the unnamed argument
	/// boundary is correctly set in cases where no keys were passed.
	///
	/// This method always transparently returns `self`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new().with_subcommand();
	/// ```
	pub const fn with_subcommand(mut self) -> Self {
		self.last_offset = true;
		self
	}

	#[must_use]
	/// # Print Name/Version.
	///
	/// Similar to `with_help()`, this method can be chained to `new()` to
	/// print the program name and version, then exit with a status code of
	/// `0`, if either "-V" or "--version" flags are present.
	///
	/// If no version flags are found, `self` is transparently passed through.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new()
	///     .with_version(b"My App", b"1.5");
	/// ```
	pub fn with_version(self, name: &[u8], version: &[u8]) -> Self {
		if self.keys.contains2("-V", "--version") {
			Msg::from([name, b" v", version]).println();
			exit(0);
		}

		self
	}

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
	/// let args: Vec<String> = Argue::new().take();
	/// ```
	pub fn take(self) -> Vec<String> { self.args }



	// ------------------------------------------------------------------------
	// Getters
	// ------------------------------------------------------------------------

	#[must_use]
	/// # First Entry.
	///
	/// Borrow the first entry, if any.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new();
	///
	/// if let Some("happy") = args.peek() { ... }
	/// ```
	pub fn peek(&self) -> Option<&str> {
		if self.args.is_empty() { None }
		else { Some(&self.args[0]) }
	}

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
	/// use fyi_menu::Argue;
	///
	/// let mut args = Argue::new().with_any();
	///
	/// // This is actually safe because with_any() would have errored out if
	/// // nothing were present.
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
	/// let mut args = Argue::new();
	/// let switch: bool = args.switch("--my-switch");
	/// ```
	pub fn switch(&self, key: &str) -> bool { self.keys.contains(key) }

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
	/// let mut args = Argue::new();
	/// let switch: bool = args.switch2("-s", "--my-switch");
	/// ```
	pub fn switch2(&self, short: &str, long: &str) -> bool {
		self.keys.contains2(short, long)
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
	/// let mut args = Argue::new();
	/// let opt: Option<&str> = args.option("--my-opt");
	/// ```
	pub fn option(&mut self, key: &str) -> Option<&str> {
		if let Some(mut idx) = self.keys.get(key) {
			idx += 1;
			if idx < self.args.len() {
				// We might need to update the arg boundary since this is +1.
				self.update_last(idx);
				return Some(&self.args[idx]);
			}
		}

		None
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
	/// let mut args = Argue::new();
	/// let opt: Option<&str> = args.option2("-o", "--my-opt");
	/// ```
	pub fn option2(&mut self, short: &str, long: &str) -> Option<&str> {
		if let Some(mut idx) = self.keys.get2(short, long) {
			idx += 1;
			if idx < self.args.len() {
				// We might need to update the arg boundary since this is +1.
				self.update_last(idx);
				return Some(&self.args[idx]);
			}
		}

		None
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
	/// let mut args = Argue::new();
	/// let extras: &[String] = args.args();
	/// ```
	pub fn args(&self) -> &[String] {
		let idx = self.arg_idx();
		if idx < self.args.len() {
			&self.args[idx..]
		}
		else { &[] }
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
	/// let mut args = Argue::new();
	/// let opt: String = args.take_arg();
	/// ```
	pub fn take_arg(&mut self) -> String {
		let idx = self.arg_idx();
		if idx >= self.args.len() {
			die(b"Missing required argument.");
			unreachable!();
		}

		self.args.remove(idx)
	}



	// ------------------------------------------------------------------------
	// Internal
	// ------------------------------------------------------------------------

	/// # Arg Index.
	///
	/// This is an internal method that returns the index at which the first
	/// unnamed argument may be found.
	///
	/// Note: the index may be out of range, but won't be used in that case.
	const fn arg_idx(&self) -> usize {
		if self.keys.is_empty() && ! self.last_offset { 0 }
		else { self.last + 1 }
	}

	/// # Update Last Read.
	///
	/// Set the position of the last known key (or key value) to `idx`. If
	/// `idx` is less than the previous value for some reason, no action is
	/// taken.
	fn update_last(&mut self, idx: usize) {
		if idx > self.last {
			self.last = idx;
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

		let mut args = Argue::from_iter(base.clone()).with_separator();
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
		assert_eq!(args.option("--key"), Some("Val"));
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
	}
}
