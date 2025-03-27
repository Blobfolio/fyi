/*!
# FYI Msg: Message Kinds (Prefixes).
*/

use crate::{
	ansi::AnsiColor,
	Msg,
};
use std::{
	borrow::Cow,
	fmt,
};



// `MsgKind` generated by build.rs.
include!(concat!(env!("OUT_DIR"), "/msg-kinds.rs"));

impl Default for MsgKind {
	#[inline]
	fn default() -> Self { Self::None }
}

impl fmt::Display for MsgKind {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		<str as fmt::Display>::fmt(self.as_str(), f)
	}
}

#[cfg(feature = "bin_kinds")]
impl From<&[u8]> for MsgKind {
	/// # From Byte Slice.
	fn from(src: &[u8]) -> Self {
		match src.trim_ascii() {
			b"aborted" => Self::Aborted,
			b"blank" => Self::Blank,
			b"confirm" | b"prompt" => Self::Confirm,
			b"crunched" => Self::Crunched,
			b"debug" => Self::Debug,
			b"done" => Self::Done,
			b"error" => Self::Error,
			b"found" => Self::Found,
			b"info" => Self::Info,
			b"notice" => Self::Notice,
			b"print" => Self::Custom,
			b"review" => Self::Review,
			b"skipped" => Self::Skipped,
			b"success" => Self::Success,
			b"task" => Self::Task,
			b"warning" => Self::Warning,
			_ => Self::None,
		}
	}
}

