/*!
# FYI Msg - Fitted Widths

This optional module contains methods for counting the display width of byte strings, and/or figuring out the closest char index to chop on to make something fit.
*/



#[must_use]
/// # Length Width.
///
/// Return the maximum byte *length* for the slice that fits a given display
/// *width*, such that `slice[0..len]` will be a valid substring likely to fit.
/// This method is Unicode-safe, but Unicode character widths are at best
/// approximations, so actual results may vary plus or minus a few spaces.
///
/// If no fit is possible, zero is returned.
///
/// See the documentation for [`width`] for more information.
///
/// For performance reasons, this method will assume the byte sequence is ASCII
/// unless/until it finds a non-ASCII code, at which point it will shift to the
/// heavier `length_width_unicode` method and finish counting there.
///
/// **This requires the `fitted` crate feature.**
///
/// ## Safety.
///
/// The byte sequence must be valid UTF-8 or undefined things will happen.
pub fn length_width(bytes: &[u8], stop: usize) -> usize {
	// Short circuit.
	let b_len: usize = bytes.len();
	if b_len <= stop { return b_len; }

	// Iterate the bytes unless/until we find something non-ASCII.
	let mut in_ansi: bool = false;
	match bytes.iter()
		.take_while(|i| i.is_ascii())
		.try_fold((0, 0), |(l, w), &i|
			// We're already inside an ANSI sequence and waiting for a
			// terminator.
			if in_ansi {
				// The sequence is over.
				if matches!(i, b'm' | b'A' | b'K') { in_ansi = false; }
				Ok((l + 1, w))
			}
			// We're starting a new ANSI sequence.
			else if i == b'\x1b' {
				in_ansi = true;
				Ok((l + 1, w))
			}
			// Control characters have no width.
			else if i == 0 || i.is_ascii_control() { Ok((l + 1, w)) }
			// We've reached out limit.
			else if w + 1 > stop { Err(l) }
			// Normal ASCII has a width of one!
			else { Ok((l + 1, w + 1)) }
		)
	{
		Ok((len, width)) =>
			// We read it all.
			if len == b_len { len }
			// Otherwise we need to end the rest of the slice to the unicode counter.
			else {
				length_width_unicode(&bytes[len..], len, width, stop)
			},
		// We reached the stopping point.
		Err(len) => len,
	}
}


#[must_use]
/// # Width.
///
/// Find the "display width" of a byte string.
///
/// Like anything having to do with width vs length, this should be considered
/// at best an approximation. For ASCII, every byte that is not a control
/// character or part of an ANSI formatting sequence is counted as having a
/// length of 1. For Unicode, the [`unicode_width`](https://crates.io/crates/unicode-width) crate is used
/// to determine width.
///
/// While not 100% perfect, results should be closer than simply trusting the
/// raw byte length.
///
/// For performance reasons, this method will assume the byte sequence is ASCII
/// unless/until it finds a non-ASCII code, at which point it will shift to the
/// heavier `width_unicode` method and finish counting there.
///
/// Note: line breaks are ignored; the cumulative width of all lines is
/// returned. If you're trying to calculate *line* widths, split the slice
/// first and pass each chunk separately.
///
/// **This requires the `fitted` crate feature.**
///
/// ## Safety.
///
/// The byte sequence must be valid UTF-8 or undefined things will happen.
pub fn width(bytes: &[u8]) -> usize {
	// Short circuit.
	if bytes.is_empty() { return 0; }

	// Iterate the bytes unless/until we find something non-ASCII.
	let mut in_ansi: bool = false;
	let (len, width) = bytes.iter()
		.take_while(|i| i.is_ascii())
		.fold((0, 0), |(l, w), &i|
			// We're already inside an ANSI sequence and waiting for a
			// terminator.
			if in_ansi {
				// The sequence is over.
				if matches!(i, b'm' | b'A' | b'K') { in_ansi = false; }
				(l + 1, w)
			}
			// We're starting a new ANSI sequence.
			else if i == b'\x1b' {
				in_ansi = true;
				(l + 1, w)
			}
			// Control characters have no width.
			else if i == 0 || i.is_ascii_control() { (l + 1, w) }
			// Normal ASCII has a width of one!
			else { (l + 1, w + 1) }
		);

	// We read it all.
	if len == bytes.len() { width }
	// Otherwise we need to end the rest of the slice to the unicode counter.
	else {
		width_unicode(&bytes[len..], width)
	}
}

