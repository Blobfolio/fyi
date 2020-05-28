/*!
# FYI Witcher Example: Find + Progress
*/

use fyi_witcher::Witcher;
use std::thread;
use std::time::Duration;



/// Do it.
fn main() {
	// Search for gzipped MAN pages.
	let witched = Witcher::new(&["/usr/share/man"], r"(:?)\.gz$");
	assert!(! witched.is_empty());

	// Do a loop for no particular reason. To make it interesting, we'll force
	// pauses relative to the path length.
	witched.progress("Gzipped MAN Pages", |p| {
		thread::sleep(Duration::from_millis(p.to_str().unwrap().len() as u64 * 2));
	});
}
