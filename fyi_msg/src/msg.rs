/*!
# FYI Msg: Message!
*/

use crate::{
	traits::FastConcat,
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
	ptr,
};



/// Buffer Indexes.
///
/// The start and end points of the malleable progress components are stored in
/// an array for easy access. These are their indexes.
const PART_INDENT: usize = 0;
const PART_TIMESTAMP: usize = 1;
const PART_PREFIX: usize = 2;
const PART_MSG: usize = 3;
const PART_SUFFIX: usize = 4;

// Configuration Flags.
//
// These flags are an alternative way to configure indentation and
// timestamping.

/// Enable Indentation (equivalent to 4 spaces).
pub const FLAG_INDENT: u8 =    0b0001;

/// Enable Timestamp.
pub const FLAG_TIMESTAMP: u8 = 0b0010;



#[derive(Clone, Copy)]
/// # Prefix Buffer.
///
/// This is a simple fixed-array buffer to store custom prefixes for
/// `MsgKind::Other`. This is implemented as a custom struct in order to take
/// advantage of `Copy`, etc.
///
/// ## Restrictions
///
/// Because the buffer is fixed at a length of `64`, including the label and
/// any ANSI formatting, this leaves roughly 45 bytes for the label itself.
/// Prefixes exceeding this limit are silently ignored.
pub struct PrefixBuffer {
	buf: [u8; 64],
	len: usize,
}

impl fmt::Debug for PrefixBuffer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("PrefixBuffer")
			.field("buf", &self)
			.finish()
	}
}

impl Default for PrefixBuffer {
	#[inline]
	fn default() -> Self {
		Self {
			buf: [0; 64],
			len: 0,
		}
	}
}

impl Deref for PrefixBuffer {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.buf[0..self.len] }
}

impl Eq for PrefixBuffer {}

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
	/// # New Instance (Unchecked).
	///
	/// Create a new instance using the given prefix and color.
	///
	/// ## Safety
	///
	/// The prefix must be valid UTF-8 and cannot exceed 45 bytes in length.
	pub unsafe fn new_unchecked(prefix: &[u8], color: u8) -> Self {
		use std::mem;

		let mut buf = [mem::MaybeUninit::<u8>::uninit(); 64];
		let dst = buf.as_mut_ptr() as *mut u8;

		// Write the color.
		let mut len: usize = utility::write_ansi_code_bold(dst, color);

		// Write the prefix.
		ptr::copy_nonoverlapping(prefix.as_ptr(), dst.add(len), prefix.len());
		len += prefix.len();

		// Write the closer.
		ptr::copy_nonoverlapping(b":\x1b[0m ".as_ptr(), dst.add(len), 6);

		// Align and return!
		Self {
			buf: mem::transmute::<_, [u8; 64]>(buf),
			len: len + 6,
		}
	}

	#[inline]
	/// # As Bytes.
	///
	/// Return the value as a slice of bytes.
	pub fn as_bytes(&self) -> &[u8] { self }

	#[inline]
	/// # Is Empty?
	pub const fn is_empty(&self) -> bool { 0 == self.len }

	#[inline]
	/// # Length.
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
	#[inline]
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
	/// # Custom Prefix.
	///
	/// A custom prefix requires a string label and [ANSI color code](https://misc.flogisoft.com/bash/tip_colors_and_formatting#colors1). The value
	/// will automatically be suffixed with a colon and space for clean joining
	/// to the message bit.
	///
	/// Custom prefixes, including any ANSI markup, are limited to 64 bytes. If
	/// this limit is exceeded, the prefix is silently ignored, making it
	/// equivalent to `MsgKind::None`.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::MsgKind;
	/// let kind = MsgKind::new("Hello", 199);
	/// ```
	pub fn new<S> (prefix: S, color: u8) -> Self
	where S: AsRef<str> {
		let prefix: &[u8] = prefix.as_ref().trim().as_bytes();

		if prefix.is_empty() || prefix.len() > 45 { Self::None }
		else {
			unsafe {
				Self::Other(PrefixBuffer::new_unchecked(prefix, color))
			}
		}
	}

	#[must_use]
	/// # As Bytes.
	///
	/// Return the formatted prefix as a byte slice.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::MsgKind;
	/// let kind: &[u8] = MsgKind::new("Hello", 199).as_bytes();
	/// ```
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
	/// # As Str.
	///
	/// Return the formatted prefix as a string slice.
	///
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::MsgKind;
	/// let kind = MsgKind::new("Hello", 199).as_str();
	/// ```
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
	}

	/// Into `Msg`.
	///
	/// Create a new [Msg] using this prefix and the specified body text.
	///
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// use fyi_msg::MsgKind;
	///
	/// let kind = MsgKind::new("Hello", 199);
	/// let msg = kind.into_msg("This is my message!");
	/// ```
	pub fn into_msg<S> (self, msg: S) -> Msg
	where S: AsRef<str> {
		Msg::prefixed(self, msg)
	}

	#[must_use]
	/// # Is Empty?
	pub const fn is_empty(&self) -> bool {
		match self {
			Self::None => true,
			Self::Other(x) => x.is_empty(),
			_ => false,
		}
	}

	#[must_use]
	/// # Length.
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
/// The `Msg` struct is a fairly straight-forward way of getting a simple ANSI-
/// formatted message printed to the terminal.
///
/// A number of basic prefixes like "Error" and "Success" are built in. Custom
/// prefixes with arbitrary coloring can be used via [`MsgKind::new`].
///
/// The [`with_indent()`](Msg::with_indent) and [`with_timestamp()`](Msg::with_timestamp) build patterns can prepend
/// indentation or a timestamp to the message, respectively.
///
/// That's it. Nice and boring!
///
/// ## Example
///
/// ```no_run
/// use fyi_msg::Msg;
/// use fyi_msg::MsgKind;
///
/// // Create a message with a custom prefix and color.
/// MsgKind::new("Yo", 199)
///     .into_msg("How are you doing today?")
///     .println();
///
/// // Built-ins work the same way.
/// MsgKind::Error.into_msg("Well darn.").println();
/// MsgKind::Success.into_msg("Oh good!").println();
///
/// // Ask a yes/no question.
/// let res: bool = MsgKind::Confirm.into_msg("Are you OK?").prompt();
/// ```
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
	#[inline]
	fn deref(&self) -> &Self::Target { &self.buf }
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

