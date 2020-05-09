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
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
pub unsafe fn print(data: &[u8], indent: u8, flags: Flags) {
	// Indentation is annoying. Let's get it over with. Haha.
	if indent > 0 {
		return print(
			&[
				whitespace(indent.saturating_mul(4) as usize),
				data,
			].concat(),
			0,
			flags
		);
	}

	// Timestamps are heavy enough to demand their own specialized creation.
	if flags.contains(Flags::TIMESTAMPED) {
		_print_timestamped(data, flags);
	}
	// Print to `Stderr`.
	else if flags.contains(Flags::TO_STDERR) {
		_print_stderr(data, flags);
	}
	// Print to `Stdout`.
	else if ! flags.contains(Flags::TO_NOWHERE) {
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
		return prompt(
			&[
				whitespace(indent.saturating_mul(4) as usize),
				data,
			].concat(),
			0,
			flags
		);
	}

	// Strip ANSI and recurse.
	if flags.contains(Flags::NO_ANSI) {
		return prompt(&data.strip_ansi(), 0, flags & !Flags::NO_ANSI);
	}

	// Actually confirm it...
	if flags.contains(Flags::TO_NOWHERE) { false }
	else {
		casual::confirm(std::str::from_utf8_unchecked(data))
	}
}

/// Print Timestamped.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
unsafe fn _print_timestamped(data: &[u8], flags: Flags) {
	let cli_width: usize = term_width();
	let msg_width: usize = data.count_width();
	let offset: &[u8] = if cli_width > msg_width + 21 {
		whitespace(cli_width - msg_width - 21)
	}
	else { &b"\n"[..] };

	// Print to `Stderr`.
	if flags.contains(Flags::TO_STDERR) {
		_print_stderr(
			&[
				data,
				offset,
				&Timestamp::new(),
			].concat(),
			flags
		);
	}
	// Print to `Stdout`.
	else if ! flags.contains(Flags::TO_NOWHERE) {
		_print_stdout(
			&[
				data,
				offset,
				&Timestamp::new(),
			].concat(),
			flags
		);
	}
}

/// Print `Stdout`.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
unsafe fn _print_stdout(data: &[u8], flags: Flags) -> bool {
	use std::io::Write;

	// Strip ANSI?
	if flags.contains(Flags::NO_ANSI) {
		return _print_stdout(&data.strip_ansi(), flags & !Flags::NO_ANSI);
	}

	let writer = std::io::stdout();
	let mut handle = writer.lock();
	if handle.write_all(data).is_ok() {
		handle.flush().is_ok()
	}
	else { false }
}

/// Print `Stderr`.
///
/// # Safety
///
/// This method accepts a raw `[u8]`; when using it, make sure the data you
/// pass is valid UTF-8.
unsafe fn _print_stderr(data: &[u8], flags: Flags) -> bool {
	use std::io::Write;

	// Strip ANSI?
	if flags.contains(Flags::NO_ANSI) {
		return _print_stderr(&data.strip_ansi(), flags & !Flags::NO_ANSI);
	}

	let writer = std::io::stderr();
	let mut handle = writer.lock();
	if handle.write_all(data).is_ok() {
		handle.flush().is_ok()
	}
	else { false }
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