/// ## Details.
impl MsgKind {
	#[must_use]
	/// # As String Slice.
	///
	/// Return the kind as a string slice, _without_ the formatting and trailing
	/// `": "` used by [`Msg`].
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::MsgKind;
	///
	/// assert_eq!(MsgKind::Error.as_str(), "Error");
	/// assert_eq!(MsgKind::Success.as_str(), "Success");
	/// ```
	pub const fn as_str(self) -> &'static str {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => "",
			#[cfg(not(feature = "bin_kinds"))] Self::None => "",
			Self::Aborted => "Aborted",
			Self::Confirm => "Confirm",
			Self::Crunched => "Crunched",
			Self::Debug => "Debug",
			Self::Done => "Done",
			Self::Error => "Error",
			Self::Found => "Found",
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
	/// # As String Slice (Prefix).
	///
	/// Return the kind as a string slice, formatted and with a trailing `": "`,
	/// same as [`Msg`] uses for prefixes.
	pub(crate) const fn as_str_prefix(self) -> &'static str {
		match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => "",
			#[cfg(not(feature = "bin_kinds"))] Self::None => "",
			Self::Aborted =>  "\x1b[1;91mAborted:\x1b[0m ",
			Self::Confirm =>  "\x1b[1;38;5;208mConfirm:\x1b[0m ",
			Self::Crunched => "\x1b[1;92mCrunched:\x1b[0m ",
			Self::Debug =>    "\x1b[1;96mDebug:\x1b[0m ",
			Self::Done =>     "\x1b[1;92mDone:\x1b[0m ",
			Self::Error =>    "\x1b[1;91mError:\x1b[0m ",
			Self::Found =>    "\x1b[1;92mFound:\x1b[0m ",
			Self::Info =>     "\x1b[1;95mInfo:\x1b[0m ",
			Self::Notice =>   "\x1b[1;95mNotice:\x1b[0m ",
			Self::Review =>   "\x1b[1;96mReview:\x1b[0m ",
			Self::Skipped =>  "\x1b[1;93mSkipped:\x1b[0m ",
			Self::Success =>  "\x1b[1;92mSuccess:\x1b[0m ",
			Self::Task =>     "\x1b[1;38;5;199mTask:\x1b[0m ",
			Self::Warning =>  "\x1b[1;93mWarning:\x1b[0m ",
		}
	}

	#[cfg(feature = "bin_kinds")]
	#[doc(hidden)]
	#[must_use]
	/// # Command.
	///
	/// Return the corresponding CLI (sub)command that triggers this kind.
	///
	/// Note: this is only intended for use by the `fyi` binary; the method
	/// may change without warning.
	pub const fn command(self) -> &'static str {
		match self {
			Self::Aborted => "aborted",
			Self::Blank => "blank",
			Self::Confirm => "confirm",
			Self::Crunched => "crunched",
			Self::Custom => "print",
			Self::Debug => "debug",
			Self::Done => "done",
			Self::Error => "error",
			Self::Found => "found",
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
	/// This returns `true` for [`MsgKind::None`], [`MsgKind::Blank`], and
	/// [`MsgKind::Custom`], `false` for everything else.
	pub const fn is_empty(self) -> bool {
		matches!(self, Self::None | Self::Blank | Self::Custom)
	}

	#[cfg(not(feature = "bin_kinds"))]
	#[must_use]
	/// # Is Empty.
	///
	/// This returns `true` for [`MsgKind::None`], `false` for everything else.
	pub const fn is_empty(self) -> bool { matches!(self, Self::None) }

	#[must_use]
	/// # Prefix Color.
	///
	/// Return the color used by this kind when playing the role of a [`Msg`]
	/// prefix, or `None` if [`MsgKind::None`], which has neither content nor
	/// formatting.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::ansi::AnsiColor;
	/// use fyi_msg::MsgKind;
	///
	/// assert_eq!(
	///     MsgKind::Info.prefix_color(),
	///     Some(AnsiColor::LightMagenta),
	/// );
	/// ```
	pub const fn prefix_color(self) -> Option<AnsiColor> {
		Some(match self {
			#[cfg(feature = "bin_kinds")] Self::None | Self::Blank | Self::Custom => return None,
			#[cfg(not(feature = "bin_kinds"))] Self::None => return None,
			Self::Aborted =>  AnsiColor::LightRed,
			Self::Confirm =>  AnsiColor::DarkOrange,
			Self::Crunched => AnsiColor::LightGreen,
			Self::Debug =>    AnsiColor::LightCyan,
			Self::Done =>     AnsiColor::LightGreen,
			Self::Error =>    AnsiColor::LightRed,
			Self::Found =>    AnsiColor::LightGreen,
			Self::Info =>     AnsiColor::LightMagenta,
			Self::Notice =>   AnsiColor::LightMagenta,
			Self::Review =>   AnsiColor::LightCyan,
			Self::Skipped =>  AnsiColor::LightYellow,
			Self::Success =>  AnsiColor::LightGreen,
			Self::Task =>     AnsiColor::Misc199,
			Self::Warning =>  AnsiColor::LightYellow,
		})
	}

	#[inline]
	#[must_use]
	/// # Into Message.
	///
	/// This is a convenience method to generate a new message using this
	/// prefix, equivalent to passing the kind to [`Msg::new`] manually.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::{Msg, MsgKind};
	///
	/// assert_eq!(
	///     MsgKind::Error.into_msg("Oops"),
	///     Msg::new(MsgKind::Error, "Oops"),
	/// );
	/// ```
	///
	/// Most kinds — everything but [`MsgKind::None`] and [`MsgKind::Confirm`] —
	/// have same-named shorthand methods on the `Msg` struct itself that work
	/// like the above, except they also add a line break to the end.
	///
	/// ```
	/// use fyi_msg::{Msg, MsgKind};
	///
	/// assert_eq!(
	///     Msg::error("Oops"),
	///     MsgKind::Error.into_msg("Oops").with_newline(true),
	/// );
	/// ```
	pub fn into_msg<S>(self, msg: S) -> Msg
	where S: AsRef<str> { Msg::new(self, msg) }
}



/// # Into Message Prefix.
///
/// This trait provides everything necessary to format prefixes passed to
/// [`Msg::new`], [`Msg::set_prefix`], and [`Msg::with_prefix`].
///
/// More specifically, it allows users to choose between the "easy" built-in
/// [`MsgKind`] prefixes and custom ones, with or without ANSI formatting.
///
/// Custom prefixes can be any of the usual string types — `&str`,
/// `String`/`&String`, or `Cow<str>`/`&Cow<str>` — optionally tupled with an
/// [`AnsiColor`] for formatting.
///
/// See [`Msg::new`] for more details.
pub trait IntoMsgPrefix {
	/// # Prefix Length.
	///
	/// Returns the total byte length of the fully-rendered prefix, including
	/// any ANSI sequences and trailing `": "` separator.
	fn prefix_len(&self) -> usize;

