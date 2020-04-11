/*!
# FYI Core: Msg
*/

use crate::{
	MSG_TIMESTAMP,
	PRINT_NEWLINE,
	PRINT_STDERR,
	traits::str::FYIStringFormat,
	util::{
		cli,
		numbers,
		strings,
		time,
	},
};
use std::{
	borrow::Cow,
	fmt,
	time::Instant,
};

#[derive(Debug, Clone, PartialEq)]
/// Generic message.
pub enum Prefix<'mp> {
	/// Custom.
	Custom(Cow<'mp, str>, u8),
	/// Debug.
	Debug,
	/// Error.
	Error,
	/// Info.
	Info,
	/// Notice.
	Notice,
	/// Success.
	Success,
	/// Warning.
	Warning,
	/// None.
	None,
}

impl Default for Prefix<'_> {
	/// Default.
	fn default() -> Self {
		Prefix::None
	}
}

impl fmt::Display for Prefix<'_> {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.prefix())
	}
}

impl<'mp> Prefix<'mp> {
	/// Custom Prefix.
	pub fn new<S> (msg: S, color: u8) -> Self
	where S: Into<Cow<'mp, str>> {
		Self::Custom(msg.into(), color)
	}

	/// Happy or sad?
	pub fn happy(&self) -> bool {
		match *self {
			Self::Error | Self::Warning => false,
			_ => true,
		}
	}

	/// Print the prefix!
	pub fn prefix(&self) -> Cow<'_, str> {
		match *self {
			Self::Custom(ref p, c) => match p.is_empty() {
				true => Cow::Borrowed(""),
				false => Cow::Owned([
					"\u{1B}[1;38;5;",
					c.to_string().as_str(),
					"m",
					p,
					":\u{1B}[0m ",
				].concat()),
			},
			Self::Debug => Cow::Borrowed("\u{1B}[96;1mDebug:\u{1B}[0m "),
			Self::Error => Cow::Borrowed("\u{1B}[91;1mError:\u{1B}[0m "),
			Self::Info => Cow::Borrowed("\u{1B}[96;1mInfo:\u{1B}[0m "),
			Self::Notice => Cow::Borrowed("\u{1B}[95;1mNotice:\u{1B}[0m "),
			Self::Success => Cow::Borrowed("\u{1B}[92;1mSuccess:\u{1B}[0m "),
			Self::Warning => Cow::Borrowed("\u{1B}[93;1mWarning:\u{1B}[0m "),
			_ => Cow::Borrowed(""),
		}
	}
}



#[derive(Debug, Default, Clone)]
/// Message.
pub struct Msg<'m> {
	indent: u8,
	prefix: Prefix<'m>,
	msg: Cow<'m, str>,
	flags: u8,
}

impl fmt::Display for Msg<'_> {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.msg())
	}
}

impl<'m> Msg<'m> {
	/// New.
	pub fn new<S> (msg: S) -> Self
	where S: Into<Cow<'m, str>> {
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
	pub fn with_prefix(mut self, prefix: Prefix<'m>) -> Self {
		self.prefix = prefix;
		self
	}



	// -------------------------------------------------------------
	// Getters
	// -------------------------------------------------------------

	/// Msg.
	pub fn msg(&self) -> Cow<'_, str> {
		// The regular message will be needed either way.
		let msg: Cow<'_, str> = Cow::Owned([
			&strings::whitespace(self.indent * 4),
			&self.prefix.prefix(),
			"\u{1B}[1m",
			&self.msg,
			"\u{1B}[0m"
		].concat());

		match 0 != (self.flags & MSG_TIMESTAMP) {
			// Include a timestamp.
			true => {
				// A formatted timestamp.
				let timestamp: Cow<'_, str> = Cow::Owned([
					"\u{1B}[2m[\u{1B}[34;2m",
					&self.timestamp(),
					"\u{1B}[0m\u{1B}[1m]\u{1B}[0m",
				].concat());

				let msg_len = &msg.fyi_width();
				let timestamp_len = &timestamp.fyi_width();
				let width = cli::term_width();

				Cow::Owned(match msg_len + timestamp_len + 1 <= width {
					true => [
						&*msg,
						&strings::whitespace(width - msg_len - timestamp_len),
						&timestamp,
					].concat(),
					false => [
						&timestamp,
						"\n",
						&msg,
					].concat(),
				})
			},
			// Just the message.
			false => msg,
		}
	}

	/// Prefix.
	pub fn prefix(&self) -> Prefix {
		self.prefix.clone()
	}

	/// Timestamp.
	pub fn timestamp(&self) -> Cow<'_, str> {
		use chrono::prelude::*;
		Cow::Owned(Local::now().format("%F %T").to_string())
	}



	// -------------------------------------------------------------
	// Misc Operations
	// -------------------------------------------------------------

	#[cfg(feature = "interactive")]
	/// Prompt instead.
	pub fn prompt(&self) -> bool {
		casual::confirm(&[
			"\u{1B}[93;1mConfirm:\u{1B}[0m \u{1B}[1m",
			&self.msg,
			"\u{1B}[0m",
		].concat())
	}

	/// Print.
	pub fn print(&self) {
		let mut flags: u8 = self.flags | PRINT_NEWLINE;
		if false == self.prefix.happy() {
			flags |= PRINT_STDERR;
		}

		cli::print(self.to_string(), flags);
	}

	// -------------------------------------------------------------
	// Message Templates
	// -------------------------------------------------------------

	/// Template: Crunched In X.
	pub fn crunched_in(num: u64, time: Instant, du: Option<(u64, u64)>) -> Self {
		let elapsed = time::human_elapsed(time.elapsed().as_secs() as usize, 0);

		Msg::new(Cow::Owned(match du {
			Some((before, after)) => [
				&strings::inflect(num as usize, "file", "files"),
				" in ",
				&elapsed,
				&match numbers::saved(before, after) {
					0 => ", but no dice".to_string(),
					x => format!(
						", saving {} bytes ({:3.*}%)",
						numbers::human_int(x),
						2,
						(1.0 - (after as f64 / before as f64)) * 100.0
					),
				},
				".",
			].concat(),
			None => [
				&strings::inflect(num as usize, "file", "files"),
				" in ",
				&elapsed,
				".",
			].concat(),
		}))
			.with_prefix(Prefix::Custom(Cow::Borrowed("Crunched"), 2))
	}

	/// Template: Finished In X.
	pub fn finished_in(time: Instant) -> Self {
		Msg::new(Cow::Owned([
			"Finished in ",
			&time::human_elapsed(time.elapsed().as_secs() as usize, 0),
			".",
		].concat()))
			.with_prefix(Prefix::Custom(Cow::Borrowed("Crunched"), 2))
	}
}
