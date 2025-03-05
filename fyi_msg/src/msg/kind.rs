/*!
# FYI Msg: Kinds
*/

use super::Msg;
use std::fmt;



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
	/// # None.
	///
	/// No prefix at all, equivalent to [`Msg::plain`].
	None,

	/// # Confirm.
	Confirm,

	/// # Crunched.
	Crunched,

	/// # Debug.
	Debug,

	/// # Done.
	Done,

	/// # Error.
	Error,

	/// # Info.
	Info,

	/// # Notice.
	Notice,

	/// # Review
	Review,

	/// # Skipped.
	Skipped,

	/// # Success.
	Success,

	/// # Task.
	Task,

	/// # Warning.
	Warning,

	#[cfg(feature = "bin_kinds")] #[doc(hidden)] Blank,
	#[cfg(feature = "bin_kinds")] #[doc(hidden)] Custom,
}

impl fmt::Display for MsgKind {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.pad(self.prefix())
	}
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
			b"review" => Self::Review,
			b"skipped" => Self::Skipped,
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
	#[doc(hidden)]
	#[cfg(feature = "bin_kinds")]
	#[must_use]
	/// # Command.
	///
	/// Return the corresponding CLI (sub)command that triggers this kind.
	pub const fn command(self) -> &'static str {
		match self {
			Self::Blank => "blank",
			Self::Confirm => "confirm",
			Self::Crunched => "crunched",
			Self::Custom => "print",
			Self::Debug => "debug",
			Self::Done => "done",
			Self::Error => "error",
			Self::Info => "info",
			Self::None => "",
			Self::Notice => "notice",
			Self::Review => "review",
			Self::Skipped => "skipped",
			Self::Success => "success",
			Self::Task => "task",
			Self::Warning => "warning",
		}
	}

	#[cfg(feature = "bin_kinds")]
	#[must_use]
	/// # Is Empty.
	///
	/// This returns true for [`MsgKind::None`], false for everything else.
	pub const fn is_empty(self) -> bool {
		matches!(self, Self::None | Self::Blank | Self::Custom)
	}

	#[cfg(not(feature = "bin_kinds"))]
	#[must_use]
	/// # Is Empty.
	///
	/// This returns true for [`MsgKind::None`], false for everything else.
	pub const fn is_empty(self) -> bool { matches!(self, Self::None) }

	#[must_use]
	/// # Length.
	///
	/// This returns the byte length of the prefix as a `u32`, worth mentioning
	/// only because most length methods think in terms of `usize`.
	///
	/// Note: this value includes "invisible" ANSI-related bytes as well as the
	/// trailing ": " bit, so will be bigger than you might expect for
	/// everything but [`MsgKind::None`], which is empty/zero.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::MsgKind;
	///
	/// assert!(4 < MsgKind::Task.len_32());
	/// ```
	pub const fn len_32(self) -> u32 {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => 0,
			#[cfg(not(feature = "bin_kinds"))] Self::None => 0,
			Self::Confirm => 26,
			Self::Crunched => 21,
			Self::Done | Self::Info => 17,
			Self::Debug | Self::Error => 18,
			Self::Notice | Self::Review => 19,
			Self::Skipped | Self::Success | Self::Warning => 20,
			Self::Task => 23,
		}
	}
}

/// ## Conversion.
impl MsgKind {
	#[must_use]
	/// # As (Formatted) Byte Slice.
	///
	/// Return a byte slice suitable for use as a [`Msg`] prefix part, complete
	/// with ANSI coloration/bolding and a trailing ": ".
	///
	/// In most contexts, [`MsgKind::prefix`] is more appropriate.
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
			Self::Review => b"\x1b[96;1mReview:\x1b[0m ",
			Self::Skipped => b"\x1b[93;1mSkipped:\x1b[0m ",
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

	#[inline]
	#[must_use]
	/// # Prefix (Label).
	///
	/// Return just the word, without any ANSI formatting or trailing
	/// punctuation.
	///
	/// In practice, this works out to be the same as the variant itself,
	/// except for [`MsgKind::None`], which is literally nothing.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::MsgKind;
	///
	/// // It's what you'd expect.
	/// assert_eq!(MsgKind::Error.prefix(), "Error");
	/// assert_eq!(MsgKind::Success.prefix(), "Success");
	///
	/// // Except maybe this, which is actually nothing.
	/// assert_eq!(MsgKind::None.prefix(), "");
	/// ```
	pub const fn prefix(self) -> &'static str {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => "",
			#[cfg(not(feature = "bin_kinds"))] Self::None => "",
			Self::Confirm => "Confirm",
			Self::Crunched => "Crunched",
			Self::Debug => "Debug",
			Self::Done => "Done",
			Self::Error => "Error",
			Self::Info => "Info",
			Self::Notice => "Notice",
			Self::Review => "Review",
			Self::Skipped => "Skipped",
			Self::Success => "Success",
			Self::Task => "Task",
			Self::Warning => "Warning",
		}
	}

	#[inline]
	#[must_use]
	/// # Prefix Color.
	///
	/// Return the ANSI color code ([256/vte](https://misc.flogisoft.com/bash/tip_colors_and_formatting#foreground_text1))
	/// used by the prefix.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::MsgKind;
	///
	/// // Roll your own coloration.
	/// let kind = MsgKind::Task;
	/// println!(
	///     "\x1b[1;38;5;{}m{kind}:\x1b[0m Who even needs this crate?!",
	///     kind.prefix_color(),
	/// );
	/// ```
	pub const fn prefix_color(self) -> u8 {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => 39,
			#[cfg(not(feature = "bin_kinds"))] Self::None => 39,
			Self::Confirm => 208,                              // Orange.
			Self::Crunched | Self::Done | Self::Success => 10, // (Light) Green.
			Self::Debug | Self::Review => 14,                  // (Light) Cyan.
			Self::Error => 9,                                  // (Light) Red.
			Self::Info | Self::Notice => 13,                   // (Light) Magenta.
			Self::Skipped | Self::Warning => 11,               // (Light) Yellow.
			Self::Task => 199,                                 // Hot Pink.
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	/// # Kinds.
	///
	/// All the normal kinds except for [`MsgKind::None`].
	const KINDS: [MsgKind; 13] = [
		MsgKind::None,
		MsgKind::Confirm,
		MsgKind::Crunched,
		MsgKind::Debug,
		MsgKind::Done,
		MsgKind::Error,
		MsgKind::Info,
		MsgKind::Notice,
		MsgKind::Review,
		MsgKind::Skipped,
		MsgKind::Success,
		MsgKind::Task,
		MsgKind::Warning,
	];

	#[test]
	fn t_len() {
		for p in KINDS {
			assert_eq!(p.len_32() as usize, p.as_bytes().len());
			assert_eq!(p.is_empty(), p.as_bytes().is_empty());
			assert_eq!(p.is_empty(), p.len_32() == 0);
		}
	}

	#[test]
	fn t_prefix() {
		for p in KINDS {
			// Prefix should match the display impl.
			let prefix = p.prefix();
			assert_eq!(prefix, p.to_string());

			// None is empty so there's nothing to look for, per se.
			if matches!(p, MsgKind::None) { assert_eq!(prefix, ""); }
			// Otherwise we should find our prefix in the formatted byte slice
			// between the closing ANSI "m" and colon.
			else {
				let s = std::str::from_utf8(p.as_bytes()).expect("Invalid UTF-8!");
				assert!(s.contains(&format!("m{prefix}:\x1b")));
			}
		}
	}
}
