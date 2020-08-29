/*!
# FYI: Argue

This is a very minimalistic CLI argument parser. It is significantly smaller
and faster than `clap`, but only because it is *opinionated*. Haha.

One important benefit (or drawback) to `Argue` is that it does not hold any
specific knowledge about the supported or expected options. This is great for
performance and allows for more flexible/dynamic parsing, but means handling is
ultimately up to the implementing library.

Nobody is in agreement on how CLI arguments should be formatted. To that end,
the assumptions this library makes are:
* A maximum of 16 keys are supported.
* Short options are only ever a single char. Anything after that single char (e.g "Val" in "-kVal") will be considered a value and broken off into its own entry.
* Long option key/value pairs (e.g. "--key=val") are likewise split into their own entries.
* Options may only ever appear once.
* Options with values may only have one value.
* Options must appear before unnamed arguments.
* Leading empty entries are trimmed from the set, but trailing ones are preserved.
* Anything appearing after a separator ("--") is requoted and glued together, replacing the separator entry.
* Unnamed trailing arguments are assumed to exist after the last detected option, or that option's value (if you e.g. called `option()` to retrieve it).

Some special helpers are provided in the form of builder options. See the
various `with_*()` methods for more information.

Once seeded, you can either dereference the struct into a slice of `String`s to
work your manual handling magic, or keep it in object form and use methods like
`switch()`, `option()`, and `args()` to retrieve values as needed.

## Examples

```no_run
use fyi_menu::Argue;

let mut args = Argue::new()
    .with_any() // Require at least one entry or error out.
    .with_version("App Name") // Print a version screen if version flags are found.
    .with_help(custom_callback); // Run help callback if help flags are found.

let something: bool = args.switch("-s");
let value: Option<&str> = args.option("--my-opt");
let remaining: &[String] = args.args();
```
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
/// Argue.
///
/// This is the main point of the library! See the module level documentation
/// for more information.
pub struct Argue {
	/// Parsed arguments.
	args: Vec<String>,
	/// Keys found mapped to their index in `self.args`.
	keys: KeyMaster,
	/// The last known key/value index.
	///
	/// Put another way, all entries in `self.args` between `0..=self.last` are
	/// either option keys or values. Beginning with `self.last + 1` are all
	/// the unnamed trailing arguments.
	last: usize,
	/// Subcommand? This modifies the arg boundary in cases where no keys are
	/// present.
	last_offset: bool,
}

impl Default for Argue {
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
	fn from_iter<I: IntoIterator<Item=String>>(src: I) -> Self {
		Self::from(src.into_iter())
	}
}

impl Argue {
	// ------------------------------------------------------------------------
	// Builder
	// ------------------------------------------------------------------------

	#[must_use]
	/// New.
	///
	/// Populate arguments from `std::env::args()`. The first (path) part is
	/// automatically excluded.
	///
	/// To construct an `Argue` from arbitrary raw values, use the
	/// `Argue::from_iter()` method (provided via the `iter::FromIterator`
	/// trait).
	pub fn new() -> Self {
		Self::from(env::args().skip(1))
	}

	/// With Entry.
	///
	/// This is used to fold iterator entries into the collection. It is
	/// slightly more efficient to blindly collect and handle keys with a
	/// secondary loop, but this approach is significantly cleaner.
	fn fold_entry(mut self, mut e: String) -> Self {
		match KeyKind::from(e.as_bytes()) {
			// Passthru.
			KeyKind::None => { self.args.push(e); },
			// Record the keys and passthru.
			KeyKind::Short | KeyKind::Long => {
				let idx: usize = self.args.len();
				if ! self.keys.insert(&e, idx) { die(b"Duplicate key."); }
				self.args.push(e);
				self.last = idx;
			},
			// Split a short key/value pair.
			KeyKind::ShortV => {
				let idx: usize = self.args.len();
				let tmp: String = e.split_off(2);
				if ! self.keys.insert(&e, idx) { die(b"Duplicate key."); }
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

				if ! self.keys.insert(&e, idx) { die(b"Duplicate key."); }
				self.args.push(e);
				self.args.push(tmp);
				self.last = idx + 1;
			},
		}

		self
	}

	#[must_use]
	/// Assert Non-Empty
	///
	/// Chain this method to `new()` to print an error and exit with status
	/// code `1` in cases where no CLI arguments were present.
	///
	/// If arguments are found, this just transparently returns `self`.
	pub fn with_any(self) -> Self {
		if self.args.is_empty() {
			die(b"Missing options, flags, arguments, and/or ketchup.");
		}

		self
	}

	#[must_use]
	/// Print Help.
	///
	/// Chain this method to `new()` to run a user-supplied callback if the
	/// options `[-h, --help, help]` are present, terminating the run
	/// afterward with exit code `0`.
	///
	/// In cases where these flags appear in the middle, and the first entry in
	/// the set is a "subcommand", that subcommand value is passed to the
	/// callback. (This allows an app to implement different help screens
	/// depending on context.) If the first entry is just another key or
	/// something, `None` is passed to the callback instead.
	///
	/// If no help flags are found, this just transparently returns `self`.
	pub fn with_help<F>(self, cb: F) -> Self
	where F: Fn(Option<&str>) {
		if let Some(x) = self.peek() {
			// Is "help" the subcommand?
			if x == "help" {
				cb(None);
				exit(0);
			}
			// Check the flags.
			else if self.keys.contains2("-h", "--help") {
				cb(
					if x.as_bytes()[0] == b'-' { None }
					else { Some(x) }
				);
				exit(0);
			}
		}

		self
	}

	#[must_use]
	/// Parse Paths From List.
	///
	/// Most of our own apps that use this library are designed to accept any
	/// number of trailing paths as unnamed arguments, or parse paths out of a
	/// text file specified by `[-l, --list]`.
	///
	/// To that end, this method can be chained to `new()` to see if there is a
	/// list flag, and if so, read that file and append each of its lines to
	/// the result set as unnamed arguments. (This way those arguments and any
	/// others can just get returned via `args()` later on.)
	///
	/// Note: this doesn't pass any judgments on the contents of the file.
	///
	/// This method always transparently returns `self`.
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
	/// With Separator.
	///
	/// If the arguments contain an "--" separator, it is replaced with all of
	/// the bits after it, requoted as necessary.
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
	/// With Subcommand.
	///
	/// In cases where no keys are present, the arg boundary defaults to the
	/// first entry. This method can be used to change the minimum to `1`. It
	/// has no effect if keys are present, since they'll set the boundary for
	/// us.
	pub const fn with_subcommand(mut self) -> Self {
		self.last_offset = true;
		self
	}

	#[must_use]
	/// Print Version.
	///
	/// Similar to `with_help()`, this method can be chained to `new()` to
	/// print the program name and version, then exit with a status code of
	/// `0` if any `[-V, --version]` flags are present.
	///
	/// If no version flags are found, `self` is transparently passed through.
	pub fn with_version(self, name: &[u8], version: &[u8]) -> Self {
		if self.keys.contains2("-V", "--version") {
			Msg::from([name, b" v", version].concat()).println();
			exit(0);
		}

		self
	}

	#[allow(clippy::missing_const_for_fn)] // Doesn't work!
	#[must_use]
	/// Take.
	///
	/// Drop the struct and return the parsed results as an owned
	/// `Vec<String>`.
	///
	/// Unless `take_arg()` was called previously, the vector should represent
	/// all of the original arguments (albeit reformatted).
	pub fn take(self) -> Vec<String> { self.args }



	// ------------------------------------------------------------------------
	// Getters
	// ------------------------------------------------------------------------

	#[must_use]
	/// First Entry.
	///
	/// Borrow the first entry, if any.
	pub fn peek(&self) -> Option<&str> {
		if self.args.is_empty() { None }
		else { Some(&self.args[0]) }
	}

	#[must_use]
	/// First Entry.
	///
	/// Borrow the first entry.
	///
	/// # Safety
	///
	/// This assumes a first argument exists. It will panic if not.
	pub unsafe fn peek_unchecked(&self) -> &str {
		&self.args[0]
	}

	#[must_use]
	/// Switch.
	///
	/// Returns `true` if the switch is present, `false` if not.
	pub fn switch(&self, key: &str) -> bool {
		self.keys.contains(key)
	}

	#[must_use]
	/// Switch (short/long).
	///
	/// This is just like `switch()`, except it checks for both a short and
	/// long key, returning `true` if either were present.
	pub fn switch2(&self, short: &str, long: &str) -> bool {
		self.keys.contains2(short, long)
	}

	/// Option.
	///
	/// Return the value corresponding to `key`, if present. Multi-value
	/// options are not supported.
	///
	/// Note: this method requires mutable access to `self` because it can
	/// potentially nudge the option/argument boundary +1 to the right. (Argue
	/// knows during parsing where the last keys are, but doesn't know which of
	/// those keys have values until `option()` or `option2()` are called.)
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

	/// Option (short/long).
	///
	/// This is just like `option()`, except it checks for both a short and
	/// long key, returning the first match found.
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
	/// (Remaining) Arguments.
	///
	/// This returns a slice from the end of the result set assumed to
	/// represent unnamed arguments. The boundary for the split defaults to
	/// after the last position of a key, but might change +1 if that last key
	/// is later found (via a call to `option()` or `option2()`) to contain a
	/// value.
	///
	/// If there are no arguments, an empty slice is returned.
	pub fn args(&self) -> &[String] {
		let idx = self.arg_idx();
		if idx < self.args.len() {
			&self.args[idx..]
		}
		else { &[] }
	}

	#[must_use]
	/// Take Arg.
	///
	/// Return the first available argument — draining it from the collection —
	/// or print an error message and exit.
	///
	/// This method is intended for use in cases where exactly one argument is
	/// expected and required. All other cases should just call `args()`.
	pub fn take_arg(&mut self) -> String {
		let idx = self.arg_idx();
		if idx >= self.args.len() {
			die(b"Missing required argument.");
		}

		self.args.remove(idx)
	}



	// ------------------------------------------------------------------------
	// Internal
	// ------------------------------------------------------------------------

	/// Arg Index.
	///
	/// Return the index arguments are expected to begin at.
	const fn arg_idx(&self) -> usize {
		if self.keys.is_empty() && ! self.last_offset { 0 }
		else { self.last + 1 }
	}

	/// Update Last Read.
	///
	/// The boundary between options/values and trailing arguments is noted
	/// by the last known key (or its value).
	fn update_last(&mut self, idx: usize) {
		if idx > self.last {
			self.last = idx;
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

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
