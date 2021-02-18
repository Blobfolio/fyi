/*!
# FYI Msg
*/

pub(super) mod buffer;
pub(super) mod kind;

use crate::{
	MsgKind,
	MsgBuffer,
};
use dactyl::NiceU8;
use std::{
	fmt::{
		self,
		Arguments,
	},
	hash,
	io,
	ops::Deref,
};

#[cfg(feature = "fitted")] use std::borrow::Cow;



#[cfg(feature = "timestamps")]      const MSGBUFFER: usize = crate::BUFFER6;
#[cfg(not(feature = "timestamps"))] const MSGBUFFER: usize = crate::BUFFER5;



/// # Helper: ToC Setup.
#[cfg(feature = "timestamps")]
macro_rules! new_toc {
	($p_end:expr, $m_end:expr) => (
		[
			0, 0,           // Indentation.
			0, 0,           // Timestamp.
			0, $p_end,      // Prefix.
			$p_end, $m_end, // Message.
			$m_end, $m_end, // Suffix.
			$m_end, $m_end, // Newline.
		]
	);
	($p_end:expr, $m_end:expr, true) => (
		[
			0, 0,               // Indentation.
			0, 0,               // Timestamp.
			0, $p_end,          // Prefix.
			$p_end, $m_end,     // Message.
			$m_end, $m_end,     // Suffix.
			$m_end, $m_end + 1, // Newline.
		]
	);
}

#[cfg(not(feature = "timestamps"))]
macro_rules! new_toc {
	($p_end:expr, $m_end:expr) => (
		[
			0, 0,           // Indentation.
			0, $p_end,      // Prefix.
			$p_end, $m_end, // Message.
			$m_end, $m_end, // Suffix.
			$m_end, $m_end, // Newline.
		]
	);
	($p_end:expr, $m_end:expr, true) => (
		[
			0, 0,               // Indentation.
			0, $p_end,          // Prefix.
			$p_end, $m_end,     // Message.
			$m_end, $m_end,     // Suffix.
			$m_end, $m_end + 1, // Newline.
		]
	);
}



// Buffer Indexes.

/// Buffer Index: Indentation.
const PART_INDENT: usize = 0;

/// Buffer Index: Timestamp.
#[cfg(feature = "timestamps")] const PART_TIMESTAMP: usize = 1;

/// Buffer Index: Prefix.
#[cfg(feature = "timestamps")] const PART_PREFIX: usize = 2;
#[cfg(not(feature = "timestamps"))] const PART_PREFIX: usize = 1;

/// Buffer Index: Message body.
#[cfg(feature = "timestamps")] const PART_MSG: usize = 3;
#[cfg(not(feature = "timestamps"))] const PART_MSG: usize = 2;

/// Buffer Index: Suffix.
#[cfg(feature = "timestamps")] const PART_SUFFIX: usize = 4;
#[cfg(not(feature = "timestamps"))] const PART_SUFFIX: usize = 3;

/// Buffer Index: Newline.
#[cfg(feature = "timestamps")] const PART_NEWLINE: usize = 5;
#[cfg(not(feature = "timestamps"))] const PART_NEWLINE: usize = 4;



// Configuration Flags.
//
// These flags are an alternative way to configure indentation and
// timestamping.

/// Enable Indentation (equivalent to 4 spaces).
pub const FLAG_INDENT: u8 =    0b0001;

#[cfg(feature = "timestamps")]
/// Enable Timestamp.
pub const FLAG_TIMESTAMP: u8 = 0b0010;

/// Enable Trailing Line.
pub const FLAG_NEWLINE: u8 =   0b0100;



