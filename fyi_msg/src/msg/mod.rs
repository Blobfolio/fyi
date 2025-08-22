/*!
# FYI Msg: Messages!
*/

pub(super) mod kind;

#[expect(unused_imports, reason = "For docs.")]
use crate::MsgKind;

#[cfg(feature = "progress")] use crate::BeforeAfter;
use fyi_ansi::{
	ansi,
	csi,
	dim,
};
use kind::IntoMsgPrefix;
use std::{
	borrow::{
		Borrow,
		Cow,
	},
	cmp::Ordering,
	fmt,
	hash,
	io,
	num::NonZeroUsize,
	ops::Range,
};



#[cfg(feature = "timestamps")]
/// # Helper: `Toc` Setup.
macro_rules! toc {
	($p_end:expr, $m_end:expr) => (
		$crate::msg::Toc([
			0,      // Indentation.
			0,      // Timestamp.
			0,      // Prefix.
			$p_end, // Message.
			$m_end, // Suffix.
			$m_end, // Newline.
			$m_end, // Total Length.
		])
	);
	($p_end:expr, $m_end:expr, true) => (
		$crate::msg::Toc([
			0,          // Indentation.
			0,          // Timestamp.
			0,          // Prefix.
			$p_end,     // Message.
			$m_end,     // Suffix.
			$m_end,     // Newline.
			$m_end + 1, // Total Length.
		])
	);
}

#[cfg(not(feature = "timestamps"))]
/// # Helper: `Toc` Setup.
macro_rules! toc {
	($p_end:expr, $m_end:expr) => (
		$crate::msg::Toc([
			0,      // Indentation.
			0,      // Prefix.
			$p_end, // Message.
			$m_end, // Suffix.
			$m_end, // Newline.
			$m_end, // Total Length.
		])
	);
	($p_end:expr, $m_end:expr, true) => (
		$crate::msg::Toc([
			0,          // Indentation.
			0,          // Prefix.
			$p_end,     // Message.
			$m_end,     // Suffix.
			$m_end,     // Newline.
			$m_end + 1, // Total Length.
		])
	);
}
use toc; // kind.rs needs this.



#[derive(Debug, Default, Clone)]
/// # Message.
///
/// The `Msg` struct provides a partitioned, contiguous byte source to hold
/// arbitrary messages of the "Error: Oh no!" variety. They can be modified
/// efficiently in place (per-part) and printed to `STDOUT` with [`Msg::print`]
/// or `STDERR` with [`Msg::eprint`] (or via [`Display`](fmt::Display)).
///
/// There are two crate feature gates that augment this struct (at the expense
/// of additional dependencies):
///
/// * `fitted` adds [`Msg::fitted`] for obtaining a slice trimmed to a specific display width.
/// * `timestamps` adds [`Msg::set_timestamp`]/[`Msg::with_timestamp`] for inserting a local datetime value before the prefix.
///
/// Everything else comes stock!
///
/// ## Examples
///
/// ```
/// use fyi_msg::{Msg, MsgKind};
///
/// Msg::new(MsgKind::Success, "You did it!")
///     .with_newline(true)
///     .print();
/// ```
///
/// There are a bunch of built-in prefix types ([`MsgKind`]), each of which
/// (except [`MsgKind::None`] and [`MsgKind::Confirm`]) has a corresponding
/// "quick" method on this struct to save the effort of chaining [`Msg::new`]
/// and [`Msg::with_newline`].
///
/// ```
/// use fyi_msg::{Msg, MsgKind};
///
/// // Same as before, but more concise.
/// Msg::success("You did it!").print();
/// ```
///
/// Confirmations have a convenience _macro_ instead, [`confirm`](crate::confirm),
/// that handles all the setup and prompting, returning a simple `bool`
/// indicating the yes/noness of the user response.
///
/// Take a look at `examples/msg.rs` for a breakdown of the various options.
///
/// ## Conversion
///
/// `Msg` objects are essentially just fancy strings.
///
/// You can borrow the the result with [`Msg::as_str`]/[`Msg::as_bytes`] or
/// steal it with [`Msg::into_string`]/[`Msg::into_bytes`].
pub struct Msg {
	/// # Actual Message.
	inner: String,

	/// # Table of Contents.
	toc: Toc,
}

impl AsRef<[u8]> for Msg {
	#[inline]
	fn as_ref(&self) -> &[u8] { self.as_bytes() }
}

impl AsRef<str> for Msg {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl Borrow<str> for Msg {
	#[inline]
	fn borrow(&self) -> &str { self.as_str() }
}

impl fmt::Display for Msg {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		<str as fmt::Display>::fmt(self.as_str(), f)
	}
}

impl Eq for Msg {}

