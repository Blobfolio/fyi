/*!
# FYI Menu: CLI Menu

The `ArgList` struct is a very simple getopt-like command line argument parser.
Unlike (much more robust) crates like `clap` or `getopts`, it doesn't require
upfront knowledge about the program or all possible flags; it merely holds the
list and drains/returns matches as they are requested.

This lack of awareness makes `ArgList` incredibly fast, but requires orderly
handling on the app side. For example, `pluck_command()` needs to be called
first, and `pluck_args()` needs to be called last, or the data returned might
not make sense. (`pluck_switch()` and `pluck_opt()` can go anywhere in between,
but it is recommended to collect all switches first to reduce the number of
iteration operations.)

`ArgList` differentiates between four different types of arguments. Using, as
an example, the command `git clone -q -b master git://url.com/repo`:
1. "Command": a leading, non-dash-prefixed word, like "clone".
2. "Switch": a dash-prefixed key that is or isn't, like "-q" or "--quiet".
3. "Option": a dash-prefixed key that is followed by a value, like "-b".
4. "Argument": all unqualified, trailing values, like the "<git://url.com/repo>".

Keys cannot be grouped or repeated. `ArgList` draws the line there. Haha. For
example, given two short keys, "q" and "v", they must be written "-q -v", not
"-qv".

Option values can be entered either of three different ways:
1. "-k value" or "--key value".
2. "-kvalue".
3. "--key=value".

If a separator "--" is present, all trailing arguments will be glued into a
single, quoted value at that spot.

For example, calling `pluck_args()` on `-q -v arg1 arg2 arg3` would return 3
values, `["arg1", "arg2", "arg3"]`, but calling it on
`-q -v -- arg1 arg2 arg3` would instead yield a single argument,
`["arg1 arg2 arg3"]`.

Additionally, because the extra bits are stringified, elements requiring shell
escaping are wrapped in single quotes (with backslashes behind any inner single
quotes in that value), so you'd get something like
"--ver mongoose --msg 'hello world'". Empty values are converted to `''` to
keep them from disappearing.

`ArgList` can also be dereferenced to a slice of strings if the built-in
fetchers are too limiting. This can be handy for fetching arguments after a
separator in cases where those arguments are "keys", which the `pluck_arg()`
method would reject.
*/

use fyi_msg::MsgKind;
use std::{
	borrow::Borrow,
	cmp::Ordering,
	env,
	ops::Deref,
	process,
};



/// Escape Chars
///
/// This returns `true` if a character requires escaping (for e.g. the shell).
fn escape_chars(ch: char) -> bool {
	match ch {
		'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '=' | '/' | ',' | '.' | '+' => false,
		_ => true,
	}
}

#[must_use]
#[allow(clippy::suspicious_else_formatting)]
/// Escape String (for shell)
///
/// For our purposes, we just want to ensure strings requiring quotes are
/// single-quoted so as not to be lost entirely, but additional validation
/// is definitely needed before attempting to use them!
pub fn escape(mut s: String) -> String {
	// Empty strings need to be quoted so as not to disappear.
	if s == "" { String::from("''") }
	// We need to quote it, and escape any single quotes already within it.
	else if s.contains(escape_chars) {
		unsafe {
			let v = s.as_mut_vec();

			{
				let mut idx: usize = 0;
				let mut len: usize = v.len();
				while idx < len {
					match v[idx] {
						// Replace existing backslashes with forward slashes.
						// This ain't a Windows library. Haha.
						b'\\' => {
							v[idx] = b'/';
							idx += 1;
						},
						// Backslash any existing quotes.
						b'\'' => {
							v.insert(idx, b'\\');
							idx += 2;
							len += 1;
						},
						// Everything else can just pass right on through.
						_ => { idx += 1; }
					}
				}
			}

			// Add quotes to the front and back when we're done!
			v.reserve(2);
			v.insert(0, b'\'');
			v.push(b'\'');
		}

		s
	}
	// Send it through unchanged.
	else { s }
}

/// Is Byte a Letter
///
/// Keys need an [a-z] or [A-Z] letter following the dash.
fn is_letter(data: u8) -> bool {
	match data {
		b'a'..=b'z' | b'A'..=b'Z' => true,
		_ => false,
	}
}

