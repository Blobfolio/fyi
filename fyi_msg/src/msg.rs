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



static CLOSE: &[u8] = &[27, 91, 48, 109];
static EMPTY: &[u8] = &[];
static OPEN_BOLD: &[u8] = &[27, 91, 49, 109];
static PREFIX_JOINER: &[u8] = &[27, 91, 51, 57, 109, 32];



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
	($method:ident, $label:literal) => {
		#[inline]
		/// New $label message.
		pub fn $method<T: Borrow<str>> (msg: T) -> Self {
			unsafe { Self::new_unchecked($label, msg.borrow().as_bytes()) }
		}
	};
}

impl Msg {
	/// The part index for indentation.
	const IDX_INDENT: usize = 0;

	/// The part index for timestamps.
	const IDX_TIMESTAMP: usize = 1;

	/// The part index for the actual message.
	const IDX_MSG: usize = 2;

	/// New message.
	pub fn new<T1, T2> (prefix: T1, prefix_color: u8, msg: T2) -> Self
	where
	T1: Borrow<str>,
	T2: Borrow<str> {
		let prefix = prefix.borrow();
		let msg = msg.borrow();

		if prefix.is_empty() {
			Self::plain(msg)
		}
		else if msg.is_empty() {
			unsafe {
				Msg(PrintBuf::from_at_with_parts_unchecked(
					&ansi_code_bold(prefix_color).iter()
						.chain(prefix.as_bytes())
						.chain(CLOSE)
						.cloned()
						.collect::<Vec<u8>>(),
					Msg::IDX_MSG,
					3
				))
			}
		}
		else {
			unsafe {
				Msg(PrintBuf::from_at_with_parts_unchecked(
					&ansi_code_bold(prefix_color).iter()
						.chain(prefix.as_bytes())
						.chain(PREFIX_JOINER)
						.chain(msg.as_bytes())
						.chain(CLOSE)
						.cloned()
						.collect::<Vec<u8>>(),
					Msg::IDX_MSG,
					3
				))
			}
		}
	}

	#[must_use]
	/// New Message w/ Default Prefix.
	///
	/// This method is used by most all the convenience methods.
	///
	/// # Safety
	///
	/// This method assumes the prefix is valid UTF-8 — and correctly colored —
	/// and the message is valid UTF-8.
	pub unsafe fn new_unchecked(prefix: &[u8], msg: &[u8]) -> Self {
		Msg(PrintBuf::from_at_with_parts_unchecked(
			&prefix.iter()
				.chain(msg)
				.chain(CLOSE)
				.cloned()
				.collect::<Vec<u8>>(),
			Msg::IDX_MSG,
			3
		))
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
				Msg(PrintBuf::from_at_with_parts_unchecked(
					&OPEN_BOLD.iter()
						.chain(msg.as_bytes())
						.chain(CLOSE)
						.cloned()
						.collect::<Vec<u8>>(),
					Msg::IDX_MSG,
					3
				))
			}
		}
	}

	#[must_use]
	/// With Indentation.
	pub fn with_indent(mut self) -> Self {
		self.indent();
		self
	}

	#[must_use]
	/// With Printer.
	pub fn with_printer(mut self, printer: PrinterKind) -> Self {
		self.set_printer(printer);
		self
	}

	#[must_use]
	/// With Timestamp.
	pub fn with_timestamp(mut self) -> Self {
		self.timestamp();
		self
	}

	new_msg_method!(confirm, b"\x1B[1;38;5;208mConfirm:\x1B[0;1m ");   // Orange.
	new_msg_method!(crunched, b"\x1B[1;92mCrunched:\x1B[0;1m ");       // Light Green.
	new_msg_method!(debug, b"\x1B[1;96mDebug:\x1B[0;1m ");             // Light Cyan.
	new_msg_method!(done, b"\x1B[1;92mDone:\x1B[0;1m ");               // Light Green.
	new_msg_method!(eg, b"\x1B[1;96me.g.\x1B[0;1m ");                  // Light Cyan.
	new_msg_method!(error, b"\x1B[1;91mError:\x1B[0;1m ");             // Light Red.
	new_msg_method!(ie, b"\x1B[1;96mi.e.\x1B[0;1m ");                  // Light Cyan.
	new_msg_method!(info, b"\x1B[1;95mInfo:\x1B[0;1m ");               // Light Magenta.
	new_msg_method!(notice, b"\x1B[1;95mNotice:\x1B[0;1m ");           // Light Magenta.
	new_msg_method!(question, b"\x1B[1;38;5;208mQuestion:\x1B[0;1m "); // Orange.
	new_msg_method!(success, b"\x1B[1;92mSuccess:\x1B[0;1m ");         // Light Green.
	new_msg_method!(task, b"\x1B[1;38;5;199mTask:\x1B[0;1m ");         // Hot Pink.
	new_msg_method!(warning, b"\x1B[1;93mWarning:\x1B[0;1m ");         // Light Yellow.

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
		unsafe { self.0.replace_part_unchecked(Self::IDX_INDENT, EMPTY); }
	}

	/// Remove Timestamp.
	pub fn remove_timestamp(&mut self) {
		unsafe { self.0.replace_part_unchecked(Self::IDX_TIMESTAMP, EMPTY); }
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
				&Timestamp::new().deref().iter()
					.chain(&[32, 32])
					.cloned()
					.collect::<Vec<u8>>(),
			);
		}
	}
}
