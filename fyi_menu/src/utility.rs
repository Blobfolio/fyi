/*!
# FYI Menu: Utility Methods
*/



#[must_use]
/// Is Byte a Letter?
///
/// Keys need an [a-z] or [A-Z] letter following the dash.
pub fn byte_is_letter(byte: u8) -> bool {
	match byte {
		b'A'..=b'Z' | b'a'..=b'z' => true,
		_ => false,
	}
}

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
pub fn slice_is_whitespace(data: &[u8]) -> bool {
	let len: usize = data.len();
	let mut idx: usize = 0;
	while idx < len {
		if ! data[idx].is_ascii_whitespace() {
			return false;
		}

		idx += 1;
	}

	true
}

/// Retain With Answer
///
/// Identical to `retain()` except it returns `true` if changes were made,
/// `false` otherwise.
pub fn vec_retain_explain<F> (data: &mut Vec<String>, mut f: F) -> bool
where F: FnMut(&String) -> bool {
	let len = data.len();
	let mut del = 0;

	let ptr = data.as_mut_ptr();
	unsafe {
		let mut idx: usize = 0;
		while idx < len {
			if !f(&*ptr.add(idx)) {
				del += 1;
			}
			else if del > 0 {
				ptr.add(idx).swap(ptr.add(idx - del));
			}

			idx += 1;
		}
	}

	if del > 0 {
		data.truncate(len - del);
		true
	}
	else { false }
}

/// Trim Start.
///
/// Remove all leading values that are empty or contain only ASCII whitespace
/// from the beginning of a Vec.
pub fn vec_trim_start(data: &mut Vec<String>) {
	let mut idx: usize = 0;
	let len: usize = data.len();

	while idx < len {
		// Got something!
		if ! slice_is_whitespace(data[idx].as_bytes()) {
			if 0 != idx {
				data.drain(0..idx);
			}
			return;
		}

		idx += 1;
	}

	// The loop exits on the first non-empty entry. If we reach this condition,
	// the whole Vec is empty!
	if idx != 0 {
		data.truncate(0);
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_byte_is_letter() {
		assert_eq!(true, byte_is_letter(b'a'));
		assert_eq!(true, byte_is_letter(b'B'));
		assert_eq!(true, byte_is_letter(b'c'));
		assert_eq!(true, byte_is_letter(b'D'));

		assert_eq!(false, byte_is_letter(b'!'));
		assert_eq!(false, byte_is_letter(b' '));
		assert_eq!(false, byte_is_letter(b'@'));
		assert_eq!(false, byte_is_letter(b'2'));
		assert_eq!(false, byte_is_letter(b'.'));
	}

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

	#[test]
	fn t_vec_retain_explain() {
		let mut test = vec![
			String::from("a"),
			String::from("b"),
			String::from("a"),
			String::from("b"),
		];

		assert_eq!(true, vec_retain_explain(&mut test, |x| x == "a"));
		assert_eq!(
			test,
			vec![
				String::from("a"),
				String::from("a"),
			]
		);

		assert_eq!(false, vec_retain_explain(&mut test, |x| x == "a"));
		assert_eq!(
			test,
			vec![
				String::from("a"),
				String::from("a"),
			]
		);

		assert_eq!(true, vec_retain_explain(&mut test, |x| x == "b"));
		assert!(test.is_empty());
	}

	#[test]
	fn t_vec_trim_start() {
		let mut test = vec![
			String::from(""),
			String::from(" "),
			String::from("one"),
			String::from("two"),
			String::from(""),
		];
		vec_trim_start(&mut test);
		assert_eq!(
			test,
			vec![
				String::from("one"),
				String::from("two"),
				String::from(""),
			]
		);

		vec_trim_start(&mut test);
		assert_eq!(
			test,
			vec![
				String::from("one"),
				String::from("two"),
				String::from(""),
			]
		);

		test.truncate(0);
		vec_trim_start(&mut test);
		assert!(test.is_empty());

		test = vec![
			String::from(""),
			String::from(" "),
			String::from(" \t      \n    "),
		];
		vec_trim_start(&mut test);
		assert!(test.is_empty());
	}
}