/// Count Leading Dashes
fn leading_dashes(data: &[u8]) -> usize {
	match data.len().cmp(&2) {
		// Options must be at least 2 letters long.
		Ordering::Less => 0,
		// This could be a short option.
		Ordering::Equal =>
			if data[0] == b'-' && is_letter(data[1]) { 1 }
			else { 0 },
		// This could be a short option (with value) or a long option (with or
		// without value) or neither.
		Ordering::Greater =>
			if data[0] == b'-' {
				if data[1] == b'-' && is_letter(data[2]) { 2 }
				else if is_letter(data[1]) { 1 }
				else { 0 }
			}
			else { 0 },
	}
}

#[must_use]
/// Parse Key and/or Value From Raw
pub fn parse_kv(raw: &str) -> (Option<usize>, Option<usize>) {
	let bytes: &[u8] = raw.as_bytes();
	let len: usize = raw.len();
	match leading_dashes(bytes).cmp(&1) {
		// No dashes, treat as a value.
		Ordering::Less => (None, Some(len)),
		// One dash.
		Ordering::Equal =>
			// If the key is all there is, we're done.
			if len == 2 { (Some(len), None) }
			// We can't have something like "-v-v".
			else if bytes[2] == b'-' { (None, None) }
			// Split it down the middle.
			else { (Some(2), Some(len - 2)) }
		// Two dashes.
		Ordering::Greater => {
			let mut idx = 3;
			while idx < len {
				if bytes[idx] == b'=' {
					if idx + 1 < len {
						return (Some(idx), Some(len - idx - 1));
					}
					else {
						return (Some(idx), Some(0));
					}
				}

				idx += 1;
			}

			(Some(len), None)
		}
	}
}



/// Traits for Vec<String>
///
/// We use `retain()` in a lot of places, but usually just want to know if
/// anything changed. We'll give Vec that power via a trait to make our lives
/// easier.
pub trait ArgListVec {
	/// Retain With Answer
	///
	/// Identical to `retain()` except a boolean is returned.
	fn retain_explain<F> (&mut self, f: F) -> bool
	where F: FnMut(&String) -> bool;
}

impl ArgListVec for Vec<String> {
	/// Retain With Answer
	///
	/// Identical to `retain()` except it returns `true` if changes were made,
	/// `false` otherwise.
	fn retain_explain<F> (&mut self, mut f: F) -> bool
	where F: FnMut(&String) -> bool {
		let len = self.len();
		let mut del = 0;

		let ptr = self.as_mut_ptr();
		unsafe {
			let mut idx: usize = 0;
			while idx < len {
				if !f(&*ptr.add(idx)) {
					del += 1;
				}
				else if del > 0 {
					ptr.add(idx).swap(ptr.add(idx - del));
				}

				idx += 1;
			}
		}

		if del > 0 {
			self.truncate(len - del);
			true
		}
		else { false }
	}
}



#[derive(Debug, Clone, Hash, PartialEq)]
/// Argument List.
///
/// This is a very crude and simple draining CLI argument parser library. It is
/// left to implementing programs to ask for the right things, in the right
/// order, and take whatever actions necessary.
pub struct ArgList (Vec<String>);

impl Default for ArgList {
	/// Default from Env
	fn default() -> Self {
		Self::from(env::args().skip(1).collect::<Vec<String>>())
	}
}

impl Deref for ArgList {
	type Target = [String];

	fn deref(&self) -> &Self::Target {
		&self.0[..]
	}
}

impl From<Vec<&str>> for ArgList {
	fn from(out: Vec<&str>) -> Self {
		Self::from(out.into_iter().map(String::from).collect::<Vec<String>>())
	}
}

impl From<Vec<String>> for ArgList {
	fn from(mut out: Vec<String>) -> Self {
		let mut len: usize = out.len();
		let mut idx: usize = 0;
		let mut any: bool = false;

		// Loop it while we've got it.
		while idx < len {
			// Empty values.
			if out[idx] == "" {
				if any { idx += 1; }
				// Drop empty values from the front.
				else {
					out.remove(idx);
					len -= 1;
				}
			}
			// Stupid junk entries can always be removed.
			else if out[idx] == "-" || out[idx] == "---" {
				out.remove(idx);
				len -= 1;
			}
			// If we hit the separator, we need to reglue what comes after,
			// then we're done.
			else if out[idx] == "--" {
				// If there's nothing after, there's nothing to preserve.
				if idx + 1 == len {
					len -= 1;
					out.truncate(len);
				}
				else {
					out[idx] = out.drain(idx+1..)
						.map(escape)
						.collect::<Vec<String>>()
						.join(" ");
					break;
				}
			}
			// We have a value, a key, or both. Need to dig deeper!
			else {
				match parse_kv(&out[idx]) {
					// Key or Value.
					(Some(kv), None) | (None, Some(kv)) => {
						any = true;

						// We might need to shrink it.
						if kv != out[idx].len() {
							out[idx].truncate(kv);
						}

						idx += 1;
					},
					// Key *and* value.
					(Some(k), Some(v)) => {
						let el_len: usize = out[idx].len();

						// Value might be empty.
						if v == 0 {
							out.insert(idx + 1, String::default());
						}
						// Otherwise chop it off the end.
						else {
							out.insert(idx + 1, String::from(&out[idx][el_len-v..]));
						}

						// Truncate the key to its appropriate length.
						if k != el_len {
							out[idx].truncate(k);
						}

						len += 1;
						idx += 2;
					},
					// Bunk Entry!
					_ => {
						out.remove(idx);
						len -= 1;
					}
				}
			}
		}

		Self(out)
	}
}

