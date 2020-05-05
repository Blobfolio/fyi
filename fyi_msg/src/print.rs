/*!
# FYI Printer

The `Printer` is an interface for sending shit to `Stdout`/`Stderr` with
various possible formatting options (see e.g. the `Flags`).

There is a corresponding trait `Printable` that `Msg` and strings implement
to make the process more ergonomical, but the methods here can be called
directly.
*/

use bytes::{
	BytesMut,
	BufMut,
};
use crate::traits::{
	DoubleTime,
	GirthExt,
	StripAnsi,
	WhiteSpace,
};
use std::borrow::Cow;
use chrono::{
	Datelike,
	Local,
	Timelike,
};



bitflags::bitflags! {
	/// Print flags.
	pub struct Flags: u32 {
		/// Empty flag.
		const NONE        = 0b0000_0000;
		/// Strip ANSI formatting.
		const NO_ANSI     = 0b0000_0001;
		/// Do not append a new line.
		const NO_LINE     = 0b0000_0010;
		/// Include a local timestamp.
		const TIMESTAMPED = 0b0000_0100;
		/// Do not actually print anything. This is meant for debugging only.
		const TO_NOWHERE  = 0b0000_1000;
		/// Send to `Stderr` instead of `Stdout`.
		const TO_STDERR   = 0b0001_0000;
	}
}

impl Default for Flags {
	/// Default.
	fn default() -> Self {
		Flags::NONE
	}
}

/// Prepare message for print.
///
/// This method mutates the data according to the prescribed flags, then sends
/// it to the right writer for output.
///
/// If `Flags::TO_NOWHERE` is set, this will still build the message, it just
/// won't do the writer part. This helps with benchmarking, etc., but is
/// admittedly useless in production.
pub fn print(data: &[u8], indent: u8, flags: Flags) {
	let mut buf = BytesMut::with_capacity(512);

	// Start with indentation.
	if 0 < indent {
		buf.extend_from_slice(<[u8]>::whitespace(indent.saturating_mul(4)));
	}

	// Add the message.
	if ! data.is_empty() {
		buf.extend_from_slice(data);
	}

	// Add a timestamp?
	if flags.contains(Flags::TIMESTAMPED) {
		_print_put_timestamp(&mut buf);
	}

	// Add a newline?
	if ! flags.contains(Flags::NO_LINE) {
		buf.put_u8(b'\n');
	}

	// No ANSI?
	if flags.contains(Flags::NO_ANSI) {
		if flags.contains(Flags::TO_STDERR) {
			_print_stderr(&buf.strip_ansi());
		}
		else if ! flags.contains(Flags::TO_NOWHERE) {
			_print_stdout(&buf.strip_ansi());
		}
	}
	// Regular way!
	else if flags.contains(Flags::TO_STDERR) {
		_print_stderr(&buf);
	}
	else if ! flags.contains(Flags::TO_NOWHERE) {
		_print_stdout(&buf);
	}
}

#[cfg(feature = "interactive")]
#[must_use]
/// Prompt.
pub fn prompt(data: &[u8], indent: u8, flags: Flags) -> bool {
	let mut buf = BytesMut::with_capacity(512);

	// Start with indentation.
	if 0 < indent {
		buf.extend_from_slice(<[u8]>::whitespace(indent.saturating_mul(4)));
	}

	// Add the message.
	if ! data.is_empty() {
		if flags.contains(Flags::NO_ANSI) {
			buf.extend_from_slice(data.strip_ansi().as_ref());
		}
		else {
			buf.extend_from_slice(data);
		}
	}

	// Timestamps, etc., are unsupported by this method.

	if flags.contains(Flags::TO_NOWHERE) { false }
	else {
		casual::confirm(unsafe {
			std::str::from_utf8_unchecked(&buf)
		})
	}
}

/// Push Timestamp.
fn _print_put_timestamp(buf: &mut BytesMut) {
	use std::fmt::Write;

	lazy_static::lazy_static! {
		static ref YEAR: Cow<'static, str> = {
			let mut tmp: String = String::with_capacity(4);
			itoa::fmt(&mut tmp, Local::now().year()).expect("Invalid year.");
			Cow::Owned(tmp)
		};
	}

	let cli_width: usize = term_width();
	let msg_width: usize = buf.count_width();
	let now = Local::now();

	// We can fit it on one line.
	if cli_width > msg_width + 21 {
		buf.extend_from_slice(<[u8]>::whitespace((cli_width - msg_width - 21) as u8));
	}
	else {
		buf.put_u8(b'\n');
	}

	write!(
		buf,
		"\x1B[2m[\x1B[34m{}-{}-{} {}:{}:{}\x1B[39m]\x1B[0m",
		YEAR.as_ref(),
		str::double_digit_time(now.month().into()),
		str::double_digit_time(now.day().into()),
		str::double_digit_time(now.hour().into()),
		str::double_digit_time(now.minute().into()),
		str::double_digit_time(now.second().into()),
	).expect("Invalid timestamp.");
}

/// Print `Stdout`.
fn _print_stdout(data: &[u8]) -> bool {
	use std::io::Write;

	let writer = std::io::stdout();
	let mut handle = writer.lock();
	if handle.write_all(data).is_ok() {
		handle.flush().is_ok()
	}
	else {
		false
	}
}

/// Print `Stderr`.
fn _print_stderr(data: &[u8]) -> bool {
	use std::io::Write;

	let writer = std::io::stderr();
	let mut handle = writer.lock();
	if handle.write_all(data).is_ok() {
		handle.flush().is_ok()
	}
	else {
		false
	}
}

#[must_use]
/// Term Width
pub fn term_width() -> usize {
	// Reserve one space at the end "just in case".
	if let Some((w, _)) = term_size::dimensions() { w - 1 }
	else { 0 }
}
