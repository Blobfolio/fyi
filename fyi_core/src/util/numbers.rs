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
		false => 0,
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn human_bytes() {
		assert_eq!(super::human_bytes(921u64), Cow::Borrowed("921B"));
		assert_eq!(super::human_bytes(922u64), Cow::Borrowed("0.90KB"));
		assert_eq!(super::human_bytes(12003u64), Cow::Borrowed("11.72KB"));
		assert_eq!(super::human_bytes(4887391u64), Cow::Borrowed("4.66MB"));
		assert_eq!(super::human_bytes(499288372u64), Cow::Borrowed("476.16MB"));
		assert_eq!(super::human_bytes(99389382145u64), Cow::Borrowed("92.56GB"));
	}

	#[test]
	fn human_int() {
		assert_eq!(super::human_int(500u64), Cow::Borrowed("500"));
		assert_eq!(super::human_int(5000u64), Cow::Borrowed("5,000"));
		assert_eq!(super::human_int(50000u64), Cow::Borrowed("50,000"));
		assert_eq!(super::human_int(500000u64), Cow::Borrowed("500,000"));
		assert_eq!(super::human_int(5000000u64), Cow::Borrowed("5,000,000"));
	}

	#[test]
	fn saved() {
		// Negatives.
		assert_eq!(super::saved(0u64, 500u64), 0u64);
		assert_eq!(super::saved(500u64, 500u64), 0u64);
		assert_eq!(super::saved(500u64, 0u64), 0u64);

		// Positives.
		assert_eq!(super::saved(1000u64, 500u64), 500u64);
		assert_eq!(super::saved(10000u64, 500u64), 9500u64);
	}
}
