/*!
# FYI Message

The `Msg` struct is a fairly straight-forward way of getting a simple ANSI-
formatted message printed to the terminal.

A number of basic prefixes like "Error" and "Success" are built in. Custom
prefixes with arbitrary coloring can be used via `MsgKind::new()`.

The `with_indent()` and `with_timestamp()` build patterns can prepend
indentation or a timestamp respectively.

That's it. Nice and boring!

## Example:

```no_run
use fyi_msg::Msg;
use fyi_msg::MsgKind;

// Create a message with a custom prefix and color.
MsgKind::new("Yo", 199)
    .into_msg("How are you doing today?")
    .println();

// Built-ins work the same way.
MsgKind::Error.into_msg("Well darn.").println();
MsgKind::Success.into_msg("Oh good!").println();

// Ask a yes/no question.
let res: bool = MsgKind::Confirm.into_msg("Are you OK?").prompt();
```
*/

use crate::utility;
use std::{
	fmt,
	io::{
		self,
		Write,
	},
};



#[derive(Debug, Clone, PartialEq)]
/// Message Kind.
///
/// This is the prefix, basically.
pub enum MsgKind {
	/// None.
	None,
	/// Confirm.
	Confirm,
	/// Crunched.
	Crunched,
	/// Debug.
	Debug,
	/// Done.
	Done,
	/// Error.
	Error,
	/// Info.
	Info,
	/// Notice.
	Notice,
	/// Success.
	Success,
	/// Task.
	Task,
	/// Warning.
	Warning,
	/// Custom.
	Other(Vec<u8>),
}

impl Default for MsgKind {
	fn default() -> Self { Self::None }
}

impl From<&str> for MsgKind {
	fn from(txt: &str) -> Self {
		match txt {
			"confirm" | "prompt" => Self::Confirm,
			"crunched" => Self::Crunched,
			"debug" => Self::Debug,
			"done" => Self::Done,
			"error" => Self::Error,
			"info" => Self::Info,
			"notice" => Self::Notice,
			"success" => Self::Success,
			"task" => Self::Task,
			"warning" => Self::Warning,
			_ => Self::None,
		}
	}
}

impl MsgKind {
	/// New.
	pub fn new<S> (prefix: S, color: u8) -> Self
	where S: AsRef<str> {
		let prefix = prefix.as_ref().trim();
		if prefix.is_empty() { Self::None }
		else {
			Self::Other([
				utility::ansi_code_bold(color),
				prefix.as_bytes(),
				b":\x1b[0m ",
			].concat())
		}
	}

	#[must_use]
	/// As Bytes.
	pub fn as_bytes(&self) -> &[u8] {
		match self {
			Self::None => &[],
			Self::Confirm => b"\x1b[1;38;5;208mConfirm:\x1b[0m ",
			Self::Crunched => b"\x1b[92;1mCrunched:\x1b[0m ",
			Self::Debug => b"\x1b[96;1mDebug:\x1b[0m ",
			Self::Done => b"\x1b[92;1mDone:\x1b[0m ",
			Self::Error => b"\x1b[91;1mError:\x1b[0m ",
			Self::Info => b"\x1b[95;1mInfo:\x1b[0m ",
			Self::Notice => b"\x1b[95;1mNotice:\x1b[0m ",
			Self::Success => b"\x1b[92;1mSuccess:\x1b[0m ",
			Self::Task => b"\x1b[1;38;5;199mTask:\x1b[0m ",
			Self::Warning => b"\x1b[93;1mWarning:\x1b[0m ",
			Self::Other(x) => &x[..],
		}
	}

	#[must_use]
	/// As Str.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
	}

	/// Into `Msg`.
	pub fn into_msg<S> (self, msg: S) -> Msg
	where S: AsRef<str> {
		Msg::new(msg).with_prefix(self)
	}
}



#[derive(Debug, Clone, Default, PartialEq)]
/// Message.
pub struct Msg {
	indent: u8,
	timestamp: bool,
	prefix: MsgKind,
	msg: Vec<u8>,
}

impl fmt::Display for Msg {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(unsafe { std::str::from_utf8_unchecked(&self.to_vec()) })
	}
}

impl From<Vec<u8>> for Msg {
	fn from(src: Vec<u8>) -> Self {
		Self {
			msg: src,
			..Self::default()
		}
	}
}

impl Msg {
	/// New.
	pub fn new<S> (msg: S) -> Self
	where S: AsRef<str> {
		Self {
			msg: msg.as_ref().as_bytes().to_vec(),
			..Self::default()
		}
	}

	#[allow(clippy::missing_const_for_fn)] // Doesn't work.
	#[must_use]
	/// With Prefix.
	pub fn with_prefix(mut self, prefix: MsgKind) -> Self {
		self.prefix = prefix;
		self
	}