/// # Helper: From (Fixed) Slice.
///
/// This leverages specialized concatenation methods for byte slices of fixed
/// length, avoiding the need to `[].concat()` or `iter().collect()`.
macro_rules! from_fast_concat {
	($num:literal) => {
		impl From<[&[u8]; $num]> for Msg {
			fn from(src: [&[u8]; $num]) -> Self {
				let src = src.fast_concat();
				let end = src.len() as u16;

				Self {
					toc: Toc::new(
						0_u16, 0_u16, // Indentation.
						0_u16, 0_u16, // Timestamp.
						0_u16, 0_u16, // Prefix.
						0_u16, end,   // Message.
						end, end,     // Suffix.
						// Unused...
						end, end, end, end, end, end, end, end, end,
						end, end, end, end, end, end, end, end, end,
						end, end, end, end
					),
					buf: src,
				}
			}
		}
	};
}

from_fast_concat!(2);
from_fast_concat!(3);
from_fast_concat!(4);
from_fast_concat!(5);
from_fast_concat!(6);
from_fast_concat!(7);
from_fast_concat!(8);

impl From<Vec<u8>> for Msg {
	fn from(src: Vec<u8>) -> Self {
		let end: u16 = src.len() as u16;
		Self {
			toc: Toc::new(
				0_u16, 0_u16, // Indentation.
				0_u16, 0_u16, // Timestamp.
				0_u16, 0_u16, // Prefix.
				0_u16, end,   // Message.
				end, end,     // Suffix.
				// Unused...
				end, end, end, end, end, end, end, end, end,
				end, end, end, end, end, end, end, end, end,
				end, end, end, end
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
	#[inline]
	/// # New Instance.
	///
	/// Create a new message without a prefix. This is basically just a string,
	/// but might have its uses, particularly when combined with the builder
	/// pattern methods.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let msg = Msg::new("Hello world.");
	/// ```
	pub fn new<S> (msg: S) -> Self
	where S: AsRef<str> { Self::from(msg.as_ref()) }

	#[must_use]
	/// # Prefixed Message.
	///
	/// This is an optimized way to generate a new message with a prefix and
	/// body.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// use fyi_msg::MsgKind;
	/// let msg = Msg::prefixed(MsgKind::Success, "Hello world.");
	/// ```
	pub fn prefixed<S> (prefix: MsgKind, msg: S) -> Self
	where S: AsRef<str> {
		if prefix == MsgKind::None { Self::new(msg) }
		else {
			let msg: &[u8] = msg.as_ref().as_bytes();
			if msg.is_empty() {
				Self::default().with_prefix(prefix)
			}
			else {
				unsafe { Self::prefixed_unchecked(prefix, msg) }
			}
		}
	}

	#[must_use]
	/// # Prefixed Message (Unchecked).
	///
	/// This method creates a prefixed message without worrying about the
	/// potential sanity of either component.
	///
	/// ## Safety
	///
	/// This method accepts raw bytes for the message body; that body should be
	/// valid UTF-8 or undefined things may happen.
	pub unsafe fn prefixed_unchecked(prefix: MsgKind, msg: &[u8]) -> Self {
		let (p_len, m_len) = (prefix.len(), msg.len());
		let mut buf: Vec<u8> = Vec::with_capacity(p_len + m_len);

		{
			let ptr = buf.as_mut_ptr();
			ptr::copy_nonoverlapping(prefix.as_bytes().as_ptr(), ptr, p_len);
			ptr::copy_nonoverlapping(msg.as_ptr(), ptr.add(p_len), m_len);
			buf.set_len(m_len + p_len);
		}

		let p_len: u16 = p_len as u16;
		let end: u16 = m_len as u16 + p_len;
		Self {
			buf,
			toc: Toc::new(
				0_u16, 0_u16, // Indentation.
				0_u16, 0_u16, // Timestamp.
				0_u16, p_len, // Prefix.
				p_len, end,   // Message.
				end, end,     // Suffix.
				// Unused...
				end, end, end, end, end, end, end, end, end,
				end, end, end, end, end, end, end, end, end,
				end, end, end, end
			),
		}
	}

	#[must_use]
	/// # With Flags.
	///
	/// Flags can be used to set or unset indentation and timestamping in a
	/// single call. This is equivalent to but more efficient than chaining
	/// both [`with_indent()`](Msg::with_indent) and [`with_timestamp()`](Msg::with_timestamp).
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::{
	///     FLAG_INDENT,
	///     FLAG_TIMESTAMP,
	///     Msg
	/// };
	/// let msg = Msg::new("Hello world.")
	///     .with_flags(FLAG_INDENT | FLAG_TIMESTAMP);
	/// ```
	pub fn with_flags(mut self, flags: u8) -> Self {
		self.set_indent(
			if 0 == flags & FLAG_INDENT { 0 }
			else { 1 }
		);
		self.set_timestamp(0 != flags & FLAG_TIMESTAMP);
		self
	}

	#[must_use]
	/// # With Indent.
	///
	/// Use this method to indent the message `indent` number of levels, each
	/// level being four spaces. Acceptable values fall in the range of `0..=4`.
	/// Anything greater than that range is simply truncated to 16 spaces.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let msg = Msg::new("Hello world.")
	///     .with_indent(1);
	/// ```
	pub fn with_indent(mut self, indent: u8) -> Self {
		self.set_indent(indent);
		self
	}

	#[allow(clippy::missing_const_for_fn)] // Doesn't work.
	#[must_use]
	/// # With Prefix.
	///
	/// Set the message prefix.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// use fyi_msg::MsgKind;
	/// let msg = Msg::new("Hello world.")
	///     .with_prefix(MsgKind::Success);
	/// ```
	pub fn with_prefix(mut self, prefix: MsgKind) -> Self {
		self.set_prefix(prefix);
		self
	}

	#[must_use]
	/// # With Timestamp.
	///
	/// Messages are not timestamped by default, but can be if `true` is passed
	/// to this method. Timestamps are formatted the Unix way, i.e. the *only*
	/// way that makes sense: `[YYYY-MM-DD hh:mm:ss]`.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let msg = Msg::new("Hello world.")
	///     .with_timestamp(true);
	/// ```
	pub fn with_timestamp(mut self, on: bool) -> Self {
		self.set_timestamp(on);
		self
	}



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	/// # Set Indent.
	///
	/// Set or reset the level of indentation. See [`Msg::with_indent`] for more
	/// information.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let mut msg = Msg::new("Hello world.");
	/// msg.set_indent(0); // "Hello World."
	/// msg.set_indent(1); // "    Hello World."
	/// msg.set_indent(2); // "        Hello World."
	/// ```
	pub fn set_indent(&mut self, indent: u8) {
		static INDENT: [u8; 16] = *b"                ";

		unsafe {
			self.toc.replace_unchecked(
				&mut self.buf,
				PART_INDENT,
				match indent {
					0 => b"",
					1 => &INDENT[0..4],
					2 => &INDENT[0..8],
					3 => &INDENT[0..12],
					_ => &INDENT,
				}
			);
		}
	}

	#[inline]
	/// # Set Message.
	///
	/// Set or reset the message body.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let mut msg = Msg::new("Hello world.");
	/// msg.set_msg("Goodbye world.");
	/// ```
	pub fn set_msg<S> (&mut self, msg: S)
	where S: AsRef<str> {
		unsafe {
			self.toc.replace_unchecked(&mut self.buf, PART_MSG, msg.as_ref().as_bytes());
		}
	}

	#[inline]
	/// # Set Prefix.
	///
	/// Set or reset the message prefix.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// use fyi_msg::MsgKind;
	///
	/// let mut msg = Msg::new("Hello world.");
	/// msg.set_prefix(MsgKind::Error);
	/// ```
	pub fn set_prefix(&mut self, prefix: MsgKind) {
		unsafe {
			self.toc.replace_unchecked(&mut self.buf, PART_PREFIX, prefix.as_bytes());
		}
	}

	/// # Set Suffix (Unchecked)
	///
	/// This method sets the suffix exactly as specified. It should have a
	/// leading space, and should probably reset ANSI formatting at the end.
	///
	/// ## Safety
	///
	/// This method is "unsafe" insofar as the data is accepted without any
	/// checks or manipulation.
	pub unsafe fn set_suffix_unchecked(&mut self, suffix: &[u8]) {
		self.toc.replace_unchecked(&mut self.buf, PART_SUFFIX, suffix);
	}

	/// # Set Timestamp.
	///
	/// Enable or disable the message timestamp. See [`Msg::with_timestamp`] for
	/// more information.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let mut msg = Msg::new("Hello world.");
	/// msg.set_timestamp(true);  // Turn it on.
	/// msg.set_timestamp(false); // Turn it off.
	/// ```
	pub fn set_timestamp(&mut self, on: bool) {
		if on == self.toc.is_empty(PART_TIMESTAMP) {
			unsafe {
				if on {
					self.write_timestamp();
				}
				else {
					self.toc.zero_unchecked(&mut self.buf, PART_TIMESTAMP);
				}
			}
		}
	}



	// ------------------------------------------------------------------------
	// Conversion
	// ------------------------------------------------------------------------

	#[must_use]
	#[inline]
	/// # As Bytes.
	///
	/// Return the message as a slice of bytes.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let mut msg = Msg::new("Hello world.");
	/// let bytes: &[u8] = msg.as_bytes();
	/// ```
	pub fn as_bytes(&self) -> &[u8] { &self.buf }

	#[must_use]
	#[inline]
	/// # As Str.
	///
	/// Return the message as a string slice.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let mut msg = Msg::new("Hello world.");
	/// let bytes: &str = msg.as_str();
	/// ```
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.buf) }
	}

	#[allow(clippy::missing_const_for_fn)] // Doesn't work!
	#[must_use]
	#[inline]
	/// # Into Vec.
	///
	/// Consume the message, converting it into an owned byte vector.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let mut msg: Vec<u8> = Msg::new("Hello world.").into_vec();
	/// ```
	pub fn into_vec(self) -> Vec<u8> { self.buf }



	// ------------------------------------------------------------------------
	// Output
	// ------------------------------------------------------------------------

	#[must_use]
	/// # Prompt.
	///
	/// This produces a simple y/N input prompt, requiring the user type "Y" or
	/// "N" to proceed. Positive values return `true`, negative values return
	/// `false`. The default (if the user just hits <enter>) is "N".
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let mut msg = Msg::new("Do you like chickens?");
	/// if msg.prompt() {
	///    println!("That's great! They like you too!");
	/// }
	/// ```
	pub fn prompt(&self) -> bool {
		// Clone the message and append a little [y/N] instructional bit to the
		// end.
		let mut q = self.clone();
		unsafe { q.set_suffix_unchecked(b" \x1b[2m[y/\x1b[4mN\x1b[0;2m]\x1b[0m "); }

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
			unsafe {
				Self::prefixed_unchecked(
					MsgKind::Error,
					b"Invalid input: enter \x1b[91mN\x1b[0m or \x1b[92mY\x1b[0m."
				).println();
			}
		}
	}

