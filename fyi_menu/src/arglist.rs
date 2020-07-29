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

use crate::{
	FLAG_ALL,
	die,
	parse_args,
	utility::vec_retain_explain,
};
use std::{
	env,
	ops::{
		BitOrAssign,
		Deref,
	},
};



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
		parse_args(&mut out, FLAG_ALL);
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

	#[must_use]
	/// Check First Entry
	///
	/// Return the first entry without draining it.
	pub fn peek(&self) -> Option<&str> {
		if self.0.is_empty() { None }
		else { Some(&self.0[0]) }
	}

	#[must_use]
	/// Pluck Next
	///
	/// Return the next thing, whatever that is. This is generally only useful
	/// as a first operation.
	pub fn pluck_next(&mut self) -> Option<String> {
		if self.0.is_empty() { None }
		else { Some(self.0.remove(0)) }
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
			die(b"Missing command.");
			unreachable!();
		}
	}

	/// Pluck Flags
	///
	/// Drain all occurrences of the flags and return the results.
	///
	/// Panics if `keys` are empty, or `keys` and `values` have different lengths.
	pub fn pluck_flags<U>(&mut self, flags: &mut U, keys: &[&str], values: &[U])
	where U: Copy + BitOrAssign {
		assert!(! keys.is_empty() && keys.len() == values.len());

		let len: usize = self.0.len();
		if 0 == len { return; }

		// This is basically what `Vec.retain()` does, except we're hitting
		// multiple patterns at once.
		let mut del: usize = 0;
		let ptr = self.0.as_mut_ptr();
		unsafe {
			let k_len: usize = keys.len();
			let k_ptr = keys.as_ptr();
			let v_ptr = values.as_ptr();

			// Outer loop; check our remaining values.
			let mut idx: usize = 0;
			while idx < len {
				let needle: &str = (*ptr.add(idx)).as_str();
				let last_del: usize = del;

				// Inner loop: check for matches.
				let mut k_idx = 0;
				while k_idx < k_len {
					// Update the flags and remove the index.
					if (*k_ptr.add(k_idx)).eq(needle) {
						*flags |= *v_ptr.add(k_idx);
						del += 1;
						break;
					}

					// Keep looking.
					k_idx += 1;
				}

				// If this wasn't a match, we might need to shift the value
				// down.
				if del > 0 && last_del == del {
					ptr.add(idx).swap(ptr.add(idx - del));
				}

				// Keep looking.
				idx += 1;
			}
		}

		// If we found anything, we need to run `truncate()` to free the
		// memory.
		if del > 0 {
			self.0.truncate(len - del);
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
		vec_retain_explain(&mut self.0, cb)
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
					die(&[
						b"Missing option value: ",
						self.0[idx].as_bytes(),
					].concat());
				}

				// Remove the key and value.
				out = self.0.drain(idx..idx+2).nth(1);
				len -= 2;
			}
			else { idx += 1; }
		}

		out
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
			die(b"Missing trailing argument(s).");
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
				! vec_retain_explain(&mut self.0, |x| x != "" && ! x.starts_with('-')) ||
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
			die(b"Missing trailing argument.");
			unreachable!();
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_extract() {
		let mut args = ArgList::from(vec!["com", "-v", "-a", "", "-q", "--", "Apple Sauce"]);

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

		// Let's start over and pull multiple flags at once.
		args = ArgList::from(vec!["-v", "--help", "--num", "100", "-q", "One Arg", "Two Arg"]);

		const FLAG_H: u8 = 0b0001;
		const FLAG_Q: u8 = 0b0010;
		const FLAG_V: u8 = 0b0100;
		const FLAG_Z: u8 = 0b1000;

		let mut flags: u8 = 0;
		args.pluck_flags(
			&mut flags,
			&["--help", "-q", "-v", "-Z"],
			&[FLAG_H, FLAG_Q, FLAG_V, FLAG_Z],
		);

		assert!(0 != (flags & FLAG_H));
		assert!(0 != (flags & FLAG_Q));
		assert!(0 != (flags & FLAG_V));
		assert!(0 == (flags & FLAG_Z));

		assert_eq!(
			*args,
			[String::from("--num"), String::from("100"), String::from("One Arg"), String::from("Two Arg")][..]
		);

		// Let's start over and look at options and multi-args.
		args = ArgList::from(vec!["-v", "1.0", "--num", "100", "-q", "One Arg", "Two Arg"]);

		// No command this time.
		assert_eq!(args.peek(), Some("-v"));
		assert_eq!(args.pluck_command(), None);

		// Grab the version.
		assert_eq!(args.pluck_opt(|x| x == "-v" || x == "--version"), Some(String::from("1.0")));
		assert_eq!(
			*args,
			[String::from("--num"), String::from("100"), String::from("-q"), String::from("One Arg"), String::from("Two Arg")][..]
		);

		// Get the remaining options out of the way.
		assert!(args.pluck_switch(|x| x != "-q"));
		assert_eq!(args.pluck_opt(|x| x == "--num"), Some(String::from("100")));

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
