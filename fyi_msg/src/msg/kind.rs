use super::Msg;
use std::ops::Deref;



#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
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
///
/// When you know the prefix at compile time and want a trailing line break,
/// it is more efficient to call the corresponding method on the [`Msg`]
/// struct, like [`Msg::error`], [`Msg::success`], etc.
///
/// Alternatively, you can just call [`Msg::new`] with the prefix, which is
/// what [`MsgKind::into_msg`] does anyway.
pub enum MsgKind {
	#[default]
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

	#[cfg(feature = "bin_kinds")] #[doc(hidden)] Blank,
	#[cfg(feature = "bin_kinds")] #[doc(hidden)] Custom,
}

impl Deref for MsgKind {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { self.as_bytes() }
}

impl From<&[u8]> for MsgKind {
	/// # From Bytes.
	///
	/// This is a reverse lookup that translates bytes back into the
	/// corresponding enum variant. This method only really exists for the
	/// benefit of the FYI binary.
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
	/// # Length.
	///
	/// This returns the byte length of the prefix as a `u32`, worth mentioning
	/// only because most length methods think in terms of `usize`.
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
	///
	/// This is the same as dereferencing.
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

	#[inline]
	/// # Into Message.
	///
	/// This is a convenience method to generate a new message using this
	/// prefix. It is equivalent to calling [`Msg::new`], and in fact, that's
	/// what it does under the hood.
	pub fn into_msg<S>(self, msg: S) -> Msg
	where S: AsRef<str> { Msg::new(self, msg) }
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
