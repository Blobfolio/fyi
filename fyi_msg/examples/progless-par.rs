/*!
# FYI Msg Example: Parallel Progress Bar w/ Task List.

This requires the `progress` crate feature.
*/

use fyi_msg::{
	Msg,
	Progless,
};
use rayon::prelude::*;
use std::time::Duration;



include!("_progless-data.rs");

/// # Do it.
fn main() {
	// Initiate a progress bar.
	let pbar = Progless::steady(FILE_TYPES.len() as u32)
		.with_title(Some(Msg::custom("Scanning", 199, "Pretending to look at files…")));

	FILE_TYPES.par_iter()
		.map(|&t| (t, Duration::from_millis(t.len() as u64 * 3)))
		.for_each(|(txt, delay)| {
			// Start a new task.
			pbar.add(txt);

			// Simulate work.
			std::thread::sleep(delay);

			// Remove said task, which increments the "done" count by one.
			pbar.remove(txt);
		});

	// Print a simple summary.
	let _ = pbar.finish();
	Msg::from(pbar).print();
}
