/*!
# FYI Message

The `Msg` struct is an efficient way to construct a "Prefix: Hello World"-type
status message.
*/

use crate::{
	print,
	Flags,
	traits::{
		AnsiCodeBold,
		Printable,
	},
};
use std::{
	borrow::Borrow,
	fmt,
	ops::Deref,
};



#[derive(Debug, Default, Clone, PartialEq, Hash)]
/// The Message!
pub struct Msg(Vec<u8>);

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

impl Msg {
	/// Bold ANSI.
	const BOLD: &'static [u8; 4] = b"\x1B[1m";
	/// Reset styles.
	const RESET_ALL: &'static [u8; 4] = b"\x1B[0m";
	/// Prefix closer.
	const PREFIX_CLOSER: &'static [u8; 7] = b":\x1B[39m ";

	/// New message.
	pub fn new<T1, T2> (prefix: T1, prefix_color: u8, msg: T2) -> Self
	where
	T1: Borrow<str>,
	T2: Borrow<str> {
		let prefix: &str = prefix.borrow();
		let msg: &str = msg.borrow();

		if prefix.is_empty() {
			if msg.is_empty() {
				Self::default()
			}
			else {
				Msg([
					Self::BOLD,
					msg.as_bytes(),
					Self::RESET_ALL,
				].concat())
			}
		}
		else {
			Msg([
				<[u8]>::ansi_code_bold(prefix_color),
				prefix.as_bytes(),
				Self::PREFIX_CLOSER,
				msg.as_bytes(),
				Self::RESET_ALL,
			].concat())
		}
	}

	/// New message (without prefix).
	pub fn plain<T> (msg: T) -> Self
	where T: Borrow<str> {
		let msg: &str = msg.borrow();
		if msg.is_empty() {
			Self::default()
		}
		else {
			Msg([
				Self::BOLD,
				msg.as_bytes(),
				Self::RESET_ALL,
			].concat())
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

	#[must_use]
	#[inline]
	/// Is Empty?
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	#[must_use]
	/// Length.
	pub fn len(&self) -> usize {
		self.0.len()
	}
}

impl Printable for Msg {
	/// Print.
	fn print(&self, indent: u8, flags: Flags) {
		print::print(self, indent, flags);
	}

	#[cfg(feature = "interactive")]
	/// Prompt.
	fn prompt(&self, indent: u8, flags: Flags) -> bool {
		print::prompt(self, indent, flags)
	}
}
