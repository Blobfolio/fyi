/*!
# FYI Msg Example: Serial Progress Bar.

**This requires the `progress` crate feature.**
*/

use fyi_msg::Msg;



#[cfg(feature = "progress")]
include!("_progless-data.txt");

#[cfg(not(feature = "progress"))]
fn main() -> std::process::ExitCode {
	Msg::error("This example requires the 'progress' feature.").eprint();
	std::process::ExitCode::FAILURE
}

#[cfg(feature = "progress")]
/// # Do it.
fn main() {
	use fyi_msg::{
		MsgKind,
		Progless,
	};
	use std::time::Duration;

	eprintln!("Text output to STDERR the normal way.");

	// Initiate a progress bar.
	let pbar = Progless::try_from(FILE_TYPES.len()).unwrap()
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
	pbar.finish();
	pbar.summary(MsgKind::Crunched, "file", "files").print();

	eprintln!("More random text that has nothing to do with Progless.");
}
