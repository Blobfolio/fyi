/*!
# FYI Msg
*/

mod buffer;
mod kind;

use fyi_num::NiceANSI;
use std::{
	borrow::Cow,
	fmt,
	hash,
	io,
	ops::Deref,
};

#[allow(unreachable_pub)]
pub use buffer::{
	MsgBuffer2,
	MsgBuffer3,
	MsgBuffer4,
	MsgBuffer5,
	MsgBuffer6,
	MsgBuffer7,
	MsgBuffer8,
	MsgBuffer9,
	MsgBuffer10,
};

#[allow(unreachable_pub)]
pub use kind::MsgKind;



// Buffer Indexes.

/// Buffer Index: Indentation.
const PART_INDENT: usize = 0;

/// Buffer Index: Timestamp.
const PART_TIMESTAMP: usize = 1;

/// Buffer Index: Prefix.
const PART_PREFIX: usize = 2;

/// Buffer Index: Message body.
const PART_MSG: usize = 3;

/// Buffer Index: Suffix.
const PART_SUFFIX: usize = 4;

/// Buffer Index: Newline.
const PART_NEWLINE: usize = 5;

// Configuration Flags.
//
// These flags are an alternative way to configure indentation and
// timestamping.

/// Enable Indentation (equivalent to 4 spaces).
pub const FLAG_INDENT: u8 =    0b0001;

/// Enable Timestamp.
pub const FLAG_TIMESTAMP: u8 = 0b0010;

/// Enable Trailing Line.
pub const FLAG_NEWLINE: u8 =   0b0100;



#[derive(Debug, Default, Clone)]
/// # Message.
pub struct Msg(MsgBuffer6);

impl AsRef<str> for Msg {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl Deref for Msg {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl fmt::Display for Msg {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<&str> for Msg {
	fn from(src: &str) -> Self { Self::plain(src) }
}

impl From<String> for Msg {
	fn from(src: String) -> Self { Self::plain(src) }
}

impl Eq for Msg {}

impl hash::Hash for Msg {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl PartialEq for Msg {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
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
	/// [`MsgKind::None`], though in that case, [`Msg::plain`] is better).
	pub fn new<S>(kind: MsgKind, msg: S) -> Self
	where S: AsRef<str> {
		let msg = msg.as_ref().as_bytes();
		let p_end = kind.len();
		let m_end = p_end + msg.len();
		Self(MsgBuffer6::from_raw_parts(
			[kind.as_bytes(), msg].concat(),
			[
				0, 0,         // Indentation.
				0, 0,         // Timestamp.
				0, p_end,     // Prefix.
				p_end, m_end, // Message.
				m_end, m_end, // Suffix.
				m_end, m_end, // Newline.
			]
		))
	}

	/// # Custom Prefix.
	///
	/// This creates a new message with a user-defined prefix and color. See
	/// [here](https://misc.flogisoft.com/bash/tip_colors_and_formatting) for a BASH color code primer.
	pub fn custom<S>(prefix: S, color: u8, msg: S) -> Self
	where S: AsRef<str> {
		let prefix = prefix.as_ref().as_bytes();
		if prefix.is_empty() {
			return Self::plain(msg);
		}

		// Start a vector with the prefix bits.
		let msg = msg.as_ref().as_bytes();
		let v = [
			NiceANSI::from(color).as_bytes(),
			prefix,
			b":\x1b[0m ",
			msg,
		].concat();

		let m_end = v.len();
		let p_end = m_end - msg.len();

		Self(MsgBuffer6::from_raw_parts(
			v,
			[
				0, 0,         // Indentation.
				0, 0,         // Timestamp.
				0, p_end,     // Prefix.
				p_end, m_end, // Message.
				m_end, m_end, // Suffix.
				m_end, m_end, // Newline.
			]
		))
	}

	/// # Custom Prefix (Unchecked)
	///
	/// Same as [`Msg::custom`], except no validation or formatting is applied
	/// to the provided prefix. This can be useful in cases where the prefix
	/// requires special spacing or delimiters.
	pub fn custom_unchecked<S>(prefix: S, msg: S) -> Self
	where S: AsRef<str> {
		let prefix = prefix.as_ref().as_bytes();
		let msg = msg.as_ref().as_bytes();

		let p_end = prefix.len();
		let m_end = p_end + msg.len();

		Self(MsgBuffer6::from_raw_parts(
			[prefix, msg].concat(),
			[
				0, 0,         // Indentation.
				0, 0,         // Timestamp.
				0, p_end,     // Prefix.
				p_end, m_end, // Message.
				m_end, m_end, // Suffix.
				m_end, m_end, // Newline.
			]
		))
	}

