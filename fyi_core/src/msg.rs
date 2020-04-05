/*!
# FYI Core: Msg
*/

#[cfg(feature = "interactive")]
use ansi_term::Colour;

use ansi_term::Style;
use chrono::prelude::*;
use crate::misc::{
	cli,
	numbers,
	strings::{
		self,
		FYIStringFormat,
	},
	time,
};
use crate::prefix::Prefix;
use std::borrow::Cow;
use std::time::Instant;





#[derive(Debug, Default, Clone)]
/// Message.
pub struct Msg<'a> {
	indent: u8,
	prefix: Prefix<'a>,
	msg: Cow<'a, str>,
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
			Style::new().bold().paint(&*self.msg)
		);

		// A timestamp?
		let timestamp = self.timestamp();
		if false == timestamp.is_empty() {
			out = append_timestamp(out, timestamp);
		}

		f.write_str(&out)
	}
}

impl<'a> Msg<'a> {
	/// New.
	pub fn new<S> (msg: S) -> Self
	where S: Into<Cow<'a, str>> {
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

	/// Crunched In...
	pub fn msg_crunched_in(count: u64, time: Instant, size: Option<(u64, u64)>) -> Self {
		let elapsed: String = time::human_elapsed(time.elapsed().as_secs() as usize, 0);
		let (before, after) = size.unwrap_or((0, 0));
		let saved = numbers::saved(before, after);

		let msg: String = match saved {
			0 => format!(
				"{} in {}{}.",
				strings::inflect(count as usize, "file", "files"),
				elapsed,
				match size.is_some() {
					true => ", but no dice",
					false => "",
				},
			),
			_ => format!(
				"{} in {}, saving {} bytes ({:3.*}%).",
				strings::inflect(count as usize, "file", "files"),
				elapsed,
				numbers::human_int(saved),
				2,
				(1.0 - (after as f64 / before as f64)) * 100.0
			),
		};

		Msg::new(msg)
			.with_prefix(Prefix::Custom("Crunched", 2))
	}

	/// Finished In...
	pub fn msg_finished_in(time: Instant) -> Self {
		Msg::new([
			"Finished in ",
			time::human_elapsed(time.elapsed().as_secs() as usize, 0).as_str(),
			"."
		].concat())
			.with_prefix(Prefix::Success)
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
			String::new()
		}
	}

	#[cfg(feature = "interactive")]
	/// Prompt instead.
	pub fn prompt(&self) -> bool {
		dialoguer::Confirmation::new()
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

		cli::print(self.to_string(), flags);
	}
}

/// Append Timestamp.
fn append_timestamp<S> (msg: S, timestamp: S) -> String
where S: Into<String> {
	let msg = msg.into();
	let msg_len = msg.fyi_width();
	let timestamp = timestamp.into();
	let timestamp_len = timestamp.fyi_width();
	let mut max_len = cli::term_width();
	if 80 < max_len {
		max_len = 80;
	}

	// We can do it inline.
	if msg_len + timestamp_len + 1 <= max_len {
		[
			msg,
			strings::whitespace(max_len - msg_len - timestamp_len),
			timestamp,
		].concat()
	}
	else {
		[
			timestamp.as_str(),
			"\n",
			msg.as_str(),
		].concat()
	}
}
