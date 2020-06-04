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
	fmt,
	ops::Deref,
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
const LBL_MSG_PRE: &[u8] = &[27, 91, 49, 109];
const LBL_PREFIX_POST: &[u8] = &[58, 27, 91, 48, 109, 32];
const LBL_RESET: &[u8] = &[27, 91, 48, 109];
const LBL_TIMESTAMP_POST: &[u8] = &[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32];
const LBL_TIMESTAMP_PRE: &[u8] = &[27, 91, 50, 109, 91, 27, 91, 48, 59, 51, 52, 109];



#[derive(Debug, Clone, Hash, PartialEq)]
/// The Message!
pub struct Msg(MsgBuf);

impl AsRef<str> for Msg {
	#[inline]
	fn as_ref(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&*self.0) }
	}
}

impl AsRef<[u8]> for Msg {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		&*self.0
	}
}

impl Borrow<str> for Msg {
	#[inline]
	fn borrow(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&*self.0) }
	}
}

impl Borrow<[u8]> for Msg {
	#[inline]
	fn borrow(&self) -> &[u8] {
		&*self.0
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
		&self.0
	}
}

impl fmt::Display for Msg {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
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
			(true, false) => Self::new_msg_unchecked(msg.as_bytes()),
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
		Self(MsgBuf::from_many(&[
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
		Self(MsgBuf::from_many(&[
			// Indentation and timestamp.
			&[], &[], &[], &[],
			prefix_pre,
			prefix,
			LBL_RESET,
			// Message.
			&[], &[], &[],
		]))
	}

	#[must_use]
	/// New Message (Unchecked)
	fn new_msg_unchecked(msg: &[u8]) -> Self {
		Self(MsgBuf::from_many(&[
			// Indentation and timestamp.
			&[], &[], &[], &[],
			// Prefix.
			&[], &[], &[],
			LBL_MSG_PRE,
			msg,
			LBL_RESET,
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
	// Convenience Methods
	// ------------------------------------------------------------------------

	new_prefix!(confirm, &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 56, 109], &[67, 111, 110, 102, 105, 114, 109]);
	new_prefix!(crunched, &[27, 91, 49, 59, 57, 50, 109], &[67, 114, 117, 110, 99, 104, 101, 100]);
	new_prefix!(debug, &[27, 91, 49, 59, 57, 54, 109], &[68, 101, 98, 117, 103]);
	new_prefix!(done, &[27, 91, 49, 59, 57, 50, 109], &[68, 111, 110, 101]);
	new_prefix!(error, &[27, 91, 49, 59, 57, 49, 109], &[69, 114, 114, 111, 114]);
	new_prefix!(info, &[27, 91, 49, 59, 57, 53, 109], &[73, 110, 102, 111]);
	new_prefix!(notice, &[27, 91, 49, 59, 57, 53, 109], &[78, 111, 116, 105, 99, 101]);
	new_prefix!(success, &[27, 91, 49, 59, 57, 50, 109], &[83, 117, 99, 99, 101, 115, 115]);
	new_prefix!(task, &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 57, 109], &[84, 97, 115, 107]);
	new_prefix!(warning, &[27, 91, 49, 59, 57, 51, 109], &[87, 97, 114, 110, 105, 110, 103]);
}
