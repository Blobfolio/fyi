/*!
# FYI Core: Numbers
*/

use num_format::{Locale, ToFormattedString};



/// Human-Readable Size.
///
/// Convert a numerical byte size into a string with the best unit
/// given the value.
pub fn human_bytes<N> (size: N) -> String
where N: Into<u64> {
	let bytes = size.into() as f64;

	let kb: f64 = 1024.0;
	let mb: f64 = 1024.0 * 1024.0;
	let gb: f64 = 1024.0 * 1024.0 * 1024.0;

	let (bytes, unit) = if bytes > gb * 0.9 {
		(bytes / gb, "GB")
	} else if bytes > mb * 0.9 {
		(bytes / mb, "MB")
	} else if bytes > kb * 0.9 {
		(bytes / kb, "KB")
	} else {
		return format!("{}B", bytes);
	};

	format!("{:.*}{}", 2, bytes, unit)
}

/// Nice Int.
pub fn human_int<N> (num: N) -> String
where N: Into<u64> {
	num.into().to_formatted_string(&Locale::en)
}

/// Saved.
pub fn saved<N> (before: N, after: N) -> u64
where N: Into<u64> {
	let before = before.into();
	let after = after.into();
	match 0 < after && after < before {
		true => before - after,
		false => 0
	}
}
