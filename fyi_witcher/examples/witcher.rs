/*!
# FYI Witcher Example: Find + Progress
*/

use fyi_msg::MsgKind;
use fyi_witcher::{
	Witcher,
	WITCHING_SUMMARIZE,
};
use std::{
	thread,
	time::Duration,
};



/// Do it.
fn main() {
	// Search for gzipped MAN pages.
	Witcher::default()
		.with_ext1(b".gz")
		.with_path("/usr/share/man")
		.into_witching()
		.with_title(MsgKind::new("Witcher Demo", 199).into_msg("Gzipped MAN Pages"))
		.with_flags(WITCHING_SUMMARIZE)
		.with_labels("manual", "manuals")
		.run(|p| {
			thread::sleep(Duration::from_millis(p.to_str().unwrap().len() as u64 * 2));
		});
}
