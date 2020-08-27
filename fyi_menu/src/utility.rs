/*!
# FYI Menu: Utility Methods
*/



#[allow(clippy::match_on_vec_items)] // We already checked the index exists.
#[allow(clippy::suspicious_else_formatting)] // What does lines have to do with it?
#[must_use]
/// Escape String Value.
///
/// This is a lightweight — many assumptions made — reverse argument parser.
/// Its primary purpose is to turn all the separated values after an "--" back
/// into the sort of string it would have started as.
///
/// In practice, values requiring quotes are single-quoted; single quotes
/// within the value are backslashed; and backslashes within the value are
/// converted to forward slashes. The latter makes this ill-advised for use
/// with e.g. Windows paths, but that's pretty much outside our scope anyway.
///
/// # Safety
///
/// This method uses unsafe methods, but in a safe way. It's fine.
pub fn esc_arg(mut s: String) -> String {
	// Strings suck. Let's work from bytes.
	let v = unsafe { s.as_mut_vec() };
	let mut needs_quote: bool = v.is_empty();

	if ! needs_quote {
		let mut idx: usize = 0;
		let mut len: usize = v.len();
		while idx < len {
			// Replace backslashes with forward slashes.
			if v[idx] ==  b'\\' {
				v[idx] = b'/';
				idx += 1;
			}
			// Backslash quotes.
			else if v[idx] == b'\'' {
				v.insert(idx, b'\\');
				idx += 2;
				len += 1;
				needs_quote = true;
			}
			// Something else?
			else {
				if
					! needs_quote &&
					! matches!(v[idx], b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b'=' | b'/' | b',' | b'.' | b'+')
				{
					needs_quote = true;
				}
				idx += 1;
			}
		}
	}

	if needs_quote {
		v.reserve(2);
		v.insert(0, b'\'');
		v.push(b'\'');
	}

	s
}

#[must_use]
/// Is A String Only Whitespace?
///
/// This will return `true` if the string is empty or only includes ASCII
/// whitespace. Other characters, including weird Unicode whitespace, result in
/// a response of `false`.
pub const fn slice_is_whitespace(data: &[u8]) -> bool {
	let len: usize = data.len();
	let mut idx: usize = 0;
	while idx < len {
		if ! matches!(data[idx], b'\t' | b'\n' | b'\x0C' | b'\r' | b' ') {
			return false;
		}

		idx += 1;
	}

	true
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_esc_arg() {
		assert_eq!("''", esc_arg("".into()));
		assert_eq!("' '", esc_arg(" ".into()));
		assert_eq!("Hello", esc_arg("Hello".into()));
		assert_eq!("'Hello World'", esc_arg("Hello World".into()));
		assert_eq!("/path/to/file", esc_arg(r"\path\to\file".into()));
		assert_eq!(r"'Eat at Joe\'s'", esc_arg("Eat at Joe's".into()));
		assert_eq!(r"'Björk\'s Vespertine'", esc_arg("Björk's Vespertine".into()));
	}

	#[test]
	fn t_slice_is_whitespace() {
		assert_eq!(true, slice_is_whitespace(b" "));
		assert_eq!(true, slice_is_whitespace(b""));
		assert_eq!(true, slice_is_whitespace(b"\t \n"));
		assert_eq!(false, slice_is_whitespace(b" Hello World "));
	}
}
