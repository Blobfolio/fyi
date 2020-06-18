/*!
# FYI Message

The `Msg` struct is an efficient way to construct a simple, printable, colored
`Prefix: Hello World`-type status message.

## Example:

```no_run
use fyi_msg::Msg;

// Create a message with a custom prefix and color.
let msg = Msg::new("Yo", 199, "How are you doing today?");

// Use a short-hand method to create a message with a pre-defined prefix:
let msg = Msg::error("Well darn.");
let msg = Msg::debug("I like cookies.");
let msg = Msg::success("Example executed!");
```
*/

use crate::{
	MsgBuf,
	utility::{
		ansi_code_bold,
		time_format_dd,
		whitespace,
	},
};
use std::{
	borrow::Borrow,
	cmp::Ordering,
	fmt,
	hash::{
		Hash,
		Hasher,
	},
	ops::Deref,
	str::FromStr,
};



/// Helper: Generate `Msg` preset methods like "Error:", "Success:", etc.
macro_rules! new_prefix {
	($fn:ident, $pre:expr, $prefix:expr) => {
		#[must_use]
		/// New Prefix + Msg
		pub fn $fn<T: Borrow<str>> (msg: T) -> Self {
			let msg = msg.borrow();
			if msg.is_empty() { Self::new_prefix_unchecked($pre, $prefix) }
			else { Self::new_prefix_msg_unchecked($pre, $prefix, msg.as_bytes()) }
		}
	};
}



/// The Message Partitions!
const IDX_INDENT: usize = 1;
const IDX_TIMESTAMP_PRE: usize = 2;  // ANSI.
const IDX_TIMESTAMP: usize = 3;
const IDX_TIMESTAMP_POST: usize = 4; // ANSI.
const IDX_PREFIX_PRE: usize = 5;     // ANSI.
const IDX_PREFIX: usize = 6;
const IDX_PREFIX_POST: usize = 7;    // ANSI.
const IDX_MSG_PRE: usize = 8;        // ANSI.
const IDX_MSG: usize = 9;
const IDX_MSG_POST: usize = 10;      // ANSI.

/// Other repeated bits.
//                                  \e   [   1    m
const LBL_MSG_PRE: &[u8] =        &[27, 91, 49, 109];
//                                   :  \e   [   0    m   •
const LBL_PREFIX_POST: &[u8] =    &[58, 27, 91, 48, 109, 32];
//                                  \e   [   0    m
const LBL_RESET: &[u8] =          &[27, 91, 48, 109];
//                                  \e   [   0   ;   2    m   ]  \e   [   0    m   •
const LBL_TIMESTAMP_POST: &[u8] = &[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32];
//                                  \e   [   2    m   [  \e   [   0   ;   3   4    m
const LBL_TIMESTAMP_PRE: &[u8] =  &[27, 91, 50, 109, 91, 27, 91, 48, 59, 51, 52, 109];



#[derive(Debug, Clone)]
/// The Message!
pub struct Msg(MsgBuf);

impl AsRef<str> for Msg {
	#[inline]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for Msg {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl Borrow<str> for Msg {
	#[inline]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl Borrow<[u8]> for Msg {
	#[inline]
	fn borrow(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl Default for Msg {
	#[inline]
	fn default() -> Self {
		Self(MsgBuf::splat(10))
	}
}

impl Deref for Msg {
	type Target = [u8];

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.as_bytes()
	}
}

impl Eq for Msg {}

impl fmt::Display for Msg {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl<'a> From<&'a str> for Msg {
	#[inline]
	fn from(msg: &'a str) -> Self {
		Self(MsgBuf::from(&[
			// Indentation and timestamp.
			&[], &[], &[], &[],
			// Prefix.
			&[], &[], &[],
			LBL_MSG_PRE,
			msg.as_bytes(),
			LBL_RESET,
		]))
	}
}

impl<'a> From<&'a [u8]> for Msg {
	#[inline]
	fn from(msg: &'a [u8]) -> Self {
		Self(MsgBuf::from(&[
			// Indentation and timestamp.
			&[], &[], &[], &[],
			// Prefix.
			&[], &[], &[],
			LBL_MSG_PRE,
			msg,
			LBL_RESET,
		]))
	}
}

impl FromStr for Msg {
	type Err = std::num::ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from(s))
	}
}

