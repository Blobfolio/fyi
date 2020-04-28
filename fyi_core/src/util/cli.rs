/*!
# FYI Core: CLI
*/

use bytes::{
	BytesMut,
	BufMut
};
use crate::traits::AnsiBitsy;
use std::{
	borrow::Borrow,
	io::{
		stderr,
		stdout,
		Write,
	},
};



/// Print.
pub fn print<S> (msg: S, flags: u8) -> bool
where S: Borrow<str> {
	// Easy abort.
	if 0 != (crate::PRINT_NOTHING & flags) {
		return false;
	}

	let mut buf = BytesMut::from(msg.borrow());
	if 0 != (crate::PRINT_NO_COLOR & flags) {
		buf.clear();
		buf.put(msg.borrow().strip_ansi().as_bytes());
	}

	// Add a new line to the end.
	if 0 != (crate::PRINT_NEWLINE & flags) {
		buf.put_u8(b'\n');
	}

	// Print it!
	if 0 == (crate::PRINT_STDERR & flags) {
		let writer = stdout();
		let mut handle = writer.lock();
		if handle.write_all(&buf.to_vec()).is_ok() {
			handle.flush().is_ok()
		}
		else {
			false
		}
	}
	else {
		let writer = stderr();
		let mut handle = writer.lock();
		if handle.write_all(&buf.to_vec()).is_ok() {
			handle.flush().is_ok()
		}
		else {
			false
		}
	}
}

#[must_use]
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