#[derive(Debug, Default, Clone)]
/// # Message.
///
/// The `Msg` struct provides a partitioned, contiguous byte source to hold
/// arbitrary messages of the "Error: Oh no!" variety. They can be modified
/// efficiently in place (per-part) and printed to `Stderr` or `Stdout`.
///
/// ## Examples
///
/// ```no_run
/// use fyi_msg::{Msg, MsgKind};
/// Msg::new(MsgKind::Success, "You did it!")
///     .with_newline(true)
///     .print();
/// ```
///
/// Take a look at the `examples/` directory for a rundown on the different
/// message types and basic usage.
pub struct Msg(MsgBuffer<MSGBUFFER>);

impl AsRef<[u8]> for Msg {
	#[inline]
	fn as_ref(&self) -> &[u8] { self.as_bytes() }
}

impl AsRef<str> for Msg {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl std::borrow::Borrow<str> for Msg {
	#[inline]
	fn borrow(&self) -> &str { self.as_str() }
}

impl Deref for Msg {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl fmt::Display for Msg {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<&str> for Msg {
	#[inline]
	fn from(src: &str) -> Self { Self::plain(src) }
}

impl From<String> for Msg {
	#[inline]
	fn from(src: String) -> Self { Self::plain(src) }
}

impl Eq for Msg {}

impl hash::Hash for Msg {
	#[inline]
	fn hash<H: hash::Hasher>(&self, state: &mut H) { self.0.hash(state); }
}

impl PartialEq for Msg {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl PartialEq<str> for Msg {
	#[inline]
	fn eq(&self, other: &str) -> bool { self.as_str() == other }
}

impl PartialEq<String> for Msg {
	#[inline]
	fn eq(&self, other: &String) -> bool { self.as_str() == other }
}

impl PartialEq<[u8]> for Msg {
	#[inline]
	fn eq(&self, other: &[u8]) -> bool { self.as_bytes() == other }
}

impl PartialEq<Vec<u8>> for Msg {
	#[inline]
	fn eq(&self, other: &Vec<u8>) -> bool { self.0 == *other }
}

/// ## Instantiation.
impl Msg {
	/// # New Message.
	///
	/// This creates a new message with a built-in prefix (which can be
	/// [`MsgKind::None`](crate::MsgKind::None), though in that case, [`Msg::plain`] is better).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, MsgKind};
	/// let msg = Msg::new(MsgKind::Info, "This is a message.");
	/// ```
	pub fn new<S>(kind: MsgKind, msg: S) -> Self
	where S: AsRef<str> {
		let msg = msg.as_ref().as_bytes();
		let p_end = kind.len_32();
		let m_end = p_end + msg.len() as u32;

		Self(MsgBuffer::from_raw_parts(
			[kind.as_bytes(), msg].concat(),
			new_toc!(p_end, m_end)
		))
	}

	/// # Custom Prefix.
	///
	/// This creates a new message with a user-defined prefix and color. See
	/// [here](https://misc.flogisoft.com/bash/tip_colors_and_formatting) for a BASH color code primer.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, MsgKind};
	/// let msg = Msg::custom("Prefix", 199, "This message has a pink prefix.");
	/// ```
	pub fn custom<S>(prefix: S, color: u8, msg: S) -> Self
	where S: AsRef<str> {
		let prefix = prefix.as_ref().as_bytes();
		if prefix.is_empty() {
			return Self::plain(msg);
		}

		// Start a vector with the prefix bits.
		let msg = msg.as_ref().as_bytes();
		let v = [
			b"\x1b[1;38;5;",
			&*NiceU8::from(color),
			b"m",
			prefix,
			b":\x1b[0m ",
			msg,
		].concat();

		let m_end = v.len() as u32;
		let p_end = m_end - msg.len() as u32;

		Self(MsgBuffer::from_raw_parts(v, new_toc!(p_end, m_end)))
	}

