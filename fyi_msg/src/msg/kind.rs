/*!
# FYI Msg: Message Kinds (Prefixes).
*/

use crate::{
	AnsiColor,
	Msg,
};
use fyi_ansi::ansi;
use std::{
	borrow::Cow,
	fmt,
};



/// # Helper: `MsgKind` Setup.
macro_rules! msg_kind {
	// A neat counting trick adapted from The Little Book of Rust Macros, used
	// here to figure out the size of the ALL array.
	(@count $odd:tt) => ( 1 );
	(@count $odd:tt $($a:tt $b:tt)+) => ( (msg_kind!(@count $($a)+) * 2) + 1 );
	(@count $($a:tt $b:tt)+) =>         (  msg_kind!(@count $($a)+) * 2      );

	// Define MsgKind, MsgKind::ALL, and MsgKind::as_str_prefix.
	(@build $($k:ident, $v:expr),+ $(,)?) => (
		#[expect(missing_docs, reason = "Redudant.")]
		#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
		/// # Message Kind.
		///
		/// This enum contains built-in prefixes for [`Msg`](crate::Msg). These are
		/// generally only used to initiate a new message with this prefix, like:
		///
		/// ## Examples
		///
		/// ```
		/// use fyi_msg::{Msg, MsgKind};
		///
		/// // Error: Oh no!
		/// assert_eq!(
		///     Msg::new(MsgKind::Error, "Oh no!"),
		///     MsgKind::Error.into_msg("Oh no!"),
		/// );
		/// ```
		///
		/// Most kinds have their own dedicated [`Msg`] helper method which, unlike the
		/// previous examples, comes with a line break at the end.
		///
		/// ```
		/// use fyi_msg::{Msg, MsgKind};
		///
		/// // Error: Oh no!\n
		/// assert_eq!(
		///     Msg::error("Oh no!"),
		///     Msg::new(MsgKind::Error, "Oh no!").with_newline(true),
		/// );
		/// ```
		pub enum MsgKind {
			$($k),+
		}

		impl MsgKind {
			/// # All Variants.
			///
			/// This array can be used to cheaply iterate through all message kinds.
			pub const ALL: [Self; msg_kind!(@count $($k)+)] = [
				$(Self::$k),+
			];

			#[inline]
			#[must_use]
			/// # As String Slice (Prefix).
			///
			/// Return the kind as a string slice, formatted and with a trailing `": "`,
			/// same as [`Msg`] uses for prefixes.
			pub(crate) const fn as_str_prefix(self) -> &'static str {
				match self {
					$(Self::$k => $v),+
				}
			}
		}
	);

	// Define MsgKind::as_str.
	(@as_str $($k:ident),+ $(,)?) => (
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
			///
			/// // Note that None is empty.
			/// assert_eq!(MsgKind::None.as_str(), "");
			/// ```
			pub const fn as_str(self) -> &'static str {
				match self {
					$(Self::$k => stringify!($k)),+,
					_ => "",
				}
			}
		}
	);

	// Define the one-shot Msg helpers.
	(@msg $($k:ident, $fn:ident, $v:expr),+ $(,)?) => (
		/// ## [`MsgKind`] One-Shots.
		impl Msg {
			$(
				#[must_use]
				#[doc = concat!("# New ", stringify!($k), ".")]
				///
				#[doc = concat!("Create a new [`Msg`] with a built-in [`MsgKind::", stringify!($k), "`] prefix _and_ trailing line break.")]
				///
				/// ## Examples.
				///
				/// ```
				/// use fyi_msg::{Msg, MsgKind};
				///
				/// assert_eq!(
				#[doc = concat!("    Msg::", stringify!($fn), "(\"Hello World\"),")]
				#[doc = concat!("    Msg::new(MsgKind::", stringify!($k), ", \"Hello World\").with_newline(true),")]
				/// );
				/// ```
				pub fn $fn<S: AsRef<str>>(msg: S) -> Self {
					// Glue it all together.
					let msg = msg.as_ref();
					let m_end = $v.len() + msg.len();
					let mut inner = String::with_capacity(m_end + 1);
					inner.push_str($v);
					inner.push_str(msg);
					inner.push('\n');

					// Done!
					Self {
						inner,
						toc: super::toc!($v.len(), m_end, true),
					}
				}
			)+
		}
	);

	// Generate an ANSI-formatted Msg prefix for a given kind.
	(@prefix $kind:ident, $color:tt) => (
		concat!(ansi!((bold, $color) stringify!($kind), ":"), " ")
	);

	// Entry point!
	($($kind:ident, $fn:ident, $color:tt),+ ,) => (
		#[cfg(feature = "bin_kinds")]
		msg_kind!{
			@build
			None, "",
			Confirm,  msg_kind!(@prefix Confirm, dark_orange),
			$($kind, msg_kind!(@prefix $kind, $color)),+,
			Blank, "",
			Custom, "",
		}

		#[cfg(not(feature = "bin_kinds"))]
		msg_kind!{
			@build
			None, "",
			Confirm,  msg_kind!(@prefix Confirm, dark_orange),
			$($kind, msg_kind!(@prefix $kind, $color)),+,
		}

		msg_kind!{ @as_str Confirm, $($kind),+ }

		msg_kind!{
			@msg
			$($kind, $fn, msg_kind!(@prefix $kind, $color)),+,
		}
	);
}

msg_kind! {
	Aborted,  aborted,  light_red,
	Crunched, crunched, light_green,
	Debug,    debug,    light_cyan,
	Done,     done,     light_green,
	Error,    error,    light_red,
	Found,    found,    light_green,
	Info,     info,     light_magenta,
	Notice,   notice,   light_magenta,
	Review,   review,   light_cyan,
	Skipped,  skipped,  light_yellow,
	Success,  success,  light_green,
	Task,     task,     199,
	Warning,  warning,  light_yellow,
}

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
	/// use fyi_msg::{AnsiColor, MsgKind};
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
	fn prefix_str(&self) -> Cow<'_, str> {
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
	fn prefix_str(&self) -> Cow<'_, str> { Cow::Borrowed(self.as_str_prefix()) }

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
}