impl ArgList {
	#[must_use]
	/// As Mut Vec.
	///
	/// This method can be used to obtain a mutable Vec of any remaining args
	/// (for e.g. manual parsing/inspection), but in general, it is safer and
	/// saner to use the struct's `pluck_*()` methods instead.
	pub fn as_mut_vec(&mut self) -> &mut Vec<String> {
		&mut self.0
	}

	/// Expect Something, Anything
	pub fn expect(&self) {
		if self.0.is_empty() {
			Self::die("Missing options, flags, arguments, and/or ketchup.");
			unreachable!();
		}
	}

	#[must_use]
	/// Check First Entry
	///
	/// Return the first entry without draining it.
	pub fn peek(&self) -> Option<&str> {
		if self.0.is_empty() { None }
		else { Some(&self.0[0]) }
	}

	/// Extract Command
	///
	/// Drain and return the first element if it is not a key-like thing.
	pub fn pluck_command(&mut self) -> Option<String> {
		if self.0.is_empty() || self.0[0].starts_with('-') { None }
		else {
			Some(self.0.remove(0))
		}
	}

	/// Expect Command
	///
	/// Wrap `pluck_command()` and fail on None.
	pub fn expect_command(&mut self) -> String {
		if let Some(com) = self.pluck_command() { com }
		else {
			Self::die("Missing command.");
			unreachable!();
		}
	}

	/// Extract Switch
	///
	/// Drain all occurrences from the store and return `true` if any were
	/// found.
	///
	/// Note: this method requires a *retaining* callback, meaning it should
	/// return the opposite of what you'd expect since we want to *remove*
	/// matching elements, not keep them.
	///
	/// In other words, `true` for non-match, `false` for match. Haha.
	pub fn pluck_switch<F> (&mut self, cb: F) -> bool
	where F: FnMut(&String) -> bool {
		self.0.retain_explain(cb)
	}

	/// Extract Option
	///
	/// Drain and return the option value from the store, if present.
	///
	/// Note: this method requires a *matching* callback, unlike
	/// `pluck_switch()`, so it should return TRUE on a match, FALSE on a non-
	/// match.
	pub fn pluck_opt<F> (&mut self, mut cb: F) -> Option<String>
	where F: FnMut(&String) -> bool {
		let mut out = None;
		let mut len: usize = self.0.len();
		let mut idx = 0;

		while idx < len {
			if cb(&self.0[idx]) {
				if idx + 1 == len {
					Self::die({
						let mut s: String = String::from("Missing option value: ");
						s.push_str(&self.0[idx]);
						s
					});
					unreachable!();
				}

				// Remove the key and value.
				out = self.0.drain(idx..idx+2).nth(1);
				len -= 2;
			}
			else { idx += 1; }
		}

		out
	}

	/// Extract Option (usize)
	///
	/// Same as `pluck_opt()`, except the value is returned as a usize. Parsing
	/// failures are treated as None, i.e. an error.
	pub fn pluck_opt_usize<F> (&mut self, cb: F) -> Option<usize>
	where F: FnMut(&String) -> bool {
		self.pluck_opt(cb)
			.and_then(|x| x.parse::<usize>().ok())
	}

	/// Convenience: Help
	pub fn pluck_help(&mut self) -> bool {
		self.0.retain_explain(|x| x != "-h" && x != "--help")
	}

	/// Convenience: Version
	pub fn pluck_version(&mut self) -> bool {
		self.0.retain_explain(|x| x != "-V" && x != "--version")
	}

