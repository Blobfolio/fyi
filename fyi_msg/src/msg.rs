/*!
# FYI Message

The `Msg` struct is a fairly straight-forward way of getting a simple ANSI-
formatted message printed to the terminal.

A number of basic prefixes like "Error" and "Success" are built in. Custom
prefixes with arbitrary coloring can be used via `MsgKind::new()`.

The `with_indent()` and `with_timestamp()` build patterns can prepend
indentation or a timestamp to the message, respectively.

That's it. Nice and boring!

## Restrictions:

Custom prefixes are limited to 64 bytes, including the formatting code. This
leaves roughly 45 bytes for the label itself.

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

use crate::{
	Toc,
	utility,
};
use std::{
	cmp::Ordering,
	fmt,
	hash::{
		Hash,
		Hasher,
	},
	io::{
		self,
		Write,
	},
	iter::FromIterator,
	ops::Deref,
};



/// Buffer Indexes.
///
/// The start and end points of the malleable progress components are stored in
/// an array for easy access. These are their indexes.
const PART_INDENT: usize = 0;
const PART_TIMESTAMP: usize = 1;
const PART_PREFIX: usize = 2;
const PART_MSG: usize = 3;

/// Configuration Flags.
///
/// These flags are an alternative way to configure indentation and
/// timestamping.

/// Indentation.
pub const FLAG_INDENT: u8 =    0b0001;

/// Timestamp.
pub const FLAG_TIMESTAMP: u8 = 0b0010;



#[derive(Clone, Copy)]
/// Prefix Buffer.
///
/// This is a simple fixed-array buffer to store custom prefixes for
/// `MsgKind::Other`. This is implemented as a custom struct in order to take
/// advantage of `Copy`, etc.
pub struct PrefixBuffer {
	buf: [u8; 64],
	len: usize,
}

impl fmt::Debug for PrefixBuffer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("PrefixBuffer")
			.field("buf", &self.as_bytes())
			.finish()
	}
}

impl Default for PrefixBuffer {
	fn default() -> Self {
		Self {
			buf: [0; 64],
			len: 0,
		}
	}
}

impl Deref for PrefixBuffer {
	type Target = [u8];
	fn deref(&self) -> &Self::Target { self.as_bytes() }
}

impl Eq for PrefixBuffer {}

impl FromIterator<u8> for PrefixBuffer {
	fn from_iter<I: IntoIterator<Item=u8>>(iter: I) -> Self {
		Self::from(iter.into_iter().collect::<Vec<u8>>())
	}
}

impl From<Vec<u8>> for PrefixBuffer {
	fn from(src: Vec<u8>) -> Self {
		match src.len() {
			1..=64 => {
				let mut out = Self::default();
				out.len = src.len();
				out.buf[0..out.len].copy_from_slice(&src);
				out
			},
			_ => Self::default(),
		}
	}
}

impl Hash for PrefixBuffer {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_bytes().hash(state);
	}
}

impl Ord for PrefixBuffer {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_bytes().cmp(other.as_bytes())
	}
}

impl PartialEq for PrefixBuffer {
	fn eq(&self, other: &Self) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}

impl PartialOrd for PrefixBuffer {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.as_bytes().cmp(other.as_bytes()))
	}
}

impl PrefixBuffer {
	/// As Bytes.
	pub fn as_bytes(&self) -> &[u8] { &self.buf[0..self.len] }

	/// Is Empty.
	pub const fn is_empty(&self) -> bool { 0 == self.len }

	/// Length.
	pub const fn len(&self) -> usize { self.len }
}



#[derive(Debug, Copy, Clone, Hash, PartialEq)]
/// Message Kind.
///
/// This is the prefix, basically. `Other` owns a `Vec<u8>` — its byte string
/// equivalent — while all other options are static.
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
	Other(PrefixBuffer),
}

impl Default for MsgKind {
	fn default() -> Self { Self::None }
}

impl fmt::Display for MsgKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(unsafe { std::str::from_utf8_unchecked(self.as_bytes()) })
	}
}

impl From<&str> for MsgKind {
	/// This is a convenience method to convert a lower case string constant to
	/// the equivalent kind. `Other` is not reachable this way; non-matches
	/// match to `None` instead.
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
	/// Custom Prefix.
	///
	/// A custom prefix requires a string label and ANSI color code. The value
	/// will automatically be suffixed with a colon and space for clean joining
	/// to the message bit.
	pub fn new<S> (prefix: S, color: u8) -> Self
	where S: AsRef<str> {
		let prefix = prefix.as_ref().trim();
		if prefix.is_empty() { Self::None }
		else {
			Self::Other(PrefixBuffer::from_iter(
				utility::ansi_code_bold(color).iter()
					.chain(prefix.as_bytes().iter())
					.chain(b":\x1b[0m ".iter())
					.copied()
			))
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
			Self::Other(x) => x,
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
		Msg::from(msg.as_ref()).with_prefix(self)
	}

	#[must_use]
	/// Is Empty.
	pub const fn is_empty(&self) -> bool {
		match self {
			Self::None => true,
			Self::Other(x) => x.is_empty(),
			_ => false,
		}
	}

	#[must_use]
	/// Length.
	pub const fn len(&self) -> usize {
		match self {
			Self::None => 0,
			Self::Confirm => 26,
			Self::Crunched => 21,
			Self::Done | Self::Info => 17,
			Self::Debug | Self::Error => 18,
			Self::Notice => 19,
			Self::Success | Self::Warning => 20,
			Self::Task => 23,
			Self::Other(x) => x.len(),
		}
	}
}



#[derive(Debug, Clone, Default)]
/// Message.
///
/// This is it! The whole point of the crate! See the library documentation for
/// more information.
pub struct Msg {
	/// The compiled buffer!
	buf: Vec<u8>,
	/// A Table of Contents.
	toc: Toc,
}

impl AsRef<str> for Msg {
	fn as_ref(&self) -> &str { self.as_str() }
}

impl Deref for Msg {
	type Target = [u8];
	fn deref(&self) -> &Self::Target { self.as_bytes() }
}

impl fmt::Display for Msg {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<&str> for Msg {
	fn from(src: &str) -> Self { Self::from(src.as_bytes().to_vec()) }
}

impl From<String> for Msg {
	fn from(src: String) -> Self { Self::from(src.into_bytes()) }
}

impl From<&[u8]> for Msg {
	fn from(src: &[u8]) -> Self { Self::from(src.to_vec()) }
}

impl From<Vec<u8>> for Msg {
	fn from(src: Vec<u8>) -> Self {
		let end: u16 = src.len() as u16;
		Self {
			toc: Toc::new(
				0_u16, 0_u16,
				0_u16, 0_u16,
				0_u16, 0_u16,
				0_u16, end,
				// Unused...
				end, end, end, end, end, end, end, end, end,
				end, end, end, end, end, end, end, end, end,
				end, end, end, end, end, end
			),
			buf: src,
		}
	}
}

impl FromIterator<u8> for Msg {
	fn from_iter<I: IntoIterator<Item=u8>>(iter: I) -> Self {
		Self::from(iter.into_iter().collect::<Vec<u8>>())
	}
}

impl Hash for Msg {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.buf.hash(state);
	}
}

impl PartialEq for Msg {
	fn eq(&self, other: &Self) -> bool {
		self.buf == other.buf
	}
}

impl PartialEq<[u8]> for Msg {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_bytes() == other
	}
}

impl PartialEq<&str> for Msg {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
	}
}

impl Msg {
	/// New.
	///
	/// Create a new message without a prefix. This is basically just a string,
	/// but might have its uses, particularly when combined with the build
	/// pattern methods.
	pub fn new<S> (msg: S) -> Self
	where S: AsRef<str> { Self::from(msg.as_ref()) }

	#[must_use]
	/// With Flags.
	///
	/// Flags can be used to set or unset indentation and timestamping.
	pub fn with_flags(mut self, flags: u8) -> Self {
		self.set_indent(
			if 0 == flags & FLAG_INDENT { 0 }
			else { 1 }
		);
		self.set_timestamp(0 != flags & FLAG_TIMESTAMP);
		self
	}

	#[must_use]
	/// With Indent.
	///
	/// Use this method to indent the message `indent` number of levels. Each
	/// level is equivalent to four spaces, e.g. `1 == "    "`,
	/// `2 == "        "`, etc.
	pub fn with_indent(mut self, indent: u8) -> Self {
		self.set_indent(indent);
		self
	}

	#[allow(clippy::missing_const_for_fn)] // Doesn't work.
	#[must_use]
	/// With Prefix.
	pub fn with_prefix(mut self, prefix: MsgKind) -> Self {
		self.set_prefix(prefix);
		self
	}

	#[must_use]
	/// With Timestamp.
	///
	/// Messages are not timestamped by default, but can be if `true` is passed
	/// to this method.
	pub fn with_timestamp(mut self, on: bool) -> Self {
		self.set_timestamp(on);
		self
	}



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	/// Set Indent.
	pub fn set_indent(&mut self, indent: u8) {
		static WHITES: [u8; 16] = [32; 16];

		let indent: usize = 4.min(indent as usize) * 4;
		if indent != self.toc.len(PART_INDENT) {
			self.toc.replace(&mut self.buf, PART_INDENT, &WHITES[0..indent]);
		}
	}

	/// Set Message.
	pub fn set_msg<S> (&mut self, msg: S)
	where S: AsRef<str> {
		self.toc.replace(&mut self.buf, PART_MSG, msg.as_ref().as_bytes());
	}

	/// Set Prefix.
	pub fn set_prefix(&mut self, prefix: MsgKind) {
		self.toc.replace(&mut self.buf, PART_PREFIX, prefix.as_bytes());
	}

	/// Set Timestamp.
	pub fn set_timestamp(&mut self, on: bool) {
		if on == self.toc.is_empty(PART_TIMESTAMP) {
			if on { self.write_timestamp(); }
			else {
				self.toc.resize(&mut self.buf, PART_TIMESTAMP, 0);
			}
		}
	}



	// ------------------------------------------------------------------------
	// Conversion
	// ------------------------------------------------------------------------

	#[must_use]
	/// As Bytes.
	pub fn as_bytes(&self) -> &[u8] { &self.buf }

	#[must_use]
	/// As Str.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.buf) }
	}

	#[allow(clippy::missing_const_for_fn)] // Doesn't work!
	#[must_use]
	/// Into Vec.
	pub fn into_vec(self) -> Vec<u8> { self.buf }



	// ------------------------------------------------------------------------
	// Output
	// ------------------------------------------------------------------------

	#[must_use]
	/// Prompt.
	///
	/// This produces a simple y/N input prompt, requiring the user type "Y" or
	/// "N" to proceed. Positive values return `true`, negative values return
	/// `false`. The default is No.
	pub fn prompt(&self) -> bool {
		// Clone the message and append a little [y/N] instructional bit to the
		// end.
		let mut q = self.clone();
		q.buf.extend_from_slice(b" \x1b[2m[y/\x1b[4mN\x1b[0;2m]\x1b[0m ");
		q.toc.increase(PART_MSG, 25);

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

	/// Print: `Stdout`.
	pub fn print(&self) {
		locked_print(&self.buf, false);
	}

	/// Print w/ Line: `Stdout`.
	pub fn println(&self) {
		locked_print(&self.buf, true);
	}

	/// Print: `Stderr`.
	pub fn eprint(&self) {
		locked_eprint(&self.buf, false);
	}

	/// Print w/ Line: `Stderr`.
	pub fn eprintln(&self) {
		locked_eprint(&self.buf, true);
	}



	// ------------------------------------------------------------------------
	// Internal
	// ------------------------------------------------------------------------

	/// Timestamp bit.
	fn write_timestamp(&mut self) {
		use chrono::{
			Datelike,
			Local,
			Timelike,
		};

		// Make sure we have something in place.
		if self.toc.is_empty(PART_TIMESTAMP) {
			self.toc.replace(
				&mut self.buf,
				PART_TIMESTAMP,
				b"\x1b[2m[\x1b[0;34m2000-00-00 00:00:00\x1b[39;2m]\x1b[0m ",
			);
		}

		// Chrono's formatter is slow as shit. It is faster for us to call
		// each of their time part methods individually, convert those
		// integers to bytes, and copy them into our static buffer.
		let now = Local::now();
		let ptr = self.buf.as_mut_ptr();
		unsafe {
			[
				(now.year() as u32).saturating_sub(2000),
				now.month(),
				now.day(),
				now.hour(),
				now.minute(),
				now.second(),
			].iter()
				.fold(
					self.toc.start(PART_TIMESTAMP) + 14,
					|dst_off, x| {
					ptr.add(dst_off).copy_from_nonoverlapping(utility::time_format_dd(*x).as_ptr(), 2);
					dst_off + 3
				});
		}
	}
}



/// Locked Print: `Stdout`.
fn locked_print(buf: &[u8], line: bool) {
	let writer = std::io::stdout();
	let mut handle = writer.lock();
	handle.write_all(buf).unwrap();

	if line {
		handle.write_all(&[10]).unwrap();
	}

	handle.flush().unwrap();
}

/// Locked Print: `Stderr`.
fn locked_eprint(buf: &[u8], line: bool) {
	let writer = std::io::stderr();
	let mut handle = writer.lock();
	handle.write_all(buf).unwrap();

	if line {
		handle.write_all(&[10]).unwrap();
	}

	handle.flush().unwrap();
}

/// Input Prompt
///
/// This is used by `Msg.prompt()` to read/normalize the user response to the
/// question.
fn read_prompt() -> io::Result<String> {
	let mut result = String::new();
	io::stdin().read_line(&mut result)?;
	Ok(result.trim().to_lowercase())
}
