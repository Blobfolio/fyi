/*!
# FYI Progress: Utility

This mod contains miscellaneous utility functions for the crate.
*/

#[must_use]
/// Chunked Seconds
///
/// This method converts seconds into hours, minutes, and seconds, returning
/// a fixed-length array with each value in order, e.g. `[h, m, s]`.
///
/// As with the rest of the methods in this module, days and beyond are not
/// considered. Large values are simply truncated to `86399`, i.e. one second
/// shy of a full day.
pub fn secs_chunks(num: u32) -> [u32; 3] {
	let mut out: [u32; 3] = [0, 0, u32::min(86399, num)];

	// Hours.
	if out[2] >= 3600 {
		out[0] = num_integer::div_floor(out[2], 3600);
		out[2] -= out[0] * 3600;
	}

	// Minutes.
	if out[2] >= 60 {
		out[1] = num_integer::div_floor(out[2], 60);
		out[2] -= out[1] * 60;
	}

	out
}

#[must_use]
/// Term Width
///
/// This is a simple wrapper around `term_size::dimensions()` to provide
/// the current terminal column width. We don't have any use for height,
/// so that property is ignored.
pub fn term_width() -> usize {
	// Reserve one space at the end "just in case".
	term_size::dimensions().map_or(0, |(w, _)| w.saturating_sub(1))
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_secs_chunks() {
		assert_eq!(secs_chunks(1), [0, 0, 1]);
		assert_eq!(secs_chunks(30), [0, 0, 30]);
		assert_eq!(secs_chunks(90), [0, 1, 30]);
		assert_eq!(secs_chunks(3600), [1, 0, 0]);
	}
}
