/*!
# FYI Msg Example: Parallel Progress Bar w/ Task List.

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
	use dactyl::NiceU16;
	use fyi_msg::{
		AnsiColor,
		Progless,
	};
	use rayon::prelude::*;
	use std::time::Duration;

	eprintln!("Text output to STDERR the normal way.");

	// Initiate a progress bar.
	let pbar = Progless::try_from(FILE_TYPES.len()).unwrap()
		.with_title(Some(Msg::new(("Scanning", AnsiColor::Misc199), "Pretending to look for \"message\" file types…")));

	FILE_TYPES.par_iter()
		.map(|&t| (t, Duration::from_millis(t.len() as u64 * 3)))
		.for_each(|(txt, delay)| {
			// Start a new task.
			pbar.add(txt);

			// Example `push_msg` usage.
			if txt.starts_with("message/") {
				// Note this shouldn't fail in practice, but if STDERR is tied
				// up for whatever reason the original message is passed back.
				let _res = pbar.push_msg(Msg::new(("Found", AnsiColor::Misc199), txt));
			}

			// Simulate work.
			std::thread::sleep(delay);

			// Remove said task, which increments the "done" count by one.
			pbar.remove(txt);
		});

	// Let's do it again! We could start a new Progless, but let's keep the
	// original one going instead.
	let nums: Vec<u16> = (10_000_u16..11_000_u16).collect();

	// This would only fail if the new total is zero, which we know is not the
	// case here.
	pbar.try_reset(nums.len() as u32).unwrap();

	// Change the title.
	pbar.set_title(Some(Msg::new(("Crunching", AnsiColor::Misc199), "Playing with numbers now…")));

	// Loop through the new tasks.
	nums.into_par_iter()
		.for_each(|n| {
			let nice = NiceU16::from(n);
			pbar.add(nice.as_str());

			// Simulate work.
			std::thread::sleep(Duration::from_millis(99));

			pbar.remove(nice.as_str());
		});

	// We're really done now.
	pbar.finish();

	// Print a generic summary. The nicer `Progless::summary` summary would
	// only reflect the last incarnation, which isn't helpful here.
	Msg::from(pbar).eprint();

	eprintln!("More random text that has nothing to do with Progless.");
}
