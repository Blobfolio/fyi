/*!
# FYI Core: Msg
*/

use bytes::{
	BytesMut,
	BufMut
};
use crate::{
	MSG_TIMESTAMP,
	PRINT_NEWLINE,
	PRINT_STDERR,
	traits::{
		AnsiBitsy,
		Elapsed,
		Inflection,
		MebiSaved,
	},
	util::{
		cli,
		strings,
	},
};
use num_format::{
	Locale,
	ToFormattedString,
};
use std::{
	borrow::Cow,
	fmt,
	time::Instant,
};



#[derive(Debug, Clone, PartialEq)]
/// Generic message.
pub enum Prefix<'mp> {
	/// Custom.
	Custom(Cow<'mp, str>),
	/// Debug.
	Debug,
	/// Error.
	Error,
	/// Info.
	Info,
	/// Notice.
	Notice,
	/// Success.
	Success,
	/// Warning.
	Warning,
	/// None.
	None,
}

impl Default for Prefix<'_> {
	/// Default.
	fn default() -> Self {
		Prefix::None
	}
}

impl fmt::Display for Prefix<'_> {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}

impl<'mp> Prefix<'mp> {
	/// Custom Prefix.
	pub fn new<S> (msg: S, color: u8) -> Self
	where S: Into<Cow<'mp, str>> {
		let msg = msg.into();
		match msg.is_empty() {
			true => Self::None,
			false => {
				let mut out: String = String::with_capacity(msg.len() + 19);
				out.push_str("\x1B[1;38;5;");
				itoa::fmt(&mut out, color).unwrap();
				out.push('m');
				out.push_str(&msg);
				out.push_str(":\x1B[0m ");
				Self::Custom(out.into())
			},
		}
	}

	/// Happy or sad?
	pub fn happy(&self) -> bool {
		match *self {
			Self::Error | Self::Warning => false,
			_ => true,
		}
	}

	/// Print the prefix!
	pub fn as_bytes(&'mp self) -> &'mp [u8] {
		match *self {
			Self::Custom(ref p) => p.as_bytes(),
			Self::Debug => "\x1B[96;1mDebug:\x1B[0m ".as_bytes(),
			Self::Error => "\x1B[91;1mError:\x1B[0m ".as_bytes(),
			Self::Info => "\x1B[96;1mInfo:\x1B[0m ".as_bytes(),
			Self::Notice => "\x1B[95;1mNotice:\x1B[0m ".as_bytes(),
			Self::Success => "\x1B[92;1mSuccess:\x1B[0m ".as_bytes(),
			Self::Warning => "\x1B[93;1mWarning:\x1B[0m ".as_bytes(),
			_ => "".as_bytes(),
		}
	}

	/// Print the prefix!
	pub fn prefix(&'mp self) -> Cow<'mp, str> {
		match *self {
			Self::Custom(ref p) => Cow::Borrowed(p),
			Self::Debug => "\x1B[96;1mDebug:\x1B[0m ".into(),
			Self::Error => "\x1B[91;1mError:\x1B[0m ".into(),
			Self::Info => "\x1B[96;1mInfo:\x1B[0m ".into(),
			Self::Notice => "\x1B[95;1mNotice:\x1B[0m ".into(),
			Self::Success => "\x1B[92;1mSuccess:\x1B[0m ".into(),
			Self::Warning => "\x1B[93;1mWarning:\x1B[0m ".into(),
			_ => "".into(),
		}
	}

	/// Prefix length.
	pub fn is_empty(&'mp self) -> bool {
		match *self {
			Self::None => true,
			_ => false,
		}
	}

	/// Prefix length.
	pub fn len(&'mp self) -> usize {
		match *self {
			Self::Custom(ref p) => p.len(),
			Self::Debug => 18,
			Self::Error => 18,
			Self::Info => 17,
			Self::Notice => 19,
			Self::Success => 20,
			Self::Warning => 20,
			_ => 0,
		}
	}

	/// Prefix width.
	pub fn width(&'mp self) -> usize {
		match *self {
			Self::Custom(ref p) => p.width(),
			Self::Debug => 7,
			Self::Error => 7,
			Self::Info => 6,
			Self::Notice => 8,
			Self::Success => 9,
			Self::Warning => 9,
			_ => 0,
		}
	}
}

impl AsRef<str> for Prefix<'_> {
	/// As Ref String.
	fn as_ref(&self) -> &str {
		match *self {
			Self::Custom(ref p) => p,
			Self::Debug => "\x1B[96;1mDebug:\x1B[0m ",
			Self::Error => "\x1B[91;1mError:\x1B[0m ",
			Self::Info => "\x1B[96;1mInfo:\x1B[0m ",
			Self::Notice => "\x1B[95;1mNotice:\x1B[0m ",
			Self::Success => "\x1B[92;1mSuccess:\x1B[0m ",
			Self::Warning => "\x1B[93;1mWarning:\x1B[0m ",
			_ => "",
		}
	}
}



