/*!
# FYI Printer

The `Printer` is an interface for sending shit to `Stdout`/`Stderr` with
various possible formatting options (see also: `Flags`).

There is a corresponding trait `Printable` that `Msg` and strings implement
to make the process more ergonomical, but the methods here can be called
directly.

Most of the methods in this module are unsafe because they accept `[u8]` inputs
that are expected to be valid UTF-8. `Msg`, etc., performs that validation, but
if for some reason you find a separate use case for these highly-specific
methods, make sure your strings are right before printing.
*/

use crate::{
	Timestamp,
	traits::GirthExt,
	utility,
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

/// Time-saving Macro #1.
macro_rules! print_format_indent {
	($data:ident, $indent:ident) => (
		&[
			utility::whitespace($indent.saturating_mul(4) as usize),
			$data,
		].concat()
	);
}

/// Time-saving Macro #2.
macro_rules! print_format_timestamp {
	($data:ident) => (
		&[
			$data,
			match utility::term_width().saturating_sub($data.count_width() + 21) {
				0 => &b"\n"[..],
				space => utility::whitespace(space)
			},
			&Timestamp::new(),
		].concat()
	);
}

/// Time-saving Macro #3.
macro_rules! print_format_indent_and_timestamp {
	($data:ident, $indent:ident) => {{
		let tmp: usize = $indent.saturating_mul(4) as usize;
		&[
			utility::whitespace(tmp),
			$data,
			match utility::term_width().saturating_sub(tmp + $data.count_width() + 21) {
				0 => &b"\n"[..],
				space => utility::whitespace(space)
			},
			&Timestamp::new(),
		].concat()
	}};
}

/// Time-saving Macro #4.
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
/// This method completes the write to the specified writer. By this point, all
/// formatting-type modifications have already been made, except for the
/// trailing new line (disabled via `Flags::NO_LINE`); that one tweak is made
/// here.
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
/// As we aren't doing the heavy lifting here, there is no support for `Flags`,
/// however prompt messages can be indented.
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



#[cfg(not(feature = "stdout_sinkhole"))]
/// Print `Stdout`.
///
/// This is a convenience wrapper to pass data to `print_to()` using `stdout()`
/// as the writer.
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
/// The `stdout_sinkhole` flag is set during benchmarking in order to silently
/// replace `stdout()` with a `sink()`. This lets us get decent comparative
/// measurements without drowning the display in thousands of prints.
///
/// If building `FYI` manually, make sure not to set this feature. Haha.
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
/// This is a convenience wrapper to pass data to `print_to()` using `stderr()`
/// as the writer.
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
