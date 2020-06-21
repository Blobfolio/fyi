/*!
# FYI Menu: CLI Menu

The `ArgList` struct is a very simple alternative to the very robust `clap`
crate, but it tackles things from a different angle.

Rather than acting as both God and Gateway — incurring setup cost and parsing
cost — `ArgList` leaves the logic and the knowing to the implementing app. It
simple holds a copy of the raw `std::env::args()` — minus the first part for
program path — and removes/returns anything as it is explicitly asked for.

Because requests are draining, request ordering might need to be considered.
For example, all named options and switches should be checked *before* fetching
trailing argument(s).

Help and version and other `clap` niceties have to be implemented separately,
but such is life.
*/

use fyi_msg::Msg;
use std::{
	borrow::Borrow,
	env,
	ops::{Deref, DerefMut},
	process,
};



#[derive(Debug, Clone, Hash, PartialEq)]
/// Argument List.
///
/// This is a very crude and simple draining CLI argument parser library. It is
/// left to implementing programs to ask for the right things, in the right
/// order, and take whatever actions necessary.
pub struct ArgList(Vec<String>);

impl Default for ArgList {
	/// Default.
	fn default() -> Self {
		Self(env::args().skip(1).collect())
	}
}

impl Deref for ArgList {
	type Target = Vec<String>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for ArgList {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl From<Vec<&str>> for ArgList {
	#[inline]
	fn from(env: Vec<&str>) -> Self {
		Self(env.into_iter().map(String::from).collect())
	}
}

impl From<Vec<String>> for ArgList {
	#[inline]
	fn from(env: Vec<String>) -> Self {
		Self(env)
	}
}

impl ArgList {
	/// Peek First.
	pub fn peek_first(&self) -> Result<&str, ()> {
		if self.0.is_empty() { Err(()) }
		else {
			Ok(&self.0[0])
		}
	}

	/// Expect Something
	///
	/// If the `ArgList` is empty, `::die()` will be called.
	pub fn expect_any(&mut self) {
		if self.0.is_empty() {
			Self::die("Missing required arguments.");
			unreachable!()
		}
	}

	/// Extract Command
	///
	/// If the first argument does not begin with a hypen, it is removed from
	/// the store and returned.
	pub fn extract_command(&mut self) -> Result<String, ()> {
		if self.0.is_empty() || self.0[0].starts_with('-') { Err(()) }
		else {
			Ok(self.0.remove(0))
		}
	}

	/// Expect Command
	///
	/// Same as `extract_command`, except it will call `::die()` on failure.
	pub fn expect_command(&mut self) -> String {
		if let Ok(com) = self.extract_command() { com }
		else {
			Self::die("Missing required subcommand.");
			unreachable!()
		}
	}

	/// Extract Switch
	///
	/// Remove all matching instances from the store and return a boolean
	/// indicating whether or not any were present.
	pub fn extract_switch(&mut self, keys: &[&str]) -> bool {
		match (self.0.len(), keys.len()) {
			(0, _) | (_, 0) => false,
			(l, 1) => {
				self.0.retain(|x| *x != keys[0]);
				l != self.0.len()
			},
			(l, _) => {
				self.0.retain(|x| ! keys.contains(&x.as_str()));
				l != self.0.len()
			}
		}
	}

	/// Extract String Option
	///
	/// Remove all matching instances from the store and return the String
	/// value if present.
	pub fn extract_opt(&mut self, keys: &[&str]) -> Result<String, ()> {
		let len: usize = self.0.len();
		if len < 2 || keys.is_empty() { Err(()) }
		else if let Some(idx) = self.0.iter().position(|x| keys.contains(&x.as_str())) {
			// There must be a non-hyphen-starting value.
			if idx + 1 < len && ! self.0[idx + 1].starts_with('-') {
				// Remove the key.
				self.0.remove(idx);

				// Make sure there are no more occurrences of any keys.
				if self.0.iter().any(|x| keys.contains(&x.as_str())) {
					Self::die("Duplicate option.");
					unreachable!();
				}

				// Remove the value.
				let val = self.0.remove(idx);
				if val.is_empty() { Err(()) }
				else { Ok(val) }
			}
			else {
				Self::die("Missing option value.");
				unreachable!();
			}
		}
		else { Err(()) }
	}

	/// Expect Option
	///
	/// Same as `extract_opt`, except it will call `::die()` on failure.
	pub fn expect_opt(&mut self, keys: &[&str]) -> String {
		if let Ok(opt) = self.extract_opt(keys) { opt }
		else {
			Self::die("Missing required option.");
			unreachable!()
		}
	}

	/// Extract Numeric Option
	///
	/// Same as `extract_opt`, except the value is parsed as a `usize` and
	/// will fail if that isn't possible.
	pub fn extract_opt_usize(&mut self, keys: &[&str]) -> Result<usize, ()> {
		self.extract_opt(keys)
			.and_then(|x| x.parse::<usize>().map_err(|_| ()))
	}

	/// Expect Usize Option
	///
	/// Same as `extract_opt_usize`, except it will call `::die()` on failure.
	pub fn expect_opt_usize(&mut self, keys: &[&str]) -> usize {
		if let Ok(opt) = self.extract_opt_usize(keys) { opt }
		else {
			Self::die("Missing required option.");
			unreachable!()
		}
	}

	/// Extract Arg
	///
	/// There should be one and only one remaining argument (possibly after an
	/// "--"). It will be removed from the store and returned if present.
	pub fn extract_arg(&mut self) -> Result<String, ()> {
		self.extract_args()
			.and_then(|mut x| match x.len() {
				0 => Err(()),
				1 => Ok(x.remove(0)),
				_ => {
					Self::die("Unexpected arguments.");
					unreachable!()
				}
			})
	}

	/// Expect Arg
	///
	/// Same as `extract_arg`, except it will call `::die()` on failure.
	pub fn expect_arg(&mut self) -> String {
		if let Ok(arg) = self.extract_arg() { arg }
		else {
			Self::die("Missing required argument.");
			unreachable!()
		}
	}

	/// Extract Arg(s)
	///
	/// Drain all remaining arguments from the store, returning any with valid
	/// length.
	pub fn extract_args(&mut self) -> Result<Vec<String>, ()> {
		if self.0.is_empty() { Err(()) }
		else {
			let out: Vec<String> = self.0.drain(..)
				.filter(|x| *x != "--" && ! x.is_empty())
				.collect();
			if out.is_empty() { Err(()) }
			else { Ok(out) }
		}
	}

	/// Expect Arg
	///
	/// Same as `extract_args`, except it will call `::die()` on failure.
	pub fn expect_args(&mut self) -> Vec<String> {
		if let Ok(arg) = self.extract_args() { arg }
		else {
			Self::die("Missing required argument(s).");
			unreachable!()
		}
	}

	/// Print an Error and Exit
	///
	/// Note: The error message should include a trailing line break.
	pub fn die<S: Borrow<str>>(msg: S) {
		eprintln!("{}", &Msg::error(msg));
		process::exit(1);
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_extract() {
		let mut args = ArgList(vec!["com".to_string(), "-v".to_string(), "-a".to_string(), "".to_string(), "-q".to_string(), "--".to_string(), "Apple Sauce".to_string()]);

		// Not an assertion, per se, but we should have arguments.
		args.expect_any();

		// Check the first.
		assert_eq!(args.peek_first(), Ok("com"));
		let com = args.extract_command();
		assert_eq!(com, Ok("com".to_string()));

		// Check for -v and -a.
		assert!(args.extract_switch(&["-v", "-a"]));

		// Let's make sure extractions have been draining.
		assert_eq!(*args, vec!["".to_string(), "-q".to_string(), "--".to_string(), "Apple Sauce".to_string()]);

		// Check for something that doesn't exist.
		assert!(! args.extract_switch(&["--perfection"]));

		// Get -q out of the way so we can look at the trailing arg.
		assert!(args.extract_switch(&["-q"]));
		assert_eq!(args.extract_arg(), Ok("Apple Sauce".to_string()));

		// Empty?
		assert!(args.is_empty());

		// Let's start over and look at options and multi-args.
		args = ArgList(vec!["-v".to_string(), "1.0".to_string(), "--num".to_string(), "100".to_string(), "-q".to_string(), "One Arg".to_string(), "Two Arg".to_string()]);
		args.expect_any();

		// No command this time.
		assert_eq!(args.peek_first(), Ok("-v"));
		assert_eq!(args.extract_command(), Err(()));

		// Grab the version.
		assert_eq!(args.extract_opt(&["-v", "--version"]), Ok("1.0".to_string()));
		assert_eq!(*args, vec!["--num".to_string(), "100".to_string(), "-q".to_string(), "One Arg".to_string(), "Two Arg".to_string()]);

		// Grab the num as a number.
		assert_eq!(args.extract_opt_usize(&["--num"]), Ok(100_usize));

		// Get the -q out of the way.
		assert!(args.extract_switch(&["-q"]));

		// Now check that we get args!
		assert_eq!(args.extract_args(), Ok(vec!["One Arg".to_string(), "Two Arg".to_string()]));
		assert!(args.is_empty());
	}
}
