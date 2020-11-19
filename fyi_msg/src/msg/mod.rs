/*!
# FYI Msg
*/

mod buffer;
mod kind;
mod prefix;

// These *are* re-exported and fully reachable. Haha.
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
#[allow(unreachable_pub)] pub use kind::MsgKind;
#[allow(unreachable_pub)] pub use prefix::MsgPrefix;

use std::{
	fmt,
	iter::FromIterator,
	ops::Deref,
	ptr,
	io::{
		self,
		Write,
	},
};



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

// Configuration Flags.
//
// These flags are an alternative way to configure indentation and
// timestamping.

/// Enable Indentation (equivalent to 4 spaces).
pub const FLAG_INDENT: u8 =    0b0001;

/// Enable Timestamp.
pub const FLAG_TIMESTAMP: u8 = 0b0010;



#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
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
pub struct Msg(MsgBuffer5);



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
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}

impl From<&str> for Msg {
	#[inline]
	fn from(src: &str) -> Self { Self::from(src.as_bytes().to_vec()) }
}

impl From<String> for Msg {
	#[inline]
	fn from(src: String) -> Self { Self::from(src.into_bytes()) }
}

impl From<&[u8]> for Msg {
	#[inline]
	fn from(src: &[u8]) -> Self { Self::from(src.to_vec()) }
}

/// # Helper: From Concatable.
///
/// This lets messages be built from a slice of slices. It just concatenates
/// them into a single byte stream that [`Msg`] can use, saving the
/// implementing library from running `.concat()` themselves.
macro_rules! from_concat_slice {
	($size:literal) => {
		impl From<[&[u8]; $size]> for Msg {
			#[inline]
			fn from(src: [&[u8]; $size]) -> Self { Self::from(src.concat()) }
		}
	};
}

from_concat_slice!(1);
from_concat_slice!(2);
from_concat_slice!(3);
from_concat_slice!(4);
from_concat_slice!(5);
from_concat_slice!(6);
from_concat_slice!(7);
from_concat_slice!(8);

impl FromIterator<u8> for Msg {
	#[inline]
	fn from_iter<I: IntoIterator<Item=u8>>(iter: I) -> Self {
		Self::from(iter.into_iter().collect::<Vec<u8>>())
	}
}

impl From<Vec<u8>> for Msg {
	fn from(src: Vec<u8>) -> Self {
		let end: usize = src.len();
		unsafe {
			Self(MsgBuffer5::from_raw_parts(
				src,
				[
					0, 0,     // Indentation.
					0, 0,     // Timestamp.
					0, 0,     // Prefix.
					0, end,   // Message.
					end, end, // Suffix.
				]
			))
		}
	}
}

/// # Instantiation and Builder Bits.
///
/// These methods cover instantiation and setup of `Msg` objects.
impl Msg {
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
			ptr::copy_nonoverlapping(prefix.as_ptr(), ptr, p_len);
			ptr::copy_nonoverlapping(msg.as_ptr(), ptr.add(p_len), m_len);
			buf.set_len(m_len + p_len);
		}

		let end: usize = m_len + p_len;
		Self(MsgBuffer5::from_raw_parts(
			buf,
			[
				0, 0,       // Indentation.
				0, 0,       // Timestamp.
				0, p_len,   // Prefix.
				p_len, end, // Message.
				end, end,   // Suffix.
			]
		))
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
		self.set_indent((0 != flags & FLAG_INDENT) as u8);
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
}

/// ## Casting.
///
/// These methods provide means of converting `Msg` instances into other data
/// structures.
///
/// Note: this struct can also be dereferenced to `&[u8]`.
impl Msg {
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
	pub fn as_bytes(&self) -> &[u8] { self }

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
	pub fn as_str(&self) -> &str { unsafe { self.0.as_str() } }

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
	pub fn into_vec(self) -> Vec<u8> { self.0.into_vec() }
}

/// ## Setters.
///
/// While `Msg` is primarily intended to be managed via builder patterns, there
/// are corresponding `set_*()` methods to work on stored mutable instances.
impl Msg {
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
		unsafe {
			self.0.replace_unchecked(
				PART_INDENT,
				&b" ".repeat(4.min(indent as usize) * 4),
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
	pub fn set_msg(&mut self, msg: &str) {
		unsafe { self.set_msg_unchecked(msg.as_bytes()) }
	}

	#[inline]
	/// # Set Message (Unchecked).
	///
	/// Set or reset the message body.
	///
	/// ## Safety
	///
	/// The message must be valid UTF-8 or undefined things will happen.
	pub unsafe fn set_msg_unchecked(&mut self, msg: &[u8]) {
		self.0.replace_unchecked(PART_MSG, msg);
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
			self.0.replace_unchecked(PART_PREFIX, &prefix);
		}
	}

	/// # Set Suffix (Unchecked)
	///
	/// This method sets the suffix exactly as specified. It should have a
	/// leading space, and should probably reset ANSI formatting at the end.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// use fyi_msg::MsgKind;
	///
	/// let mut msg = Msg::new("Hello world.");
	/// unsafe { msg.set_suffix_unchecked(b" (100%)"); }
	/// ```
	///
	/// ## Safety
	///
	/// This method is "unsafe" insofar as the data is accepted without any
	/// checks or manipulation.
	pub unsafe fn set_suffix_unchecked(&mut self, suffix: &[u8]) {
		self.0.replace_unchecked(PART_SUFFIX, suffix);
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
		unsafe {
			if on == self.0.is_empty_unchecked(PART_TIMESTAMP) {
				if on {
					// Shove the result into the buffer.
					self.0.replace_unchecked(
						PART_TIMESTAMP,
						format!(
							"\x1b[2m[\x1b[0;34m{}\x1b[39;2m]\x1b[0m ",
							chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
						).as_bytes()
					);
				}
				else {
					self.0.zero_unchecked(PART_TIMESTAMP);
				}
			}
		}
	}
}

/// Helper: Printing.
macro_rules! locked_print {
	($fn:ident, $writer:ident, true) => {
		/// # Print Helper (w/ trailing line break).
		pub fn $fn(&self) {
			let writer = io::$writer();
			let mut handle = writer.lock();
			let _ = handle.write_all(&self.0)
				.and_then(|_| handle.write_all(b"\n"))
				.and_then(|_| handle.flush());
		}
	};

	($fn:ident, $writer:ident, false) => {
		/// # Print Helper.
		pub fn $fn(&self) {
			let writer = io::$writer();
			let mut handle = writer.lock();
			let _ = handle.write_all(&self.0).and_then(|_| handle.flush());
		}
	};
}

/// ## Printing.
///
/// These methods provide means of kicking `Msg` content to the terminal.
///
/// The [`Msg::print`], [`Msg::println`], [`Msg::eprint`], and [`Msg::eprintln`] methods
/// work more or less like the Rust macros sharing their names, except the
/// writer is locked and flushed to ensure every byte actually makes it out.
impl Msg {
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
			unsafe {
				Self::prefixed_unchecked(
					MsgKind::Error,
					b"Invalid input: enter \x1b[91mN\x1b[0m or \x1b[92mY\x1b[0m."
				).println();
			}
		}
	}

	locked_print!(print, stdout, false);
	locked_print!(println, stdout, true);
	locked_print!(eprint, stderr, false);
	locked_print!(eprintln, stderr, true);
}



#[cfg(test)]
mod tests {
	use super::*;
	use criterion as _;

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
