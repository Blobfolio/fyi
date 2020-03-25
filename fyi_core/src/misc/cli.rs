/*!
# FYI Core: CLI
*/

use crate::misc::strings;
use std::io::{stderr, stdout, Write};



/// Print.
pub fn print<S> (msg: S, flags: u8) -> bool
where S: Into<String> {
	let mut msg: String = msg.into();

	// Strip colors.
	if 0 != (crate::PRINT_NO_COLOR & flags) {
		msg = strings::strip_styles(&msg);
	}

	// Add a new line to the end.
	if 0 != (crate::PRINT_NEWLINE & flags) {
		msg = format!("{}\n", &msg).to_string();
	}

	// Print it!
	match 0 == (crate::PRINT_STDERR & flags) {
		true => {
			let mut writer = stdout();
			print_custom(&mut writer, &msg.as_bytes())
		},
		false => {
			let mut writer = stderr();
			print_custom(&mut writer, &msg.as_bytes())
		},
	}
}

/// Print (to a specific writer).
pub fn print_custom<W> (writer: &mut W, msg: &[u8]) -> bool
where W: Write {
	match writer.write_all(&msg).is_ok() {
		true => writer.flush().is_ok(),
		false => false,
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
