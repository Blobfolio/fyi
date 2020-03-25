/*!
# FYI Core: Strings
*/



/// Inflect.
///
/// Return a string like "NUMBER LABEL" where the label is
/// appropriately singular or plural given the value.
pub fn inflect<N, S> (num: N, singular: S, plural: S) -> String
where N: Into<usize>, S: Into<String> {
	let num = num.into();
	match num {
		1 => format!("1 {}", singular.into()),
		_ => format!("{} {}", num, plural.into()),
	}
}

/// Indentation in Spaces.
///
/// Return a string consisting of 4 spaces for each requested tab.
pub fn indentation<N> (indent: N) -> String
where N: Into<usize> {
	whitespace(indent.into() * 4)
}

/// Line Count.
pub fn lines<S> (text: S) -> usize
where S: Into<String> {
	let mut total: usize = 0;
	let text = strip_styles(text).trim().to_string();
	for _ in text.lines() {
		total += 1;
	}

	total
}

/// Oxford Join
///
/// Join a `Vec<String>` with correct comma usage and placement. If
/// there is one item, that item is returned. If there are two, they
/// are joined with the operator. Three or more entries will use
/// the Oxford Comma.
pub fn oxford_join<S> (mut list: Vec<String>, glue: S) -> String
where S: Into<String> {
	let glue: String = glue.into().trim().to_string();

	match list.len() {
		0 => "".to_string(),
		1 => list[0].to_string(),
		2 => list.join(&format!(" {} ", &glue)),
		_ => {
			let last = list.pop().unwrap_or("".to_string());
			format!("{}, and {}", list.join(", "), last)
		}
	}
}

/// Find Padding Needed.
fn pad_diff<S> (text: S, len: usize) -> usize
where S: Into<String> {
	let text_len: usize = text.into().len();
	match text_len >= len {
		true => 0,
		false => len - text_len,
	}
}

/// Pad String On Left.
pub fn pad_left<S>(text: S, pad_len: usize, pad_fill: u8) -> String
where S: Into<String> {
	let text = text.into();

	match pad_diff(&text, pad_len) {
		0 => text,
		x => format!(
			"{}{}",
			String::from_utf8(vec![pad_fill; x]).unwrap_or("".to_string()),
			text,
		),
	}
}

/// Pad String On Right.
pub fn pad_right<S>(text: S, pad_len: usize, pad_fill: u8) -> String
where S: Into<String> {
	let text = text.into();

	match pad_diff(&text, pad_len) {
		0 => text,
		x => format!(
			"{}{}",
			text,
			String::from_utf8(vec![pad_fill; x]).unwrap_or("".to_string()),
		),
	}
}

/// Shorten String From Left (Keeping Right).
pub fn shorten_left<S>(text: S, len: usize) -> String
where S: Into<String> {
	match len {
		0 => "".to_string(),
		1 => "…".to_string(),
		_ => {
			// Pull text details.
			let text = text.into();
			let text_len: usize = text.len();

			// Shorten away!
			match text_len <= len {
				true => text,
				false => {
					let short: String = text.chars()
						.skip(text_len - len + 1)
						.collect();
					format!("…{}", short.trim())
				},
			}
		}
	}
}

/// Shorten String From Right (Keeping Left).
pub fn shorten_right<S>(text: S, len: usize) -> String
where S: Into<String> {
	match len {
		0 => "".to_string(),
		1 => "…".to_string(),
		_ => {
			// Pull text details.
			let text = text.into();
			let text_len: usize = text.len();

			// Shorten away!
			match text_len <= len {
				true => text,
				false => {
					let short: String = text.chars()
						.take(len - 1)
						.collect();
					format!("{}…", short.trim())
				},
			}
		}
	}
}

/// Stripped Length.
///
/// Return the length of a string without counting any ANSI codes, etc.
pub fn stripped_len<S> (text: S) -> usize
where S: Into<String> {
	strip_styles(text).len()
}

/// Strip Styles
///
/// Remove ANSI codes, etc., from a string.
pub fn strip_styles<S> (text: S) -> String
where S: Into<String> {
	let text = strip_ansi_escapes::strip(text.into())
		.unwrap_or(Vec::new());
	std::str::from_utf8(&text)
		.unwrap_or("")
		.to_string()
}

/// Make whitespace.
///
/// Generate a string consisting of X spaces.
pub fn whitespace<N> (count: N) -> String
where N: Into<usize> {
	let count = count.into();
	if 0 < count {
		String::from_utf8(vec![b' '; count]).unwrap_or("".to_string())
	}
	else {
		"".to_string()
	}
}
