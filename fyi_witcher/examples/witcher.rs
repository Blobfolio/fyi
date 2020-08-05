/*!
# FYI Witcher Example: Find + Progress
*/

use fyi_msg::MsgKind;
use fyi_progress::{
	Progress,
	ProgressParallelism,
};
use fyi_witcher::Witcher;
use std::{
	path::PathBuf,
	thread,
	time::Duration,
};



/// Do it.
fn main() {
	// Search for gzipped MAN pages.
	let witched: Vec<PathBuf> = Witcher::from(PathBuf::from("/usr/share/man"))
		.filter_and_collect(r"(:?)\.gz$");
	assert!(! witched.is_empty());

	// A progress bar is a good way to visualize the results!
	let pbar = Progress::new(
		witched,
		MsgKind::new("Witcher Demo", 199).into_msg("Gzipped MAN Pages")
	)
		.with_threads(ProgressParallelism::Heavy);

	// Simulate callback runtime variation by calculating a sleep period from
	// the file path length.
	pbar.run(|p| {
		thread::sleep(Duration::from_millis(p.to_str().unwrap().len() as u64 * 2));
	});

	// And print a summary when it's over.
	pbar.print_summary("manual", "manuals");
}