	/// # New Message Without Prefix.
	///
	/// This is equivalent to [`Msg::new`] with [`MsgKind::None`], but
	/// streamlined.
	pub fn plain<S>(msg: S) -> Self
	where S: AsRef<str> {
		let msg = msg.as_ref().as_bytes();
		let len = msg.len();
		Self(MsgBuffer6::from_raw_parts(
			msg.to_vec(),
			[
				0, 0,     // Indentation.
				0, 0,     // Timestamp.
				0, 0,     // Prefix.
				0, len,   // Message.
				len, len, // Suffix.
				len, len, // Newline.
			]
		))
	}

	/// # Error
	///
	/// This is a convenience method for quickly creating a new error with a
	/// terminating line break. After creation, it is a normal [`Msg`] that can
	/// be manipulated in the usual ways.
	pub fn error<S>(msg: S) -> Self
	where S: AsRef<str> {
		let msg = msg.as_ref().as_bytes();
		let len = msg.len();
		let mut v = Vec::with_capacity(19 + len);
		v.extend_from_slice(MsgKind::Error.as_bytes());
		v.extend_from_slice(msg);
		v.extend_from_slice(b"\n");

		let m_end = len + 18;
		Self(MsgBuffer6::from_raw_parts(
			v,
			[
				0, 0,             // Indentation.
				0, 0,             // Timestamp.
				0, 18,            // Prefix.
				18, m_end,        // Message.
				m_end, m_end,     // Suffix.
				m_end, m_end + 1, // Newline.
			]
		))
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
	pub fn with_flags(mut self, flags: u8) -> Self {
		if 0 != flags & FLAG_INDENT {
			self.set_indent(1);
		}
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
	pub fn with_indent(mut self, indent: u8) -> Self {
		self.set_indent(indent);
		self
	}

	#[must_use]
	/// # With Timestamp.
	///
	/// Disable, enable, and/or recalculate the timestamp prefix for the
	/// message.
	pub fn with_timestamp(mut self, timestamp: bool) -> Self {
		self.set_timestamp(timestamp);
		self
	}

	#[must_use]
	/// # With Linebreak.
	///
	/// Add a trailing linebreak to the end of the message. This is either one
	/// or none; calling it multiple times won't add more lines.
	pub fn with_newline(mut self, newline: bool) -> Self {
		self.set_newline(newline);
		self
	}

	#[must_use]
	/// # With Prefix.
	///
	/// Set or reset the message prefix.
	pub fn with_prefix(mut self, kind: MsgKind) -> Self {
		self.set_prefix(kind);
		self
	}

	#[must_use]
	/// # With Prefix.
	///
	/// Set or reset the message with a user-defined prefix and color.
	pub fn with_custom_prefix<S>(mut self, prefix: S, color: u8) -> Self
	where S: AsRef<str> {
		self.set_custom_prefix(prefix, color);
		self
	}

	#[must_use]
	/// # With Message.
	///
	/// Set or reset the message portion of the message.
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
	pub fn with_suffix<S>(mut self, suffix: S) -> Self
	where S: AsRef<str> {
		self.set_suffix(suffix);
		self
	}
}

/// ## Setters.
impl Msg {
	/// # Set Indentation.
	pub fn set_indent(&mut self, indent: u8) {
		self.0.replace(PART_INDENT, &b" ".repeat(4.min(indent as usize) * 4));
	}

	/// # Set Timestamp.
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
	pub fn set_prefix(&mut self, kind: MsgKind) {
		self.0.replace(PART_PREFIX, kind.as_bytes());
	}

	/// # Set Custom Prefix.
	pub fn set_custom_prefix<S>(&mut self, prefix: S, color: u8)
	where S: AsRef<str> {
		let prefix = prefix.as_ref().as_bytes();

		if prefix.is_empty() { self.0.truncate(PART_PREFIX, 0); }
		else {
			self.0.replace(
				PART_PREFIX,
				&[
					NiceANSI::from(color).as_bytes(),
					prefix,
					b":\x1b[0m ",
				].concat(),
			);
		}
	}

	/// # Set Message.
	pub fn set_msg<S>(&mut self, msg: S)
	where S: AsRef<str> {
		self.0.replace(PART_MSG, msg.as_ref().as_bytes());
	}

	/// # Set Suffix.
	pub fn set_suffix<S>(&mut self, suffix: S)
	where S: AsRef<str> {
		self.0.replace(PART_SUFFIX, suffix.as_ref().as_bytes());
	}
}

/// ## Conversion.
impl Msg {
	#[must_use]
	/// # As Bytes.
	pub fn as_bytes(&self) -> &[u8] { &self.0 }

	#[must_use]
	/// # As Str.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.0) }
	}

	#[must_use]
	/// # Into Vec.
	pub fn into_vec(self) -> Vec<u8> { self.0.into_vec() }

	#[must_use]
	/// # Into String.
	pub fn into_string(self) -> String {
		unsafe { String::from_utf8_unchecked(self.0.into_vec()) }
	}

	#[cfg(feature = "fitted")]
	#[must_use]
	/// # Capped Width Slice.
	///
	/// This will return a byte string that should fit a given console width if
	/// printed. This is subject to the usual disclaimers of "Unicode is
	/// monstrous and unpredictable", but it does its best, and will be more
	/// accurate than simply chopping to `len()`.
	///
	/// Space will be trimmed from the message portion as needed, leaving
	/// prefixes, suffixes, and other parts unchanged.
	///
	/// If the message cannot be made to fit, an empty byte string is returned.
	pub fn fitted(&self, width: usize) -> Cow<[u8]> {
		// Quick length bypass; length will only ever be greater or equal to
		// width, so if that fits, the message fits.
		if self.len() <= width {
			return Cow::Borrowed(self);
		}

		// Count up the actual width to see if it fits.
		let (total_width, msg_width) = self.width();
		if total_width <= width {
			return Cow::Borrowed(self);
		}

		// Only the `PART_MSG` gets trimmed; it has to be long enough to make
		// the difference or we'll return an empty slice.
		let trim = total_width - width;
		if msg_width < trim {
			return Cow::Owned(Vec::new());
		}

		// Find out how much of the string can be made to fit.
		let keep = unsafe { self.0.fitted(PART_MSG, msg_width - trim) };
		if keep == 0 {
			Cow::Owned(Vec::new())
		}
		else {
			// We have to trim the message to fit. Let's do it on a copy.
			let mut tmp = self.clone();
			tmp.0.truncate(PART_MSG, keep);

			// We might need to append an ANSI reset to be safe.
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
	/// The buffers don't necessarily end on partitioned space, but they do for
	/// [`Msg`] structs, so we can make this `const` by inferring the length
	/// from the end of the last part.
	pub const fn len(&self) -> usize { self.0.end(PART_NEWLINE) }

	#[must_use]
	/// # Is Empty.
	pub const fn is_empty(&self) -> bool { self.len() == 0 }

	#[cfg(feature = "fitted")]
	/// # Message width.
	///
	/// This returns a tuple containing the total width as well as the width
	/// of the message part.
	///
	/// This implementation takes various shortcuts given the nature of the
	/// struct that would not necessarily work for all buffers. For example,
	/// indentation is always ASCII, so length is equivalent to width, and
	/// timestamps always have 21 printable characters.
	fn width(&self) -> (usize, usize) {
		unsafe {
			let msg_width = self.0.width(PART_MSG);
			let mut total =
				self.0.len(PART_INDENT) +
				self.0.width(PART_PREFIX) +
				self.0.width(PART_SUFFIX) +
				msg_width;

			// If present, the printable bits are always [YYYY-MM-DD HH:MM:SS].
			if 0 != self.0.len(PART_TIMESTAMP) {
				total += 21;
			}

			(total, msg_width)
		}
	}
}

/// ## Printing.
impl Msg {
	/// # Locked Print to STDOUT.
	///
	/// This is equivalent to calling either `print!()` or `println()`
	/// depending on whether or not a trailing linebreak has been set.
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
			Self::new(
				MsgKind::Error,
				"Invalid input: enter \x1b[91mN\x1b[0m or \x1b[92mY\x1b[0m."
			)
				.with_flags(FLAG_NEWLINE)
				.print();
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use criterion as _;

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
	}
}