	/// # Push Prefix.
	///
	/// Push the complete prefix to an existing string.
	fn prefix_push(&self, dst: &mut String);

	#[inline]
	/// # Prefix String.
	///
	/// Returns the complete prefix for rendering.
	///
	/// [`MsgKind`] prefixes are static and require no allocation, but custom
	/// types (unless empty) do to join all the pieces together.
	fn prefix_str(&self) -> Cow<str> {
		let mut out = String::with_capacity(self.prefix_len());
		self.prefix_push(&mut out);
		Cow::Owned(out)
	}
}

impl IntoMsgPrefix for MsgKind {
	#[inline]
	/// # Prefix Length.
	fn prefix_len(&self) -> usize { self.as_str_prefix().len() }

	#[inline]
	/// # Prefix String.
	fn prefix_str(&self) -> Cow<str> { Cow::Borrowed(self.as_str_prefix()) }

	#[inline]
	/// # Push Prefix.
	fn prefix_push(&self, dst: &mut String) { dst.push_str(self.as_str_prefix()); }
}

/// # Helper: `IntoMsgPrefix`.
macro_rules! into_prefix {
	($($ty:ty),+) => ($(
		impl IntoMsgPrefix for $ty {
			#[inline]
			/// # Prefix Length.
			fn prefix_len(&self) -> usize {
				let len = self.len();
				if len == 0 { 0 }
				else { len + 2 } // For the ": " separator.
			}

			#[inline]
			/// # Push Prefix.
			fn prefix_push(&self, dst: &mut String) {
				if ! self.is_empty() {
					dst.push_str(self);
					dst.push_str(": ");
				}
			}
		}

		impl IntoMsgPrefix for ($ty, AnsiColor) {
			#[inline]
			/// # Prefix Length.
			fn prefix_len(&self) -> usize {
				let len = self.0.len();
				if len == 0 { 0 }
				else {
					self.1.as_str_bold().len() + self.0.len() +
					AnsiColor::RESET_PREFIX.len()
				}
			}

			#[inline]
			/// # Push Prefix.
			fn prefix_push(&self, dst: &mut String) {
				if ! self.0.is_empty() {
					dst.push_str(self.1.as_str_bold());
					dst.push_str(&self.0);
					dst.push_str(AnsiColor::RESET_PREFIX);
				}
			}
		}

		impl IntoMsgPrefix for ($ty, u8) {
			#[inline]
			/// # Prefix Length.
			fn prefix_len(&self) -> usize {
				let len = self.0.len();
				if len == 0 { 0 }
				else {
					let color = AnsiColor::from_u8(self.1);
					color.as_str_bold().len() + self.0.len() +
					AnsiColor::RESET_PREFIX.len()
				}
			}

			#[inline]
			/// # Push Prefix.
			fn prefix_push(&self, dst: &mut String) {
				if ! self.0.is_empty() {
					dst.push_str(AnsiColor::from_u8(self.1).as_str_bold());
					dst.push_str(&self.0);
					dst.push_str(AnsiColor::RESET_PREFIX);
				}
			}
		}
	)+);
}
into_prefix!(&str, &String, String, &Cow<'_, str>, Cow<'_, str>);



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn t_as_str_prefix() {
		// Make sure our hardcoded prefix strings look like they're supposed
		// to, using the ansi builder/color as an alternative.
		for kind in MsgKind::ALL {
			// Not all kinds have formatted versions.
			let Some(color) = kind.prefix_color() else { continue; };
			let manual = format!("{}{kind}:{} ", color.as_str_bold(), AnsiColor::RESET);

			assert_eq!(manual, kind.as_str_prefix());
		}
	}

	// `MsgKind` generated by build.rs.
	include!(concat!(env!("OUT_DIR"), "/msg-kinds-tests.rs"));
}
