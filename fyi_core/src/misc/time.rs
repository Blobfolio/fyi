/*!
# FYI Core: Time
*/

use crate::misc::strings;
use std::time::{
	Duration,
	SystemTime,
	UNIX_EPOCH
};



/// Human-Readable Elapsed Time.
///
/// The short format will return a string in "HH:MM:SS" format, unless
/// your duration has crossed into days, in which case it will be in
/// "DD:HH:MM:SS" format.
///
/// The long format will be a list of the non-empty bits in English,
/// like "15 seconds" or "3 hours and 2 seconds" or "4 days, 3 hours,
/// 2 minutes, and 13 seconds".
pub fn human_elapsed<N> (elapsed: N, flags: u8) -> String
where N: Into<usize> {
	let elapsed = elapsed.into();
	let compact: bool = 0 != (crate::PRINT_COMPACT & flags);

	if elapsed <= 0 {
		return match compact {
			true => "00:00:00".to_string(),
			false => "0 seconds".to_string(),
		};
	}

	// Break down the time.
	let bits: Vec<(usize, &str, &str)> = vec![
		(elapsed / 86400, "day", "days"),
		((elapsed % 86400) / 3600, "hour", "hours"),
		((elapsed % 86400 % 3600) / 60, "minute", "minutes"),
		(elapsed % 86400 % 3600 % 60, "second", "seconds"),
	];

	// Return a shortened version.
	if true == compact {
		return bits.iter()
			.filter_map(|(num, singular, _)| match (*num > 0) | (&"day" != singular) {
				true => Some(format!("{:02}", num)),
				false => None,
			})
			.collect::<Vec<String>>()
			.join(":");
	}

	// A longer version.
	let out = bits.iter()
		.filter_map(|(num, singular, plural)| match *num {
			0 => None,
			_ => Some(strings::inflect(*num, *singular, *plural)),
		})
		.collect::<Vec<String>>();

	// Let's grammar-up the response with Oxford joins.
	let joined = strings::oxford_join(out, " and ");
	match joined.len() {
		0 => "0 seconds".to_string(),
		_ => joined
	}
}

/// Unix Time.
pub fn unixtime() -> usize {
	SystemTime::now().duration_since(UNIX_EPOCH)
		.unwrap_or(Duration::new(5, 0))
		.as_secs() as usize
}