#[derive(Debug, Default, Clone)]
/// Message.
pub struct Msg<'m> {
	indent: u8,
	prefix: Prefix<'m>,
	msg: Cow<'m, str>,
	flags: u8,
}

impl fmt::Display for Msg<'_> {
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.msg())
	}
}

impl<'m> Msg<'m> {
	/// New.
	pub fn new<S> (msg: S) -> Self
	where S: Into<Cow<'m, str>> {
		Msg {
			msg: msg.into(),
			..Msg::default()
		}
	}

	/// Set Flags.
	pub fn with_flags(mut self, flags: u8) -> Self {
		self.flags = flags;
		self
	}

	/// Set Indentation.
	pub fn with_indent(mut self, indent: u8) -> Self {
		self.indent = indent;
		self
	}

	/// Set Prefix.
	pub fn with_prefix(mut self, prefix: Prefix<'m>) -> Self {
		self.prefix = prefix;
		self
	}



	// -------------------------------------------------------------
	// Getters
	// -------------------------------------------------------------

	/// Msg.
	pub fn msg(&self) -> Cow<'_, str> {
		let mut buf: BytesMut = BytesMut::with_capacity(256);
		let mut width: usize = 0;

		if 0 != self.indent {
			width += self._msg_put_indent(&mut buf);
		}
		if ! self.prefix.is_empty() {
			width += self._msg_put_prefix(&mut buf);
		}
		if ! self.msg.is_empty() {
			width += self._msg_put_msg(&mut buf);
		}
		if 0 != (MSG_TIMESTAMP & self.flags) {
			self._msg_put_timestamp(&mut buf, width);
		}

		Cow::Owned(unsafe { String::from_utf8_unchecked(buf.to_vec()) })
	}

	/// Msg indent.
	fn _msg_put_indent(&self, buf: &mut BytesMut) -> usize {
		let indent: usize = self.indent as usize * 4;
		buf.put(strings::whitespace(indent).as_bytes());
		indent
	}

	/// Msg prefix.
	fn _msg_put_prefix(&self, buf: &mut BytesMut) -> usize {
		buf.put(self.prefix.as_bytes());
		self.prefix.width()
	}

	/// Msg.
	fn _msg_put_msg(&self, buf: &mut BytesMut) -> usize {
		buf.extend_from_slice(b"\x1B[1m");
		buf.put(self.msg.as_bytes());
		buf.extend_from_slice(b"\x1B[0m");
		self.msg.width() + 8
	}

	/// Timestamp.
	fn _msg_put_timestamp(&self, buf: &mut BytesMut, old_width: usize) {
		let width: usize = cli::term_width();
		let ts: String = chrono::Local::now().format("%F %T").to_string();
		let ts_width: usize = ts.len() + 25;

		// Can it fit on one line?
		if width >= old_width + ts_width + 1 {
			buf.put(strings::whitespace(width - ts_width - old_width).as_bytes());

			buf.extend_from_slice(b"\x1B[2m[\x1B[34;2m");
			buf.put(ts.as_bytes());
			buf.extend_from_slice(b"\x1B[0m\x1B[2m]\x1B[0m");
		}
		// Well shit.
		else {
			let b = buf.split();
			if 0 != self.indent {
				self._msg_put_indent(buf);
			}
			buf.extend_from_slice(b"\x1B[2m[\x1B[34;2m");
			buf.put(ts.as_bytes());
			buf.extend_from_slice(b"\x1B[0m\x1B[2m]\x1B[0m\n");
			buf.unsplit(b);
		}
	}

	/// Prefix.
	pub fn prefix(&self) -> Prefix {
		self.prefix.clone()
	}



	// -------------------------------------------------------------
	// Misc Operations
	// -------------------------------------------------------------

	#[cfg(feature = "interactive")]
	/// Prompt instead.
	pub fn prompt(&self) -> bool {
		casual::confirm(&[
			"\x1B[93;1mConfirm:\x1B[0m \x1B[1m",
			&self.msg,
			"\x1B[0m",
		].concat())
	}

	/// Print.
	pub fn print(&self) {
		let mut flags: u8 = self.flags | PRINT_NEWLINE;
		if false == self.prefix.happy() {
			flags |= PRINT_STDERR;
		}

		cli::print(self.msg(), flags);
	}

	// -------------------------------------------------------------
	// Message Templates
	// -------------------------------------------------------------

	/// Template: Crunched In X.
	pub fn crunched_in(num: u64, time: Instant, du: Option<(u64, u64)>) -> Self {
		let elapsed = time.elapsed().as_secs().elapsed();

		Msg::new(Cow::Owned(match du {
			Some((before, after)) => [
				num.inflect("file", "files").as_ref(),
				" in ",
				&elapsed,
				&match before.saved(after) {
					0 => ", but no dice".to_string(),
					x => format!(
						", saving {} bytes ({:3.*}%)",
						x.to_formatted_string(&Locale::en),
						2,
						(1.0 - (after as f64 / before as f64)) * 100.0
					),
				},
				".",
			].concat(),
			None => [
				num.inflect("file", "files").as_ref(),
				" in ",
				&elapsed,
				".",
			].concat(),
		}))
			.with_prefix(Prefix::new("Crunched", 2))
	}

	/// Template: Finished In X.
	pub fn finished_in(time: Instant) -> Self {
		Msg::new(Cow::Owned([
			"Finished in ",
			&time.elapsed().as_secs().elapsed(),
			".",
		].concat()))
			.with_prefix(Prefix::new("Crunched", 2))
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn prefix() {
		assert_eq!(Prefix::Debug.prefix().strip_ansi(), "Debug: ");
		assert_eq!(Prefix::Error.prefix().strip_ansi(), "Error: ");
		assert_eq!(Prefix::Info.prefix().strip_ansi(), "Info: ");
		assert_eq!(Prefix::None.prefix().strip_ansi(), "");
		assert_eq!(Prefix::Notice.prefix().strip_ansi(), "Notice: ");
		assert_eq!(Prefix::Success.prefix().strip_ansi(), "Success: ");
		assert_eq!(Prefix::Warning.prefix().strip_ansi(), "Warning: ");
	}

	#[test]
	fn prefix_happy() {
		assert!(Prefix::Debug.happy());
		assert!(Prefix::Info.happy());
		assert!(Prefix::None.happy());
		assert!(Prefix::Notice.happy());
		assert!(Prefix::Success.happy());

		// These two are sad.
		assert!(! Prefix::Error.happy());
		assert!(! Prefix::Warning.happy());
	}

	#[test]
	fn prefix_new() {
		assert_eq!(Prefix::new("", 199), Prefix::None);
		assert_eq!(Prefix::new("Hello", 199).prefix(), "\x1B[1;38;5;199mHello:\x1B[0m ");
		assert_eq!(Prefix::new("Hello", 2).prefix(), "\x1B[1;38;5;2mHello:\x1B[0m ");
	}

	#[test]
	fn msg_new() {
		// Just a message.
		let msg: Msg = Msg::new("Hello World");
		assert_eq!(msg.msg(), "\x1B[1mHello World\x1B[0m");
		assert_eq!(msg.prefix(), Prefix::None);

		// With prefix.
		let msg: Msg = Msg::new("Hello World")
			.with_prefix(Prefix::Success);
		assert_eq!(msg.msg(), "\x1B[92;1mSuccess:\x1B[0m \x1B[1mHello World\x1B[0m");
		assert_eq!(msg.prefix(), Prefix::Success);

		// Indentation. We've tested color enough now; let's strip ANSI
		// to make this more readable.
		let msg: Msg = Msg::new("Hello World")
			.with_indent(1);
		assert_eq!(msg.msg().strip_ansi(), "    Hello World");
		let msg: Msg = Msg::new("Hello World")
			.with_indent(2);
		assert_eq!(msg.msg().strip_ansi(), "        Hello World");
	}
}
