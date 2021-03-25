/*!
# FYI Msg
*/

pub(super) mod buffer;
pub(super) mod kind;

use crate::{
	MsgKind,
	MsgBuffer,
};

#[cfg(feature = "progress")] use crate::BeforeAfter;

use dactyl::NiceU8;
use std::{
	borrow::Borrow,
	fmt,
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

/// # Helper: ToC Setup.
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

/// # Helper: Built-in `MsgKind` Checked Methods.
macro_rules! impl_builtin_checked {
	($name:expr, $ex:expr, $fn:ident, $fn2:ident) => {
		#[doc(inline)]
		#[doc = $name]
		///
		/// This is a convenience method to create a thusly prefixed message
		/// with a trailing line break.
		///
		/// This is equivalent to combining [`Msg::new`] with [`Msg::with_newline`],
		/// but optimized for this specific prefix.
		///
		/// ## Examples
		///
		/// ```no_run
		/// use fyi_msg::Msg;
		#[doc = $ex]
		/// ```
		pub fn $fn<S>(msg: S) -> Self
		where S: AsRef<str> {
			unsafe { Self::$fn2(msg.as_ref().as_bytes()) }
		}
	};
}

/// # Helper: Built-in `MsgKind` Unchecked Methods.
macro_rules! impl_builtin_unchecked {
	($name:expr, $desc:expr, $fn:ident, $kind:expr, $p_len:literal) => {
		#[must_use]
		#[doc = $name]
		///
		#[doc = $desc]
		///
		/// ## Safety
		///
		/// This method does not itself do unsafe things, however most of this
		/// struct's methods rely on the stored message being valid UTF-8.
		/// Because this method does not validate the inputs, it leaves the
		/// door open for undefined behaviors, and as such, is labeled "unsafe".
		///
		/// With that in mind, the value of `msg` must be valid UTF-8, or later
		/// use of the instance might panic or act out in undefined ways.
		pub unsafe fn $fn(msg: &[u8]) -> Self {
			let len = msg.len();
			let m_end = len as u32 + $p_len;

			let mut v: Vec<u8> = Vec::with_capacity($p_len + 1 + len);
			v.extend_from_slice($kind.as_bytes());
			v.extend_from_slice(msg);
			v.push(b'\n');

			Self(MsgBuffer::from_raw_parts(v, new_toc!($p_len, m_end, true)))
		}
	};
}

/// # Helper: Impl Built-in `MsgKind` Methods.
macro_rules! impl_builtins {
	($name:literal, $fn:ident, $fn2:ident, $kind:expr, $p_len:literal) => (
		impl_builtin_checked!(
			concat!("# New ", $name),
			concat!("Msg::", stringify!($fn), r#"("Hello World").print(); // "#, $name, ": Hello World"),
			$fn,
			$fn2
		);

		impl_builtin_unchecked!(
			concat!("# New ", $name, " (Unchecked)"),
			concat!("This is equivalent to (the safe) [`Msg::", stringify!($fn), "`], except it takes a raw byte slice."),
			$fn2,
			$kind,
			$p_len
		);
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
/// efficiently in place (per-part) and printed to `STDOUT` with [`Msg::print`]
/// or `STDERR` with [`Msg::eprint`].
///
/// Take a look at `examples/msg`, which covers just about all the standard use
/// cases.
///
/// There are two crate feature gates that augment this struct (at the expense
/// of additional dependencies):
///
/// * `fitted` adds the [`Msg::fitted`] method, which returns a byte slice that should fit within a given display width, shrinking the message part of the message as necessary to make room (leaving prefixes and suffixes in tact).
/// * `timestamps` adds [`Msg::with_timestamp`] and [`Msg::set_timestamp`] methods for adding a local datetime value before the prefix.
///
/// Everything else comes stock!
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
/// There are a bunch of built-in prefix types, each of which (except
/// [`MsgKind::Confirm`]) has a corresponding "quick" method on this struct,
/// like [`Msg::error`], [`Msg::success`], etc. See [`MsgKind`] for the full
/// list. These are equivalent to chaining [`Msg::new`] and [`Msg::with_newline`]
/// for the given type.
///
/// Confirmations have a convenience macro instead, [`confirm`](crate::confirm),
/// that handles all the setup and prompting, returning a simple `bool`
/// indicating the yes/noness of the user response.
///
/// Again, the `examples/msg` demo covers just about all the possibilities.
///
/// ## Conversion
///
/// While [`Msg`] objects are perfectly usable as-is, they can be easily
/// converted to other formats. They dereference to a byte slice and implement
/// `AsRef<[u8]>` and `Borrow<[u8]>`. They also implement `AsRef<str>` and
/// `Borrow<str>` for stringy situations. And if you want to consume the struct
/// into an owned type, there's also [`Msg::into_vec`] and [`Msg::into_string`].
pub struct Msg(MsgBuffer<MSGBUFFER>);

impl AsRef<[u8]> for Msg {
	#[inline]
	fn as_ref(&self) -> &[u8] { self.as_bytes() }
}

impl AsRef<str> for Msg {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl Borrow<[u8]> for Msg {
	#[inline]
	fn borrow(&self) -> &[u8] { self }
}

impl Borrow<str> for Msg {
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
	#[inline]
	/// # New Message.
	///
	/// This creates a new message with a built-in prefix (which can be
	/// [`MsgKind::None`](crate::MsgKind::None), though in that case, [`Msg::plain`]
	/// is better).
	///
	/// If your prefix choice is built-in and known at compile time and you
	/// want the message to have a trailing line break, consider using the
	/// corresponding dedicated method instead ([`Msg::error`], [`Msg::success`],
	/// etc.), as it will be slightly faster.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, MsgKind};
	/// let msg = Msg::new(MsgKind::Info, "This is a message.");
	/// ```
	pub fn new<S>(kind: MsgKind, msg: S) -> Self
	where S: AsRef<str> {
		unsafe { Self::new_unchecked(kind, msg.as_ref().as_bytes()) }
	}

	#[allow(clippy::cast_possible_truncation)] // MsgBuffer checks fit.
	#[must_use]
	/// # New Message (Unchecked).
	///
	/// This is just like [`Msg::new`], except the message is passed as a raw
	/// byte slice.
	///
	/// ## Safety
	///
	/// The message slice must be valid UTF-8 or undefined things will happen.
	/// When in doubt, use the string-based [`Msg::new`] instead.
	pub unsafe fn new_unchecked(kind: MsgKind, msg: &[u8]) -> Self {
		let p_end = kind.len_32();
		let m_end = p_end + msg.len() as u32;

		Self(MsgBuffer::from_raw_parts(
			[kind.as_bytes(), msg].concat(),
			new_toc!(p_end, m_end)
		))
	}

	#[allow(clippy::cast_possible_truncation)] // MsgBuffer checks fit.
	/// # Custom Prefix.
	///
	/// This creates a new message with a user-defined prefix and color. See
	/// [here](https://misc.flogisoft.com/bash/tip_colors_and_formatting) for a BASH color code primer.
	///
	/// The prefix you provide will automatically have a `": "` added to the
	/// end, so you should pass "Prefix" rather than "Prefix:".
	///
	/// If you don't like the colonics, use [`Msg::custom_preformatted`]
	/// instead.
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

	#[allow(clippy::cast_possible_truncation)] // MsgBuffer checks fit.
	/// # Custom Prefix (Pre-formatted)
	///
	/// Same as [`Msg::custom`], except no validation or formatting is applied
	/// to the provided prefix. This can be useful in cases where the prefix
	/// requires special spacing, delimiters, or formatting that don't match
	/// the default format.
	///
	/// It is worth noting that when using this method, you must provide the
	/// punctuation and space that will separate the prefix and message parts,
	/// otherwise you'll end up with "prefixmessage" glued together.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, MsgKind};
	/// let msg = Msg::custom_preformatted(
	///     "e.g. ",
	///     "This message has an unformatted prefix."
	/// );
	/// ```
	pub fn custom_preformatted<S>(prefix: S, msg: S) -> Self
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

	#[inline]
	/// # New Message Without Any Prefix.
	///
	/// This is a streamlined equivalent of calling [`Msg::new`] with a
	/// [`MsgKind::None`].
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Msg;
	/// let msg = Msg::plain("This message has no prefix.");
	/// ```
	pub fn plain<S>(msg: S) -> Self
	where S: AsRef<str> {
		unsafe { Self::plain_unchecked(msg.as_ref().as_bytes()) }
	}

	#[allow(clippy::cast_possible_truncation)] // MsgBuffer checks fit.
	#[must_use]
	/// # New Message Without Any Prefix (Unchecked).
	///
	/// This is just like [`Msg::plain`], except the message is passed as a raw
	/// byte slice.
	///
	/// ## Safety
	///
	/// The message slice must be valid UTF-8 or undefined things will happen.
	/// When in doubt, use the string-based [`Msg::plain`] instead.
	pub unsafe fn plain_unchecked(msg: &[u8]) -> Self {
		let len = msg.len() as u32;

		Self(MsgBuffer::from_raw_parts(
			msg.to_vec(),
			new_toc!(0, len)
		))
	}
}

/// # Built-ins.
///
/// This contains convenience methods for creating a new message with a
/// built-in prefix and trailing line break. All of the stock kinds are covered
/// except for [`MsgKind::Confirm`], which does not have trailing line breaks
/// in its prompt form, and is kind of weird to use without a prompt.
///
/// Speaking of, there is a dedicated [`confirm`](crate::confirm) macro, that
/// renders the message with the right prefix, pops the prompt, and returns the
/// `bool`.
impl Msg {
	impl_builtins!("Crunched", crunched, crunched_unchecked, MsgKind::Crunched, 21);
	impl_builtins!("Debug", debug, debug_unchecked, MsgKind::Debug, 18);
	impl_builtins!("Done", done, done_unchecked, MsgKind::Done, 17);
	impl_builtins!("Info", info, info_unchecked, MsgKind::Info, 17);
	impl_builtins!("Error", error, error_unchecked, MsgKind::Error, 18);
	impl_builtins!("Notice", notice, notice_unchecked, MsgKind::Notice, 19);
	impl_builtins!("Success", success, success_unchecked, MsgKind::Success, 20);
	impl_builtins!("Task", task, task_unchecked, MsgKind::Task, 23);
	impl_builtins!("Warning", warning, warning_unchecked, MsgKind::Warning, 20);
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
	/// There are 2-3 flags total (depending on whether or not the `timestamps`
	/// feature has been enabled):
	///
	/// * [`FLAG_INDENT`] indents the message one level (four spaces);
	/// * [`FLAG_NEWLINE`] adds a trailing line break to the end;
	/// * [`FLAG_TIMESTAMP`] adds a `[YYYY-MM-DD HH:MM:SS]`-style timestamp between the indentation and prefix;
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
	/// indentation, pass `0`. Large values are capped at a maximum of `4`
	/// levels of indentation.
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
	/// **This requires the `timestamps` crate feature.**
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
	/// Unlike prefixes, there are no built-in suffixes, and as such, no
	/// assumptions or automatic formatting is applied. The value you set must
	/// include any spacing, delimiters, and formatting needed to have it look
	/// right. Generally you'll want to at least have a leading space,
	/// otherwise you'll get "messagesuffix" all glued together.
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
	/// This is the setter companion to the [`Msg::with_indent`] builder
	/// method. Refer to that documentation for more information.
	pub fn set_indent(&mut self, indent: u8) {
		static SPACES: [u8; 16] = [32_u8; 16];
		self.0.replace(PART_INDENT, &SPACES[0..4.min(usize::from(indent)) << 2]);
	}

	#[cfg(feature = "timestamps")]
	#[allow(clippy::cast_possible_truncation)] // Date pieces have known values.
	#[allow(clippy::cast_sign_loss)] // Date pieces have known values.
	/// # Set Timestamp.
	///
	/// This is the setter companion to the [`Msg::with_timestamp`] builder
	/// method. Refer to that documentation for more information.
	///
	/// **This requires the `timestamps` crate feature.**
	pub fn set_timestamp(&mut self, timestamp: bool) {
		use chrono::{Datelike, Local, Timelike};

		if timestamp {
			let now = Local::now();
			let (y1, y2) = num_integer::div_mod_floor(now.year() as u16, 100);

			// Running each datetime part through `NiceU8` looks a bit
			// terrible, but is roughly twice as fast as issuing a single call
			// to `DateTime::<Local>::format`, and shaves about 30KiB off FYI's
			// binary size.
			self.0.replace(
				PART_TIMESTAMP,
				&[
					b"\x1b[2m[\x1b[0;34m",
					NiceU8::from(y1 as u8).as_bytes2(),
					NiceU8::from(y2 as u8).as_bytes2(),
					b"-",
					NiceU8::from(now.month() as u8).as_bytes2(),
					b"-",
					NiceU8::from(now.day() as u8).as_bytes2(),
					b" ",
					NiceU8::from(now.hour() as u8).as_bytes2(),
					b":",
					NiceU8::from(now.minute() as u8).as_bytes2(),
					b":",
					NiceU8::from(now.second() as u8).as_bytes2(),
					b"\x1b[39;2m]\x1b[0m ",
				].concat()
			);
		}
		else if 0 != self.0.len(PART_TIMESTAMP) {
			self.0.truncate(PART_TIMESTAMP, 0);
		}
	}

	/// # Set Linebreak.
	///
	/// This is the setter companion to the [`Msg::with_newline`] builder
	/// method. Refer to that documentation for more information.
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

	#[inline]
	/// # Set Prefix.
	///
	/// This is the setter companion to the [`Msg::with_prefix`] builder
	/// method. Refer to that documentation for more information.
	pub fn set_prefix(&mut self, kind: MsgKind) {
		self.0.replace(PART_PREFIX, kind.as_bytes());
	}

	/// # Set Custom Prefix.
	///
	/// This is the setter companion to the [`Msg::with_custom_prefix`] builder
	/// method. Refer to that documentation for more information.
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

	#[inline]
	/// # Set Message.
	///
	/// This is the setter companion to the [`Msg::with_msg`] builder method.
	/// Refer to that documentation for more information.
	pub fn set_msg<S>(&mut self, msg: S)
	where S: AsRef<str> {
		self.0.replace(PART_MSG, msg.as_ref().as_bytes());
	}

	#[inline]
	/// # Set Suffix.
	///
	/// This is the setter companion to the [`Msg::with_suffix`] builder
	/// method. Refer to that documentation for more information.
	pub fn set_suffix<S>(&mut self, suffix: S)
	where S: AsRef<str> {
		self.0.replace(PART_SUFFIX, suffix.as_ref().as_bytes());
	}
}

#[cfg(feature = "progress")]
/// ## Bytes Saved Suffix.
///
/// A lot of our own programs crunch data and report the savings as a suffix.
/// This section just adds a quick helper for that.
impl Msg {
	#[must_use]
	/// # Bytes Saved Suffix.
	///
	/// A lot of our own programs using this lib crunch data and report the
	/// savings as a suffix. This method just provides a quick way to generate
	/// that.
	pub fn with_bytes_saved(mut self, state: BeforeAfter) -> Self {
		use dactyl::{NicePercent, NiceU64};

		if let Some(saved) = state.less() {
			self.0.replace(
				PART_SUFFIX,
				&state.less_percent().map_or_else(
					|| [
						&b" \x1b[2m(Saved "[..],
						NiceU64::from(saved).as_bytes(),
						b" bytes.)\x1b[0m",
					].concat(),
					|percent| [
						&b" \x1b[2m(Saved "[..],
						NiceU64::from(saved).as_bytes(),
						b" bytes, ",
						NicePercent::from(percent).as_bytes(),
						b".)\x1b[0m",
					].concat()
				)
			);
		}
		else {
			self.0.replace(PART_SUFFIX, b" \x1b[2m(No savings.)\x1b[0m");
		}

		self
	}
}

/// ## Conversion.
impl Msg {
	#[must_use]
	#[inline]
	/// # As Bytes.
	///
	/// Return the entire message as a byte slice. Alternatively, you could
	/// dereference the struct or use [`Msg::as_ref`] or [`Msg::borrow`].
	pub fn as_bytes(&self) -> &[u8] { &self.0 }

	#[must_use]
	#[inline]
	/// # As Str.
	///
	/// Return the entire message as a string slice. Alternatively, you could
	/// use [`Msg::as_ref`] or [`Msg::borrow`].
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.0) }
	}

	#[must_use]
	#[inline]
	/// # Into Vec.
	///
	/// Consume the message, returning an owned `Vec<u8>`.
	pub fn into_vec(self) -> Vec<u8> { self.0.into_vec() }

	#[must_use]
	#[inline]
	/// # Into String.
	///
	/// Consume the message, returning an owned string.
	pub fn into_string(self) -> String {
		unsafe { String::from_utf8_unchecked(self.0.into_vec()) }
	}

	#[cfg(feature = "fitted")]
	#[allow(clippy::cast_possible_truncation)] // MsgBuffer checks fit.
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
	///
	/// **This requires the `fitted` crate feature.**
	pub fn fitted(&self, width: usize) -> Cow<[u8]> {
		// Quick length bypass; length will only ever be greater or equal to
		// width, so if that fits, the message fits.
		if self.len() <= width {
			return Cow::Borrowed(self);
		}

		// If the fixed width bits are themselves too big, we can't fit print.
		#[cfg(feature = "timestamps")]
		let fixed_width: usize =
			self.0.len(PART_INDENT) as usize +
			crate::width(self.0.get(PART_PREFIX)) +
			crate::width(self.0.get(PART_SUFFIX)) +
			if 0 == self.0.len(PART_TIMESTAMP) { 0 }
			else { 21 };

		#[cfg(not(feature = "timestamps"))]
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
	#[inline]
	/// # Length.
	///
	/// This returns the total length of the entire `Msg`, ANSI markup and all.
	pub const fn len(&self) -> usize {
		// Because the buffers used by `Msg` end on partitioned space, the end
		// of the last part is equal to the total length. Let's use that method
		// since it is constant!
		self.0.end(PART_NEWLINE) as usize
	}

	#[must_use]
	#[inline]
	/// # Is Empty.
	pub const fn is_empty(&self) -> bool { self.len() == 0 }
}

/// ## Printing.
impl Msg {
	/// # Locked Print to `STDOUT`.
	///
	/// This is equivalent to calling either `print!()` or `println()`
	/// depending on whether or not a trailing linebreak has been set.
	///
	/// In fact, [`Msg`] does implement `Display`, so you could do just that,
	/// but this method avoids the allocation penalty.
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
		let _res = handle.write_all(&self.0).and_then(|_| handle.flush());
	}

	/// # Locked Print to `STDERR`.
	///
	/// This is equivalent to calling either `eprint!()` or `eprintln()`
	/// depending on whether or not a trailing linebreak has been set.
	///
	/// In fact, [`Msg`] does implement `Display`, so you could do just that,
	/// but this method avoids the allocation penalty.
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
		let _res = handle.write_all(&self.0).and_then(|_| handle.flush());
	}

	/// # Print and Die.
	///
	/// This is a convenience method for printing a message to `STDERR` and
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
	/// unreachable!();
	/// ```
	pub fn die(&self, code: i32) -> ! {
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
	/// Every example in the docs shows this in combination with the built-in
	/// [`MsgKind::Confirm`] prefix, but this can be called on any [`Msg`]
	/// object. The main thing worth noting is the suffix portion is
	/// overridden for display, so don't bother putting anything there.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::{confirm, Msg, MsgKind};
	///
	/// // The manual way:
	/// if Msg::new(MsgKind::Confirm, "Do you like chickens?").prompt() {
	///     println!("That's great! They like you too!");
	/// }
	///
	/// // The macro way:
	/// if confirm!("Do you like chickens?") {
	///     println!("That's great! They like you too!");
	/// }
	/// ```
	pub fn prompt(&self) -> bool {
		// Clone the message and append a little [y/N] instructional bit to the
		// end. This might not be necessary, but preserves the original message
		// in case it is needed again.
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
	use rayon as _;

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