impl Hash for Msg {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state);
	}
}

impl PartialEq for Msg {
	fn eq(&self, other: &Self) -> bool {
		self.as_str() == other.as_str()
	}
}

impl PartialEq<&str> for Msg {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<[u8]> for Msg {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_str() == unsafe { std::str::from_utf8_unchecked(other) }
	}
}

impl PartialEq<&[u8]> for Msg {
	fn eq(&self, other: &&[u8]) -> bool {
		self.as_str() == unsafe { std::str::from_utf8_unchecked(*other) }
	}
}

impl PartialOrd for Msg {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.as_str().cmp(other.as_str()))
	}
}

impl PartialOrd<&str> for Msg {
	fn partial_cmp(&self, other: &&str) -> Option<Ordering> {
		Some(self.as_str().cmp(*other))
	}
}

impl PartialOrd<[u8]> for Msg {
	fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
		Some(self.as_str().cmp(unsafe { std::str::from_utf8_unchecked(other) }))
	}
}

impl PartialOrd<&[u8]> for Msg {
	fn partial_cmp(&self, other: &&[u8]) -> Option<Ordering> {
		Some(self.as_str().cmp(unsafe { std::str::from_utf8_unchecked(*other) }))
	}
}



impl Msg {
	// ------------------------------------------------------------------------
	// Public Static Methods
	// ------------------------------------------------------------------------

	#[must_use]
	/// New Prefix + Msg
	pub fn new<T1, T2> (prefix: T1, prefix_color: u8, msg: T2) -> Self
	where
	T1: Borrow<str>,
	T2: Borrow<str> {
		let prefix = prefix.borrow();
		let msg = msg.borrow();

		match (prefix.is_empty(), msg.is_empty()) {
			// Neither.
			(true, true) => Self::default(),
			// Both.
			(false, false) => Self::new_prefix_msg_unchecked(
				ansi_code_bold(prefix_color),
				prefix.as_bytes(),
				msg.as_bytes()
			),
			// Message only.
			(true, false) => Self::from(msg.as_bytes()),
			// Prefix only.
			(false, true) => Self::new_prefix_unchecked(
				ansi_code_bold(prefix_color),
				prefix.as_bytes()
			),
		}
	}



	// ------------------------------------------------------------------------
	// Private Static Methods
	// ------------------------------------------------------------------------

	#[must_use]
	/// New Prefix + Msg (Unchecked)
	fn new_prefix_msg_unchecked(prefix_pre: &[u8], prefix: &[u8], msg: &[u8]) -> Self {
		Self(MsgBuf::from(&[
			// Indentation and timestamp.
			&[], &[], &[], &[],
			prefix_pre,
			prefix,
			LBL_PREFIX_POST,
			LBL_MSG_PRE,
			msg,
			LBL_RESET,
		]))
	}

	#[must_use]
	/// New Prefix (Unchecked)
	fn new_prefix_unchecked(prefix_pre: &[u8], prefix: &[u8]) -> Self {
		Self(MsgBuf::from(&[
			// Indentation and timestamp.
			&[], &[], &[], &[],
			prefix_pre,
			prefix,
			LBL_RESET,
			// Message.
			&[], &[], &[],
		]))
	}



	// ------------------------------------------------------------------------
	// Public Methods
	// ------------------------------------------------------------------------

	/// Indent.
	pub fn set_indent(&mut self, indent: usize) {
		let len: usize = usize::min(10, indent) * 4;
		if 0 == len {
			self.0.clear_part(IDX_INDENT);
		}
		else {
			self.0.replace_part(IDX_INDENT, whitespace(len));
		}
	}

