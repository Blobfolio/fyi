/*!
# FYI Msg Traits: Printable

This trait adds a single `print()` method to structs that ties into
`fyi_msg::print::print()`.
*/

use crate::{
	Flags,
	print,
	prompt,
};



/// Printable
pub trait Printable {
	/// Print.
	fn print(&self, indent: u8, flags: Flags);

	#[cfg(feature = "interactive")]
	#[must_use]
	/// Prompt.
	fn prompt(&self, indent: u8, flags: Flags) -> bool;
}

impl Printable for [u8] {
	/// Print.
	fn print(&self, indent: u8, flags: Flags) {
		print(self, indent, flags);
	}

	#[cfg(feature = "interactive")]
	#[must_use]
	/// Prompt.
	fn prompt(&self, indent: u8, flags: Flags) -> bool {
		prompt(self, indent, flags)
	}
}