/// # Helper: From Stringlike.
macro_rules! from_stringlike {
	($ty:ty, $op:ident) => (
		impl From<$ty> for Msg {
			#[inline]
			fn from(src: $ty) -> Self { Self::from(src.$op()) }
		}
	);
}
from_stringlike!(&str, to_owned);
from_stringlike!(&String, clone);
from_stringlike!(Cow<'_, str>, into_owned);

impl From<String> for Msg {
	#[inline]
	fn from(src: String) -> Self {
		let m_end = src.len();
		Self {
			inner: src,
			toc: toc!(0, m_end),
		}
	}
}

impl From<Msg> for String {
	#[inline]
	fn from(src: Msg) -> Self { src.into_string() }
}

impl hash::Hash for Msg {
	#[inline]
	fn hash<H: hash::Hasher>(&self, state: &mut H) { self.inner.hash(state); }
}

impl PartialEq for Msg {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl PartialEq<str> for Msg {
	#[inline]
	fn eq(&self, other: &str) -> bool { self.as_str() == other }
}
impl PartialEq<Msg> for str {
	#[inline]
	fn eq(&self, other: &Msg) -> bool { <Msg as PartialEq<Self>>::eq(other, self) }
}

impl PartialEq<[u8]> for Msg {
	#[inline]
	fn eq(&self, other: &[u8]) -> bool { self.as_bytes() == other }
}
impl PartialEq<Msg> for [u8] {
	#[inline]
	fn eq(&self, other: &Msg) -> bool { <Msg as PartialEq<Self>>::eq(other, self) }
}

/// # Helper: Reciprocal `PartialEq`.
macro_rules! eq {
	($parent:ty: $($ty:ty),+) => ($(
		impl PartialEq<$ty> for Msg {
			#[inline]
			fn eq(&self, other: &$ty) -> bool { <Self as PartialEq<$parent>>::eq(self, other) }
		}
		impl PartialEq<Msg> for $ty {
			#[inline]
			fn eq(&self, other: &Msg) -> bool { <Msg as PartialEq<$parent>>::eq(other, self) }
		}
	)+);
}
eq!(str:  &str,  &String,  String,  &Cow<'_, str>,  Cow<'_, str>,  &Box<str>,  Box<str>);
eq!([u8]: &[u8], &Vec<u8>, Vec<u8>, &Cow<'_, [u8]>, Cow<'_, [u8]>, &Box<[u8]>, Box<[u8]>);

impl Ord for Msg {
	#[inline]
	fn cmp(&self, other: &Self) -> Ordering { self.inner.cmp(&other.inner) }
}

impl PartialOrd for Msg {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

/// ## Construction.
impl Msg {
	#[must_use]
	#[expect(clippy::needless_pass_by_value, reason = "Trait covers owned and referenced types.")]
	/// # New Message.
	///
	/// This creates a new [`Msg`] with prefix and message parts.
	///
	/// The prefix can be a built-in [`MsgKind`], or something custom, with or
	/// without ANSI formatting.
	///
	/// Custom prefixes can be any of the usual string types — `&str`,
	/// `String`/`&String`, or `Cow<str>`/`&Cow<str>` — optionally tupled with
	/// an [`AnsiColor`](crate::AnsiColor) for sex appeal.
	///
	/// Custom prefixes should _not_ include the `": "` separator, as that is
	/// appended automatically to all non-empty values.
	///
	/// To create a message without a prefix, just pass the content to
	/// [`Msg::from`] instead.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::{
	///     AnsiColor,
	///     Msg,
	///     MsgKind,
	/// };
	///
	/// // Built-in prefix. Easy!
	/// assert_eq!(
	///     Msg::new(MsgKind::Info, "This is a message."),
	///     "\x1b[1;95mInfo:\x1b[0m This is a message.",
	/// );
	///
	/// // Custom prefix, no formatting.
	/// assert_eq!(
	///     Msg::new("Best Picture", "C.H.U.D."),
	///     "Best Picture: C.H.U.D.",
	/// );
	///
	/// // Custom prefix, red and bold.
	/// assert_eq!(
	///     Msg::new(("Crap", AnsiColor::Red), "Something broke!"),
	///     "\x1b[1;31mCrap:\x1b[0m Something broke!"
	/// );
	///
	/// // Same as above, but with the color as a `u8`.
	/// assert_eq!(
	///     Msg::new(("Crap", 1), "Something broke!"),
	///     "\x1b[1;31mCrap:\x1b[0m Something broke!"
	/// );
	///
	/// // If for some reason you pass an empty string, the prefix will be
	/// // omitted.
	/// assert_eq!(
	///     Msg::new(("", AnsiColor::Misc199), "Plain Jane."),
	///     "Plain Jane.",
	/// );
	/// ```
	pub fn new<P, S>(prefix: P, msg: S) -> Self
	where
		P: IntoMsgPrefix,
		S: AsRef<str>,
	{
		let msg = msg.as_ref();

		let p_end = prefix.prefix_len();
		let mut inner = String::with_capacity(p_end + msg.len());
		prefix.prefix_push(&mut inner);
		inner.push_str(msg);
		let m_end = inner.len();

		// Done!
		Self {
			inner,
			toc: toc!(p_end, m_end),
		}
	}
}

/// ## Getters.
impl Msg {
	#[inline]
	#[must_use]
	/// # As Byte Slice.
	///
	/// Return the formatted message as a byte slice.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// assert_eq!(
	///     Msg::from("Hello world").as_bytes(),
	///     b"Hello world",
	/// );
	/// ```
	pub const fn as_bytes(&self) -> &[u8] { self.inner.as_bytes() }

	#[inline]
	#[must_use]
	/// # As String Slice.
	///
	/// Return the formatted message as a string slice.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// assert_eq!(
	///     Msg::from("Hello world").as_str(),
	///     "Hello world",
	/// );
	/// ```
	pub const fn as_str(&self) -> &str { self.inner.as_str() }

	#[cfg(feature = "fitted")]
	#[cfg_attr(docsrs, doc(cfg(feature = "fitted")))]
	#[must_use]
	#[inline]
	/// # Fit to Width.
	///
	/// Return the message as a string with its lines capped to the given
	/// display `width`.
	///
	/// This is essentially just a convenience wrapper around [`fit_to_width`](crate::fit_to_width);
	/// refer to that method documentation for more details.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::{AnsiColor, Msg};
	///
	/// let msg = Msg::new(("Name", AnsiColor::Blue), "Björk")
	///     .with_suffix(" (the Great)")
	///     .with_newline(true); // Trailing line breaks are fine.
	///
	/// // As it is:
	/// assert_eq!(
	///     msg.as_str(),
	///     "\x1b[1;34mName:\x1b[0m Björk (the Great)\n",
	/// );
	///
	/// // Fitting to 20 columns loses some of the suffix, but the trailing
	/// // line break is preserved.
	/// assert_eq!(
	///     msg.fitted(20),
	///     "\x1b[1;34mName:\x1b[0m Björk (the Gre\n",
	/// );
	///
	/// // Fitting to 10 columns drops the suffix entirely, loses a bit of
	/// // the message part, but the line break hangs on.
	/// // the trailing line break.
	/// assert_eq!(
	///     msg.fitted(10),
	///     "\x1b[1;34mName:\x1b[0m Björ\n",
	/// );
	///
	/// // Fitting to 4 columns kills most everything, but the ANSI reset and
	/// // line break are preserved.
	/// assert_eq!(
	///     msg.fitted(4),
	///     "\x1b[1;34mName\x1b[0m\n",
	/// );
	/// ```
	pub fn fitted(&self, width: usize) -> Cow<'_, str> {
		crate::fit_to_width(self.as_str(), width)
	}

	#[inline]
	#[must_use]
	/// # Into Bytes.
	///
	/// Consume self, returning an owned byte vector.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// assert_eq!(
	///     Msg::from("Hello world").into_bytes(),
	///     b"Hello world",
	/// );
	/// ```
	pub fn into_bytes(self) -> Vec<u8> { self.inner.into_bytes() }

	#[inline]
	#[must_use]
	/// # Into String.
	///
	/// Consume self, returning the inner string.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// assert_eq!(
	///     Msg::from("Hello world").into_string(),
	///     "Hello world",
	/// );
	/// ```
	pub fn into_string(self) -> String { self.inner }

	#[inline]
	#[must_use]
	/// # Is Empty?
	///
	/// Returns `true` if the message is empty.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// // One way to get an empty message.
	/// assert!(Msg::from("").is_empty());
	/// ```
	pub const fn is_empty(&self) -> bool { self.inner.is_empty() }

	#[inline]
	#[must_use]
	/// # Message Length.
	///
	/// Return the total number of bytes in the formatted message.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// assert_eq!(Msg::from("ABC").len(), 3);
	/// assert_eq!(
	///     Msg::done("Goodbye.").len(),
	///     26,
	/// ); // Don't forget about ANSI…
	/// ```
	pub const fn len(&self) -> usize { self.inner.len() }
}

/// ## Setters.
impl Msg {
	/// # Set Indentation.
	///
	/// (Re)set the message's indentation level to `tabs` "tabs" (four spaces
	/// each), up to a maximum depth of eight (thirty-two spaces total).
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::from("Hello world.");
	///
	/// msg.set_indent(1);
	/// assert_eq!(msg, "    Hello world.");
	///
	/// msg.set_indent(2);
	/// assert_eq!(msg, "        Hello world.");
	///
	/// msg.set_indent(3);
	/// assert_eq!(msg, "            Hello world.");
	///
	/// // …
	///
	/// msg.set_indent(7);
	/// assert_eq!(msg, "                            Hello world.");
	///
	/// msg.set_indent(8);
	/// assert_eq!(msg, "                                Hello world.");
	///
	/// msg.set_indent(9);
	/// assert_eq!(msg, "                                Hello world."); // Same as 8.
	///
	/// // …
	///
	/// msg.set_indent(u8::MAX);
	/// assert_eq!(msg, "                                Hello world."); // Same as 8.
	///
	/// // Back to zero!
	/// msg.set_indent(0);
	/// assert_eq!(msg, "Hello world.");
	/// ```
	pub fn set_indent(&mut self, tabs: u8) {
		/// # Thirty-Two Spaces.
		///
		/// For indentation, alignment, etc.
		static SPACES: &str = "                                ";

		self.replace_part(
			TocId::Indent,
			SPACES.get(..usize::from(tabs) * 4).unwrap_or(SPACES),
		);
	}

	/// # Set Message Content.
	///
	/// (Re)set the actual message part of the message.
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::from("Hello");
	/// assert_eq!(msg, "Hello");
	///
	/// msg.set_msg("Goodbye");
	/// assert_eq!(msg, "Goodbye");
	/// ```
	pub fn set_msg<S: AsRef<str>>(&mut self, msg: S) {
		self.replace_part(TocId::Message, msg.as_ref());
	}

	/// # Set Trailing Linebreak.
	///
	/// Add/remove the message's trailing line break.
	///
	/// ## Examples
	///
	/// Messages created with [`Msg::from`], [`Msg::new`], and
	/// [`MsgKind::into_msg`] have no trailing line break by default:
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::from("Hello World!");
	/// assert_eq!(msg, "Hello World!");
	///
	/// msg.set_newline(true); // Add it.
	/// assert_eq!(msg, "Hello World!\n");
	/// ```
	///
	/// Messages created with the kind-specific helper methods, however, _do_
	/// have a line break by default:
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::info("Hello World!");
	/// assert_eq!(msg, "\x1b[1;95mInfo:\x1b[0m Hello World!\n");
	///
	/// msg.set_newline(false); // Remove it.
	/// assert_eq!(msg, "\x1b[1;95mInfo:\x1b[0m Hello World!");
	/// ```
	pub fn set_newline(&mut self, enabled: bool) {
		let out = if enabled { "\n" } else { "" };
		self.replace_part(TocId::Newline, out);
	}

	#[expect(clippy::needless_pass_by_value, reason = "Impl is on referenced and owned types.")]
	/// # Set Prefix.
	///
	/// (Re/un)set the message prefix.
	///
	/// As with [`Msg::new`], prefixes can be a built-in [`MsgKind`] or custom
	/// string, with or without formatting.
	///
	/// To remove the prefix entirely, pass [`MsgKind::None`] or `""`.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::{
	///     AnsiColor,
	///     Msg,
	///     MsgKind,
	/// };
	///
	/// let mut msg = Msg::new(MsgKind::Error, "Uh oh!");
	/// assert_eq!(
	///     msg,
	///     "\x1b[1;91mError:\x1b[0m Uh oh!"
	/// );
	///
	/// // Downgrade to warning.
	/// msg.set_prefix(MsgKind::Warning);
	/// assert_eq!(
	///     msg,
	///     "\x1b[1;93mWarning:\x1b[0m Uh oh!"
	/// );
	///
	/// // Escalate it to profanity.
	/// msg.set_prefix(("Shit", AnsiColor::Misc199));
	/// assert_eq!(
	///     msg,
	///     "\x1b[1;38;5;199mShit:\x1b[0m Uh oh!"
	/// );
	///
	/// // Remove the prefix altogether.
	/// msg.set_prefix(MsgKind::None);
	/// assert_eq!(msg, "Uh oh!");
	/// ```
	pub fn set_prefix<P: IntoMsgPrefix>(&mut self, prefix: P) {
		self.replace_part(TocId::Prefix, &prefix.prefix_str());
	}

	/// # Set Suffix.
	///
	/// (Re)set the message suffix.
	///
	/// Unlike prefixes, no automatic formatting is applied to suffixes. For
	/// example, if you want a space separating the message content and suffix,
	/// the suffix should start with a leading space.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::from("Checked!");
	/// msg.set_suffix(" ✓");
	///
	/// assert_eq!(
	///     msg,
	///     "Checked! ✓",
	/// );
	pub fn set_suffix<S: AsRef<str>>(&mut self, suffix: S) {
		self.replace_part(TocId::Suffix, suffix.as_ref());
	}

	#[cfg(feature = "timestamps")]
	#[cfg_attr(docsrs, doc(cfg(feature = "timestamps")))]
	/// # Set Timestamp.
	///
	/// Add/remove a timestamp to/from the beginning the of the message.
	///
	/// ## Examples.
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::from("Parsed log.");
	/// msg.set_timestamp(true); // [YYYY-MM-DD hh:mm:ss] Parsed log.
	/// ```
	pub fn set_timestamp(&mut self, enabled: bool) {
		if enabled {
			let now = utc2k::Local2k::now().formatted();
			let mut out = String::with_capacity(43);
			out.push_str(concat!(csi!(dim), "[", csi!(reset, blue)));
			out.push_str(now.as_str());
			out.push_str(concat!(ansi!((reset, dim) "]"), " "));
			self.replace_part(TocId::Timestamp, &out);
		}
		else { self.replace_part(TocId::Timestamp, ""); }
	}

	/// # Strip ANSI Formatting.
	///
	/// Remove colors, bold, etc. from the message.
	///
	/// This is best called last, as changes made after this might reintroduce
	/// fancy formatting.
	///
	/// See also [`Msg::without_ansi`].
	///
	/// Returns `true` if the content was modified.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::info("5,000 matching files were found.");
	/// assert!(msg.strip_ansi());
	///
	/// // Now it reads:
	/// assert_eq!(
	///     msg,
	///     "Info: 5,000 matching files were found.\n",
	/// );
	/// ```
	pub fn strip_ansi(&mut self) -> bool {
		// Iterate through all the parts (except indent and newline), replacing
		// the content as needed.
		let mut changed = false;
		for id in TocId::ANSI_PARTS {
			// The original part location and size.
			let old_rng = self.toc.part_rng(id);
			let old_len = old_rng.len();

			// Remove ANSI in-place across the part range, but wait to
			// reconcile the table of contents until the end.
			let mut removed = 0;
			let mut start = old_rng.start;
			let mut stop = old_rng.end;
			while let Some(mut ansi_rng) = self.inner.get(start..stop).and_then(crate::ansi::next_ansi) {
				// Make the range absolute.
				ansi_rng.start += start;
				ansi_rng.end += start;

				// Update the counters and remove the chunk from the buffer.
				let ansi_len = ansi_rng.len();
				removed += ansi_len;    // We removed the whole range.
				start = ansi_rng.start; // Pick up from here next time around.
				stop -= ansi_len;       // But stop earlier, since we removed shit.
				self.inner.replace_range(ansi_rng, "");
			}

			// Update the table of contents, if necessary.
			if removed != 0 {
				self.toc.resize_part(id, old_len - removed);
				changed = true;
			}
		}

		changed
	}
}

/// ## Builder Setters.
impl Msg {
	#[inline]
	#[must_use]
	/// # With/Without Indentation.
	///
	/// (Re)set the message's indentation level to `tabs` "tabs" (four spaces
	/// each), up to a maximum depth of eight (thirty-two spaces total).
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let msg = Msg::from("Hello world.").with_indent(1);
	/// assert_eq!(msg, "    Hello world.");
	///
	/// let msg = Msg::from("Hello world.").with_indent(2);
	/// assert_eq!(msg, "        Hello world.");
	///
	/// let msg = Msg::from("Hello world.").with_indent(3);
	/// assert_eq!(msg, "            Hello world.");
	///
	/// // …
	///
	/// let msg = Msg::from("Hello world.").with_indent(7);
	/// assert_eq!(msg, "                            Hello world.");
	///
	/// let msg = Msg::from("Hello world.").with_indent(8);
	/// assert_eq!(msg, "                                Hello world.");
	///
	/// let msg = Msg::from("Hello world.").with_indent(9);
	/// assert_eq!(msg, "                                Hello world."); // Same as 8.
	///
	/// // …
	///
	/// let msg = Msg::from("Hello world.").with_indent(u8::MAX);
	/// assert_eq!(msg, "                                Hello world."); // Same as 8.
	/// ```
	pub fn with_indent(mut self, tabs: u8) -> Self {
		self.set_indent(tabs);
		self
	}

	#[inline]
	#[must_use]
	/// # With Message Content.
	///
	/// In most cases where the message content needs to change, [`Msg::set_msg`]
	/// probably makes more sense, but everything else gets a builder method,
	/// so why not?
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let msg = Msg::from("Hello")
	///     .with_msg("Goodbye");
	/// assert_eq!(msg, "Goodbye");
	/// ```
	pub fn with_msg<S: AsRef<str>>(mut self, msg: S) -> Self {
		self.set_msg(msg);
		self
	}

	#[inline]
	#[must_use]
	/// # With/Without Trailing Linebreak.
	///
	/// Add/remove the message's trailing line break.
	///
	/// ## Examples
	///
	/// Messages created with [`Msg::from`], [`Msg::new`], and
	/// [`MsgKind::into_msg`] have no trailing line break by default:
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::from("Hello World!");
	/// assert_eq!(
	///     msg,
	///     "Hello World!",
	/// );
	///
	/// assert_eq!(
	///     msg.with_newline(true), // Add line break.
	///     "Hello World!\n",
	/// );
	/// ```
	///
	/// Messages created with the built-in helper methods, however, do:
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let mut msg = Msg::info("Hello World!");
	/// assert_eq!(
	///     msg,
	///     "\x1b[1;95mInfo:\x1b[0m Hello World!\n",
	/// );
	///
	/// assert_eq!(
	///     msg.with_newline(false), // Remove line break.
	///     "\x1b[1;95mInfo:\x1b[0m Hello World!",
	/// );
	/// ```
	pub fn with_newline(mut self, enabled: bool) -> Self {
		self.set_newline(enabled);
		self
	}

	#[inline]
	#[must_use]
	/// # With Prefix.
	///
	/// (Re/un)set the message prefix.
	///
	/// As with [`Msg::new`], prefixes can be a built-in [`MsgKind`] or custom
	/// string, with or without formatting.
	///
	/// To remove the prefix entirely, pass [`MsgKind::None`] or `""`.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::{Msg, MsgKind};
	///
	/// // A built-in.
	/// assert_eq!(
	///     Msg::from("Uh oh!").with_prefix(MsgKind::Error),
	///     "\x1b[1;91mError:\x1b[0m Uh oh!",
	/// );
	///
	/// // A custom and plain prefix.
	/// assert_eq!(
	///     Msg::from("Uh oh!").with_prefix("Nope"),
	///     "Nope: Uh oh!",
	/// );
	///
	/// // A custom and blue prefix.
	/// assert_eq!(
	///     Msg::from("Uh oh!").with_prefix(("Meh", 4)),
	///     "\x1b[1;34mMeh:\x1b[0m Uh oh!",
	/// );
	/// ```
	pub fn with_prefix<P: IntoMsgPrefix>(mut self, prefix: P) -> Self {
		self.set_prefix(prefix);
		self
	}

	#[inline]
	#[must_use]
	/// # With Suffix.
	///
	/// (Re)set the message suffix.
	///
	/// Unlike prefixes, no automatic formatting is applied to suffixes. For
	/// example, if you want a space separating the message content and suffix,
	/// the suffix should start with a leading space.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let msg = Msg::from("Checked!")
	///     .with_suffix(" ✓");
	///
	/// assert_eq!(
	///     msg,
	///     "Checked! ✓",
	/// );
	pub fn with_suffix<S: AsRef<str>>(mut self, suffix: S) -> Self {
		self.set_suffix(suffix);
		self
	}

	#[cfg(feature = "timestamps")]
	#[cfg_attr(docsrs, doc(cfg(feature = "timestamps")))]
	#[inline]
	#[must_use]
	/// # With/Without Timestamp.
	///
	/// Add/remove a timestamp to/from the beginning the of the message.
	///
	/// ## Examples.
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let msg = Msg::from("Parsed log.")
	///     .with_timestamp(true); // [YYYY-MM-DD hh:mm:ss] Parsed log.
	/// ```
	pub fn with_timestamp(mut self, enabled: bool) -> Self {
		self.set_timestamp(enabled);
		self
	}

	#[must_use]
	/// # Without ANSI Formatting.
	///
	/// Remove colors, bold, etc. from the message.
	///
	/// This is best called last, as changes made after this might reintroduce
	/// fancy formatting.
	///
	/// For unchained usage, see [`Msg::strip_ansi`].
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// assert_eq!(
	///     Msg::info("5,000 matching files were found.").without_ansi(),
	///     "Info: 5,000 matching files were found.\n",
	/// );
	/// ```
	pub fn without_ansi(mut self) -> Self {
		self.strip_ansi();
		self
	}
}

/// ## Printing.
impl Msg {
	#[inline]
	/// # Print to `STDOUT`.
	///
	/// This is a convenience method for printing a message to `STDOUT` without
	/// having to go through the standard library's [`print`] macro.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let msg = Msg::info("Hello World!");
	///
	/// // You've got two choices to print.
	/// print!("{msg}"); // \x1b[1;95mInfo:\x1b[0m Hello World!\n
	/// msg.print();     // \x1b[1;95mInfo:\x1b[0m Hello World!\n
	/// // This line break is embedded in the message itself.   ^
	///
	/// // As such, you probably don't want to do this:
	/// println!("{msg}"); // \x1b[1;95mInfo:\x1b[0m Hello World!\n\n
	///
	/// // Of course, messages don't _have to_ embed the break:
	/// let msg = Msg::info("Hello World!").with_newline(false);
	/// println!("{msg}"); // \x1b[1;95mInfo:\x1b[0m Hello World!\n
	/// ```
	pub fn print(&self) {
		use io::Write;

		let mut handle = io::stdout().lock();
		let _res = handle.write_all(self.as_bytes()).and_then(|()| handle.flush());
	}

	#[inline]
	/// # Print to `STDERR`.
	///
	/// This is a convenience method for printing a message to `STDERR` without
	/// having to go through the standard library's [`eprint`] macro.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_msg::Msg;
	///
	/// let msg = Msg::info("Hello World!");
	///
	/// // You've got two choices to print.
	/// eprint!("{msg}"); // \x1b[1;95mInfo:\x1b[0m Hello World!\n
	/// msg.eprint();     // \x1b[1;95mInfo:\x1b[0m Hello World!\n
	/// // This line break is embedded in the message itself.   ^
	///
	/// // As such, you probably don't want to do this:
	/// eprintln!("{msg}"); // \x1b[1;95mInfo:\x1b[0m Hello World!\n\n
	///
	/// // Of course, messages don't _have to_ embed the break:
	/// let msg = Msg::info("Hello World!").with_newline(false);
	/// eprintln!("{msg}"); // \x1b[1;95mInfo:\x1b[0m Hello World!\n
	/// ```
	pub fn eprint(&self) {
		use io::Write;

		let mut handle = io::stderr().lock();
		let _res = handle.write_all(self.as_bytes()).and_then(|()| handle.flush());
	}

	#[must_use]
	#[inline]
	/// # Prompt.
	///
	/// This produces a simple y/N input prompt, requiring the user type "Y" or
	/// "N" to proceed. Positive values return `true`, negative values return
	/// `false`. The default (if the user just hits `<ENTER>`) is "N".
	///
	/// Note: the prompt normalizes the suffix and newline parts for display.
	/// If your message contains these parts, they will be ignored by the
	/// prompt action, but will be retained in the original struct should you
	/// wish to use it in some other manner later in your code.
	///
	/// Every example in the docs shows this in combination with the built-in
	/// [`MsgKind::Confirm`] prefix, but this can be called on any [`Msg`]
	/// object.
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
	pub fn prompt(&self) -> bool { self.prompt_with_default(false) }

	#[must_use]
	#[inline]
	/// # Prompt (w/ Default).
	///
	/// Same as [`Msg::prompt`], but with the option of specifying the default
	/// return value — `true` for Yes, `false` for No — that will be returned
	/// if the user just hits `<ENTER>`.
	pub fn prompt_with_default(&self, default: bool) -> bool {
		self.prompt__(default, false)
	}

	#[must_use]
	#[inline]
	/// # Prompt (`STDERR`).
	///
	/// Same as [`Msg::prompt`], but printed to `STDERR` instead of `STDOUT`.
	pub fn eprompt(&self) -> bool { self.eprompt_with_default(false) }

	#[must_use]
	#[inline]
	/// # Prompt (w/ Default, `STDERR`).
	///
	/// Same as [`Msg::prompt_with_default`], but printed to `STDERR` instead of
	/// `STDOUT`.
	pub fn eprompt_with_default(&self, default: bool) -> bool {
		self.prompt__(default, true)
	}

	/// # Internal Prompt Handling.
	///
	/// This prints the prompt, handling the desired default and output.
	fn prompt__(&self, default: bool, stderr: bool) -> bool {
		// Clone the message and append a little [y/N] instructional bit to the
		// end. This might not be necessary, but preserves the original message
		// in case it is needed again.
		let q = self.clone()
			.with_suffix(
				if default { concat!(" ", dim!("[", csi!(underline), "Y", csi!(!underline), "/n]"), " ") }
				else       { concat!(" ", dim!("[y/", csi!(underline), "N", csi!(!underline), "]"), " ") },
			)
			.with_newline(false);

		// Ask and collect input, looping until a valid response is typed.
		let mut result = String::new();
		loop {
			if stderr { q.eprint(); }
			else { q.print(); }

			if let Some(res) = io::stdin().read_line(&mut result)
				.ok()
				.and_then(|_| match result.to_lowercase().trim() {
					"" => Some(default),
					"n" | "no" => Some(false),
					"y" | "yes" => Some(true),
					_ => None,
				})
			{ break res; }

			// Print an error and do it all over again.
			result.truncate(0);
			let err = Self::error(concat!(
				"Invalid input; enter ",
				ansi!((light_red) "N"),
				" or ",
				ansi!((light_green) "Y"),
				".",
			));
			if stderr { err.eprint(); }
			else { err.print(); }
		}
	}
}

#[cfg(feature = "progress")]
/// ## Miscellaneous.
impl Msg {
	#[cfg_attr(docsrs, doc(cfg(feature = "progress")))]
	#[must_use]
	/// # Bytes Saved Suffix.
	///
	/// A lot of our own programs using this lib crunch data and report the
	/// savings as a suffix. This method just provides a quick way to generate
	/// that.
	pub fn with_bytes_saved(mut self, state: BeforeAfter) -> Self {
		use dactyl::{NicePercent, NiceU64};
		use fyi_ansi::csi;

		if let Some(saved) = state.less() {
			let saved = NiceU64::from(saved);
			let buf = state.less_percent().map_or_else(
				// Just the bytes.
				|| {
					let mut buf = String::with_capacity(24 + saved.len());
					buf.push_str(concat!(" ", csi!(dim), "(Saved "));
					buf.push_str(saved.as_str());
					buf.push_str(concat!(" bytes.)", csi!()));
					buf
				},
				// With percent.
				|per| {
					let per = NicePercent::from(per);
					let mut buf = String::with_capacity(26 + saved.len() + per.len());
					buf.push_str(concat!(" ", csi!(dim), "(Saved "));
					buf.push_str(saved.as_str());
					buf.push_str(" bytes, ");
					buf.push_str(per.as_str());
					buf.push_str(concat!(".)", csi!()));
					buf
				}
			);

			self.replace_part(TocId::Suffix, &buf);
		}
		else {
			self.replace_part(
				TocId::Suffix,
				concat!(" ", dim!("(No savings.)")),
			);
		}

		self
	}
}

/// # Internal.
impl Msg {
	/// # Replace Part.
	fn replace_part(&mut self, id: TocId, content: &str) {
		// Update the content.
		let rng = self.toc.part_rng(id);
		self.inner.replace_range(rng, content);

		// Update the table of contents.
		self.toc.resize_part(id, content.len());
	}
}



#[derive(Debug, Clone, Copy, Default)]
/// # Table of Contents.
struct Toc([usize; Self::SIZE]);

impl Toc {
	/// # Table of Contents Size.
	///
	/// The struct's inner array holds the starting positions for each part,
	/// plus an extra holding the exclusive end.
	///
	/// The specific number of parts varies by crate feature, but
	/// [`TocId::Newline`] is always last. Add one for the extra end "part",
	/// and one more to convert index to length.
	const SIZE: usize = TocId::Newline as usize + 2;

	/// # Part Length.
	const fn part_len(&self, id: TocId) -> Option<NonZeroUsize> {
		let start = self.0[id as usize];   // All TocIds are in range.
		let end = self.0[id as usize + 1]; // And so is +1.
		if let Some(len) = end.checked_sub(start) { NonZeroUsize::new(len) }
		else { None }
	}

	/// # Part Range.
	const fn part_rng(&self, id: TocId) -> Range<usize> {
		let start = self.0[id as usize];
		let end = self.0[id as usize + 1];
		start..end
	}

	/// # Resize Part(s).
	///
	/// Change the size of `part`, realigning the subsequent boundaries as
	/// needed.
	fn resize_part(&mut self, id: TocId, new_len: usize) {
		let old_len = self.part_len(id).map_or(0, NonZeroUsize::get);

		match old_len.cmp(&new_len) {
			// The new length is bigger; increase the remaining positions.
			Ordering::Less => {
				let diff = new_len - old_len;
				for v in self.0.iter_mut().skip(id as usize + 1) { *v += diff; }
			},
			// The new length is smaller; decrease the remaining positions.
			Ordering::Greater => {
				let diff = old_len - new_len;
				for v in self.0.iter_mut().skip(id as usize + 1) { *v -= diff; }
			},
			// No change.
			Ordering::Equal => {},
		}
	}
}



#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # Table of Contents: Parts.
///
/// This enum holds the name/index of each [`Toc`] "chapter".
enum TocId {
	/// # Indentation.
	Indent = 0_u8,

	#[cfg(feature = "timestamps")]
	/// # Timestamps.
	Timestamp = 1_u8,

	#[cfg(feature = "timestamps")]
	/// # Prefix.
	Prefix = 2_u8,

	#[cfg(feature = "timestamps")]
	/// # Message.
	Message = 3_u8,

	#[cfg(feature = "timestamps")]
	/// # Suffix.
	Suffix = 4_u8,

	#[cfg(feature = "timestamps")]
	/// # Line Break.
	Newline = 5_u8,

	#[cfg(not(feature = "timestamps"))]
	/// # Prefix.
	Prefix = 1_u8,

	#[cfg(not(feature = "timestamps"))]
	/// # Message.
	Message = 2_u8,

	#[cfg(not(feature = "timestamps"))]
	/// # Suffix.
	Suffix = 3_u8,

	#[cfg(not(feature = "timestamps"))]
	/// # Line Break.
	Newline = 4_u8,
}

impl TocId {
	#[cfg(feature = "timestamps")]
	/// # Parts w/ ANSI.
	///
	/// These parts _might_ have formatting.
	const ANSI_PARTS: [Self; 4] = [
		Self::Timestamp, Self::Prefix, Self::Message, Self::Suffix,
	];

	#[cfg(not(feature = "timestamps"))]
	/// # Parts w/ ANSI.
	///
	/// These parts _might_ have formatting.
	const ANSI_PARTS: [Self; 3] = [Self::Prefix, Self::Message, Self::Suffix];
}



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn t_strip_ansi() {
		// Create an ANSIfull message and make sure its various parts turn out
		// as expected.
		let mut msg = Msg::error("Oops: \x1b[2mDaisy\x1b[0m.")
			.with_suffix(" \x1b[1mYikes!\x1b[0m");
		assert_eq!(
			msg,
			"\x1b[1;91mError:\x1b[0m Oops: \x1b[2mDaisy\x1b[0m. \x1b[1mYikes!\x1b[0m\n",
		);
		assert_eq!(
			&msg.inner[msg.toc.part_rng(TocId::Prefix)],
			"\x1b[1;91mError:\x1b[0m ",
		);
		assert_eq!(
			&msg.inner[msg.toc.part_rng(TocId::Message)],
			"Oops: \x1b[2mDaisy\x1b[0m.",
		);
		assert_eq!(
			&msg.inner[msg.toc.part_rng(TocId::Suffix)],
			" \x1b[1mYikes!\x1b[0m",
		);
		assert_eq!(
			&msg.inner[msg.toc.part_rng(TocId::Newline)],
			"\n",
		);

		// Strip the ANSI and reverify.
		msg.strip_ansi();
		assert_eq!(
			msg,
			"Error: Oops: Daisy. Yikes!\n",
		);
		assert_eq!(
			&msg.inner[msg.toc.part_rng(TocId::Prefix)],
			"Error: ",
		);
		assert_eq!(
			&msg.inner[msg.toc.part_rng(TocId::Message)],
			"Oops: Daisy.",
		);
		assert_eq!(
			&msg.inner[msg.toc.part_rng(TocId::Suffix)],
			" Yikes!",
		);
		assert_eq!(
			&msg.inner[msg.toc.part_rng(TocId::Newline)],
			"\n",
		);

		// Make sure double-stripping doesn't break anything.
		assert_eq!(
			msg.clone().without_ansi(),
			msg,
		);
	}
}
