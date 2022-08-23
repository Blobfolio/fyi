/*!
# FYI Msg Example: Parallel Progress Bar w/ Task List.

**This requires the `progress` crate feature.**
*/

use fyi_msg::Msg;



#[cfg(feature = "progress")]
include!("_progless-data.txt");

#[cfg(not(feature = "progress"))]
fn main() {
	Msg::error("This example requires the 'progress' feature.").die(1);
}

#[cfg(feature = "progress")]
/// # Do it.
fn main() {
	use fyi_msg::Progless;
	use rayon::prelude::*;
	use std::time::Duration;

	// Initiate a progress bar.
	let pbar = Progless::try_from(FILE_TYPES.len()).unwrap()
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
	pbar.finish();
	Msg::from(pbar).print();
}
