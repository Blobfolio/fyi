/*!
# FYI Msg - Progless Signals.
*/

#[cfg(feature = "signals_sigint")]   use crate::Msg;
#[cfg(feature = "signals_sigint")]   use signal_hook::consts::SIGINT;
#[cfg(feature = "signals_sigwinch")] use signal_hook::consts::SIGWINCH;
#[cfg(feature = "signals_sigwinch")] use signal_hook::SigId;
use std::sync::{
	Arc,
	atomic::{
		AtomicBool,
		Ordering::SeqCst,
	},
};
#[cfg(feature = "signals_sigint")] use super::Progless;
#[cfg(feature = "signals_sigint")] use std::sync::OnceLock;
use super::ProglessInner;



#[cfg(feature = "signals_sigint")]
/// # `SIGINT` Handler.
///
/// There can only be one…
static SIGINT_HANLDER: OnceLock<Arc<AtomicBool>> = OnceLock::new();



#[cfg(feature = "signals_sigint")]
#[cfg_attr(docsrs, doc(cfg(feature = "signals_sigint")))]
impl Progless {
	#[must_use]
	/// # Two-Strike `SIGINT` Handler.
	///
	/// Implement a two-strike `SIGINT`-handling policy, performing some
	/// cleanup if and when the first signal is received, but only dying if
	/// there's a second.
	///
	/// The returned state variable can be used to check whether or not a
	/// `SIGINT` has come in, and change course accordingly.
	///
	/// ## Warnings
	///
	/// `SIGINT`-handling strategies are mutually exclusive and custom
	/// bindings are **not supported**.
	///
	/// This library offers three different options — default, two-strike, and
	/// keepalive — and implements whichever happens to be registered _first_.
	///
	/// As such, this method should be called as early as possible, and _before_
	/// any [`Progless`] instances are created, otherwise you'll be stuck with
	/// the default "immediate death" handling.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Progless;
	/// use std::sync::atomic::Ordering::SeqCst;
	///
	/// // Register a shutdown handler.
	/// let killed = Progless::sigint_two_strike();
	///
	/// loop {
	///     // CTRL+C was pressed once. Clean up and apologize!
	///     if killed.load(SeqCst) {
	///         eprintln!("So long and thanks for the fish!");
	///         return;
	///     }
	///
	///     // Do stuff as usual…
	/// }
	/// ```
	///
	/// ## Panics
	///
	/// This will panic if the handler cannot be registered for whatever
	/// reason. If another handler was already present, it will simply return
	/// _its_ switch instead.
	pub fn sigint_two_strike() -> &'static Arc<AtomicBool> {
		SIGINT_HANLDER.get_or_init(sigint_two_strike)
	}

	#[must_use]
	/// # Keepalive `SIGINT` Handler.
	///
	/// Implement a keepalive `SIGINT`-handling policy, performing some
	/// cleanup if and when the first signal is received, _without_ triggering
	/// any early exit. (The program will keep on keeping on.)
	///
	/// The returned state variable can be used to check whether or not a
	/// `SIGINT` has come in, and change course accordingly.
	///
	/// ## Warnings
	///
	/// `SIGINT`-handling strategies are mutually exclusive and custom
	/// bindings are **not supported**.
	///
	/// This library offers three different options — default, two-strike, and
	/// keepalive — and implements whichever happens to be registered _first_.
	///
	/// As such, this method should be called as early as possible, and _before_
	/// any [`Progless`] instances are created, otherwise you'll be stuck with
	/// the default "immediate death" handling.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Progless;
	/// use std::sync::atomic::Ordering::SeqCst;
	///
	/// // Register a non-shutdown shutdown handler.
	/// let killed = Progless::sigint_keepalive();
	/// let mut warned = false;
	///
	/// loop {
	///     // The first CTRL+C has arrived.
	///     if ! warned && killed.load(SeqCst) {
	///         warned = true;
	///         eprintln!("I hear you, but do not care!");
	///     }
	///
	///     // Do stuff as usual…
	/// }
	/// ```
	///
	/// ## Panics
	///
	/// This will panic if the handler cannot be registered for whatever
	/// reason. If another handler was already present, it will simply return
	/// _its_ switch instead.
	pub fn sigint_keepalive() -> &'static Arc<AtomicBool> {
		SIGINT_HANLDER.get_or_init(sigint_keepalive)
	}
}



/// # Signal Handlers.
///
/// This struct holds the custom signal handler(s) that are generated as a
/// consequence of initializing a new steady ticker, mostly just to keep things
/// nice and tidy.
///
/// Signals are feature-gated, so the specifics will vary.
pub(super) struct ProglessSignals {
	#[cfg(feature = "signals_sigint")]
	/// # `SIGINT` Handler.
	///
	/// This is used to help sneak some critical cleanup in before premature
	/// termination sets in. This in turn allows us to safely do dirty things
	/// at runtime, like hiding the cursor during progress render.
	sigint: &'static Arc<AtomicBool>,

	#[cfg(feature = "signals_sigwinch")]
	/// # `SIGWINCH` Handler.
	///
	/// This is used to reduce resize-related tick latency. Without it, the
	/// terminal size has to be freshly queried with each tick.
	sigwinch: Option<ResizeHandler>,
}

impl Default for ProglessSignals {
	#[inline]
	fn default() -> Self {
		#[cfg(feature = "signals_sigint")]
		let sigint = SIGINT_HANLDER.get_or_init(sigint_default);

		#[cfg(feature = "signals_sigint")]
		if ! sigint.load(SeqCst) { eprint!("{}", Progless::CURSOR_HIDE); }

		Self {
			#[cfg(feature = "signals_sigint")]
			sigint,

			#[cfg(feature = "signals_sigwinch")]
			sigwinch: ResizeHandler::new(),
		}
	}
}

