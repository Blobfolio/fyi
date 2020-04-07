/*!
# FYI Core: Numbers
*/

use num_format::{
	Locale,
	ToFormattedString,
};
use num_traits::cast::ToPrimitive;
use std::borrow::Cow;



/// Human-Readable Size.
///
/// Convert a numerical byte size into a string with the best unit
/// given the value.
pub fn human_bytes<N> (size: N) -> Cow<'static, str>
where N: ToPrimitive {
	let bytes:f64 = size.to_f64().unwrap_or(0.0);

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
		return Cow::Owned(format!("{}B", bytes));
	};

	Cow::Owned(format!("{:.*}{}", 2, bytes, unit))
}

/// Nice Int.
pub fn human_int<N> (num: N) -> Cow<'static, str>
where N: ToPrimitive {
	Cow::Owned(num.to_u64()
		.unwrap_or(0)
		.to_formatted_string(&Locale::en))
}

/// Saved.
pub fn saved<N> (before: N, after: N) -> u64
where N: ToPrimitive {
	let before = before.to_u64().unwrap_or(0);
	let after = after.to_u64().unwrap_or(0);
	match 0 < after && after < before {
		true => before - after,
		false => 0
	}
}