	/// # Custom Prefix (Unchecked)
	///
	/// Same as [`Msg::custom`], except no validation or formatting is applied
	/// to the provided prefix. This can be useful in cases where the prefix
	/// requires special spacing or delimiters.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, MsgKind};
	/// let msg = Msg::custom_unchecked("Prefix:", "This message has an unformatted prefix.");
	/// ```
	pub fn custom_unchecked<S>(prefix: S, msg: S) -> Self
	where S: AsRef<str> {
		let prefix = prefix.as_ref().as_bytes();
		let msg = msg.as_ref().as_bytes();

		let p_end = prefix.len() as u32;
		let m_end = p_end + msg.len() as u32;

		Self(MsgBuffer::from_raw_parts(
			[prefix, msg].concat(),
			new_toc!(p_end, m_end)
		))
	}

	/// # New Message Without Prefix.
	///
	/// This is equivalent to [`Msg::new`] with [`MsgKind::None`], but
	/// streamlined.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let msg = Msg::plain("This message has no prefix.");
	/// ```
	pub fn plain<S>(msg: S) -> Self
	where S: AsRef<str> {
		let msg = msg.as_ref().as_bytes();
		let len = msg.len() as u32;

		Self(MsgBuffer::from_raw_parts(
			msg.to_vec(),
			new_toc!(0, len)
		))
	}

	/// # Error
	///
	/// This is a convenience method for quickly creating a new error with a
	/// terminating line break. After creation, it is a normal [`Msg`] that can
	/// be manipulated in the usual ways.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, MsgKind};
	/// assert_eq!(
	///     Msg::new(MsgKind::Error, "Oh no!"),
	///     Msg::error("Oh no!")
	/// );
	/// ```
	pub fn error<S>(msg: S) -> Self
	where S: AsRef<str> {
		let msg = msg.as_ref().as_bytes();
		let len = msg.len();
		let mut v = Vec::with_capacity(19 + len);
		v.extend_from_slice(MsgKind::Error.as_bytes());
		v.extend_from_slice(msg);
		v.extend_from_slice(b"\n");

		let m_end = len as u32 + 18;

		Self(MsgBuffer::from_raw_parts(v, new_toc!(18, m_end, true)))
	}
}

/// ## Formatted Instantiation.
impl Msg {
	#[must_use]
	/// # Plain Formatted.
	///
	/// ## Panics
	///
	/// This will panic if not well formed.
	pub fn fmt(args: Arguments) -> Self {
		use std::io::Write;

		let mut v: Vec<u8> = Vec::new();
		v.write_fmt(args).unwrap();

		let len: u32 = v.len() as u32;
		Self(MsgBuffer::from_raw_parts(v, new_toc!(0, len)))
	}

	#[must_use]
	/// # Prefixed Formatted.
	///
	/// ## Panics
	///
	/// This will panic if not well formed.
	pub fn fmt_prefixed(kind: MsgKind, args: Arguments) -> Self {
		use std::io::Write;

		let mut v: Vec<u8> = kind.as_bytes().to_vec();
		v.write_fmt(args).unwrap();

		let p_end = kind.len_32();
		let m_end = v.len() as u32;

		Self(MsgBuffer::from_raw_parts(v, new_toc!(p_end, m_end)))
	}

	#[must_use]
	/// # Prefixed Formatted.
	///
	/// ## Panics
	///
	/// This will panic if not well formed.
	pub fn fmt_custom<S>(prefix: S, color: u8, args: Arguments) -> Self
	where S: AsRef<str> {
		use std::io::Write;

		let prefix = prefix.as_ref().as_bytes();
		if prefix.is_empty() {
			return Self::fmt(args);
		}

		// Start with the prefix.
		let mut v: Vec<u8> = [
			b"\x1b[1;38;5;",
			&*NiceU8::from(color),
			b"m",
			prefix,
			b":\x1b[0m ",
		].concat();
		let p_end: u32 = v.len() as u32;

		// Add the message.
		v.write_fmt(args).unwrap();
		let m_end: u32 = v.len() as u32;

		Self(MsgBuffer::from_raw_parts(v, new_toc!(p_end, m_end)))
	}
}

/// ## Builders.
impl Msg {
	#[must_use]
	/// # With Flags.
	///
	/// This can be used to quickly set indentation, timestamps, and/or a
	/// trailing line break, but only affirmatively; it will not unset any
	/// missing value.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, FLAG_INDENT, FLAG_NEWLINE};
	/// let msg = Msg::plain("Indented message with trailing line.")
	///     .with_flags(FLAG_INDENT | FLAG_NEWLINE);
	/// ```
	pub fn with_flags(mut self, flags: u8) -> Self {
		if 0 != flags & FLAG_INDENT {
			self.set_indent(1);
		}

		#[cfg(feature = "timestamps")]
		if 0 != flags & FLAG_TIMESTAMP {
			self.set_timestamp(true);
		}

		if 0 != flags & FLAG_NEWLINE {
			self.set_newline(true);
		}
		self
	}

