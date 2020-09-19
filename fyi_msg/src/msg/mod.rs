/*!
# FYI Msg
*/

mod buffer;
mod kind;
mod prefix;

// These *are* re-exported and fully reachable. Haha.
#[allow(unreachable_pub)] pub use buffer::MsgBuffer;
#[allow(unreachable_pub)] pub use kind::MsgKind;
#[allow(unreachable_pub)] pub use prefix::MsgPrefix;

use crate::{
	traits::FastConcat,
	utility,
};
use std::{
	fmt,
	iter::FromIterator,
	mem,
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
pub struct Msg(MsgBuffer);



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

impl FromIterator<u8> for Msg {
	#[inline]
	fn from_iter<I: IntoIterator<Item=u8>>(iter: I) -> Self {
		Self::from(iter.into_iter().collect::<Vec<u8>>())
	}
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

				unsafe {
					Self(MsgBuffer::from_raw_parts(
						src,
						0_u16, 0_u16, // Indentation.
						0_u16, 0_u16, // Timestamp.
						0_u16, 0_u16, // Prefix.
						0_u16, end,   // Message.
						end, end,     // Suffix.
						// Unused...
						end, end, end, end, end, end, end, end, end,
						end, end, end, end, end, end, end, end, end,
						end, end, end, end
					))
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
		unsafe {
			Self(MsgBuffer::from_raw_parts(
				src,
				0_u16, 0_u16, // Indentation.
				0_u16, 0_u16, // Timestamp.
				0_u16, 0_u16, // Prefix.
				0_u16, end,   // Message.
				end, end,     // Suffix.
				// Unused...
				end, end, end, end, end, end, end, end, end,
				end, end, end, end, end, end, end, end, end,
				end, end, end, end
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

		let p_len: u16 = p_len as u16;
		let end: u16 = m_len as u16 + p_len;
		Self(MsgBuffer::from_raw_parts(
			buf,
			0_u16, 0_u16, // Indentation.
			0_u16, 0_u16, // Timestamp.
			0_u16, p_len, // Prefix.
			p_len, end,   // Message.
			end, end,     // Suffix.
			// Unused...
			end, end, end, end, end, end, end, end, end,
			end, end, end, end, end, end, end, end, end,
			end, end, end, end
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
		static INDENT: [u8; 16] = *b"                ";

		unsafe {
			self.0.replace_unchecked(
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
					let mut buf = [mem::MaybeUninit::<u8>::uninit(); 44];
					let dst = buf.as_mut_ptr() as *mut u8;

					// Write the opener.
					ptr::copy_nonoverlapping(b"\x1b[2m[\x1b[0;34m".as_ptr(), dst, 12);
					// Write the space.
					ptr::write(dst.add(22), b' ');
					// Write the closer.
					ptr::copy_nonoverlapping(b"\x1b[39;2m]\x1b[0m ".as_ptr(), dst.add(31), 13);

					// Now the time bits.
					{
						use chrono::{Datelike, Local, Timelike};
						let now = Local::now();

						utility::write_date(
							dst.add(12),
							(now.year() as u16).saturating_sub(2000) as u8,
							now.month() as u8,
							now.day() as u8
						);
						utility::write_time(
							dst.add(23),
							now.hour() as u8,
							now.minute() as u8,
							now.second() as u8,
						);
					}

					// Shove the result into the buffer.
					self.0.replace_unchecked(
						PART_TIMESTAMP,
						&mem::transmute::<_, [u8; 44]>(buf),
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
			handle.write_all(&self.0).unwrap();
			handle.write_all(b"\n").unwrap();
			handle.flush().unwrap();
		}
	};

	($fn:ident, $writer:ident, false) => {
		/// # Print Helper.
		pub fn $fn(&self) {
			let writer = io::$writer();
			let mut handle = writer.lock();
			handle.write_all(&self.0).unwrap();
			handle.flush().unwrap();
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

	locked_print!(print, stdout, false);
	locked_print!(println, stdout, true);
	locked_print!(eprint, stderr, false);
	locked_print!(eprintln, stderr, true);
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
