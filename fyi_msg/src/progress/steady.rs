/*!
# FYI Msg - Progless Steady Ticker
*/

use std::{
	sync::{
		Arc,
		Mutex,
		Condvar,
		LockResult,
	},
	thread::JoinHandle,
	time::Duration,
};
use super::{
	mutex,
	ProglessInner,
};

#[cfg(any(feature = "signals_sigint", feature = "signals_sigwinch"))]
use super::signals::ProglessSignals;



#[derive(Debug)]
/// # Steady Ticker.
///
/// Steady ticking is achieved by spawning a loop in a new thread that tries
/// to tick the progress bar once every 100ms.
///
/// The struct itself exists to hold the handle from that thread so that it can
/// run while it needs running, and stop once it needs to stop.
pub(super) struct ProglessSteady {
	/// # Ticker Thread Handle.
	ticker: Mutex<Option<JoinHandle<()>>>,

	/// # Ticker State.
	///
	/// Because `ProglessInner` cannot implement `Drop`, we need an independent
	/// state control to prevent zombie ticking in cases where the user
	/// accidentally leaves things unfinished.
	state: Arc<(Mutex<bool>, Condvar)>,
}

impl Default for ProglessSteady {
	#[inline]
	fn default() -> Self {
		Self {
			ticker: Mutex::new(None),
			state: Arc::new((Mutex::new(true), Condvar::new())),
		}
	}
}

impl From<Arc<ProglessInner>> for ProglessSteady {
	#[inline]
	fn from(t_inner: Arc<ProglessInner>) -> Self {
		let state = Arc::new((Mutex::new(false), Condvar::new()));
		let t_state = Arc::clone(&state);

		Self {
			state,
			ticker:  Mutex::new(Some(spawn_ticker(t_state, t_inner))),
		}
	}
}

impl ProglessSteady {
	/// # Tick Rate.
	///
	/// Progress "animation" is more _Speed Racer_ than _Lion King_; painting
	/// every hundred milliseconds or so is plenty.
	const TICK_RATE: Duration = Duration::from_millis(100);

	/// # Start.
	///
	/// Make sure the steady ticker is up and running!
	pub(super) fn start(&self, t_inner: Arc<ProglessInner>) {
		// Make sure the old steady ticker is dead.
		self.stop();

		// Reset!
		*mutex!(self.state.0) = false;
		let t_state = Arc::clone(&self.state);
		mutex!(self.ticker).replace(spawn_ticker(t_state, t_inner));
	}

	#[inline]
	/// # Stop.
	///
	/// Make sure the steady ticker has actually aborted. This is called
	/// automatically when [`Progless::finish`](crate::Progless::finish) is called or the instance is
	/// dropped.
	pub(super) fn stop(&self) {
		let handle = mutex!(self.ticker).take();
		if let Some(handle) = handle {
			if ! *mutex!(self.state.0) {
				*mutex!(self.state.0) = true;
				self.state.1.notify_all();
			}
			handle.join().unwrap();
		}
	}
}

impl Drop for ProglessSteady {
	#[inline]
	fn drop(&mut self) { self.stop(); }
}



#[inline]
/// # Spawn Ticker.
///
/// Spawn a new thread to issue steady-ish ticks until the associated progress
/// completes or a hard stop gets issued.
///
/// This may or may not make use of signals, depending on which crate features
/// are enabled.
fn spawn_ticker(t_state: Arc<(Mutex<bool>, Condvar)>, t_inner: Arc<ProglessInner>)
-> JoinHandle<()> {
	std::thread::spawn(move || {
		#[cfg(any(feature = "signals_sigint", feature = "signals_sigwinch"))]
		let signals = ProglessSignals::default();

		// Tick while the ticking's good.
		let (t_dead, t_cond) = &*t_state;
		let mut state = mutex!(t_dead);
		while let LockResult::Ok(res) = t_cond.wait_timeout(state, ProglessSteady::TICK_RATE) {
			state = res.0;
			if *state { return; } // Dead!

			#[cfg(any(feature = "signals_sigint", feature = "signals_sigwinch"))]
			// Dead, but from the other end.
			if ! signals.pretick(&t_inner) || ! t_inner.tick(false) {
				*state = true; // Update the state to match.
				drop(state);
				drop(signals);
				return;
			}

			#[cfg(not(any(feature = "signals_sigint", feature = "signals_sigwinch")))]
			// Dead, but from the other end.
			if ! t_inner.tick(false) {
				*state = true; // Update the state to match.
				drop(state);
				return;
			}
		}
	})
}
