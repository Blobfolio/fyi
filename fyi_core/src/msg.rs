/*!
# FYI Core: Msg
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

use ansi_term::Style;
use chrono::prelude::*;
use crate::prefix::Prefix;



#[derive(Debug, Clone)]
/// Message.
pub struct Msg {
	indent: u8,
	prefix: Option<Prefix>,
	msg: String,
	flags: u8,
}

impl std::fmt::Display for Msg {
	/// Display.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// The message.
		let mut out: String = format!(
			"{}{}{}",
			indentation(self.indent),
			match self.prefix {
				Some(ref p) => p.to_string(),
				None => "".to_string(),
			},
			Style::new().bold().paint(&self.msg)
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

impl Default for Msg {
	/// Default.
	fn default() -> Self {
		Msg {
			indent: 0,
			prefix: None,
			msg: "".to_string(),
			flags: 0,
		}
	}
}

impl Msg {
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
	pub fn with_prefix(mut self, prefix: Option<Prefix>) -> Self {
		if let Some(x) = prefix {
			self.prefix = match x.to_string().is_empty() {
				true => None,
				false => Some(x),
			};
		}
		else {
			self.prefix = None;
		}

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

	/// Print.
	pub fn print(&self) {
		match self.prefix {
			Some(Prefix::Error) | Some(Prefix::Warning) => eprintln!("{}", self.to_string()),
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
