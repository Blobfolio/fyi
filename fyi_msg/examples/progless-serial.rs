/*!
# FYI Msg Example: Serial Progress Bar.

**This requires the `progress` crate feature.**
*/

use fyi_msg::{
	Msg,
	Progless,
};
use std::time::Duration;



include!("_progless-data.rs");

/// # Do it.
fn main() {
	// Initiate a progress bar.
	let pbar = Progless::steady(FILE_TYPES.len() as u32).unwrap()
		.with_title(Some(Msg::custom("Scanning", 199, "Pretending to look at files one by one…")));

	FILE_TYPES.iter()
		.map(|&t| Duration::from_millis(t.len() as u64 / 2))
		.for_each(|delay| {
			// Simulate work.
			std::thread::sleep(delay);

			// Increment done count.
			pbar.increment();
		});

	// Print a simple summary.
	let _ = pbar.finish();
	Msg::from(pbar).print();
}