#[cfg(feature = "signals_sigint")]
impl Drop for ProglessSignals {
	#[inline]
	fn drop(&mut self) {
		// The `ProglessSignal` drop glue handles unregistration; at this
		// level we just need to make sure the cursor gets unhidden.
		eprint!("{}", Progless::CURSOR_UNHIDE);
	}
}

impl ProglessSignals {
	/// # Pre-Tick Actions.
	///
	/// Perform some signal-specific touch-ups, if any, before ticking for
	/// real.
	///
	/// This will return `false` if ticking has stopped.
	pub(super) fn pretick(&self, inner: &Arc<ProglessInner>) -> bool {
		#[cfg(feature = "signals_sigwinch")]
		// Did we resize?
		if
			self.sigwinch.as_ref().is_none_or(|e| e.switch.swap(false, SeqCst)) &&
			! inner.tick_resize()
		{
			return false;
		}

		#[cfg(feature = "signals_sigint")]
		// Is imminent death pending?
		if self.sigint.load(SeqCst) && ! inner.sigint() {
			return false;
		}

		true
	}
}



#[cfg(feature = "signals_sigwinch")]
/// # Resize Handler.
///
/// This struct holds the information for a custom `SIGWINCH` signal listener
/// to track screen resize events. On drop, it will unregister itself.
struct ResizeHandler {
	/// # Switch.
	switch: Arc<AtomicBool>,

	/// # Signal ID.
	id: SigId,
}

#[cfg(feature = "signals_sigwinch")]
impl Drop for ResizeHandler {
	#[inline]
	/// # Unbind Handler.
	fn drop(&mut self) { signal_hook::low_level::unregister(self.id); }
}

#[cfg(feature = "signals_sigwinch")]
impl ResizeHandler {
	/// # New `SIGWINCH` handler.
	///
	/// Bind a `SIGWINCH` signal handler and return it.
	fn new() -> Option<Self> {
		// Start with a value of "true" to force a dimension query on first
		// tick.
		let switch = Arc::new(AtomicBool::new(true));
		let id = signal_hook::flag::register(SIGWINCH, Arc::clone(&switch)).ok()?;

		// Return it.
		Some(Self { switch, id })
	}
}



#[cfg(feature = "signals_sigint")]
#[expect(unsafe_code, reason = "For signal listener.")]
/// # "Default" `SIGINT` Handler.
///
/// Bind a new listener for `SIGINT` signals that sneaks in a little cleanup
/// before terminating as usual.
///
/// ## Panics
///
/// This will panic if the handler cannot be registered.
fn sigint_default() -> Arc<AtomicBool> {
	let switch = Arc::new(AtomicBool::new(false));

	// Safety: signal-hook marks manual registration unsafe because such
	// callbacks can be race-prone, but our inner operations are atomic.
	unsafe {
		let t_switch = Arc::clone(&switch);
		if signal_hook::low_level::register(SIGINT, move || {
			// Unhide our cursor if we changed state.
			if ! t_switch.swap(true, SeqCst) {
				eprint!("{}", Progless::CURSOR_UNHIDE);
			}

			// One strike and you're out!
			signal_hook::low_level::exit(1);
		}).is_err() { sigint_error(); }
	}

	switch
}

#[cfg(feature = "signals_sigint")]
#[expect(unsafe_code, reason = "For signal listener.")]
/// # Two-Strike `SIGINT` Handler.
///
/// Bind a new listener for `SIGINT` signals that performs cleanup on the first
/// signal, and shuts down on the second.
///
/// ## Panics
///
/// This will panic if the handler cannot be registered.
fn sigint_two_strike() -> Arc<AtomicBool> {
	let switch = Arc::new(AtomicBool::new(false));

	// Safety: signal-hook marks manual registration unsafe because such
	// callbacks can be race-prone, but our inner operations are atomic.
	unsafe {
		let t_switch = Arc::clone(&switch);
		if signal_hook::low_level::register(SIGINT, move || {
			// Terminate the process if the switch was already true.
			if t_switch.swap(true, SeqCst) { signal_hook::low_level::exit(1); }
			// Otherwise just unhide the cursor.
			else { eprint!("{}", Progless::CURSOR_UNHIDE); }
		}).is_err() { sigint_error(); }
	}

	switch
}

#[cfg(feature = "signals_sigint")]
#[expect(unsafe_code, reason = "For signal listener.")]
/// # Keepalive `SIGINT` Handler.
///
/// Bind a new listener for `SIGINT` signals that performs cleanup on the first
/// signal but otherwise allows everything to continue running.
///
/// ## Panics
///
/// This will panic if the handler cannot be registered.
fn sigint_keepalive() -> Arc<AtomicBool> {
	let switch = Arc::new(AtomicBool::new(false));

	// Safety: signal-hook marks manual registration unsafe because such
	// callbacks can be race-prone, but our inner operations are atomic.
	unsafe {
		let t_switch = Arc::clone(&switch);
		if signal_hook::low_level::register(SIGINT, move || {
			// Unhide the cursor.
			if ! t_switch.swap(true, SeqCst) {
				eprint!("{}", Progless::CURSOR_UNHIDE);
			}
		}).is_err() { sigint_error(); }
	}

	switch
}

#[cfg(feature = "signals_sigint")]
#[inline]
/// # `SIGINT` Handler Registration Error.
///
/// Print an error and die.
fn sigint_error() -> ! {
	Msg::error("Unable to register a SIGINT handler!").eprint();
	std::process::exit(1);
}
