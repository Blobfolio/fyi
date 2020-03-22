/*!
# FYI Core: Msg
*/

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
			out = append_timestamp(out, timestamp);
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

/// Append Timestamp.
fn append_timestamp<S> (msg: S, timestamp: S) -> String
where S: Into<String> {
	let msg = msg.into();
	let msg_len = stripped_len(&msg);
	let timestamp = timestamp.into();
	let timestamp_len = stripped_len(&timestamp);
	let mut max_len = term_width();
	if 80 < max_len {
		max_len = 80;
	}

	// We can do it inline.
	if msg_len + timestamp_len + 1 <= max_len {
		format!(
			"{}{}{}",
			&msg,
			whitespace(max_len - msg_len - timestamp_len),
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

/// Indent.
fn indentation(indent: u8) -> String {
	whitespace((indent * 4) as usize)
}

/// Stripped Length.
fn stripped_len<S> (text: S) -> usize
where S: Into<String> {
	strip_styles(text).len()
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

/// Obtain the terminal cli width.
fn term_width() -> usize {
	match term_size::dimensions() {
		Some((w, _)) => w,
		_ => 0,
	}
}

/// Make whitespace.
fn whitespace(count: usize) -> String {
	if 0 < count {
		String::from_utf8(vec![b' '; count]).unwrap_or("".to_string())
	}
	else {
		"".to_string()
	}
}