	#[must_use]
	/// # With Indent.
	///
	/// Indent the message using four spaces per `indent`. To remove
	/// indentation, pass `0`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let msg = Msg::plain("Indented message.")
	///     .with_indent(1);
	/// ```
	pub fn with_indent(mut self, indent: u8) -> Self {
		self.set_indent(indent);
		self
	}

	#[cfg(feature = "timestamps")]
	#[must_use]
	/// # With Timestamp.
	///
	/// Disable, enable, and/or recalculate the timestamp prefix for the
	/// message.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let msg = Msg::plain("Timestamped message.")
	///     .with_timestamp(true);
	/// ```
	pub fn with_timestamp(mut self, timestamp: bool) -> Self {
		self.set_timestamp(timestamp);
		self
	}

	#[must_use]
	/// # With Linebreak.
	///
	/// Add a trailing linebreak to the end of the message. This is either one
	/// or none; calling it multiple times won't add more lines.
	///
	/// Without a linebreak, [`Msg::print`] is analogous to `print!()`,
	/// whereas with a linebreak, it is more like `println!()`. (The newline
	/// isn't limited to print contexts, but that's mainly what it is for.)
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let msg = Msg::plain("This has a trailing newline.")
	///     .with_newline(true);
	/// ```
	pub fn with_newline(mut self, newline: bool) -> Self {
		self.set_newline(newline);
		self
	}

	#[must_use]
	/// # With Prefix.
	///
	/// Set or reset the message prefix.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, MsgKind};
	/// assert_eq!(
	///     Msg::plain("Hello world.").with_prefix(MsgKind::Success),
	///     Msg::new(MsgKind::Success, "Hello world.")
	/// );
	/// ```
	pub fn with_prefix(mut self, kind: MsgKind) -> Self {
		self.set_prefix(kind);
		self
	}

	#[must_use]
	/// # With Custom Prefix.
	///
	/// Set or reset the message with a user-defined prefix and color.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// assert_eq!(
	///     Msg::plain("Hello world.").with_custom_prefix("Prefix", 4),
	///     Msg::custom("Prefix", 4, "Hello world.")
	/// );
	/// ```
	pub fn with_custom_prefix<S>(mut self, prefix: S, color: u8) -> Self
	where S: AsRef<str> {
		self.set_custom_prefix(prefix, color);
		self
	}

	#[must_use]
	/// # With Message.
	///
	/// Set or reset the message portion of the message.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	///
	/// // A contrived example…
	/// let mut msg = Msg::plain("Should I say this?")
	///     .with_msg("No, this!");
	/// ```
	pub fn with_msg<S>(mut self, msg: S) -> Self
	where S: AsRef<str> {
		self.set_msg(msg);
		self
	}

	#[must_use]
	/// # With Suffix.
	///
	/// Set or reset the message suffix.
	///
	/// Note: suffixes immediately follow the message portion and should
	/// explicitly include any spaces or separators in their value.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	///
	/// // A contrived example…
	/// let mut msg = Msg::plain("5,000 matching files were found.")
	///     .with_suffix(" (75%)");
	/// ```
	pub fn with_suffix<S>(mut self, suffix: S) -> Self
	where S: AsRef<str> {
		self.set_suffix(suffix);
		self
	}
}

/// ## Setters.
impl Msg {
	/// # Set Indentation.
	///
	/// This is the setter companion to the [`Msg::with_indent`] builder method.
	/// Refer to that documentation for more information.
	pub fn set_indent(&mut self, indent: u8) {
		static SPACES: [u8; 16] = [32_u8; 16];
		self.0.replace(PART_INDENT, &SPACES[0..4.min(usize::from(indent)) << 2]);
	}

