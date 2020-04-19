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
		let msg = msg.into();
		match msg.is_empty() {
			true => Self::None,
			false => Self::Custom(msg.into(), color),
		}
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
					"\x1B[1;38;5;",
					c.to_string().as_str(),
					"m",
					p,
					":\x1B[0m ",
				].concat()),
			},
			Self::Debug => Cow::Borrowed("\x1B[96;1mDebug:\x1B[0m "),
			Self::Error => Cow::Borrowed("\x1B[91;1mError:\x1B[0m "),
			Self::Info => Cow::Borrowed("\x1B[96;1mInfo:\x1B[0m "),
			Self::Notice => Cow::Borrowed("\x1B[95;1mNotice:\x1B[0m "),
			Self::Success => Cow::Borrowed("\x1B[92;1mSuccess:\x1B[0m "),
			Self::Warning => Cow::Borrowed("\x1B[93;1mWarning:\x1B[0m "),
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
		match self.flags & MSG_TIMESTAMP {
			0 => self.msg_straight(),
			_ => self.msg_timestamped(),
		}
	}

	/// Straight message.
	fn msg_straight(&self) -> Cow<'_, str> {
		Cow::Owned({
			let prefix = self.prefix.prefix();
			let indent_len: usize = self.indent as usize * 4;

			let mut p: String = String::with_capacity(
				indent_len +
				prefix.len() +
				self.msg.len() +
				8
			);
			if indent_len > 0 {
				p.push_str(&strings::whitespace(indent_len));
			}
			p.push_str(&prefix);
			p.push_str("\x1B[1m");
			p.push_str(&self.msg);
			p.push_str("\x1B[0m");

			p
		})
	}

	/// Message w/ Timestamp.
	fn msg_timestamped(&self) -> Cow<'_, str> {
		let prefix = self.prefix.prefix();
		let ts = self.timestamp();

		let prefix_len: usize = prefix.len();
		let indent_len: usize = self.indent as usize * 4;
		let msg_len: usize = indent_len +
			prefix_len +
			self.msg.len() +
			8;
		let msg_width: usize = indent_len +
			prefix.fyi_width() +
			self.msg.fyi_width();

		let ts_width: usize = ts.len() + 2;
		let ts_len: usize = ts_width + 23;

		let width: usize = cli::term_width();
		let overflow: bool = msg_width + ts_width + 1 > width;
		let total_len = match overflow {
			true => ts_len + 1 + msg_len + indent_len,
			false => msg_len + (width - msg_width),
		};

		// A formatted timestamp.
		Cow::Owned({
			let mut p: String = String::with_capacity(total_len);

			// Message first?
			if ! overflow {
				if indent_len > 0 {
					p.push_str(&strings::whitespace(indent_len));
				}
				if prefix_len > 0 {
					p.push_str(&prefix);
				}
				p.push_str("\x1B[1m");
				p.push_str(&self.msg);
				p.push_str("\x1B[0m");
				p.push_str(&strings::whitespace(width - msg_width - ts_width));
			}
			else if indent_len > 0 {
				p.push_str(&strings::whitespace(indent_len));
			}

			p.push_str("\x1B[2m[\x1B[34;2m");
			p.push_str(&ts);
			p.push_str("\x1B[0m\x1B[1m]\x1B[0m");

			// Message last?
			if overflow {
				p.push('\n');
				if indent_len > 0 {
					p.push_str(&strings::whitespace(self.indent * 4));
				}
				if prefix_len > 0 {
					p.push_str(&prefix);
				}
				p.push_str("\x1B[1m");
				p.push_str(&self.msg);
				p.push_str("\x1B[0m");
			}

			p
		})
	}

	/// Prefix.
	pub fn prefix(&self) -> Prefix {
		self.prefix.clone()
	}

	/// Timestamp.
	pub fn timestamp(&self) -> Cow<'_, str> {
		Cow::Owned(chrono::Local::now().format("%F %T").to_string())
	}



	// -------------------------------------------------------------
	// Misc Operations
	// -------------------------------------------------------------

	#[cfg(feature = "interactive")]
	/// Prompt instead.
	pub fn prompt(&self) -> bool {
		casual::confirm(&[
			"\x1B[93;1mConfirm:\x1B[0m \x1B[1m",
			&self.msg,
			"\x1B[0m",
		].concat())
	}

	/// Print.
	pub fn print(&self) {
		let mut flags: u8 = self.flags | PRINT_NEWLINE;
		if false == self.prefix.happy() {
			flags |= PRINT_STDERR;
		}

		cli::print(self.msg(), flags);
	}

	// -------------------------------------------------------------
	// Message Templates
	// -------------------------------------------------------------

	/// Template: Crunched In X.
	pub fn crunched_in(num: u64, time: Instant, du: Option<(u64, u64)>) -> Self {
		let elapsed = time::elapsed(time.elapsed().as_secs() as usize);

		Msg::new(Cow::Owned(match du {
			Some((before, after)) => [
				strings::inflect(num as usize, "file", "files").as_ref(),
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
			&time::elapsed(time.elapsed().as_secs() as usize),
			".",
		].concat()))
			.with_prefix(Prefix::Custom(Cow::Borrowed("Crunched"), 2))
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn prefix() {
		assert_eq!(Prefix::Debug.prefix().fyi_strip_ansi(), "Debug: ");
		assert_eq!(Prefix::Error.prefix().fyi_strip_ansi(), "Error: ");
		assert_eq!(Prefix::Info.prefix().fyi_strip_ansi(), "Info: ");
		assert_eq!(Prefix::None.prefix().fyi_strip_ansi(), "");
		assert_eq!(Prefix::Notice.prefix().fyi_strip_ansi(), "Notice: ");
		assert_eq!(Prefix::Success.prefix().fyi_strip_ansi(), "Success: ");
		assert_eq!(Prefix::Warning.prefix().fyi_strip_ansi(), "Warning: ");
	}

	#[test]
	fn prefix_happy() {
		assert!(Prefix::Debug.happy());
		assert!(Prefix::Info.happy());
		assert!(Prefix::None.happy());
		assert!(Prefix::Notice.happy());
		assert!(Prefix::Success.happy());

		// These two are sad.
		assert!(! Prefix::Error.happy());
		assert!(! Prefix::Warning.happy());
	}

	#[test]
	fn prefix_new() {
		assert_eq!(Prefix::new("", 199), Prefix::None);
		assert_eq!(Prefix::new("Hello", 199).prefix(), "\x1B[1;38;5;199mHello:\x1B[0m ");
		assert_eq!(Prefix::new("Hello", 2).prefix(), "\x1B[1;38;5;2mHello:\x1B[0m ");
	}

	#[test]
	fn msg_new() {
		// Just a message.
		let msg: Msg = Msg::new("Hello World");
		assert_eq!(msg.msg(), "\x1B[1mHello World\x1B[0m");
		assert_eq!(msg.prefix(), Prefix::None);

		// With prefix.
		let msg: Msg = Msg::new("Hello World")
			.with_prefix(Prefix::Success);
		assert_eq!(msg.msg(), "\x1B[92;1mSuccess:\x1B[0m \x1B[1mHello World\x1B[0m");
		assert_eq!(msg.prefix(), Prefix::Success);

		// Indentation. We've tested color enough now; let's strip ANSI
		// to make this more readable.
		let msg: Msg = Msg::new("Hello World")
			.with_indent(1);
		assert_eq!(msg.msg().fyi_strip_ansi(), "    Hello World");
		let msg: Msg = Msg::new("Hello World")
			.with_indent(2);
		assert_eq!(msg.msg().fyi_strip_ansi(), "        Hello World");
	}
}
