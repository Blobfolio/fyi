/*!
# FYI Progress: Utility Methods
*/

/// String Inflection
///
/// Given a number, come up with a byte string like "1 thing" or "2 things".
pub fn inflect<T> (num: u64, one: T, many: T) -> Vec<u8>
where T: AsRef<str> {
	if 1 == num {
		[
			&[49, 32],
			one.as_ref().as_bytes(),
		].concat()
	}
	else if num < 1000 {
		let mut buf = itoa::Buffer::new();
		[
			buf.format(num).as_bytes(),
			&[32],
			many.as_ref().as_bytes(),
		].concat()
	}
	else {
		let mut buf = num_format::Buffer::default();
		buf.write_formatted(&num, &num_format::Locale::en);
		[
			buf.as_bytes(),
			&[32],
			many.as_ref().as_bytes(),
		].concat()
	}
}

#[must_use]
/// Chunked Seconds
///
/// This method converts seconds into hours, minutes, and seconds, returning
/// a fixed-length array with each value in order, e.g. `[h, m, s]`.
///
/// As with the rest of the methods in this module, days and beyond are not
/// considered. Large values are simply truncated to `86399`, i.e. one second
/// shy of a full day.
pub fn secs_chunks(num: u32) -> [u32; 3] {
	let mut out: [u32; 3] = [0, 0, u32::min(86399, num)];

	// Hours.
	if out[2] >= 3600 {
		out[0] = num_integer::div_floor(out[2], 3600);
		out[2] -= out[0] * 3600;
	}

	// Minutes.
	if out[2] >= 60 {
		out[1] = num_integer::div_floor(out[2], 60);
		out[2] -= out[1] * 60;
	}

	out
}

#[must_use]
/// Term Width
///
/// This is a simple wrapper around `term_size::dimensions()` to provide
/// the current terminal column width. We don't have any use for height,
/// so that property is ignored.
///
/// Note: The actual width returned is `1` less than the true value. This helps
/// account for inconsistent handling of trailing whitespace, etc.
pub fn term_width() -> usize {
	term_size::dimensions().map_or(0, |(w, _)| w.saturating_sub(1))
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_inflect() {
		assert_eq!(&inflect(0, "book", "books").as_ref(), b"0 books");
		assert_eq!(&inflect(1, "book", "books").as_ref(), b"1 book");
		assert_eq!(&inflect(1000, "book", "books").as_ref(), b"1,000 books");
	}

	#[test]
	fn t_secs_chunks() {
		assert_eq!(secs_chunks(1), [0, 0, 1]);
		assert_eq!(secs_chunks(30), [0, 0, 30]);
		assert_eq!(secs_chunks(90), [0, 1, 30]);
		assert_eq!(secs_chunks(3600), [1, 0, 0]);
	}
}