	#[cfg(feature = "timestamps")]
	/// # Set Timestamp.
	///
	/// This is the setter companion to the [`Msg::with_timestamp`] builder method.
	/// Refer to that documentation for more information.
	pub fn set_timestamp(&mut self, timestamp: bool) {
		if timestamp {
			self.0.replace(
				PART_TIMESTAMP,
				format!(
					"\x1b[2m[\x1b[0;34m{}\x1b[39;2m]\x1b[0m ",
					chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
				).as_bytes()
			);
		}
		else if 0 != self.0.len(PART_TIMESTAMP) {
			self.0.truncate(PART_TIMESTAMP, 0);
		}
	}

	/// # Set Linebreak.
	///
	/// This is the setter companion to the [`Msg::with_newline`] builder method.
	/// Refer to that documentation for more information.
	pub fn set_newline(&mut self, newline: bool) {
		if newline {
			if 0 == self.0.len(PART_NEWLINE) {
				self.0.extend(PART_NEWLINE, b"\n");
			}
		}
		else if 0 != self.0.len(PART_NEWLINE) {
			self.0.truncate(PART_NEWLINE, 0);
		}
	}

	/// # Set Prefix.
	///
	/// This is the setter companion to the [`Msg::with_prefix`] builder method.
	/// Refer to that documentation for more information.
	pub fn set_prefix(&mut self, kind: MsgKind) {
		self.0.replace(PART_PREFIX, kind.as_bytes());
	}

	/// # Set Custom Prefix.
	///
	/// This is the setter companion to the [`Msg::with_custom_prefix`] builder method.
	/// Refer to that documentation for more information.
	pub fn set_custom_prefix<S>(&mut self, prefix: S, color: u8)
	where S: AsRef<str> {
		let prefix = prefix.as_ref().as_bytes();

		if prefix.is_empty() { self.0.truncate(PART_PREFIX, 0); }
		else {
			self.0.replace(
				PART_PREFIX,
				&[
					b"\x1b[1;38;5;",
					&*NiceU8::from(color),
					b"m",
					prefix,
					b":\x1b[0m ",
				].concat(),
			);
		}
	}

	/// # Set Message.
	///
	/// This is the setter companion to the [`Msg::with_msg`] builder method.
	/// Refer to that documentation for more information.
	pub fn set_msg<S>(&mut self, msg: S)
	where S: AsRef<str> {
		self.0.replace(PART_MSG, msg.as_ref().as_bytes());
	}

	/// # Set Suffix.
	///
	/// This is the setter companion to the [`Msg::with_suffix`] builder method.
	/// Refer to that documentation for more information.
	pub fn set_suffix<S>(&mut self, suffix: S)
	where S: AsRef<str> {
		self.0.replace(PART_SUFFIX, suffix.as_ref().as_bytes());
	}
}

/// ## Conversion.
impl Msg {
	#[must_use]
	/// # As Bytes.
	///
	/// Return the entire message as a byte slice.
	pub fn as_bytes(&self) -> &[u8] { &self.0 }

	#[must_use]
	/// # As Str.
	///
	/// Return the entire message as a string slice.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.0) }
	}

	#[must_use]
	/// # Into Vec.
	///
	/// Consume the message, returning a `Vec<u8>`.
	pub fn into_vec(self) -> Vec<u8> { self.0.into_vec() }

	#[must_use]
	/// # Into String.
	///
	/// Consume the message, returning an owned string.
	pub fn into_string(self) -> String {
		unsafe { String::from_utf8_unchecked(self.0.into_vec()) }
	}