	/// # Print.
	///
	/// Print the message to `Stdout`, equivalent to the `print!` macro.
	pub fn print(&self) { locked_print(&self.buf, false); }

	/// # Print w/ Line.
	///
	/// Print the message with a trailing line break to `Stdout`, equivalent to
	/// the `println!` macro.
	pub fn println(&self) { locked_print(&self.buf, true); }

	/// # Print Error.
	///
	/// Print the message to `Stderr`, equivalent to the `eprint!` macro.
	pub fn eprint(&self) { locked_eprint(&self.buf, false); }

	/// # Print Error w/ Line.
	///
	/// Print the message with a trailing line break to `Stderr`, equivalent to
	/// the `eprintln!` macro.
	pub fn eprintln(&self) { locked_eprint(&self.buf, true); }



	// ------------------------------------------------------------------------
	// Internal
	// ------------------------------------------------------------------------

	/// # Write the timestamp bit.
	///
	/// This method updates the timestamp slice of the message buffer. While
	/// `chrono` provides formatting helpers of its own, they are too slow for
	/// our use cases.
	unsafe fn write_timestamp(&mut self) {
		use chrono::{
			Datelike,
			Local,
			Timelike,
		};

		self.toc.replace_unchecked(
			&mut self.buf,
			PART_TIMESTAMP,
			b"\x1b[2m[\x1b[0;34m2000-00-00 00:00:00\x1b[39;2m]\x1b[0m ",
		);

		// Chrono's formatter is slow as shit. It is faster for us to call
		// each of their time part methods individually, convert those
		// integers to bytes, and copy them into our static buffer.
		let now = Local::now();

		let mut ptr = self.buf.as_mut_ptr().add(self.toc.start_unchecked(PART_TIMESTAMP) + 14);
		utility::write_time_dd(ptr, (now.year() as u16).saturating_sub(2000) as u8);

		ptr = ptr.add(3);
		utility::write_time_dd(ptr, now.month() as u8);

		ptr = ptr.add(3);
		utility::write_time_dd(ptr, now.day() as u8);

		ptr = ptr.add(3);
		utility::write_time_dd(ptr, now.hour() as u8);

		ptr = ptr.add(3);
		utility::write_time_dd(ptr, now.minute() as u8);

		ptr = ptr.add(3);
		utility::write_time_dd(ptr, now.second() as u8);
	}
}