	/// Set Message.
	pub fn set_msg<T: Borrow<str>>(&mut self, msg: T) {
		let msg = msg.borrow();

		// Remove the message.
		if msg.is_empty() {
			if ! self.0.part_is_empty(IDX_MSG_PRE) {
				self.0.clear_part(IDX_MSG_PRE);
				self.0.clear_part(IDX_MSG);
				self.0.clear_part(IDX_MSG_POST);
			}

			// We might need to change the end of the prefix too.
			if ! self.0.part_is_empty(IDX_PREFIX_POST) {
				self.0.replace_part(IDX_PREFIX_POST, LBL_RESET);
			}
		}
		// Add or change it.
		else {
			// The opening and closing needs to be taken care of.
			if self.0.part_is_empty(IDX_MSG_PRE) {
				self.0.replace_part(IDX_MSG_PRE, LBL_MSG_PRE);
				self.0.replace_part(IDX_MSG_POST, LBL_RESET);
			}

			self.0.replace_part(IDX_MSG, msg.as_bytes());

			// We might need to change the end of the prefix too.
			if ! self.0.part_is_empty(IDX_PREFIX_POST) {
				self.0.replace_part(IDX_PREFIX_POST, LBL_PREFIX_POST);
			}
		}
	}

	/// Set Prefix.
	pub fn set_prefix<T: Borrow<str>>(&mut self, prefix: T, prefix_color: u8) {
		let prefix = prefix.borrow();

		// Remove the prefix.
		if prefix.is_empty() {
			if ! self.0.part_is_empty(IDX_PREFIX_PRE) {
				self.0.clear_part(IDX_PREFIX_PRE);
				self.0.clear_part(IDX_PREFIX);
				self.0.clear_part(IDX_PREFIX_POST);
			}
		}
		// Add or change it.
		else {
			self.0.replace_part(IDX_PREFIX_PRE, ansi_code_bold(prefix_color));
			self.0.replace_part(IDX_PREFIX, prefix.as_bytes());
			if self.0.part_is_empty(IDX_MSG_PRE) {
				self.0.replace_part(IDX_PREFIX_POST, LBL_RESET);
			}
			else {
				self.0.replace_part(IDX_PREFIX_POST, LBL_PREFIX_POST);
			}
		}
	}

	/// Clear Timestamp.
	pub fn clear_timestamp(&mut self) {
		self.0.clear_part(IDX_TIMESTAMP_PRE);
		self.0.clear_part(IDX_TIMESTAMP);
		self.0.clear_part(IDX_TIMESTAMP_POST);
	}

	/// Timestamp.
	pub fn set_timestamp(&mut self) {
		use chrono::{
			Datelike,
			Local,
			Timelike,
		};

		// If there wasn't already a timestamp, we need to set the defaults.
		if self.0.part_is_empty(IDX_TIMESTAMP_PRE) {
			self.0.replace_part(IDX_TIMESTAMP_PRE, LBL_TIMESTAMP_PRE);
			self.0.replace_part(IDX_TIMESTAMP_POST, LBL_TIMESTAMP_POST);
			//                                    2   0   0   0   -   0   0   -   0   0   •   0   0   :   0   0   :   0   0
			self.0.replace_part(IDX_TIMESTAMP, &[50, 48, 48, 48, 45, 48, 48, 45, 48, 48, 32, 48, 48, 58, 48, 48, 58, 48, 48]);
		}

		// And of course, the timestamp.
		let buf = &mut self.0[IDX_TIMESTAMP];
		let now = Local::now();

		// Y2.1K!!! We're ignoring the century because, duh, but we'll need to
		// implement something more robust over the next 80 years. Haha.
		buf[2..4].copy_from_slice(time_format_dd((now.year() as u32).saturating_sub(2000)));
		buf[5..7].copy_from_slice(time_format_dd(now.month()));
		buf[8..10].copy_from_slice(time_format_dd(now.day()));
		buf[11..13].copy_from_slice(time_format_dd(now.hour()));
		buf[14..16].copy_from_slice(time_format_dd(now.minute()));
		buf[17..19].copy_from_slice(time_format_dd(now.second()));
	}



	// ------------------------------------------------------------------------
	// Conversion
	// ------------------------------------------------------------------------

	#[inline]
	#[must_use]
	/// As Bytes
	pub fn as_bytes(&self) -> &[u8] {
		&*self.0
	}

