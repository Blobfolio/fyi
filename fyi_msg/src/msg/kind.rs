/*!
# FYI Msg: Kind
*/

use super::Msg;
use super::MsgPrefix;
use std::{
	fmt,
	ops::Deref,
};



#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// Message Kind.
///
/// This is the prefix, basically. `Other` owns a `MsgPrefix` — a fixed-length
/// byte string — while all other options are static built-ins, with formatting
/// and labels pre-defined.
///
/// ## Examples
///
/// These are not really useful outside the context of the [`Msg`] struct. A
/// `Msg` can be prefixed with a `MsgKind` via:
///
/// ```no_run
/// use fyi_msg::Msg;
/// use fyi_msg::MsgKind;
///
/// // As builder:
/// let msg1 = Msg::from("Hello World").with_prefix(MsgKind::Success);
///
/// // Manually.
/// let msg2 = unsafe { Msg::prefixed_unchecked(MsgKind::Success, b"Hello World") };
///
/// // Or the other way around.
/// let msg3 = MsgKind::Success.into_msg("Hello World");
/// ```
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
	Other(MsgPrefix),
}

impl Default for MsgKind {
	#[inline]
	fn default() -> Self { Self::None }
}

impl Deref for MsgKind {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
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
}

impl fmt::Display for MsgKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(std::str::from_utf8(self).map_err(|_| fmt::Error::default())?)
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

/// ## Casting.
///
/// These methods provide means of converting `MsgKind` instances into other
/// data structures.
///
/// Note: this enum can also be dereferenced to `&[u8]`.
impl MsgKind {
	#[must_use]
	#[inline]
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
	pub fn as_bytes(&self) -> &[u8] { self }

	#[must_use]
	/// # As Pointer.
	///
	/// Return a raw pointer to the underlying bytes.
	pub fn as_ptr(&self) -> *const u8 {
		if let Self::Other(x) = self { x.as_ptr() }
		else { self.as_bytes().as_ptr() }
	}

	#[must_use]
	#[inline]
	/// # As Str.
	///
	/// Return the formatted prefix as a string slice.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::MsgKind;
	/// let kind = MsgKind::new("Hello", 199).as_str();
	/// ```
	///
	/// ## Safety
	///
	/// The string's UTF-8 is not validated for sanity!
	pub unsafe fn as_str(&self) -> &str { std::str::from_utf8_unchecked(self) }

	#[must_use]
	#[inline]
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
	pub fn into_msg(self, msg: &str) -> Msg {
		unsafe { Msg::prefixed_unchecked(self, msg.as_bytes()) }
	}
}

/// ## The rest!
///
/// There really isn't a lot going on here. Haha.
impl MsgKind {
	#[must_use]
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
	pub fn new(prefix: &str, color: u8) -> Self {
		if prefix.is_empty() || prefix.len() > 45 { Self::None }
		else {
			unsafe { Self::new_unchecked(prefix.as_bytes(), color) }
		}
	}

	#[must_use]
	#[inline]
	/// # Custom Prefix (Unchecked).
	///
	/// This is the same as [`MsgKind::new`] but does not validate the inputs.
	///
	/// ## Safety
	///
	/// As this does not validate the inputs, undefined things will happen if
	/// the prefix is invalid UTF-8 or exceeds 45 bytes.
	pub unsafe fn new_unchecked(prefix: &[u8], color: u8) -> Self {
		Self::Other(MsgPrefix::new_unchecked(prefix, color))
	}

	#[must_use]
	/// # Is Empty?
	///
	/// This returns `true` if the prefix is empty. In practice, this should
	/// only match [`MsgKind::None`].
	pub const fn is_empty(&self) -> bool {
		match self {
			Self::None => true,
			Self::Other(x) => x.is_empty(),
			_ => false,
		}
	}

	#[must_use]
	/// # Length.
	///
	/// This returns the byte length for the message kind. [`MsgKind::None`] is
	/// zero-length, but the rest typically occupy a couple dozen bytes.
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