	#[cfg(feature = "fitted")]
	#[must_use]
	/// # Capped Width Slice.
	///
	/// This will return a byte string that should fit a given console width if
	/// printed. This is subject to the usual disclaimers of "Unicode is
	/// monstrously complicated…", but it does its best, and will be more
	/// accurate than simply chopping to the [`Msg::len`].
	///
	/// Only the user-defined message portion of the `Msg` will be trimmed for
	/// space. Prefixes, suffixes, the trailing newline, etc., are left
	/// unchanged.
	///
	/// If the message cannot be made to fit, an empty byte string is returned.
	pub fn fitted(&self, width: usize) -> Cow<[u8]> {
		// Quick length bypass; length will only ever be greater or equal to
		// width, so if that fits, the message fits.
		if self.len() <= width {
			return Cow::Borrowed(self);
		}

		#[cfg(feature = "timestamps")]
		// If the fixed width bits are themselves too big, we can't fit print.
		let fixed_width: usize =
			self.0.len(PART_INDENT) as usize +
			crate::width(self.0.get(PART_PREFIX)) +
			crate::width(self.0.get(PART_SUFFIX)) +
			if 0 == self.0.len(PART_TIMESTAMP) { 0 }
			else { 21 };

		#[cfg(not(feature = "timestamps"))]
		// If the fixed width bits are themselves too big, we can't fit print.
		let fixed_width: usize =
			self.0.len(PART_INDENT) as usize +
			crate::width(self.0.get(PART_PREFIX)) +
			crate::width(self.0.get(PART_SUFFIX));

		if fixed_width > width {
			return Cow::Owned(Vec::new());
		}

		// Check the length again; the fixed bits might just have a lot of
		// ANSI.
		let keep = crate::length_width(self.0.get(PART_MSG), width - fixed_width) as u32;
		if keep == 0 { Cow::Owned(Vec::new()) }
		else if keep == self.0.len(PART_MSG) { Cow::Borrowed(self) }
		else {
			// We have to trim the message to fit. Let's do it on a copy.
			let mut tmp = self.clone();
			tmp.0.truncate(PART_MSG, keep);

			// We might need to append an ANSI reset to be safe. This might be
			// unnecessary, but nitpicking is more expensive than redundancy
			// here.
			if tmp.0.get(PART_MSG).contains(&b'\x1b') {
				tmp.0.extend(PART_MSG, b"\x1b[0m");
			}

			Cow::Owned(tmp.into_vec())
		}
	}
}

/// ## Details.
impl Msg {
	#[must_use]
	/// # Length.
	///
	/// This returns the total length of the entire `Msg`, ANSI markup and all.
	pub const fn len(&self) -> usize {
		// Because the buffers used by `Msg` end on partitioned space, we can
		// infer the length given the ending index of the newline part, making
		// this method `const`.
		self.0.end(PART_NEWLINE) as usize
	}

	#[must_use]
	/// # Is Empty.
	pub const fn is_empty(&self) -> bool { self.len() == 0 }
}

/// ## Printing.
impl Msg {
	/// # Locked Print to STDOUT.
	///
	/// This is equivalent to calling either `print!()` or `println()`
	/// depending on whether or not a trailing linebreak has been set.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// Msg::plain("Hello world!").with_newline(true).print();
	/// ```
	pub fn print(&self) {
		use io::Write;

		let writer = io::stdout();
		let mut handle = writer.lock();
		let _ = handle.write_all(&self.0)
			.and_then(|_| handle.flush());
	}

	/// # Locked Print to STDERR.
	///
	/// This is equivalent to calling either `eprint!()` or `eprintln()`
	/// depending on whether or not a trailing linebreak has been set.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// Msg::error("Oh no!").with_newline(true).eprint();
	/// ```
	pub fn eprint(&self) {
		use io::Write;

		let writer = io::stderr();
		let mut handle = writer.lock();
		let _ = handle.write_all(&self.0)
			.and_then(|_| handle.flush());
	}

