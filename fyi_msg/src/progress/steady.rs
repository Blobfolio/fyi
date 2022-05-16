/*!
# FYI Msg - Progless Steady Ticker
*/

use std::{
	sync::{
		Arc,
		atomic::{
			AtomicBool,
			Ordering::{
				Acquire,
				Release,
			},
		},
	},
	thread::JoinHandle,
	time::Duration,
};
use super::{
	Mutex,
	mutex,
	ProglessInner,
};



#[derive(Debug)]
/// # Steady Ticker.
///
/// Steady ticking is achieved by spawning a loop in a new thread that tries
/// to tick the progress bar once every 60ms.
///
/// The struct itself exists to hold the handle from that thread so that it can
/// run while it needs running, and stop once it needs to stop.
///
/// Stopping is triggered automatically in cases where the tick fails (because
/// i.e. the progress has reached 100%), or manually when the `enabled` field
/// is set to `false`. The latter is a failsafe for cases where the iterations
/// fail to add up to the declared total.
pub(super) struct ProglessSteady {
	ticker: Mutex<Option<JoinHandle<()>>>,
	enabled: Arc<AtomicBool>,
}

impl From<Arc<ProglessInner>> for ProglessSteady {
	fn from(t_inner: Arc<ProglessInner>) -> Self {
		const SLEEP: Duration = Duration::from_millis(60);
		let enabled = Arc::new(AtomicBool::new(true));
		let t_enabled = enabled.clone();

		Self {
			enabled,
			ticker:  Mutex::new(Some(std::thread::spawn(move || loop {
				// This will abort if we've manually shut off the "enabled"
				// field, or if "inner" has reached 100%. Otherwise this will
				// initiate a "tick", which may or may not paint an update to
				// the CLI.
				if ! t_enabled.load(Acquire) || ! t_inner.tick() { break; }

				// Sleep for a short while before checking again.
				std::thread::sleep(SLEEP);
			}))),
		}
	}
}

impl ProglessSteady {
	/// # Stop.
	///
	/// Make sure the steady ticker has actually aborted. This is called
	/// automatically when [`Progless::finish`] is called.
	pub(super) fn stop(&self) {
		if let Some(handle) = mutex!(self.ticker).take() {
			self.enabled.store(false, Release);
			handle.join().unwrap();
		}
	}
}

impl Drop for ProglessSteady {
	#[inline]
	/// # Drop.
	///
	/// Make sure the spawned steady tick thread has actually stopped. If the
	/// caller forgot to run [`Progless::finish`] it might keep doing its
	/// thing.
	fn drop(&mut self) { self.stop(); }
}
