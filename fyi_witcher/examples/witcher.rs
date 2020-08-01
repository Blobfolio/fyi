/*!
# FYI Witcher Example: Find + Progress
*/

use fyi_witcher::{
	Witcher,
	progress,
};
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

	// Do a loop for no particular reason. To make it interesting, we'll force
	// pauses relative to the path length.
	progress(&witched, "Gzipped MAN Pages", |p| {
		thread::sleep(Duration::from_millis(p.to_str().unwrap().len() as u64 * 2));
	});
}