	#[inline]
	#[must_use]
	/// As Str
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&*self.0) }
	}



	// ------------------------------------------------------------------------
	// Convenience Methods
	// ------------------------------------------------------------------------

	new_prefix!(
		confirm,
		//\e   [   1   ;   3   8   ;   5   ;   2   0   8    m
		&[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 56, 109],
		// C    o    n    f    i    r    m
		&[67, 111, 110, 102, 105, 114, 109]
	);
	new_prefix!(
		crunched,
		//\e   [   1   ;   9   2    m
		&[27, 91, 49, 59, 57, 50, 109],
		// C    r    u    n   c    h    e    d
		&[67, 114, 117, 110, 99, 104, 101, 100]
	);
	new_prefix!(
		debug,
		//\e   [   1   ;   9   6    m
		&[27, 91, 49, 59, 57, 54, 109],
		// D    e   b    u    g
		&[68, 101, 98, 117, 103]
	);
	new_prefix!(
		done,
		//\e   [   1   ;   9   2    m
		&[27, 91, 49, 59, 57, 50, 109],
		// D    o    n    e
		&[68, 111, 110, 101]
	);
	new_prefix!(
		error,
		//\e   [   1   ;   9   1    m
		&[27, 91, 49, 59, 57, 49, 109],
		// E    r    r    o    r
		&[69, 114, 114, 111, 114]
	);
	new_prefix!(
		info,
		//\e   [   1   ;   9   5    m
		&[27, 91, 49, 59, 57, 53, 109],
		// I    n    f    o
		&[73, 110, 102, 111]
	);
	new_prefix!(
		notice,
		//\e   [   1   ;   9   5    m
		&[27, 91, 49, 59, 57, 53, 109],
		// N    o    t    i   c    e
		&[78, 111, 116, 105, 99, 101]
	);
	new_prefix!(
		success,
		//\e   [   1   ;   9   2    m
		&[27, 91, 49, 59, 57, 50, 109],
		// S    u   c   c    e    s    s
		&[83, 117, 99, 99, 101, 115, 115]
	);
	new_prefix!(
		task,
		//\e   [   1   ;   3   8   ;   5   ;   1   9   9    m
		&[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 57, 109],
		// T   a    s    k
		&[84, 97, 115, 107]
	);
	new_prefix!(
		warning,
		//\e   [   1   ;   9   3    m
		&[27, 91, 49, 59, 57, 51, 109],
		// W   a    r    n    i    n    g
		&[87, 97, 114, 110, 105, 110, 103]
	);
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_impl() {
		use std::collections::hash_map::DefaultHasher;

		let empty = Msg::default();
		let one = Msg::error("Oh no!");
		let one2 = Msg::error("Oh no!");
		let two = Msg::warning("Oh no!");
		let plain = Msg::from("Dogs are better than cats.");
		let plain2 = Msg::from("Cats are better than dogs.");

		// These should match.
		assert_eq!(empty, Msg::default());
		assert_eq!(one, one2);
		assert_eq!(plain, "\x1b[1mDogs are better than cats.\x1b[0m");
		assert_eq!(plain, &b"\x1b[1mDogs are better than cats.\x1b[0m"[..]);

		// These shouldn't.
		assert!(empty != one);
		assert!(plain != "Dogs are better than cats.");
		assert!(plain != plain2);

		// Check matching hashes.
		let mut h1 = DefaultHasher::new();
		let mut h2 = DefaultHasher::new();
		one.hash(&mut h1);
		one2.hash(&mut h2);
		assert_eq!(h1.finish(), h2.finish());

		h1 = DefaultHasher::new();
		let mut h3 = DefaultHasher::new();
		two.hash(&mut h3);
		one.hash(&mut h1);
		assert!(h1.finish() != h3.finish());

		// Let's also check ordering.
		assert_eq!(one.cmp(&one2), Ordering::Equal);
		assert_eq!(plain.cmp(&plain2), Ordering::Greater);
		assert_eq!(plain2.cmp(&plain), Ordering::Less);
	}
}
