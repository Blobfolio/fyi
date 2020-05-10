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
	traits::GirthExt,
};
use std::io::Write;



bitflags::bitflags! {
	/// Print flags.
	pub struct Flags: u16 {
		/// Empty flag.
		const NONE        = 0b0000;
		/// Do not append a new line.
		const NO_LINE     = 0b0010;
		/// Include a local timestamp.
		const TIMESTAMPED = 0b0100;
		/// Send to `Stderr` instead of `Stdout`.
		const TO_STDERR   = 0b1000;
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

macro_rules! print_format_indent_and_timestamp {
	($data:ident, $indent:ident) => {{
		let tmp: usize = $indent.saturating_mul(4) as usize;
		&[
			whitespace(tmp),
			$data,
			match term_width().saturating_sub(tmp + $data.count_width() + 21) {
				0 => &b"\n"[..],
				space => whitespace(space)
			},
			&Timestamp::new(),
		].concat()
	}};
}

macro_rules! print_format {
	($handle:ident, $data:ident, $indent:ident, $flags:ident) => {
		// Indentation is annoying. Let's get it over with. Haha.
		if $indent > 0 {
			if $flags.contains(Flags::TIMESTAMPED) {
				print_to(
					&mut $handle,
					print_format_indent_and_timestamp!($data, $indent),
					$flags
				)
			}
			else {
				print_to(
					&mut $handle,
					print_format_indent!($data, $indent),
					$flags
				)
			}
		}
		// Timetsamp?
		else if $flags.contains(Flags::TIMESTAMPED) {
			print_to(
				&mut $handle,
				print_format_timestamp!($data),
				$flags
			)
		}
		else {
			print_to(&mut $handle, $data, $flags)
		}
	};
}

/// Prepare message for print.
///
/// This method mutates the data according to the prescribed flags, then sends
/// it to the right writer for output.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
pub unsafe fn print(data: &[u8], indent: u8, flags: Flags) {
	// Print to `Stderr`.
	if flags.contains(Flags::TO_STDERR) {
		_print_stderr(data, indent, flags);
	}
	// Print to `Stdout`.
	else {
		_print_stdout(data, indent, flags);
	}
}

/// Print To.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
pub unsafe fn print_to<W: Write> (writer: &mut W, data: &[u8], flags: Flags) -> bool {
	if ! flags.contains(Flags::NO_LINE) {
		writeln!(writer, "{}", std::str::from_utf8_unchecked(data)).is_ok()
	}
	else if writer.write_all(data).is_ok() {
		writer.flush().is_ok()
	}
	else { false }
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
pub unsafe fn prompt(data: &[u8], indent: u8) -> bool {
	// Attach the indent and recurse.
	if 0 < indent {
		prompt(print_format_indent!(data, indent), 0)
	}
	else {
		casual::confirm(std::str::from_utf8_unchecked(data))
	}
}

#[must_use]
/// Term Width
pub fn term_width() -> usize {
	// Reserve one space at the end "just in case".
	if let Some((w, _)) = term_size::dimensions() { w.saturating_sub(1) }
	else { 0 }
}

#[must_use]
/// Whitespace maker.
pub fn whitespace(num: usize) -> &'static [u8] {
	static WHITES: &[u8; 255] = &[b' '; 255];

	if num >= 255 { &WHITES[..] }
	else { &WHITES[0..num] }
}



#[cfg(not(feature = "stdout_sinkhole"))]
/// Print `Stdout`.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
unsafe fn _print_stdout(data: &[u8], indent: u8, flags: Flags) -> bool {
	let writer = std::io::stdout();
	let mut handle = writer.lock();
	print_format!(handle, data, indent, flags)
}

#[cfg(feature = "stdout_sinkhole")]
/// Print `Sink`.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
unsafe fn _print_stdout(data: &[u8], indent: u8, flags: Flags) -> bool {
	let mut handle = std::io::sink();
	print_format!(handle, data, indent, flags)
}

/// Print `Stderr`.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
unsafe fn _print_stderr(data: &[u8], indent: u8, flags: Flags) -> bool {
	let writer = std::io::stderr();
	let mut handle = writer.lock();
	print_format!(handle, data, indent, flags)
}
