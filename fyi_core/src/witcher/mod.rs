/*!
# FYI Core: Miscellany: Paths
*/

pub mod formats;
pub mod mass;
pub mod ops;
pub mod props;

use crate::misc::numbers;
use crate::misc::strings;
use crate::misc::time;
use crate::msg::Msg;
use crate::prefix::Prefix;
use std::time::Instant;



/// Walk Pattern.
///
/// This is just a convenience method, negating the need to add Regex
/// deps directly to projects using this library.
pub fn pattern_to_regex<S> (pat: S) -> regex::Regex
where S: Into<&'static str> {
	regex::Regex::new(pat.into()).expect("Invalid pattern.")
}

/// Before/After Summary
///
/// For walker tasks that alter contents, this method can be used to
/// print a simple summary at the end of the run (or whatever).
pub fn walk_summary(
	count: u64,
	time: Instant,
	before: u64,
	after: u64
) {
	// Bad data.
	if 0 == count || 0 == before || 0 == after {
		return;
	}

	// Print the time.
	{
		let elapsed: String = time::human_elapsed(time.elapsed().as_secs() as usize, 0);
		let msg: String = format!(
			"Crunched {} in {}.",
			strings::inflect(count as usize, "file", "files"),
			elapsed
		);
		Msg::new(msg.as_str())
			.with_prefix(Prefix::Custom("Finished", 199))
			.print();
	}

	// Print the before and after.
	if 0 < numbers::saved(before, after) {
		let old_bytes: String = format!(
			"{} bytes",
			numbers::human_int(before)
		);
		let old_bytes_len: usize = old_bytes.len();

		let new_bytes: String = format!(
			"{} bytes",
			numbers::human_int(after)
		);
		let diff_bytes: String = format!(
			"{} bytes",
			numbers::human_int(before - after)
		);

		// Print original.
		Msg::new(old_bytes.as_str())
			.with_prefix(Prefix::Custom("Original", 4))
			.print();

		{
			// Print minified.
			let msg: String = format!(
				"{}{}",
				strings::whitespace(old_bytes_len - new_bytes.len()),
				new_bytes
			);
			Msg::new(msg.as_str())
				.with_prefix(Prefix::Custom("Minified", 6))
				.print();
		}

		{
			// Print savings.
			let msg: String = format!(
				"{}{} ({:3.*}%)",
				strings::whitespace(old_bytes_len - diff_bytes.len()),
				diff_bytes,
				2,
				(1.0 - (after as f64 / before as f64)) * 100.0
			);
			Msg::new(msg.as_str())
				.with_prefix(Prefix::Custom(" Savings", 2))
				.print();
		}
	}
	else {
		Msg::new("No changes were made.")
			.with_prefix(Prefix::Warning)
			.print();
	}
}