/// # Length Width (Unicode)
///
/// This method finishes fitting a byte string containing Unicode characters.
/// The counts begin from the provided `len` and `width` values passed to it
/// (rather than zero)
///
/// See the documentation for the public [`len_width`] method for more information.
///
/// ## Safety.
///
/// The byte sequence must be valid UTF-8 or undefined things will happen.
fn length_width_unicode(bytes: &[u8], len: usize, width: usize, stop: usize) -> usize {
	use unicode_width::UnicodeWidthChar;

	// Build a string from the bytes so we can get access to the inner chars.
	// This shouldn't fail, but if it does, it will return the length the call
	// was seeded with.
	let strung = match std::str::from_utf8(bytes) {
		Ok(s) => s,
		Err(_) => { return len; },
	};

	let mut in_ansi: bool = false;
	match strung.chars()
		.try_fold((len, width), |(l, w), c| {
			// Find the "length" of this char.
			let ch_len: usize = c.len_utf8();

			// We're already inside an ANSI sequence and waiting for a
			// terminator.
			if in_ansi {
				// The sequence is over.
				if matches!(c, 'm' | 'A' | 'K') { in_ansi = false; }
				Ok((l + ch_len, w))
			}
			// We're starting a new ANSI sequence.
			else if c == '\x1b' {
				in_ansi = true;
				Ok((l + ch_len, w))
			}
			else {
				// Widths can creep up unevenly. If we've gone over, we need to
				// stop.
				let w = UnicodeWidthChar::width(c).map_or(w, |w2| w2 + w);
				if w > stop { Err(l) }
				else { Ok((l + ch_len, w)) }
			}
		})
	{
		Ok((len, _)) | Err(len) => len,
	}
}

/// # Width (Unicode)
///
/// This method finishes counting a byte string containing Unicode characters.
/// The count begins from the `width` value passed to it (rather than zero).
///
/// See the documentation for the public [`width`] method for more information.
///
/// ## Safety.
///
/// The byte sequence must be valid UTF-8 or undefined things will happen.
fn width_unicode(bytes: &[u8], width: usize) -> usize {
	use unicode_width::UnicodeWidthChar;

	// Build a string from the bytes so we can get access to the inner chars.
	// This shouldn't fail, but if it does, it will default to a byte count.
	let strung = match std::str::from_utf8(bytes) {
		Ok(s) => s,
		Err(_) => { return width + bytes.len(); },
	};

	let mut in_ansi: bool = false;
	strung.chars()
		.fold(width, |w, c|
			// We're already inside an ANSI sequence and waiting for a
			// terminator.
			if in_ansi {
				// The sequence is over.
				if matches!(c, 'm' | 'A' | 'K') { in_ansi = false; }
				w
			}
			// We're starting a new ANSI sequence.
			else if c == '\x1b' {
				in_ansi = true;
				w
			}
			else {
				UnicodeWidthChar::width(c).map_or(w, |w2| w2 + w)
			}
		)
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_length_width() {
		for &(slice, stop, expected) in &[
			("Hello", 5, 5),
			("Hello", 6, 5),
			("Hello", 4, 4),
			("Hello\nWorld", 10, 11),
			("Björk Guðmundsdóttir", 5, 6),
			("Björk Guðmundsdóttir", 3, 4),
			("Björk Guðmundsdóttir", 2, 2),
			("\x1b[2mBjörk\x1b[0m Guðmundsdóttir", 5, 14),
			("\x1b[2mHello\x1b[0m World", 5, 13),
		] {
			assert_eq!(
				length_width(slice.as_bytes(), stop),
				expected,
				"Invalid length/width for {:?} fit to {}.",
				slice,
				stop
			);
		}
	}

	#[test]
	fn t_width() {
		for &(slice, expected) in &[
			("Hello\nWorld", 10),
			("Hello World", 11),
			("\x1b[2mHello\x1b[0m World", 11),
			("Hello World…", 12),
			("Björk Guðmundsdóttir", 20),
			("\x1b[2mBjörk\x1b[0m Guðmundsdóttir", 20),
			("Björk \x1b[2mGuðmundsdóttir\x1b[0m", 20),
		] {
			assert_eq!(
				width(slice.as_bytes()), expected,
				"Invalid width for {:?}.", slice
			);
		}
	}
}
