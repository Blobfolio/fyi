/*!
# FYI Core: CLI
*/

use crate::traits::str::FYIStringFormat;
use std::io::{
	stderr,
	stdout,
	Write
};



/// Print.
pub fn print<S> (msg: S, flags: u8) -> bool
where S: Into<String> {
	let mut msg: String = msg.into();

	// Strip colors.
	if 0 != (crate::PRINT_NO_COLOR & flags) {
		msg = msg.fyi_strip_ansi().to_string();
	}

	// Add a new line to the end.
	if 0 != (crate::PRINT_NEWLINE & flags) {
		msg.push_str("\n");
	}

	// Print it!
	match 0 == (crate::PRINT_STDERR & flags) {
		true => {
			let writer = stdout();
			let mut handle = writer.lock();
			match handle.write_all(&msg.as_bytes()).is_ok() {
				true => handle.flush().is_ok(),
				false => false,
			}
		},
		false => {
			let writer = stderr();
			let mut handle = writer.lock();
			match handle.write_all(&msg.as_bytes()).is_ok() {
				true => handle.flush().is_ok(),
				false => false,
			}
		},
	}
}

/// Obtain the terminal cli width.
///
/// The current terminal width.
pub fn term_width() -> usize {
	match term_size::dimensions() {
		// It looks a little weird being squished up against the right edge.
		Some((w, _)) => w - 1,
		_ => 0,
	}
}
