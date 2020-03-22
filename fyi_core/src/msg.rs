/*!
# FYI Core: Msg
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

use ansi_term::{Colour, Style};
use chrono::prelude::*;
use crate::prefix::Prefix;
use dialoguer::Confirmation;



#[derive(Debug, Clone, Copy)]
/// Message.
pub struct Msg<'a> {
	indent: u8,
	prefix: Prefix<'a>,
	msg: &'a str,
	flags: u8,
}

impl std::fmt::Display for Msg<'_> {
	/// Display.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// The message.
		let mut out: String = format!(
			"{}{}{}",
			indentation(self.indent),
			self.prefix.to_string(),
			Style::new().bold().paint(self.msg)
		);

		// A timestamp?
		let timestamp = self.timestamp();
		if false == timestamp.is_empty() {
			out = format!(
				"{}{}\n{}",
				indentation(self.indent),
				timestamp,
				out
			);
		}

		// Strip color?
		if 0 != (super::NO_COLOR & self.flags) {
			out = strip_styles(out);
		}

		f.write_str(&out)
	}
}

impl Default for Msg<'_> {
	/// Default.
	fn default() -> Self {
		Msg {
			indent: 0,
			prefix: Prefix::None,
			msg: "",
			flags: 0,
		}
	}
}

impl<'a> Msg<'a> {
	/// New.
	pub fn new<S> (msg: S) -> Self
	where S: Into<&'a str> {
		Msg {
			msg: msg.into(),
			..Msg::default()
		}
	}

	/// Set Flags.
	pub fn with_flags(mut self, flags: u8) -> Self {
		self.flags = flags;
		self
	}

	/// Set Indentation.
	pub fn with_indent(mut self, indent: u8) -> Self {
		self.indent = indent;
		self
	}

	/// Set Prefix.
	pub fn with_prefix(mut self, prefix: Prefix<'a>) -> Self {
		self.prefix = prefix;
		self
	}

	/// Formatted Timestamp.
	fn timestamp(&self) -> String {
		if 0 != (super::TIMESTAMP & self.flags) {
			format!(
				"[{}]",
				Style::new().dimmed().paint(format!(
					"{}",
					Local::now().format("%F %T"),
				))
			)
		}
		else {
			"".to_string()
		}
	}

	/// Prompt instead.
	pub fn prompt(&self) -> bool {
		Confirmation::new()
			.with_text(&format!(
				"{} {}",
				Colour::Yellow.bold().paint("Confirm:"),
				Style::new().bold().paint(self.msg)
			))
			.interact()
			.unwrap_or(false)
	}

	/// Print.
	pub fn print(&self) {
		match self.prefix {
			Prefix::Error | Prefix::Warning => eprintln!("{}", self.to_string()),
			_ => println!("{}", self.to_string()),
		}
	}
}

/// Indent.
fn indentation(indent: u8) -> String {
	if 0 < indent {
		String::from_utf8(vec![b' '; (indent * 4) as usize]).unwrap_or("".to_string())
	}
	else {
		"".to_string()
	}
}

/// Strip Styles
fn strip_styles<S> (text: S) -> String
where S: Into<String> {
	let text = strip_ansi_escapes::strip(text.into())
		.unwrap_or(Vec::new());
	std::str::from_utf8(&text)
		.unwrap_or("")
		.to_string()
}
