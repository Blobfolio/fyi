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
	let mut buf = BytesMut::with_capacity(256);
	if 0 != (crate::PRINT_NO_COLOR & flags) {
		buf.put(msg.borrow().strip_ansi().as_bytes());
	}
	else {
		buf.put(msg.borrow().as_bytes());
	}

	// Add a new line to the end.
	if 0 != (crate::PRINT_NEWLINE & flags) {
		buf.put_u8(b'\n');
	}

	// Print it!
	match 0 == (crate::PRINT_STDERR & flags) {
		true => {
			let writer = stdout();
			let mut handle = writer.lock();
			match handle.write_all(&buf.to_vec()).is_ok() {
				true => handle.flush().is_ok(),
				false => false,
			}
		},
		false => {
			let writer = stderr();
			let mut handle = writer.lock();
			match handle.write_all(&buf.to_vec()).is_ok() {
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
