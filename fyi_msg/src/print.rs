/*!
# FYI Printer

The `Printer` is an interface for sending shit to `Stdout`/`Stderr` with
various possible formatting options (see e.g. the `Flags`).

There is a corresponding trait `Printable` that `Msg` and strings implement
to make the process more ergonomical, but the methods here can be called
directly.
*/

use crate::{
	Timestamp,
	traits::{
		GirthExt,
		StripAnsi,
	},
};
use std::io::Write;



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

macro_rules! print_format_indent {
	($data:ident, $indent:ident) => (
		&[
			whitespace($indent.saturating_mul(4) as usize),
			$data,
		].concat()
	);
}

macro_rules! print_format_timestamp {
	($data:ident) => (
		&[
			$data,
			match term_width().saturating_sub($data.count_width() + 21) {
				0 => &b"\n"[..],
				space => whitespace(space)
			},
			&Timestamp::new(),
		].concat()
	);
}

/// Prepare message for print.
///
/// This method mutates the data according to the prescribed flags, then sends
/// it to the right writer for output.
///
/// If `Flags::TO_NOWHERE` is set, this will still build the message, it just
/// won't do the writer part. This helps with benchmarking, etc., but is
/// admittedly useless in production.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
pub unsafe fn print(data: &[u8], indent: u8, flags: Flags) {
	// Indentation is annoying. Let's get it over with. Haha.
	if indent > 0 {
		print(print_format_indent!(data, indent), 0, flags)
	}
	// Timetsamp?
	else if flags.contains(Flags::TIMESTAMPED) {
		print(print_format_timestamp!(data), 0, flags & !Flags::TIMESTAMPED)
	}
	// Print to `Stderr`.
	else if flags.contains(Flags::TO_STDERR) {
		_print_stderr(data, flags);
	}
	// Print to `Stdout`.
	else {
		_print_stdout(data, flags);
	}
}

#[cfg(feature = "interactive")]
#[must_use]
/// Prompt.
///
/// This is a simple print wrapper around `casual::confirm()`.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
pub unsafe fn prompt(data: &[u8], indent: u8, flags: Flags) -> bool {
	// Attach the indent and recurse.
	if 0 < indent {
		prompt(print_format_indent!(data, indent), 0, flags)
	}
	// Strip ANSI and recurse.
	else if flags.contains(Flags::NO_ANSI) {
		prompt(&data.strip_ansi(), 0, flags & !Flags::NO_ANSI)
	}
	else {
		casual::confirm(std::str::from_utf8_unchecked(data))
	}
}

#[must_use]
/// Term Width
pub fn term_width() -> usize {
	// Reserve one space at the end "just in case".
	if let Some((w, _)) = term_size::dimensions() { w - 1 }
	else { 0 }
}

#[must_use]
/// Whitespace maker.
pub fn whitespace(num: usize) -> &'static [u8] {
	static WHITES: &[u8; 255] = &[b' '; 255];

	if num >= 255 { &WHITES[..] }
	else { &WHITES[0..num] }
}



/// Print `Stdout`.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
unsafe fn _print_stdout(data: &[u8], flags: Flags) -> bool {
	// Strip ANSI?
	if flags.contains(Flags::NO_ANSI) {
		return _print_stdout(&data.strip_ansi(), flags & !Flags::NO_ANSI);
	}

	// Emulate write with a sink.
	if flags.contains(Flags::TO_NOWHERE) {
		let mut handle = std::io::sink();
		if ! flags.contains(Flags::NO_LINE) {
			writeln!(handle, "{}", std::str::from_utf8_unchecked(data)).is_ok()
		}
		else if handle.write_all(data).is_ok() {
			handle.flush().is_ok()
		}
		else { false }
	}
	// Go to `Stdout` proper.
	else {
		let writer = std::io::stdout();
		let mut handle = writer.lock();
		if ! flags.contains(Flags::NO_LINE) {
			writeln!(handle, "{}", std::str::from_utf8_unchecked(data)).is_ok()
		}
		else if handle.write_all(data).is_ok() {
			handle.flush().is_ok()
		}
		else { false }
	}
}

/// Print `Stderr`.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
unsafe fn _print_stderr(data: &[u8], flags: Flags) -> bool {
	// Strip ANSI?
	if flags.contains(Flags::NO_ANSI) {
		return _print_stderr(&data.strip_ansi(), flags & !Flags::NO_ANSI);
	}

	let writer = std::io::stderr();
	let mut handle = writer.lock();
	if ! flags.contains(Flags::NO_LINE) {
		writeln!(handle, "{}", std::str::from_utf8_unchecked(data)).is_ok()
	}
	else if handle.write_all(data).is_ok() {
		handle.flush().is_ok()
	}
	else { false }
}