#[inline]
/// # Locked Print.
///
/// Print data to `Stdout`, locking the writer until all data has been flushed.
fn locked_print(buf: &[u8], line: bool) {
	let writer = std::io::stdout();
	let mut handle = writer.lock();
	handle.write_all(buf).unwrap();

	if line {
		handle.write_all(&[10]).unwrap();
	}

	handle.flush().unwrap();
}

#[inline]
/// # Locked Error Print.
///
/// Print data to `Stderr`, locking the writer until all data has been flushed.
fn locked_eprint(buf: &[u8], line: bool) {
	let writer = std::io::stderr();
	let mut handle = writer.lock();
	handle.write_all(buf).unwrap();

	if line {
		handle.write_all(&[10]).unwrap();
	}

	handle.flush().unwrap();
}

/// # Input Prompt
///
/// This is used by [`Msg::prompt`] to read/normalize the user response to the
/// question.
fn read_prompt() -> io::Result<String> {
	let mut result = String::new();
	io::stdin().read_line(&mut result)?;
	Ok(result.trim().to_lowercase())
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_msg() {
		let mut msg = Msg::from("My dear aunt sally.");
		assert_eq!(&*msg, b"My dear aunt sally.");

		msg.set_prefix(MsgKind::Error);
		assert!(msg.starts_with(MsgKind::Error.as_bytes()));
		assert!(msg.ends_with(b"My dear aunt sally."));

		msg.set_indent(1);
		assert!(msg.starts_with(b"    "));
		msg.set_indent(3);
		assert!(msg.starts_with(b"            "));
		msg.set_indent(0);
		assert!(msg.starts_with(MsgKind::Error.as_bytes()));

		unsafe { msg.set_suffix_unchecked(b" Heyo"); }
		assert!(msg.ends_with(b" Heyo"), "{:?}", msg.as_str());
		unsafe { msg.set_suffix_unchecked(b""); }
		assert!(msg.ends_with(b"My dear aunt sally."));

		msg.set_msg("My dear aunt");
		assert!(msg.ends_with(b"My dear aunt"));
	}
}
