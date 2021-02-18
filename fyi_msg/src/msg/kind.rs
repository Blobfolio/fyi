use super::Msg;
use std::{
	fmt,
	ops::Deref,
};



#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
/// # Message Kind.
///
/// This enum contains built-in prefixes for [`Msg`](crate::Msg). These are
/// generally only used to initiate a new message with this prefix, like:
///
/// ## Examples
///
/// ```no_run
/// use fyi_msg::{Msg, MsgKind};
///
/// assert_eq!(
///     Msg::new(MsgKind::Error, "Oh no!"),
///     MsgKind::Error.into_msg("Oh no!")
/// );
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

	#[cfg(feature = "bin_kinds")] Blank,
	#[cfg(feature = "bin_kinds")] Custom,
}

impl AsRef<str> for MsgKind {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl Default for MsgKind {
	#[inline]
	fn default() -> Self { Self::None }
}

impl Deref for MsgKind {
	type Target = [u8];
	fn deref(&self) -> &Self::Target { self.as_bytes() }
}

impl fmt::Display for MsgKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<&[u8]> for MsgKind {
	fn from(txt: &[u8]) -> Self {
		match txt {
			b"confirm" | b"prompt" => Self::Confirm,
			b"crunched" => Self::Crunched,
			b"debug" => Self::Debug,
			b"done" => Self::Done,
			b"error" => Self::Error,
			b"info" => Self::Info,
			b"notice" => Self::Notice,
			b"success" => Self::Success,
			b"task" => Self::Task,
			b"warning" => Self::Warning,
			#[cfg(feature = "bin_kinds")] b"blank" => Self::Blank,
			#[cfg(feature = "bin_kinds")] b"print" => Self::Custom,
			_ => Self::None,
		}
	}
}

/// ## Details.
impl MsgKind {
	#[must_use]
	/// # Is Empty?
	pub const fn is_empty(self) -> bool { matches!(self, Self::None) }

	#[must_use]
	/// # Length.
	pub const fn len(self) -> usize {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => 0,
			#[cfg(not(feature = "bin_kinds"))] Self::None => 0,
			Self::Confirm => 26,
			Self::Crunched => 21,
			Self::Done | Self::Info => 17,
			Self::Debug | Self::Error => 18,
			Self::Notice => 19,
			Self::Success | Self::Warning => 20,
			Self::Task => 23,
		}
	}

	#[must_use]
	/// # Length.
	pub const fn len_32(self) -> u32 {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => 0,
			#[cfg(not(feature = "bin_kinds"))] Self::None => 0,
			Self::Confirm => 26,
			Self::Crunched => 21,
			Self::Done | Self::Info => 17,
			Self::Debug | Self::Error => 18,
			Self::Notice => 19,
			Self::Success | Self::Warning => 20,
			Self::Task => 23,
		}
	}
}

/// ## Conversion.
impl MsgKind {
	#[must_use]
	/// # As Bytes.
	pub const fn as_bytes(self) -> &'static [u8] {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => &[],
			#[cfg(not(feature = "bin_kinds"))] Self::None => &[],
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
		}
	}

	#[must_use]
	/// # As Str.
	pub const fn as_str(self) -> &'static str {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => "",
			#[cfg(not(feature = "bin_kinds"))] Self::None => "",
			Self::Confirm => "\x1b[1;38;5;208mConfirm:\x1b[0m ",
			Self::Crunched => "\x1b[92;1mCrunched:\x1b[0m ",
			Self::Debug => "\x1b[96;1mDebug:\x1b[0m ",
			Self::Done => "\x1b[92;1mDone:\x1b[0m ",
			Self::Error => "\x1b[91;1mError:\x1b[0m ",
			Self::Info => "\x1b[95;1mInfo:\x1b[0m ",
			Self::Notice => "\x1b[95;1mNotice:\x1b[0m ",
			Self::Success => "\x1b[92;1mSuccess:\x1b[0m ",
			Self::Task => "\x1b[1;38;5;199mTask:\x1b[0m ",
			Self::Warning => "\x1b[93;1mWarning:\x1b[0m ",
		}
	}

	/// # Into Message.
	pub fn into_msg<S>(self, msg: S) -> Msg
	where S: AsRef<str> {
		Msg::new(self, msg)
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_len() {
		for p in &[
			MsgKind::Confirm,
			MsgKind::Crunched,
			MsgKind::Debug,
			MsgKind::Done,
			MsgKind::Error,
			MsgKind::Info,
			MsgKind::None,
			MsgKind::Notice,
			MsgKind::Success,
			MsgKind::Task,
			MsgKind::Warning,
		] {
			assert_eq!(p.len(), p.as_bytes().len());
			assert_eq!(p.is_empty(), p.as_bytes().is_empty());
		}
	}
}