	/// Pluck Arg(s)
	///
	/// Call this method last to grab whatever is left.
	pub fn pluck_args(&mut self) -> Option<Vec<String>> {
		let out: Vec<String> = self.0.drain(..)
			.filter(|x| x != "" && ! x.starts_with('-'))
			.collect();

		if out.is_empty() { None }
		else { Some(out) }
	}

	/// Expect Arg(s)
	///
	/// Wrap `pluck_args()` and fail on None.
	pub fn expect_args(&mut self) -> Vec<String> {
		if let Some(args) = self.pluck_args() { args }
		else {
			Self::die("Missing trailing argument(s).");
			unreachable!();
		}
	}

	/// Pluck Arg
	///
	/// Call this method last to grab the first of whatever is left.
	pub fn pluck_arg(&mut self) -> Option<String> {
		if
			! self.0.is_empty() &&
			(
				! self.0.retain_explain(|x| x != "" && ! x.starts_with('-')) ||
				! self.0.is_empty()
			)
		{
			Some(self.0.remove(0))
		}
		else { None }
	}

	/// Expect Arg
	///
	/// Wrap `pluck_arg()` and fail on None.
	pub fn expect_arg(&mut self) -> String {
		if let Some(arg) = self.pluck_arg() { arg }
		else {
			Self::die("Missing trailing argument.");
			unreachable!();
		}
	}

	/// Print an Error and Exit
	pub fn die<S: Borrow<str>>(msg: S) {
		MsgKind::Error.as_msg(msg).eprintln();
		process::exit(1);
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_extract() {
		let mut args = ArgList::from(vec!["com", "-v", "-a", "", "-q", "--", "Apple Sauce"]);

		// Not an assertion, per se, but we should have arguments.
		args.expect();

		// Check the first.
		assert_eq!(args.peek(), Some("com"));
		let com = args.pluck_command();
		assert_eq!(com, Some(String::from("com")));

		// Check for -v and -a.
		assert!(args.pluck_switch(|x| x != "-v" && x != "-a"));

		// Let's make sure extractions have been draining.
		assert_eq!(
			*args,
			[
				String::from(""),
				String::from("-q"),
				String::from("'Apple Sauce'")
			][..]
		);

		// Check for something that doesn't exist.
		assert!(! args.pluck_switch(|x| x != "--perfection"));

		// Get -q out of the way so we can look at the trailing arg.
		assert!(args.pluck_switch(|x| x != "-q"));
		assert_eq!(args.pluck_arg(), Some(String::from("'Apple Sauce'")));

		// Empty?
		assert!(args.is_empty());

		// Let's start over and look at options and multi-args.
		args = ArgList::from(vec!["-v", "1.0", "--num", "100", "-q", "One Arg", "Two Arg"]);
		args.expect();

		// No command this time.
		assert_eq!(args.peek(), Some("-v"));
		assert_eq!(args.pluck_command(), None);

		// Grab the version.
		assert_eq!(args.pluck_opt(|x| x == "-v" || x == "--version"), Some(String::from("1.0")));
		assert_eq!(
			*args,
			[String::from("--num"), String::from("100"), String::from("-q"), String::from("One Arg"), String::from("Two Arg")][..]
		);

		// Grab the num as a number.
		assert_eq!(args.pluck_opt_usize(|x| x == "--num"), Some(100_usize));

		// Get the -q out of the way.
		assert!(args.pluck_switch(|x| x != "-q"));

		// Now check that we get args!
		assert_eq!(args.pluck_args(), Some(vec![String::from("One Arg"), String::from("Two Arg")]));
		assert!(args.is_empty());
	}

	#[test]
	fn t_splitting() {
		let mut al = ArgList::from(vec!["-vValue", "--normal", "Normal", "--long=Long", "--empty="]);
		assert_eq!(al.pluck_opt(|x| x == "-v"), Some(String::from("Value")));
		assert_eq!(al.pluck_opt(|x| x == "--normal"), Some(String::from("Normal")));
		assert_eq!(al.pluck_opt(|x| x == "--long"), Some(String::from("Long")));
		assert_eq!(al.pluck_opt(|x| x == "--empty"), Some(String::from("")));
	}

	#[test]
	fn t_sep() {
		let al = ArgList::from(vec!["--", "--normal", "Normal", "", "--long=Long", "Hello 'Kitty'"]);
		assert_eq!(*al[0], String::from(
			"--normal Normal '' --long=Long 'Hello \\'Kitty\\''"
		));
	}
}