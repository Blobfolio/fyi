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

	if bytes > gb * 0.9 {
		Cow::Owned(format!("{:.*}GB", 2, bytes / gb))
	}
	else if bytes > mb * 0.9 {
		Cow::Owned(format!("{:.*}MB", 2, bytes / mb))
	}
	else if bytes > kb * 0.9 {
		Cow::Owned(format!("{:.*}KB", 2, bytes / kb))
	}
	else {
		Cow::Owned(format!("{}B", bytes))
	}
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
