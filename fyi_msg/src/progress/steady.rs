/*!
# FYI Msg - Progless Steady Ticker
*/

use std::{
	sync::{
		Arc,
		atomic::{
			AtomicBool,
			Ordering::{
				Relaxed,
				SeqCst,
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



/// # Tick Rate.
///
/// Delay between ticks in milliseconds.
pub(super) const TICK_RATE: u32 = 100;

/// # Sleep Duration.
///
/// `ProglessSteady` nap duration. This is half the value of the desired
/// `TICK_RATE` because there are two such naps per cycle.
const SLEEP: Duration = Duration::from_millis(TICK_RATE.wrapping_div(2) as u64);



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
	/// # Ticker Handle.
	ticker: Mutex<Option<JoinHandle<()>>>,

	/// # Is It Dead?
	dead: Arc<AtomicBool>,
}

impl Default for ProglessSteady {
	#[inline]
	fn default() -> Self {
		Self {
			ticker: Mutex::new(None),
			dead: Arc::new(AtomicBool::new(true)),
		}
	}
}

impl From<Arc<ProglessInner>> for ProglessSteady {
	fn from(t_inner: Arc<ProglessInner>) -> Self {
		let dead = Arc::new(AtomicBool::new(false));
		let t_dead = Arc::clone(&dead);

		Self {
			dead,
			ticker:  Mutex::new(Some(std::thread::spawn(move || loop {
				// This will abort if we've manually turned "dead" on, or if
				// "inner" has reached 100%. Until then, this will initiate a
				// steady "tick", which may or may not paint an update to the
				// CLI.
				if t_dead.load(Relaxed) || ! t_inner.tick(false) { break; }

				// Sleep for a short while before checking again.
				std::thread::sleep(SLEEP);
				if t_dead.load(Relaxed) { break; }
				std::thread::sleep(SLEEP);
			}))),
		}
	}
}

impl ProglessSteady {
	/// # Start.
	///
	/// Make sure the steady ticker is running.
	pub(super) fn start(&self, t_inner: Arc<ProglessInner>) {
		// Make sure the old steady ticker is dead.
		self.stop();

		// Reset!
		self.dead.store(false, SeqCst);
		let t_dead = Arc::clone(&self.dead);
		mutex!(self.ticker).replace(std::thread::spawn(move || loop {
			// This will abort if we've manually turned "dead" on, or if
			// "inner" has reached 100%. Until then, this will initiate a
			// steady "tick", which may or may not paint an update to the CLI.
			if t_dead.load(Relaxed) || ! t_inner.tick(false) { break; }

			// Sleep for a short while before checking again.
			std::thread::sleep(SLEEP);
			if t_dead.load(Relaxed) { break; }
			std::thread::sleep(SLEEP);
		}));
	}

	#[inline]
	/// # Stop.
	///
	/// Make sure the steady ticker has actually aborted. This is called
	/// automatically when [`Progless::finish`] is called.
	pub(super) fn stop(&self) {
		let handle = mutex!(self.ticker).take();
		if let Some(handle) = handle {
			self.dead.store(true, SeqCst);
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
