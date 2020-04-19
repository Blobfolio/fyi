/*!
# FYI Core: Time
*/

use crate::util::strings;
use num_traits::cast::ToPrimitive;
use std::{
	borrow::Cow,
	time::{
		Duration,
		SystemTime,
		UNIX_EPOCH,
	},
};



/// Chunked Time.
///
/// Split seconds into days, hours, minutes, seconds.
pub fn chunked(s: usize) -> [usize; 4] {
	let mut out: [usize; 4] = [0, 0, 0, s];

	// Days.
	if out[3] >= 86400 {
		out[0] = out[3] / 86400;
		out[3] -= out[0] * 86400;
	}

	// Hours.
	if out[3] >= 3600 {
		out[1] = out[3] / 3600;
		out[3] -= out[1] * 3600;
	}

	// Minutes.
	if out[3] >= 60 {
		out[2] = out[3] / 60;
		out[3] -= out[2] * 60;
	}

	out
}

/// Human-Readable Elapsed Time
///
/// This turns seconds into a human list like 1 hour and 2 seconds.
pub fn elapsed<N> (elapsed: N) -> Cow<'static, str>
where N: ToPrimitive {
	let elapsed: usize = elapsed.to_usize().unwrap_or(0);
	if 0 == elapsed {
		Cow::Borrowed("0 seconds")
	}
	else if elapsed == 1 {
		Cow::Borrowed("1 second")
	}
	else if elapsed < 60 {
		Cow::Owned([&elapsed.to_string(), " seconds"].concat())
	}
	else {
		// This is ugly but faster than elegant iter() and whatnot.
		let parts: Vec<String> = {
			let mut out: Vec<String> = Vec::with_capacity(4);
			let c = chunked(elapsed);

			if c[0] == 1 {
				out.push("1 day".to_string());
			}
			else if c[0] > 1 {
				out.push([&c[0].to_string(), " days"].concat());
			}

			if c[1] == 1 {
				out.push("1 hour".to_string());
			}
			else if c[1] > 1 {
				out.push([&c[1].to_string(), " hours"].concat());
			}

			if c[2] == 1 {
				out.push("1 minute".to_string());
			}
			else if c[2] > 1 {
				out.push([&c[2].to_string(), " minutes"].concat());
			}

			if c[3] == 1 {
				out.push("1 second".to_string());
			}
			else if c[3] > 1 {
				out.push([&c[3].to_string(), " seconds"].concat());
			}

			out
		};

		strings::oxford_join(
			&parts,
			"and"
		)
	}
}

/// Elapsed Time (Compact)
///
/// This turns seconds into a 00:00:00-style display. Days are included
/// only if positive.
pub fn elapsed_short<N> (elapsed: N) -> Cow<'static, str>
where N: ToPrimitive {
	let elapsed: usize = elapsed.to_usize().unwrap_or(0);
	if 0 == elapsed {
		Cow::Borrowed("00:00:00")
	}
	else {
		let c = chunked(elapsed);
		if 0 != c[0] {
			Cow::Owned(format!(
				"{:02}:{:02}:{:02}:{:02}",
				c[0], c[1], c[2], c[3]
			))
		}
		else {
			Cow::Owned(format!(
				"{:02}:{:02}:{:02}",
				c[1], c[2], c[3]
			))
		}
	}
}

/// Unix Time.
pub fn unixtime() -> usize {
	SystemTime::now().duration_since(UNIX_EPOCH)
		.unwrap_or(Duration::new(5, 0))
		.as_secs() as usize
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn chunked() {
		assert_eq!(super::chunked(0), [0, 0, 0, 0]);
		assert_eq!(super::chunked(50), [0, 0, 0, 50]);
		assert_eq!(super::chunked(100), [0, 0, 1, 40]);
		assert_eq!(super::chunked(1000), [0, 0, 16, 40]);
		assert_eq!(super::chunked(10000), [0, 2, 46, 40]);
		assert_eq!(super::chunked(100000), [1, 3, 46, 40]);
		assert_eq!(super::chunked(1000000), [11, 13, 46, 40]);
	}

	#[test]
	fn elapsed() {
		assert_eq!(super::elapsed(0), Cow::Borrowed("0 seconds"));
		assert_eq!(super::elapsed(1), Cow::Borrowed("1 second"));
		assert_eq!(super::elapsed(50), Cow::Borrowed("50 seconds"));
		assert_eq!(super::elapsed(100), Cow::Borrowed("1 minute and 40 seconds"));
		assert_eq!(super::elapsed(1000), Cow::Borrowed("16 minutes and 40 seconds"));
		assert_eq!(super::elapsed(10000), Cow::Borrowed("2 hours, 46 minutes, and 40 seconds"));
		assert_eq!(super::elapsed(100000), Cow::Borrowed("1 day, 3 hours, 46 minutes, and 40 seconds"));
		assert_eq!(super::elapsed(1000000), Cow::Borrowed("11 days, 13 hours, 46 minutes, and 40 seconds"));
	}

	#[test]
	fn elapsed_short() {
		assert_eq!(super::elapsed_short(0), Cow::Borrowed("00:00:00"));
		assert_eq!(super::elapsed_short(1), Cow::Borrowed("00:00:01"));
		assert_eq!(super::elapsed_short(50), Cow::Borrowed("00:00:50"));
		assert_eq!(super::elapsed_short(100), Cow::Borrowed("00:01:40"));
		assert_eq!(super::elapsed_short(1000), Cow::Borrowed("00:16:40"));
		assert_eq!(super::elapsed_short(10000), Cow::Borrowed("02:46:40"));
		assert_eq!(super::elapsed_short(100000), Cow::Borrowed("01:03:46:40"));
		assert_eq!(super::elapsed_short(1000000), Cow::Borrowed("11:13:46:40"));
	}

	#[test]
	fn unixtime() {
		let tmp: usize = super::unixtime();
		assert!(tmp > 999999999);
	}
}
