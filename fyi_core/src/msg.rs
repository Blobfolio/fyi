/*!
# FYI Core: Msg
*/

use ansi_term::{Colour, Style};
use chrono::prelude::*;
use crate::misc::{cli, strings};
use crate::prefix::Prefix;
use dialoguer::Confirmation;



#[derive(Debug, Clone, Copy)]
/// Message.
pub struct Msg<'a> {
	indent: u8,
	prefix: Prefix<'a>,
	msg: String,
	flags: u8,
}

impl std::fmt::Display for Msg<'_> {
	/// Display.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// The message.
		let mut out: String = format!(
			"{}{}{}",
			strings::indentation(self.indent),
			self.prefix.to_string(),
			Style::new().bold().paint(self.msg.clone())
		);

		// A timestamp?
		let timestamp = self.timestamp();
		if false == timestamp.is_empty() {
			out = append_timestamp(out, timestamp);
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
			msg: "".to_string(),
			flags: 0,
		}
	}
}

impl<'a> Msg<'a> {
	/// New.
	pub fn new<S> (msg: S) -> Self
	where S: Into<String> {
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
		if 0 != (super::MSG_TIMESTAMP & self.flags) {
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
				Style::new().bold().paint(self.msg.clone())
			))
			.interact()
			.unwrap_or(false)
	}

	/// Print.
	pub fn print(&self) {
		let mut flags: u8 = self.flags | crate::PRINT_NEWLINE;
		if false == self.prefix.happy() {
			flags |= crate::PRINT_STDERR;
		}

		cli::print(&self.to_string(), flags);
	}
}

/// Append Timestamp.
fn append_timestamp<S> (msg: S, timestamp: S) -> String
where S: Into<String> {
	let msg = msg.into();
	let msg_len = strings::stripped_len(&msg);
	let timestamp = timestamp.into();
	let timestamp_len = strings::stripped_len(&timestamp);
	let mut max_len = cli::term_width();
	if 80 < max_len {
		max_len = 80;
	}

	// We can do it inline.
	if msg_len + timestamp_len + 1 <= max_len {
		format!(
			"{}{}{}",
			&msg,
			strings::whitespace(max_len - msg_len - timestamp_len),
			&timestamp
		)
	}
	else {
		format!(
			"{}\n{}",
			&timestamp,
			&msg
		)
	}
}
