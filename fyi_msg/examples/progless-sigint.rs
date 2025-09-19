/*!
# FYI Msg Example: Progress w/ graceful SIGINT handling.

**This requires the `progress` and `signals_sigint` crate features.**
*/

use fyi_msg::Msg;



#[cfg(not(all(feature = "progress", feature = "signals_sigint")))]
fn main() -> std::process::ExitCode {
	Msg::error("This example requires the 'progress' and 'signals_sigint' features.").eprint();
	std::process::ExitCode::FAILURE
}

#[cfg(all(feature = "progress", feature = "signals_sigint"))]
/// # Do it.
fn main() {
	use fyi_msg::Progless;
	use std::{
		num::NonZeroU32,
		sync::atomic::Ordering::SeqCst,
		time::Duration,
	};

	eprintln!("Press CTRL+C once for a speed-up.");
	eprintln!("Press it again for immediate shutdown.");

	// In this example, we want to perform some "cleanup" on the first SIGINT,
	// only accepting premature death on the second. There's a helper for this
	// precise use case, it just has to be called before the first Progless
	// instance is created:
	let killed = Progless::sigint_two_strike();

	// Initialize a progress bar the usual way.
	let pbar = Progless::try_from(500).unwrap()
		.with_title(Some(Msg::task("Counting slowly…")));

	// Simulate work and impatience.
	for _ in 0..500 {
		let duration =
			if killed.load(SeqCst) { Duration::from_millis(1) } // Early shutdown.
			else { Duration::from_millis(90) };                 // Normal.
		std::thread::sleep(duration);
		pbar.increment();
	}

	// If we make it this far, death will be a natural one.
	pbar.finish();

	// Go again?
	if fyi_msg::confirm!(@yes "Count some more?") {
		pbar.reset(NonZeroU32::new(250).unwrap());
		pbar.set_title(Some(Msg::task("More numbers…")));
		for _ in 500..750 {
			let duration =
				if killed.load(SeqCst) { Duration::from_millis(1) } // Early shutdown.
				else { Duration::from_millis(90) };                 // Normal.
			std::thread::sleep(duration);
			pbar.increment();
		}
	}

	Msg::success("The program terminated naturally.").eprint();
}