	/// # Print and Die.
	///
	/// This is a convenience method for printing a message to STDERR and
	/// terminating the thread with the provided exit code. Generally you'd
	/// want to pass a non-zero value here.
	///
	/// Be careful calling this method in parallel contexts as it will only
	/// stop the current thread, not the entire program execution.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// Msg::error("Oh no!").with_newline(true).die(1);
	/// ```
	pub fn die(&self, code: i32) {
		self.eprint();
		std::process::exit(code);
	}

	#[must_use]
	/// # Prompt.
	///
	/// This produces a simple y/N input prompt, requiring the user type "Y" or
	/// "N" to proceed. Positive values return `true`, negative values return
	/// `false`. The default (if the user just hits <enter>) is "N".
	///
	/// Note: the prompt normalizes the suffix and newline parts for display.
	/// If your message contains these parts, they will be ignored by the
	/// prompt action, but will be retained in the original struct should you
	/// wish to use it in some other manner later in your code.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::{Msg, MsgKind};
	///
	/// if Msg::new(MsgKind::Confirm, "Do you like chickens?").prompt() {
	///    println!("That's great! They like you too!");
	/// }
	/// ```
	pub fn prompt(&self) -> bool {
		// Clone the message and append a little [y/N] instructional bit to the
		// end.
		let q = self.clone()
			.with_suffix(" \x1b[2m[y/\x1b[4mN\x1b[0;2m]\x1b[0m ")
			.with_newline(false);

		// Ask and collect input, looping until a valid response is typed.
		let mut result = String::new();
		loop {
			q.print();

			if let Some(res) = io::stdin().read_line(&mut result)
				.ok()
				.and_then(|_| match result.to_lowercase().trim() {
					"" | "n" | "no" => Some(false),
					"y" | "yes" => Some(true),
					_ => None,
				})
			{ break res; }

			// Print an error and do it all over again.
			result.truncate(0);
			Self::error("Invalid input; enter \x1b[91mN\x1b[0m or \x1b[92mY\x1b[0m.")
				.print();
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;

	#[test]
	fn t_msg() {
		let mut msg = Msg::plain("My dear aunt sally.");
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

		msg.set_suffix(" Heyo");
		assert!(msg.ends_with(b" Heyo"), "{:?}", msg.as_str());
		msg.set_suffix("");
		assert!(msg.ends_with(b"My dear aunt sally."));

		msg.set_msg("My dear aunt");
		assert!(msg.ends_with(b"My dear aunt"));
	}

	#[cfg(feature = "fitted")]
	#[test]
	fn t_fitted() {
		let mut msg = Msg::plain("Hello World");

		assert_eq!(msg.fitted(5), &b"Hello"[..]);
		assert_eq!(msg.fitted(20), &b"Hello World"[..]);

		// Try it with a new line.
		msg.set_newline(true);
		assert_eq!(msg.fitted(5), &b"Hello\n"[..]);

		// Give it a prefix.
		msg.set_prefix(MsgKind::Error);
		assert_eq!(msg.fitted(5), Vec::<u8>::new());
		assert_eq!(msg.fitted(12), &b"\x1b[91;1mError:\x1b[0m Hello\n"[..]);

		// Colorize the message.
		msg.set_msg("\x1b[1mHello\x1b[0m World");
		assert_eq!(msg.fitted(12), &b"\x1b[91;1mError:\x1b[0m \x1b[1mHello\x1b[0m\x1b[0m\n"[..]);
		assert_eq!(msg.fitted(11), &b"\x1b[91;1mError:\x1b[0m \x1b[1mHell\x1b[0m\n"[..]);

		// Try it with Unicode!
		msg.set_msg("Björk Guðmundsdóttir");
		assert_eq!(msg.fitted(12), "\x1b[91;1mError:\x1b[0m Björk\n".as_bytes());
	}
}
