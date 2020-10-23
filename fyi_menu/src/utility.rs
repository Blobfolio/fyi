/*!
# FYI Menu: Utility Methods.
*/



#[allow(clippy::suspicious_else_formatting)] // What does lines have to do with it?
#[must_use]
/// # Escape Arg String.
///
/// This is a very *crude* reverse argument parser that 'quotes' values needing
/// quoting, used by [`Argue`](crate::Argue) to recombine the arguments
/// following an "--" entry when [`FLAG_SEPARATOR`](crate::FLAG_SEPARATOR) is set.
///
/// This method is optimized for speed rather than robustness — hence its
/// crudeness — so should probably not be used in other contexts unless you're
/// expecting fairly straight-forward data.
///
/// The following adjustments are made:
///
/// * Backslashes are converted to forward slashes.
/// * Single quotes are escaped with a backslash.
/// * If the string is empty or contains anything other than `-`, `_`, `=`, `+`, `/`, `,`, `.`, `a-z`, or `0-9`, the value is wrapped in single quotes.
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
}
