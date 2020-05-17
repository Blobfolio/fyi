/*!
# FYI Message

The `Msg` struct is an efficient way to construct and/or print a simple,
colored "Prefix: Hello World"-type status message.

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
	PrintBuf,
	PrinterKind,
	PrintFlags,
	Timestamp,
	utility::ansi_code_bold,
};
use std::{
	borrow::Borrow,
	fmt,
	ops::Deref,
};



#[derive(Debug, Clone, Hash, PartialEq)]
/// The Message!
pub struct Msg(PrintBuf);

impl AsRef<str> for Msg {
	#[inline]
	/// As Str.
	fn as_ref(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
	}
}

impl AsRef<[u8]> for Msg {
	#[inline]
	/// As Str.
	fn as_ref(&self) -> &[u8] {
		self
	}
}

impl Borrow<str> for Msg {
	#[inline]
	fn borrow(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
	}
}

impl Borrow<[u8]> for Msg {
	#[inline]
	fn borrow(&self) -> &[u8] {
		self
	}
}

impl Default for Msg {
	/// Default.
	fn default() -> Self {
		Msg(PrintBuf::with_parts(3))
	}
}

impl Deref for Msg {
	type Target = [u8];

	/// Deref.
	///
	/// We deref to `&[u8]` as most contexts want bytes.
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl fmt::Display for Msg {
	#[inline]
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}

/// Shorthand for defining new messages.
macro_rules! new_msg_method {
	($method:ident, $label:literal, $color:literal) => {
		#[inline]
		/// New $label message.
		pub fn $method<T: Borrow<str>> (msg: T) -> Self {
			Self::new($label, $color, msg)
		}
	};
}

/// Shorthand for defining new messages.
macro_rules! new_msg_wc_method {
	($method:ident, $label:literal, $color:literal) => {
		#[inline]
		/// New $label message.
		pub fn $method<T: Borrow<str>> (msg: T) -> Self {
			Self::new_no_colon($label, $color, msg)
		}
	};
}

impl Msg {
	/// The part index for indentation.
	const IDX_INDENT: usize = 0;

	/// The part index for timestamps.
	const IDX_TIMESTAMP: usize = 1;

	/// The part index for the actual message.
	#[allow(dead_code)] const IDX_MSG: usize = 2;

	/// New message.
	pub fn new<T1, T2> (prefix: T1, prefix_color: u8, msg: T2) -> Self
	where
	T1: Borrow<str>,
	T2: Borrow<str> {
		let prefix = prefix.borrow();
		if prefix.is_empty() {
			return Self::plain(msg);
		}

		let msg = msg.borrow();
		if msg.is_empty() {
			unsafe {
				Msg(PrintBuf::from_parts_unchecked(&[
					// Indentation.
					&[],
					// Timestamp.
					&[],
					// Message.
					&[
						ansi_code_bold(prefix_color),
						prefix.as_bytes(),
						&[58, 27, 91, 48, 109],
					].concat(),
				]))
			}
		}
		else {
			unsafe {
				Msg(PrintBuf::from_parts_unchecked(&[
					// Indentation.
					&[],
					// Timestamp.
					&[],
					// Message.
					&[
						ansi_code_bold(prefix_color),
						prefix.as_bytes(),
						&[58, 27, 91, 51, 57, 109, 32],
						msg.as_bytes(),
						&[27, 91, 48, 109],
					].concat(),
				]))
			}
		}
	}

	/// New message (without prefix colon).
	pub fn new_no_colon<T1, T2> (prefix: T1, prefix_color: u8, msg: T2) -> Self
	where
	T1: Borrow<str>,
	T2: Borrow<str> {
		let prefix = prefix.borrow();
		if prefix.is_empty() {
			return Self::plain(msg);
		}

		let msg = msg.borrow();
		if msg.is_empty() {
			unsafe {
				Msg(PrintBuf::from_parts_unchecked(&[
					// Indentation.
					&[],
					// Timestamp.
					&[],
					// Message.
					&[
						ansi_code_bold(prefix_color),
						prefix.as_bytes(),
						&[27, 91, 48, 109],
					].concat(),
				]))
			}
		}
		else {
			unsafe {
				Msg(PrintBuf::from_parts_unchecked(&[
					// Indentation.
					&[],
					// Timestamp.
					&[],
					// Message.
					&[
						ansi_code_bold(prefix_color),
						prefix.as_bytes(),
						&[27, 91, 51, 57, 109, 32],
						msg.as_bytes(),
						&[27, 91, 48, 109],
					].concat(),
				]))
			}
		}
	}

	/// New message (without prefix).
	pub fn plain<T> (msg: T) -> Self
	where T: Borrow<str> {
		let msg = msg.borrow();
		if msg.is_empty() {
			Self::default()
		}
		else {
			unsafe {
				Msg(PrintBuf::from_parts_unchecked(&[
					// Indentation.
					&[],
					// Timestamp.
					&[],
					// Message.
					&[
						&[27, 91, 49, 109],
						msg.as_bytes(),
						&[27, 91, 48, 109],
					].concat(),
				]))
			}
		}
	}

	new_msg_method!(confirm, "Confirm", 208);   // Orange.
	new_msg_method!(crunched, "Crunched", 10);  // Light Green.
	new_msg_method!(debug, "Debug", 14);        // Light Cyan.
	new_msg_method!(done, "Done", 10);          // Light Green.
	new_msg_method!(error, "Error", 9);         // Light Red.
	new_msg_method!(info, "Info", 13);          // Light Magenta.
	new_msg_method!(notice, "Notice", 13);      // Light Magenta.
	new_msg_method!(question, "Question", 208); // Orange.
	new_msg_method!(success, "Success", 10);    // Light Green.
	new_msg_method!(task, "Task", 199);         // Hot Pink.
	new_msg_method!(warning, "Warning", 11);    // Light Yellow.

	new_msg_wc_method!(eg, "e.g.", 14);         // Light Cyan.
	new_msg_wc_method!(ie, "i.e.", 14);         // Light Cyan.

	#[must_use]
	#[inline]
	/// As Str.
	pub fn as_str(&self) -> &str {
		self.as_ref()
	}

	#[must_use]
	#[inline]
	/// As Bytes.
	pub fn as_bytes(&self) -> &[u8] {
		self
	}

	/// Indent.
	pub fn indent(&mut self) {
		unsafe { self.0.replace_part_unchecked(Self::IDX_INDENT, &[32, 32, 32, 32]); }
	}

	/// Print.
	pub fn print(&mut self, flags: PrintFlags) {
		self.0.print(flags);
	}

	#[cfg(feature = "interactive")]
	#[must_use]
	/// Prompt.
	///
	/// This is a simple print wrapper around `casual::confirm()`.
	///
	/// As we aren't doing the heavy lifting here, there is no support for `Flags`,
	/// however prompt messages can be indented.
	///
	/// # Safety
	///
	/// This method accepts a raw `[u8]`; when using it, make sure the data you
	/// pass is valid UTF-8.
	pub fn prompt(&self) -> bool {
		casual::confirm(unsafe { std::str::from_utf8_unchecked(&self.0) })
	}

	/// Remove Indent.
	pub fn remove_indent(&mut self) {
		unsafe { self.0.replace_part_unchecked(Self::IDX_INDENT, &[]); }
	}

	/// Remove Timestamp.
	pub fn remove_timestamp(&mut self) {
		unsafe { self.0.replace_part_unchecked(Self::IDX_TIMESTAMP, &[]); }
	}

	/// Set Printer.
	/// Set Printer.
	pub fn set_printer(&mut self, printer: PrinterKind) {
		self.0.set_printer(printer);
	}

	/// Add/Update timestamp.
	pub fn timestamp(&mut self) {
		unsafe {
			self.0.replace_part_unchecked(
				Self::IDX_TIMESTAMP,
				&[
					Timestamp::new().deref(),
					&[32_u8, 32_u8],
				].concat()
			);
		}
	}
}