	#[must_use]
	/// With Timestamp.
	pub const fn with_timestamp(mut self, on: bool) -> Self {
		self.timestamp = on;
		self
	}

	#[must_use]
	/// With Indent.
	pub const fn with_indent(mut self, indent: u8) -> Self {
		self.indent = indent;
		self
	}

	#[must_use]
	/// To Vec.
	pub fn to_vec(&self) -> Vec<u8> {
		[
			self.indent(),
			self.timestamp(),
			self.prefix.as_bytes(),
			self.msg.as_slice(),
		].concat()
	}

	/// Indentation bit.
	fn indent(&self) -> &[u8] {
		utility::whitespace(self.indent as usize * 4)
	}

	/// Timestamp bit.
	fn timestamp(&self) -> &[u8] {
		static mut BUF: [u8; 44] = *b"\x1b[2m[\x1b[0;34m2000-00-00 00:00:00\x1b[39;2m]\x1b[0m ";

		// Update the timestamp.
		if self.timestamp {
			use chrono::{
				Datelike,
				Local,
				Timelike,
			};

			// Chrono's formatter is slow as shit. It is faster for us to call
			// each of their time part methods individually, convert those
			// integers to bytes, and copy them into our static buffer.
			let now = Local::now();
			unsafe {
				BUF[14..16].copy_from_slice(utility::time_format_dd((now.year() as u32).saturating_sub(2000)));
				BUF[17..19].copy_from_slice(utility::time_format_dd(now.month()));
				BUF[20..22].copy_from_slice(utility::time_format_dd(now.day()));
				BUF[23..25].copy_from_slice(utility::time_format_dd(now.hour()));
				BUF[26..28].copy_from_slice(utility::time_format_dd(now.minute()));
				BUF[29..31].copy_from_slice(utility::time_format_dd(now.second()));

				&BUF
			}
		}
		else { &[] }
	}

	#[must_use]
	/// Prompt.
	pub fn prompt(&self) -> bool {
		// Clone the message and append a little [y/N] instructional bit to the
		// end.
		let mut q = self.clone();
		q.msg.extend_from_slice(b" \x1b[2m[y/\x1b[4mN\x1b[0;2m]\x1b[0m ");

		// Ask and collect input, looping until a valid response is typed.
		loop {
			q.print();

			if let Ok(s) = read_prompt() {
				match s.as_str() {
					"" | "n" | "no" => break false,
					"y" | "yes" => break true,
					_ => {},
				}
			}

			// Print an error and do it all over again.
			MsgKind::Error.into_msg("Invalid input: enter \x1b[91mN\x1b[34m or \x1b[92mY\x1b[34m.")
				.println();
		}
	}

	/// Print.
	pub fn print(&self) {
		locked_print(&self.to_vec(), false);
	}

	/// Print.
	pub fn println(&self) {
		locked_print(&self.to_vec(), true);
	}

	/// Print.
	pub fn eprint(&self) {
		locked_eprint(&self.to_vec(), false);
	}

	/// Print.
	pub fn eprintln(&self) {
		locked_eprint(&self.to_vec(), true);
	}

	/// Print.
	pub fn sink(&self) {
		locked_sink(&self.to_vec(), false);
	}

	/// Print.
	pub fn sinkln(&self) {
		locked_sink(&self.to_vec(), true);
	}
}



/// Locked Print: Stdout.
fn locked_print(buf: &[u8], line: bool) {
	let writer = std::io::stdout();
	let mut handle = writer.lock();
	handle.write_all(buf).unwrap();

	if line {
		handle.write_all(&[10]).unwrap();
	}

	handle.flush().unwrap();
}

/// Locked Print: Stderr.
fn locked_eprint(buf: &[u8], line: bool) {
	let writer = std::io::stderr();
	let mut handle = writer.lock();
	handle.write_all(buf).unwrap();

	if line {
		handle.write_all(&[10]).unwrap();
	}

	handle.flush().unwrap();
}

/// Locked Print: Sink.
fn locked_sink(buf: &[u8], line: bool) {
	let mut handle = io::sink();
	handle.write_all(buf).unwrap();

	if line {
		handle.write_all(&[10]).unwrap();
	}

	handle.flush().unwrap();
}

/// Input Prompt
///
/// This prints a question and collects the raw answer. It is up to
/// `Msg::prompt()` to figure out if it makes sense or not. (If not, that
/// method's loop will just recall this method.)
fn read_prompt() -> io::Result<String> {
	let mut result = String::new();
	io::stdin().read_line(&mut result)?;
	Ok(result.trim().to_lowercase())
}
