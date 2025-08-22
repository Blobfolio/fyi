/*!
# FYI Msg - Progless
*/

pub(super) mod ba;
pub(super) mod error;
mod steady;
pub(super) mod guard;

#[cfg(any(feature = "signals_sigint", feature = "signals_sigwinch"))]
pub(super) mod signals;



use crate::{
	ansi::{
		AnsiColor,
		NoAnsi,
	},
	Msg,
	MsgKind,
	ProglessError,
	ProglessTaskGuard,
};
use dactyl::{
	NiceClock,
	NiceElapsed,
	NicePercent,
	NiceU32,
	traits::{
		NiceInflection,
		SaturatingFrom,
	},
};
use fyi_ansi::csi;
use std::{
	collections::BTreeSet,
	io::{
		IoSlice,
		StderrLock,
		Write,
	},
	num::{
		NonZeroU8,
		NonZeroU16,
		NonZeroU32,
		NonZeroU64,
		NonZeroUsize,
		NonZeroU128,
	},
	sync::{
		Arc,
		Mutex,
		atomic::{
			AtomicU8,
			AtomicU16,
			AtomicU32,
			AtomicU64,
			Ordering::SeqCst,
		},
	},
	time::{
		Duration,
		Instant,
	},
};
use steady::ProglessSteady;



/// # Bar Filler (Done).
static BAR_DONE:   [u8; 256] = [b'#'; 256];

/// # Dash Filler (TBD).
static BAR_UNDONE: [u8; 256] = [b'-'; 256];

/// # Clear Screen.
///
/// This ANSI sequence is used to clear the screen from the current cursor
/// position (i.e. everything _after_).
const CLS: &[u8] = b"\x1b[J";

/// # Helper: Mutex Unlock.
///
/// This just moves tedious code out of the way.
macro_rules! mutex {
	($m:expr) => ($m.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
}

/// # Helper: Extract Done.
///
/// The `done` value is stored in the upper 32 bits of the 64-bit `done_total`.
macro_rules! done {
	($done_total:expr) => ($done_total >> 32);
}

/// # Helper: Extract Total.
///
/// The `total` value is stored in the lower 32 bits of the 64-bit `done_total`.
macro_rules! total {
	($done_total:expr) => ($done_total & 0x0000_0000_FFFF_FFFF_u64);
}

/// # Helper: Merge Done and Total.
///
/// Merge two `u32` values into a single `u64` by shifting the first into the
/// upper bits.
macro_rules! done_total {
	($done:expr, $total:expr) => (($done << 32) | $total);
}

use mutex;



// Tick Flags.
// These flags indicate whether or not a given component has changed since the
// last tick, saving the overhead of recalculating the buffer values each time
// a value changes. (Instead they're only recalculated at most once per tick.)

/// # Flag: Initial State.
const TICK_NEW: u8 =
	TICK_BAR | TICK_TOTAL | TICKING;

/// # Flag: Percent.
const TICK_PERCENT: u8 =
	TICK_DONE | TICK_TOTAL;

/// # Flag: Reset.
const TICK_RESET: u8 =
	TICK_BAR | TICK_DOING | TICK_DONE | TICK_TITLE | TICK_TOTAL | TICKING;

/// # Flag: Resized.
const TICK_RESIZED: u8 =
	TICK_BAR | TICK_DOING | TICK_TITLE;

/// # Flag: Drawables.
const TICK_DRAWABLE: u8 =
	TICK_BAR | TICK_DOING | TICK_DONE | TICK_TITLE | TICK_TOTAL;

/// # Flag: Repaint Bar.
const TICK_BAR: u8 =     0b0000_0001;

/// # Flag: Repaint Task List.
const TICK_DOING: u8 =   0b0000_0010;

/// # Flag: Repaint Done Value.
const TICK_DONE: u8 =    0b0000_0100;

/// # Flag: Repaint Title.
const TICK_TITLE: u8 =   0b0000_1000;

/// # Flag: Repaint Total Value.
const TICK_TOTAL: u8 =   0b0001_0000;

/// # Flag: Is Ticking?
const TICKING: u8 =      0b0010_0000;

#[cfg(feature = "signals_sigint")]
/// # Flag: SIGINT Received?
const SIGINT: u8 =       0b0100_0000;

/// # Minimum Bar Width.
const MIN_BARS_WIDTH: u8 = 10;

/// # Minimum Draw Width.
const MIN_DRAW_WIDTH: u8 = 10;



#[derive(Debug)]
/// # Progless Inner Data.
///
/// This holds most of the actual progress state information. The public
/// struct holds an instance of this behind an [`std::sync::Arc`] for easier
/// thread-sharing.
struct ProglessInner {
	/// # Buffer.
	buf: Mutex<ProglessBuffer>,

	/// # Flags.
	flags: AtomicU8,

	/// # Last Width/Height.
	///
	/// The screen dimensions (columns and rows) from the last print (so we
	/// know when it changes). They're always accessed together so share the
	/// same storage to improve consistency and reduce atomic ops.
	last_size: AtomicU16,

	/// # Cycle Number.
	///
	/// This value is incremented with each call to [`Progless::reset`] and
	/// serves as a control to help prevent a [`ProglessTaskGuard`] from one
	/// cycle affecting the totals of a subsequent one.
	cycle: AtomicU8,

	/// # Start Time.
	///
	/// The instant the object was first created. All timings are derived from
	/// this value.
	started: Instant,

	/// # Elapsed Seconds.
	///
	/// The number of elapsed seconds as of the last tick (so we know when to
	/// update the corresponding buffer part).
	elapsed: AtomicU32,

	/// # Title.
	title: Mutex<Option<Msg>>,

	/// # Done/Total Tasks.
	///
	/// Like the screen dimensions, the done and total values are tightly
	/// bound to one another so are merged together for storage to improve
	/// consistency and reduce the atomic ops.
	done_total: AtomicU64,

	/// # Active Task List.
	doing: Mutex<BTreeSet<String>>,
}

impl Default for ProglessInner {
	#[inline]
	fn default() -> Self {
		Self {
			buf: Mutex::new(ProglessBuffer::DEFAULT),
			flags: AtomicU8::new(0),

			last_size: AtomicU16::new(0),

			cycle: AtomicU8::new(0),
			started: Instant::now(),
			elapsed: AtomicU32::new(0),

			title: Mutex::new(None),
			done_total: AtomicU64::new(1),
			doing: Mutex::new(BTreeSet::default()),
		}
	}
}

impl From<NonZeroU32> for ProglessInner {
	#[inline]
	fn from(total: NonZeroU32) -> Self {
		Self {
			flags: AtomicU8::new(TICK_NEW),
			done_total: AtomicU64::new(u64::from(total.get())),
			..Self::default()
		}
	}
}

/// # Helper: generate `From` for small non-zero types.
macro_rules! inner_nz_from {
	($($ty:ty),+ $(,)?) => ($(
		impl From<$ty> for ProglessInner {
			#[inline]
			fn from(total: $ty) -> Self {
				Self {
					flags: AtomicU8::new(TICK_NEW),
					done_total: AtomicU64::new(u64::from(total.get())),
					..Self::default()
				}
			}
		}
	)+)
}
inner_nz_from!(NonZeroU8, NonZeroU16);

/// # Helper: generate `TryFrom` for large non-zero types.
macro_rules! inner_nz_tryfrom {
	($($ty:ty),+ $(,)?) => ($(
		impl TryFrom<$ty> for ProglessInner {
			type Error = ProglessError;

			#[inline]
			#[allow(
				clippy::allow_attributes,
				trivial_numeric_casts,
				reason = "We don't need another goddamn macro. Haha.",
			)]
			#[allow(clippy::cast_possible_truncation, reason = "We're checking for fit.")]
			fn try_from(total: $ty) -> Result<Self, Self::Error> {
				let total = total.get();
				if total <= 4_294_967_295 {
					Ok(Self {
						flags: AtomicU8::new(TICK_NEW),
						done_total: AtomicU64::new(total as u64),
						..Self::default()
					})
				}
				else { Err(ProglessError::TotalOverflow) }
			}
		}
	)+)
}
inner_nz_tryfrom!(NonZeroU64, NonZeroUsize, NonZeroU128);

/// # Helper: generate `TryFrom` for all non-`u32` integer types.
macro_rules! inner_tryfrom {
	($($ty:ty),+ $(,)?) => ($(
		impl TryFrom<$ty> for ProglessInner {
			type Error = ProglessError;

			#[inline]
			fn try_from(total: $ty) -> Result<Self, Self::Error> {
				u32::try_from(total)
					.map_err(|_| ProglessError::TotalOverflow)
					.and_then(Self::try_from)
			}
		}
	)+)
}

inner_tryfrom!(
	u8, u16,      u64, usize, u128,
	i8, i16, i32, i64, isize, i128,
);

impl TryFrom<u32> for ProglessInner {
	type Error = ProglessError;

	#[inline]
	fn try_from(total: u32) -> Result<Self, Self::Error> {
		NonZeroU32::new(total)
			.ok_or(ProglessError::EmptyTotal)
			.map(Self::from)
	}
}

/// # Construction/Destruction.
impl ProglessInner {
	/// # Stop.
	///
	/// Force an end to progress. This may be called manually to abort in the
	/// middle, otherwise it will trigger automatically once the done count
	/// reaches the total count.
	///
	/// Calling this will freeze the elapsed time (for future reference as
	/// needed), set "done" equal to "total", and clear any active tasks. It
	/// will also erase the CLI progress bar from the screen.
	fn stop(&self) {
		// Shut 'er down!
		let flags = self.flags.swap(0, SeqCst);
		if TICKING == flags & TICKING {
			// Acquire the lock a little early just in case there is a
			// final in-progress tick.
			let mut handle = std::io::stderr().lock();

			// Make sure "done" equals "total".
			let done_total = self.done_total.load(SeqCst);
			let total = total!(done_total);
			if total != done!(done_total) {
				self.done_total.store(done_total!(total, total), SeqCst);
			}

			// Freeze the time.
			self.elapsed.store(
				u32::saturating_from(self.started.elapsed().as_secs()),
				SeqCst
			);

			// Clear the tasks.
			mutex!(self.doing).clear();

			// Clear the screen for good measure.
			let _res = handle.write_all(CLS).and_then(|()| handle.flush());
		}
	}
}

/// # Getters.
impl ProglessInner {
	#[inline]
	/// # Cycle Number.
	fn cycle(&self) -> u8 { self.cycle.load(SeqCst) }

	#[inline]
	/// # Is Ticking.
	///
	/// This is `true` so long as `done` does not equal `total`, and `total`
	/// is greater than `0`. Otherwise it is `false`.
	///
	/// For the most part, this struct's setter methods only work while
	/// progress is happening; after that they're frozen.
	fn running(&self) -> bool { TICKING == self.flags.load(SeqCst) & TICKING }
}

/// # Setters.
impl ProglessInner {
	/// # Add Task Guard.
	///
	/// Add a new task to the `doing` list, returning a copy of the sanitized
	/// text that was used (if any).
	///
	/// If the instance is inactive, `None` is returned.
	fn add_guard(&self, task: &str) -> Option<String> {
		if self.running() {
			// Sanitize the task and try to add it to the list.
			if let Some(task) = progless_task(task) && mutex!(self.doing).insert(task.clone()) {
				self.flags.fetch_or(TICK_DOING, SeqCst);
				Some(task)
			}

			// Return an empty string if the task came up empty or wasn't
			// unique.
			else { Some(String::new()) }
		}
		else { None }
	}

	/// # Remove Task Guard.
	///
	/// Remove the task (if any) from the list and increment the done count
	/// by one (unless `! increment`), if the instance is still running and
	/// the cycle hasn't changed.
	///
	/// This is called by [`ProglessTaskGuard::drop`].
	fn remove_guard(&self, task: &str, cycle: u8, increment: bool) {
		if self.running() && self.cycle.load(SeqCst) == cycle {
			// Remove the task, if any.
			if ! task.is_empty() && mutex!(self.doing).remove(task) {
				self.flags.fetch_or(TICK_DOING, SeqCst);
			}

			// Increment?
			if increment { self.increment_n(1); }
		}
	}

	#[inline]
	/// # Increment Done by N.
	///
	/// Increase the completed count by `n`. This is safer to use than `set_done()`
	/// and more efficient than calling `increment()` a million times in a row.
	fn increment_n(&self, n: u32) {
		if n != 0 && self.running() {
			// Don't bother recasting the parts to u32; leaving them as-is
			// moots addition overflow and simplifies the subsequent joining.
			let done_total = self.done_total.load(SeqCst);
			let done = done!(done_total) + u64::from(n);
			let total = total!(done_total);

			if done < total {
				self.done_total.store(done_total!(done, total), SeqCst);
				self.flags.fetch_or(TICK_DONE | TICK_BAR, SeqCst);
			}
			// Time to call it quits!
			else { self.stop(); }
		}
	}

	/// # Push Message.
	///
	/// "Insert" (print) a line (to STDERR) above the running progress bar,
	/// useful for realtime debug logs, warnings, etc., that would otherwise
	/// have to wait for the [`Progless`] instance to finish hogging the
	/// display.
	///
	/// ## Errors
	///
	/// In practice this should never fail, but if for some reason STDERR is
	/// tied up the original message is passed back as an error in case you
	/// want to try to deal with it yourself.
	fn push_msg(&self, msg: Msg) -> Result<(), Msg> {
		let msg = msg.with_newline(true);

		// If the progress is active, we have to do some things.
		if self.running() {
			// Clear the screen, then print the message.
			let mut handle = std::io::stderr().lock();
			let res = handle.write_all(CLS)
				.and_then(|()| handle.write_all(msg.as_bytes()))
				.and_then(|()| handle.flush())
				.is_err();
			drop(handle);

			// To complete the illusion, restore the progress bits.
			self.tick(true);

			// This shouldn't happen.
			if res { return Err(msg); }
		}
		// Otherwise we can just print it directly.
		else { msg.eprint(); }

		Ok(())
	}

	/// # Reset.
	///
	/// Stop the current run (if any), clear the done/doing metrics, and assign
	/// a new total so you can re-use the [`Progless`] instance for a new set
	/// of tasks.
	///
	/// Note: the start/elapsed times for a given [`Progless`] instance are
	/// _continuous_. If you need the time counter to reset to `[00:00:00]`,
	/// you need start a brand new instance instead of resetting an existing
	/// one.
	fn reset(&self, total: NonZeroU32) {
		self.stop();
		self.cycle.fetch_add(1, SeqCst); // Bump the cycle.
		self.done_total.store(u64::from(total.get()), SeqCst);
		self.flags.store(TICK_RESET, SeqCst);
	}

	/// # Set Done.
	///
	/// Set the done count to a specific value. Be careful in cases where
	/// things are happening in parallel; in such cases `increment` is probably
	/// better.
	fn set_done(&self, done: u32) {
		if self.running() {
			let done = u64::from(done);
			let done_total = self.done_total.load(SeqCst);
			if done != done!(done_total) {
				let total = total!(done_total);
				if done < total {
					self.done_total.store(done_total!(done, total), SeqCst);
					self.flags.fetch_or(TICK_DONE | TICK_BAR, SeqCst);
				}
				// Time to call it quits!
				else { self.stop(); }
			}
		}
	}

	/// # Set Title.
	///
	/// Give the progress bar a title, which will be shown above the progress
	/// bits while progress is progressing, and removed afterward with
	/// everything else.
	fn set_title(&self, title: Option<Msg>) {
		*mutex!(self.title) = title.map(|m| m.with_newline(false));
		if self.running() { self.flags.fetch_or(TICK_TITLE, SeqCst); }
	}

	/// # Set Title Message.
	///
	/// Change (only) the message part — what follows the prefix — of the title
	/// to something else.
	///
	/// Note: this has no effect for instances without a title component.
	fn set_title_msg(&self, msg: &str) {
		if let Some(title) = mutex!(self.title).as_mut() {
			title.set_msg(msg);
			if self.running() { self.flags.fetch_or(TICK_TITLE, SeqCst); }
		}
	}

	#[cfg(feature = "signals_sigint")]
	/// # Set SIGINT.
	///
	/// This method is used to indicate that a SIGINT was received and that
	/// the tasks are being wound down (early).
	///
	/// For the running [`Progless`], all this really means is that the title
	/// will be changed to "Early shutdown in progress." (This is purely a
	/// visual thing.)
	///
	/// The caller must still run [`Progless::finish`] to close everything up
	/// when the early shutdown actually arrives.
	///
	/// This will return `false` if ticking has stopped, otherwise `true`.
	fn sigint(&self) -> bool {
		let flags = self.flags.load(SeqCst);
		if TICKING == flags & (SIGINT | TICKING) {
			mutex!(self.title).replace(Msg::new(MsgKind::Warning, "Early shutdown in progress."));
			self.flags.fetch_or(SIGINT | TICK_TITLE, SeqCst);
			true
		}
		else { TICKING == flags & TICKING }
	}
}

/// # Ticks.
impl ProglessInner {
	#[expect(clippy::cast_possible_truncation, reason = "It is what it is.")]
	/// # Tick.
	///
	/// Ticking takes all of the changed values (since the last tick), updates
	/// their corresponding parts in the buffer, and prints the result, if any.
	///
	/// To avoid pointless work, this has no effect unless something drawable
	/// changed between ticks or `force` is `true`.
	fn tick(&self, force: bool) -> bool {
		// We aren't running!
		if ! self.running() { return false; }

		// Lock STDERR as early as possible to keep the state as consistent as
		// possible, even though we may well not end up using it.
		let mut handle = std::io::stderr().lock();

		// Pull the terminal dimensions.
		let Some((width, height)) = self.tick_set_size() else {
			// The size either changed between ticks or cannot be determined.
			// Either way, let's skip a turn and wait for the state to
			// stabilize.
			return true;
		};

		// If we don't even have enough space for a percentage, clear the
		// screen and call it a day.
		if width.get() < MIN_DRAW_WIDTH {
			let _res = handle.write_all(CLS).and_then(|()| handle.flush());
			return true;
		}

		// Did anything paint-worthy happen between ticks?
		let mut ticked = match self.tick_set_drawable() {
			// Yes!
			Some(t) => t,
			None =>
				// No, but it doesn't matter; "push_msg" cleared the screen so
				// a repaint is required.
				if force { 0 }
				// No, and there's nothing for us to do.
				else { return true; },
		};

		// Lock the internal buffer holding the print-formatted progress
		// components. Everything we do from here on out will require it.
		let mut buf = mutex!(self.buf);

		// The actual progress-related parts of the progress output are all
		// interrelated, so it's best to handle their buffer-patching together.
		if 0 != ticked & (TICK_DONE | TICK_TOTAL | TICK_BAR) {
			// The bar and percentage parts depend on the values of done and
			// total just as done and total depend on themselves, so whether or
			// not they updated, we'll need to know what they are.
			let done_total = self.done_total.load(SeqCst);
			let done = done!(done_total) as u32;
			let total = total!(done_total) as u32;

			// If the done value changed, update its buffer.
			if TICK_DONE == ticked & TICK_DONE { buf.done.replace(done); }

			// Likewise but less likely, the total.
			if TICK_TOTAL == ticked & TICK_TOTAL { buf.total.replace(total); }

			// The percentage is tied to both done and total, so if either
			// value changed, we'll need to update its buffer.
			if 0 != ticked & TICK_PERCENT {
				let percent =
					if done == 0 || total == 0 { 0.0 }
					else if done >= total { 1.0 }
					else { (f64::from(done) / f64::from(total)) as f32 };
				buf.percent.replace(percent);
			}

			// The bar formatting depends on both the values and sizing of the
			// other components, so their buffers will always need to be
			// recalculated, and recalculated _last_.
			buf.set_bars(width, done, total);
		}

		// Titles don't change very often, but they're given display priority
		// over the tasks so need to be checked first.
		if TICK_TITLE == ticked & TICK_TITLE {
			// Did we have a title and tasks last time?
			let before = buf.doing.is_empty() || ! buf.title.is_empty();

			// Update it.
			buf.set_title(mutex!(self.title).as_ref(), width, height);

			// If we now have a title and didn't before, and there were tasks
			// potentially competing for space, force a task redraw to make
			// sure the extra (title) line isn't one line too many.
			if ! before && ! buf.title.is_empty() {
				ticked |= TICK_DOING;
			}
		}

		// If the task list changed, update its buffer.
		if TICK_DOING == ticked & TICK_DOING {
			buf.set_doing(&mutex!(self.doing), width, height);
		}

		// We made it! Print and return.
		buf.print(width, &mut handle);
		true
	}

	/// # Tick Drawable Changes.
	///
	/// Compute and unset the drawable changes since the last tick and update
	/// the timestamp.
	///
	/// Returns `None` if nothing paint-worthy changed, or the sections (as a
	/// bitflag) requiring buffer updates before display.
	fn tick_set_drawable(&self) -> Option<u8> {
		let secs = self.tick_set_secs();
		let flags = self.flags.fetch_and(! TICK_DRAWABLE, SeqCst) & TICK_DRAWABLE;
		if secs || flags != 0 { Some(flags) }
		else { None }
	}

	/// # Tick Elapsed Seconds.
	///
	/// The precision of `Instant` is greater than we need for printing
	/// purposes; here we're just looking to see if one or more seconds have
	/// elapsed since the last tick.
	///
	/// Because this is relative to the tick rather than the overall state of
	/// progress, it has no corresponding tick flag.
	///
	/// Returns `true` if the seconds have changed since the last check,
	/// otherwise false.
	///
	/// Note that unlike the other components, this manages both the raw and
	/// formatted values.
	fn tick_set_secs(&self) -> bool {
		// No change to the seconds bit.
		let secs: u32 = u32::saturating_from(self.started.elapsed().as_secs());
		if secs == self.elapsed.swap(secs, SeqCst) { false }
		else {
			mutex!(self.buf).elapsed.replace(secs);
			true
		}
	}

	#[cfg(feature = "signals_sigwinch")]
	/// # Set Tick Width/Height.
	///
	/// When signal support is enabled, this method is used to query and set
	/// the terminal dimensions and toggle the corresponding flags.
	///
	/// This will return `false` if progress has stopped, otherwise `true`.
	fn tick_resize(&self) -> bool {
		if self.running() {
			if let Some((width, height)) = term_size() {
				let wh = u16::from_le_bytes([width.get(), height.get()]);
				if wh != self.last_size.swap(wh, SeqCst) {
					self.flags.fetch_or(TICK_RESIZED, SeqCst);
				}
			}
			true
		}
		else { false }
	}

	#[cfg(feature = "signals_sigwinch")]
	/// # Tick Width/Height.
	///
	/// When signal support is enabled, this doesn't need to set anything; it
	/// simply returns the cached terminal dimensions, unless zero.
	fn tick_set_size(&self) -> Option<(NonZeroU8, NonZeroU8)> {
		let [width, height] = self.last_size.load(SeqCst).to_le_bytes();
		let width = NonZeroU8::new(width)?;
		let height = NonZeroU8::new(height)?;
		Some((width, height))
	}

	#[cfg(not(feature = "signals_sigwinch"))]
	/// # Tick Width/Height.
	///
	/// Without signal support, we need to query the terminal dimensions with
	/// each tick and work backwards to figure out if anything changed.
	///
	/// This version of this method does that, returning the result if
	/// non-zero.
	fn tick_set_size(&self) -> Option<(NonZeroU8, NonZeroU8)> {
		let (width, height) = term_size()?;
		let wh = u16::from_le_bytes([width.get(), height.get()]);
		if wh == self.last_size.swap(wh, SeqCst) { Some((width, height)) }
		else {
			self.flags.fetch_or(TICK_RESIZED, SeqCst);
			None
		}
	}
}



#[derive(Debug)]
/// # Progless Output Buffers.
///
/// This holds formatted copies of the various progress parts (from a
/// `ProglessInner` instance), serving as a sort of custom `MsgBuffer`.
///
/// These values are only updated as-needed during ticks, then passed to
/// STDERR.
struct ProglessBuffer {
	/// # Title (Width-Constrained).
	title: Vec<u8>,

	/// # Elapsed Time (HH:MM:SS).
	elapsed: NiceClock,

	/// # The "Done" Part of the Bar.
	bar_done: &'static [u8],

	/// # The "TBD" Part of the Bar.
	bar_undone: &'static [u8],

	/// # Number Done (Formatted).
	done: NiceU32,

	/// # Number Total (Formatted).
	total: NiceU32,

	/// # Percentage Done (Formatted).
	percent: NicePercent,

	/// # Tasks (Width-Constrained).
	doing: Vec<u8>,

	/// # Task Lines.
	lines_doing: u8,
}

impl ProglessBuffer {
	/// # Default.
	const DEFAULT: Self = Self {
		title: Vec::new(),
		elapsed: NiceClock::MIN,
		bar_done: &[],
		bar_undone: &[],
		done: NiceU32::MIN,
		total: NiceU32::MIN,
		percent: NicePercent::MIN,
		doing: Vec::new(),
		lines_doing: 0,
	};
}

impl ProglessBuffer {
	#[inline(never)]
	/// # Write It!
	///
	/// This writes the fully-formatted progress data to STDERR, returning the
	/// status as a bool.
	fn print(&self, width: NonZeroU8, handle: &mut StderrLock<'static>) -> bool {
		use std::io::ErrorKind;

		/// # Progress Output Closer.
		///
		/// Reset the styles, add a line break, and rewind to the start.
		static CLOSE: [&[u8]; 256] = [
			b"\x1b[0m\r", b"\x1b[0m\r\x1b[1A", b"\x1b[0m\r\x1b[2A", b"\x1b[0m\r\x1b[3A", b"\x1b[0m\r\x1b[4A", b"\x1b[0m\r\x1b[5A", b"\x1b[0m\r\x1b[6A", b"\x1b[0m\r\x1b[7A", b"\x1b[0m\r\x1b[8A", b"\x1b[0m\r\x1b[9A", b"\x1b[0m\r\x1b[10A", b"\x1b[0m\r\x1b[11A", b"\x1b[0m\r\x1b[12A", b"\x1b[0m\r\x1b[13A", b"\x1b[0m\r\x1b[14A", b"\x1b[0m\r\x1b[15A",
			b"\x1b[0m\r\x1b[16A", b"\x1b[0m\r\x1b[17A", b"\x1b[0m\r\x1b[18A", b"\x1b[0m\r\x1b[19A", b"\x1b[0m\r\x1b[20A", b"\x1b[0m\r\x1b[21A", b"\x1b[0m\r\x1b[22A", b"\x1b[0m\r\x1b[23A", b"\x1b[0m\r\x1b[24A", b"\x1b[0m\r\x1b[25A", b"\x1b[0m\r\x1b[26A", b"\x1b[0m\r\x1b[27A", b"\x1b[0m\r\x1b[28A", b"\x1b[0m\r\x1b[29A", b"\x1b[0m\r\x1b[30A", b"\x1b[0m\r\x1b[31A",
			b"\x1b[0m\r\x1b[32A", b"\x1b[0m\r\x1b[33A", b"\x1b[0m\r\x1b[34A", b"\x1b[0m\r\x1b[35A", b"\x1b[0m\r\x1b[36A", b"\x1b[0m\r\x1b[37A", b"\x1b[0m\r\x1b[38A", b"\x1b[0m\r\x1b[39A", b"\x1b[0m\r\x1b[40A", b"\x1b[0m\r\x1b[41A", b"\x1b[0m\r\x1b[42A", b"\x1b[0m\r\x1b[43A", b"\x1b[0m\r\x1b[44A", b"\x1b[0m\r\x1b[45A", b"\x1b[0m\r\x1b[46A", b"\x1b[0m\r\x1b[47A",
			b"\x1b[0m\r\x1b[48A", b"\x1b[0m\r\x1b[49A", b"\x1b[0m\r\x1b[50A", b"\x1b[0m\r\x1b[51A", b"\x1b[0m\r\x1b[52A", b"\x1b[0m\r\x1b[53A", b"\x1b[0m\r\x1b[54A", b"\x1b[0m\r\x1b[55A", b"\x1b[0m\r\x1b[56A", b"\x1b[0m\r\x1b[57A", b"\x1b[0m\r\x1b[58A", b"\x1b[0m\r\x1b[59A", b"\x1b[0m\r\x1b[60A", b"\x1b[0m\r\x1b[61A", b"\x1b[0m\r\x1b[62A", b"\x1b[0m\r\x1b[63A",
			b"\x1b[0m\r\x1b[64A", b"\x1b[0m\r\x1b[65A", b"\x1b[0m\r\x1b[66A", b"\x1b[0m\r\x1b[67A", b"\x1b[0m\r\x1b[68A", b"\x1b[0m\r\x1b[69A", b"\x1b[0m\r\x1b[70A", b"\x1b[0m\r\x1b[71A", b"\x1b[0m\r\x1b[72A", b"\x1b[0m\r\x1b[73A", b"\x1b[0m\r\x1b[74A", b"\x1b[0m\r\x1b[75A", b"\x1b[0m\r\x1b[76A", b"\x1b[0m\r\x1b[77A", b"\x1b[0m\r\x1b[78A", b"\x1b[0m\r\x1b[79A",
			b"\x1b[0m\r\x1b[80A", b"\x1b[0m\r\x1b[81A", b"\x1b[0m\r\x1b[82A", b"\x1b[0m\r\x1b[83A", b"\x1b[0m\r\x1b[84A", b"\x1b[0m\r\x1b[85A", b"\x1b[0m\r\x1b[86A", b"\x1b[0m\r\x1b[87A", b"\x1b[0m\r\x1b[88A", b"\x1b[0m\r\x1b[89A", b"\x1b[0m\r\x1b[90A", b"\x1b[0m\r\x1b[91A", b"\x1b[0m\r\x1b[92A", b"\x1b[0m\r\x1b[93A", b"\x1b[0m\r\x1b[94A", b"\x1b[0m\r\x1b[95A",
			b"\x1b[0m\r\x1b[96A", b"\x1b[0m\r\x1b[97A", b"\x1b[0m\r\x1b[98A", b"\x1b[0m\r\x1b[99A", b"\x1b[0m\r\x1b[100A", b"\x1b[0m\r\x1b[101A", b"\x1b[0m\r\x1b[102A", b"\x1b[0m\r\x1b[103A", b"\x1b[0m\r\x1b[104A", b"\x1b[0m\r\x1b[105A", b"\x1b[0m\r\x1b[106A", b"\x1b[0m\r\x1b[107A", b"\x1b[0m\r\x1b[108A", b"\x1b[0m\r\x1b[109A", b"\x1b[0m\r\x1b[110A", b"\x1b[0m\r\x1b[111A",
			b"\x1b[0m\r\x1b[112A", b"\x1b[0m\r\x1b[113A", b"\x1b[0m\r\x1b[114A", b"\x1b[0m\r\x1b[115A", b"\x1b[0m\r\x1b[116A", b"\x1b[0m\r\x1b[117A", b"\x1b[0m\r\x1b[118A", b"\x1b[0m\r\x1b[119A", b"\x1b[0m\r\x1b[120A", b"\x1b[0m\r\x1b[121A", b"\x1b[0m\r\x1b[122A", b"\x1b[0m\r\x1b[123A", b"\x1b[0m\r\x1b[124A", b"\x1b[0m\r\x1b[125A", b"\x1b[0m\r\x1b[126A", b"\x1b[0m\r\x1b[127A",
			b"\x1b[0m\r\x1b[128A", b"\x1b[0m\r\x1b[129A", b"\x1b[0m\r\x1b[130A", b"\x1b[0m\r\x1b[131A", b"\x1b[0m\r\x1b[132A", b"\x1b[0m\r\x1b[133A", b"\x1b[0m\r\x1b[134A", b"\x1b[0m\r\x1b[135A", b"\x1b[0m\r\x1b[136A", b"\x1b[0m\r\x1b[137A", b"\x1b[0m\r\x1b[138A", b"\x1b[0m\r\x1b[139A", b"\x1b[0m\r\x1b[140A", b"\x1b[0m\r\x1b[141A", b"\x1b[0m\r\x1b[142A", b"\x1b[0m\r\x1b[143A",
			b"\x1b[0m\r\x1b[144A", b"\x1b[0m\r\x1b[145A", b"\x1b[0m\r\x1b[146A", b"\x1b[0m\r\x1b[147A", b"\x1b[0m\r\x1b[148A", b"\x1b[0m\r\x1b[149A", b"\x1b[0m\r\x1b[150A", b"\x1b[0m\r\x1b[151A", b"\x1b[0m\r\x1b[152A", b"\x1b[0m\r\x1b[153A", b"\x1b[0m\r\x1b[154A", b"\x1b[0m\r\x1b[155A", b"\x1b[0m\r\x1b[156A", b"\x1b[0m\r\x1b[157A", b"\x1b[0m\r\x1b[158A", b"\x1b[0m\r\x1b[159A",
			b"\x1b[0m\r\x1b[160A", b"\x1b[0m\r\x1b[161A", b"\x1b[0m\r\x1b[162A", b"\x1b[0m\r\x1b[163A", b"\x1b[0m\r\x1b[164A", b"\x1b[0m\r\x1b[165A", b"\x1b[0m\r\x1b[166A", b"\x1b[0m\r\x1b[167A", b"\x1b[0m\r\x1b[168A", b"\x1b[0m\r\x1b[169A", b"\x1b[0m\r\x1b[170A", b"\x1b[0m\r\x1b[171A", b"\x1b[0m\r\x1b[172A", b"\x1b[0m\r\x1b[173A", b"\x1b[0m\r\x1b[174A", b"\x1b[0m\r\x1b[175A",
			b"\x1b[0m\r\x1b[176A", b"\x1b[0m\r\x1b[177A", b"\x1b[0m\r\x1b[178A", b"\x1b[0m\r\x1b[179A", b"\x1b[0m\r\x1b[180A", b"\x1b[0m\r\x1b[181A", b"\x1b[0m\r\x1b[182A", b"\x1b[0m\r\x1b[183A", b"\x1b[0m\r\x1b[184A", b"\x1b[0m\r\x1b[185A", b"\x1b[0m\r\x1b[186A", b"\x1b[0m\r\x1b[187A", b"\x1b[0m\r\x1b[188A", b"\x1b[0m\r\x1b[189A", b"\x1b[0m\r\x1b[190A", b"\x1b[0m\r\x1b[191A",
			b"\x1b[0m\r\x1b[192A", b"\x1b[0m\r\x1b[193A", b"\x1b[0m\r\x1b[194A", b"\x1b[0m\r\x1b[195A", b"\x1b[0m\r\x1b[196A", b"\x1b[0m\r\x1b[197A", b"\x1b[0m\r\x1b[198A", b"\x1b[0m\r\x1b[199A", b"\x1b[0m\r\x1b[200A", b"\x1b[0m\r\x1b[201A", b"\x1b[0m\r\x1b[202A", b"\x1b[0m\r\x1b[203A", b"\x1b[0m\r\x1b[204A", b"\x1b[0m\r\x1b[205A", b"\x1b[0m\r\x1b[206A", b"\x1b[0m\r\x1b[207A",
			b"\x1b[0m\r\x1b[208A", b"\x1b[0m\r\x1b[209A", b"\x1b[0m\r\x1b[210A", b"\x1b[0m\r\x1b[211A", b"\x1b[0m\r\x1b[212A", b"\x1b[0m\r\x1b[213A", b"\x1b[0m\r\x1b[214A", b"\x1b[0m\r\x1b[215A", b"\x1b[0m\r\x1b[216A", b"\x1b[0m\r\x1b[217A", b"\x1b[0m\r\x1b[218A", b"\x1b[0m\r\x1b[219A", b"\x1b[0m\r\x1b[220A", b"\x1b[0m\r\x1b[221A", b"\x1b[0m\r\x1b[222A", b"\x1b[0m\r\x1b[223A",
			b"\x1b[0m\r\x1b[224A", b"\x1b[0m\r\x1b[225A", b"\x1b[0m\r\x1b[226A", b"\x1b[0m\r\x1b[227A", b"\x1b[0m\r\x1b[228A", b"\x1b[0m\r\x1b[229A", b"\x1b[0m\r\x1b[230A", b"\x1b[0m\r\x1b[231A", b"\x1b[0m\r\x1b[232A", b"\x1b[0m\r\x1b[233A", b"\x1b[0m\r\x1b[234A", b"\x1b[0m\r\x1b[235A", b"\x1b[0m\r\x1b[236A", b"\x1b[0m\r\x1b[237A", b"\x1b[0m\r\x1b[238A", b"\x1b[0m\r\x1b[239A",
			b"\x1b[0m\r\x1b[240A", b"\x1b[0m\r\x1b[241A", b"\x1b[0m\r\x1b[242A", b"\x1b[0m\r\x1b[243A", b"\x1b[0m\r\x1b[244A", b"\x1b[0m\r\x1b[245A", b"\x1b[0m\r\x1b[246A", b"\x1b[0m\r\x1b[247A", b"\x1b[0m\r\x1b[248A", b"\x1b[0m\r\x1b[249A", b"\x1b[0m\r\x1b[250A", b"\x1b[0m\r\x1b[251A", b"\x1b[0m\r\x1b[252A", b"\x1b[0m\r\x1b[253A", b"\x1b[0m\r\x1b[254A", b"\x1b[0m\r\x1b[255A",
		];

		// We're discontiguous enough to warrant vectored writes, I think…
		let mut parts: &mut [IoSlice] =
			// If the screen is too small for everything, print the percentage
			// by itself to give them some indication of progress.
			if width.get() < 40 {
				&mut [
					IoSlice::new(concat!(
						"\x1b[J ",
						csi!(reset, bold, light_cyan),
						"» ",
						csi!(!fg),
					).as_bytes()), // Clear + Prefix.
					IoSlice::new(self.percent.as_bytes()), // Percent.
					IoSlice::new(concat!(csi!(), "\r").as_bytes()), // Reset and rewind.
				]
			}
			// Otherwise give it all we've got!
			else {
				// The number of lines we'll need to move up after printing to
				// get back to the start.
				let lines =
					if self.title.is_empty() { self.lines_doing }
					else { self.lines_doing.saturating_add(1) };

				&mut [
					// Clear.
					IoSlice::new(CLS),

					// Title.
					IoSlice::new(&self.title),

					// Elapsed.
					IoSlice::new(concat!(
						csi!(reset, dim),
						"[",
						csi!(reset, bold),
					).as_bytes()),
					IoSlice::new(self.elapsed.as_bytes()),
					IoSlice::new(concat!(
						csi!(reset, dim),
						"]  [",
						csi!(reset, bold, light_cyan),
					).as_bytes()),

					// Bars.
					IoSlice::new(self.bar_done),
					IoSlice::new(csi!(blue).as_bytes()),
					IoSlice::new(self.bar_undone),
					IoSlice::new(concat!(
						csi!(reset, dim),
						"]",
						csi!(reset, bold, light_cyan),
						"  ",
					).as_bytes()),

					// Done/total.
					IoSlice::new(self.done.as_bytes()),
					IoSlice::new(concat!(
						csi!(reset, dim),
						"/",
						csi!(reset, bold, blue),
					).as_bytes()),
					IoSlice::new(self.total.as_bytes()),

					// Percent.
					IoSlice::new(concat!(csi!(!fg), "  ").as_bytes()),
					IoSlice::new(self.percent.as_bytes()),

					// Tasks.
					IoSlice::new(csi!(reset, magenta).as_bytes()),
					IoSlice::new(&self.doing),

					// The end!
					IoSlice::new(CLOSE[usize::from(lines)]),
				]
			};

		// TODO: remove once `write_all_vectored` is stable.
		IoSlice::advance_slices(&mut parts, 0);
		loop {
			match handle.write_vectored(parts) {
				Ok(0) => return false,
				Ok(n) => IoSlice::advance_slices(&mut parts, n),
				Err(e) =>
					if e.kind() == ErrorKind::Interrupted {} // Keep trying.
					else { return false; },
			}
			if parts.is_empty() { break; }
		}
		handle.flush().is_ok()
	}
}

impl ProglessBuffer {
	/// # Set Bars.
	fn set_bars(&mut self, width: NonZeroU8, done: u32, total: u32) {
		// Default sizes.
		let mut w_done = 0_u8;
		let mut w_undone = 0_u8;

		// How much room do we have for the bar(s)?
		// The magic "19" is made up of the following hard-coded pieces:
		// 10: elapsed time and braces;
		// 2: spaces after elapsed time;
		// 1: the "/" between done and total;
		// 2: the spaces after total;
		// 2: the braces around the bar itself;
		// 2: the spaces after the bar itself;
		let space: u8 = width.get().saturating_sub(u8::saturating_from(
			19 +
			self.done.len() +
			self.total.len() +
			self.percent.len()
		));

		// If we have any space, divide it up proportionately.
		if total != 0 && MIN_BARS_WIDTH <= space {
			// Nothing is done.
			if done == 0 { w_undone = space; }
			// Everything is done!
			else if done == total { w_done = space; }
			// Working on it!
			else {
				w_done = u8::saturating_from((done * u32::from(space)).wrapping_div(total));
				w_undone = space.saturating_sub(w_done);
			}

			debug_assert_eq!(
				w_done + w_undone,
				space,
				"BUG: bar space was miscalculated."
			);
		}

		// Update the parts!.
		self.bar_done =     &BAR_DONE[..usize::from(w_done)];
		self.bar_undone = &BAR_UNDONE[..usize::from(w_undone)];
	}

	/// # Update Tasks.
	fn set_doing(
		&mut self,
		doing: &BTreeSet<String>,
		width: NonZeroU8,
		height: NonZeroU8,
	) {
		/// # Task Prefix.
		///
		/// This translates to:           •   •   •   •   ↳             •
		const PREFIX: &[u8; 9] = &[b'\n', 32, 32, 32, 32, 226, 134, 179, 32];

		// Reset.
		self.doing.truncate(0);
		self.lines_doing = 0;

		// The actual width we can work with is minus six for padding, six for
		// the prefix.
		let width = usize::from(width.get().saturating_sub(12));

		// Add each task as its own line, assuming we have the room.
		if
			2 <= width &&
			usize::from(! self.title.is_empty()) + 1 + doing.len() <= usize::from(height.get())
		{
			for line in doing {
				let keep = crate::length_width(line, width);
				if keep != 0 {
					self.doing.extend_from_slice(PREFIX);
					self.doing.extend(line.bytes().take(keep));
					self.lines_doing += 1;
				}
			}
		}
	}

	/// # Update Title.
	fn set_title(&mut self, title: Option<&Msg>, width: NonZeroU8, height: NonZeroU8) {
		// Reset the title.
		self.title.truncate(0);

		// We need at least two lines of screen space to fit a title.
		if 2 <= height.get() && let Some(title) = title {
			let title = title.fitted(usize::from(width.get()));
			if ! title.is_empty() {
				// Truncate to first line.
				let slice = title.as_bytes();
				let end = slice.iter().copied().position(|b| b == b'\n').unwrap_or(slice.len());
				if end != 0 {
					self.title.extend_from_slice(&slice[..end]);
					self.title.push(b'\n');
				}
			}
		}
	}
}



#[cfg_attr(docsrs, doc(cfg(feature = "progress")))]
#[derive(Debug, Clone, Default)]
/// # Progless.
///
/// This is a simple, thread-safe, steady-ticking CLI progress bar that can be
/// used to entertain users while long jobs are running.
///
/// To use it, enable the `progress` crate flag.
///
/// ## Examples
///
/// Initialize and use as follows:
///
/// ```no_run
/// use fyi_msg::Progless;
///
/// // You can use [`Progless::try_from`] for any unsigned integer type, or the
/// // infallible [`Progless::from`] on an [`std::num::NonZeroU32`].
/// let pbar = Progless::try_from(1001_u32).unwrap();
///
/// // Iterate your taskwork or whatever.
/// for i in 0..1001 {
///     // Do some work.
///     // ...
///
///     // Increment the count.
///     pbar.increment();
/// }
///
/// // Close it off.
/// pbar.finish();
/// ```
///
/// [`Progless`] is thread-safe so can be called from parallel iterators like
/// those from [`rayon`](https://crates.io/crates/rayon) without any special fuss.
///
/// When doing parallel work, many tasks might be "in progress" simultaneously.
/// To that end, you may wish to use the [`Progless::task`] instead of
/// manually incrementing the counts to let the user know what's happening.
///
/// Doing this, a list of active tasks will be maintained and printed along
/// with the numerical progress. Removing a task automatically increments the
/// done count, so you should *not* call [`Progless::increment`] when using
/// this feature.
///
/// ```no_run
/// # use fyi_msg::Progless;
/// # use rayon::prelude::*;
///
/// # let pbar = Progless::try_from(1001_u32).unwrap();
///
/// // ... snip
///
/// // Iterate in Parallel.
/// (0..1001).into_par_iter().for_each(|i| {
///     // Announce the new task at the start.
///     let task = pbar.task(format!("Task #{}.", i));
///
///     // Do some work.
///
///     // Drop the guard when finished.
///     drop(task);
/// });
///
/// // ... snip
/// ```
pub struct Progless {
	/// # Steady Ticker.
	steady: Arc<ProglessSteady>,

	/// # Progress Data.
	inner: Arc<ProglessInner>,
}

impl<T> From<T> for Progless
where ProglessInner: From<T> {
	#[inline]
	fn from(total: T) -> Self {
		let inner = Arc::new(ProglessInner::from(total));
		Self {
			steady: Arc::new(ProglessSteady::from(Arc::clone(&inner))),
			inner,
		}
	}
}

impl From<Progless> for Msg {
	#[inline]
	/// # Into [`Msg`]
	///
	/// This provides a simple way to convert a (finished) [`Progless`]
	/// instance into a generic summary [`Msg`] that can e.g. be printed.
	///
	/// For a more advanced summary, use the [`Progless::summary`] method.
	fn from(src: Progless) -> Self {
		let elapsed = NiceElapsed::from(src.inner.started);
		let mut msg = String::with_capacity(13 + elapsed.len());
		msg.push_str("Finished in ");
		msg.push_str(elapsed.as_str());
		msg.push('.');

		Self::done(msg).with_newline(true)
	}
}

/// # Helper: generate `TryFrom for Progless` for all the
/// `TryFrom for ProglessInner` types since we can't use generics for this
/// trait.
macro_rules! outer_tryfrom {
	($($ty:ty),+ $(,)?) => ($(
		impl TryFrom<$ty> for Progless {
			type Error = ProglessError;

			#[inline]
			fn try_from(total: $ty) -> Result<Self, Self::Error> {
				let inner = Arc::new(ProglessInner::try_from(total)?);
				Ok(Self {
					steady: Arc::new(ProglessSteady::from(Arc::clone(&inner))),
					inner,
				})
			}
		}
	)+)
}

outer_tryfrom!(
	u8, u16, u32, u64, usize, u128,
	i8, i16, i32, i64, isize, i128,
	NonZeroU64, NonZeroUsize, NonZeroU128,
);

/// # Constants.
impl Progless {
	/// # ANSI Sequence: Hide Cursor.
	///
	/// Emit this sequence to STDERR to hide the terminal cursor.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Progless;
	///
	/// // Hide the cursor.
	/// eprint!("{}", Progless::CURSOR_HIDE);
	/// ```
	pub const CURSOR_HIDE: &str = "\x1b[?25l";

	/// # ANSI Sequence: Unhide Cursor.
	///
	/// Emit this sequence to STDERR to restore the terminal cursor.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Progless;
	///
	/// // Hide the cursor.
	/// eprint!("{}", Progless::CURSOR_HIDE);
	///
	/// // Do some stuff.
	///
	/// // Bring the cursor back.
	/// eprint!("{}", Progless::CURSOR_UNHIDE);
	/// ```
	pub const CURSOR_UNHIDE: &str = "\x1b[?25h";

	#[cfg(target_pointer_width = "16")]
	/// # Max Total.
	///
	/// A [`Progless`] instance cannot have a total higher than this value.
	/// This is technically `u32::MAX`, but in practice `usize` is used more
	/// often, so this value reflects whichever of the two is smaller.
	/// Regardless, it's an awful lot of tasks to try to visualize. Haha.
	pub const MAX_TOTAL: usize = 65_535;

	#[cfg(not(target_pointer_width = "16"))]
	/// # Max Total.
	///
	/// A [`Progless`] instance cannot have a total higher than this value.
	/// This is technically `u32::MAX`, but in practice `usize` is used more
	/// often, so this value reflects whichever of the two is smaller.
	/// Regardless, it's an awful lot of tasks to try to visualize. Haha.
	pub const MAX_TOTAL: usize = 4_294_967_295;

	/// # Total Error.
	///
	/// This is the error message that is returned when a total is too high for
	/// a [`Progless`] instance.
	pub const MAX_TOTAL_ERROR: ProglessError = ProglessError::TotalOverflow;
}

/// # Construction/Destruction.
impl Progless {
	#[must_use]
	#[inline]
	/// # With Title.
	///
	/// Add a title to the progress bar. When present, this will print on its
	/// own line immediately before the progress line.
	///
	/// Titles are formatted as [`Msg`] objects. You can pass a [`Msg`]
	/// directly, or something that can be converted to one, like a string
	/// slice.
	///
	/// Pass `None` to remove the title entirely.
	///
	/// Note: titles cannot have line breaks; this will automatically replace
	/// any non-space whitespace with regular horizontal spaces.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{Msg, Progless};
	///
	/// // Initialize with a `u32` total.
	/// let pbar = Progless::try_from(1001_u32).unwrap()
	///     .with_title(Some(Msg::info("Doing things!")));
	///
	/// // Iterate your taskwork or whatever.
	/// for i in 0..1001 {
	///     // Do some work.
	///     // ...
	///
	///     // Increment the done count.
	///     pbar.increment();
	/// }
	///
	/// pbar.finish();
	/// ```
	pub fn with_title<S>(self, title: Option<S>) -> Self
	where S: Into<Msg> {
		let title = title.and_then(|m| {
			let m = m.into();
			if m.is_empty() { None }
			else { Some(m) }
		});
		self.inner.set_title(title);
		self
	}

	#[must_use]
	#[inline]
	/// # Set Title As X: Reticulating Splines…
	///
	/// This is simply shorthand for generating a "Reticulating Splines…"
	/// title, where X is the value passed in (usually the app name).
	///
	/// It's a sort of default…
	pub fn with_reticulating_splines<S>(self, app: S) -> Self
	where S: AsRef<str> {
		self.set_reticulating_splines(app);
		self
	}

	#[expect(clippy::must_use_candidate, reason = "Caller might not care.")]
	#[inline]
	/// # Stop.
	///
	/// Finish the progress bar, shut down the steady ticker, and return the
	/// time elapsed.
	///
	/// Calling this method will also erase any previously-printed progress
	/// information from the CLI screen.
	///
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Progless;
	///
	/// // Initialize with a `u32` total.
	/// let pbar = Progless::try_from(1001_u32).unwrap();
	///
	/// // Iterate your taskwork or whatever.
	/// for i in 0..1001 {
	///     // Do some work.
	///     // ...
	///
	///     // Increment the done count.
	///     pbar.increment();
	/// }
	///
	/// // Finish it off!
	/// pbar.finish();
	/// ```
	pub fn finish(&self) -> Duration {
		self.inner.stop();
		self.steady.stop();
		self.inner.started.elapsed()
	}

	#[must_use]
	/// # Summarize.
	///
	/// Generate a formatted [`Msg`] summary of the (finished) progress using
	/// the supplied verb and noun.
	///
	/// If you just want a generic "Finished in X." message, use [`Msg::from`]
	/// instead.
	///
	/// Note: if you called [`Progless::reset`] anywhere along the way, this
	/// won't include totals from the previous run(s). (The duration is the
	/// only constant.)
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::{MsgKind, Progless};
	///
	/// // Initialize with a `u32` total.
	/// let pbar = Progless::try_from(1001_u32).unwrap();
	///
	/// // Iterate your taskwork or whatever.
	/// for i in 0..1001 {
	///     // Do some work.
	///     // ...
	///
	///     // Increment the done count.
	///     pbar.increment();
	/// }
	///
	/// pbar.finish();
	///
	/// // Print something like "Crunched X files in Y seconds."
	/// pbar.summary(MsgKind::Crunched, "file", "files").print();
	/// ```
	pub fn summary<S>(&self, kind: MsgKind, singular: S, plural: S) -> Msg
	where S: AsRef<str> {
		let done = done!(self.inner.done_total.load(SeqCst)) as u32;
		Msg::new(kind, format!(
			"{} in {}.",
			done.nice_inflect(singular.as_ref(), plural.as_ref()),
			NiceElapsed::from(self.inner.started),
		))
			.with_newline(true)
	}
}

/// # Passthrough Setters.
impl Progless {
	#[inline]
	/// # Increment Done.
	///
	/// Increase the completed count by exactly one. This is safer to use than
	/// `set_done()` in cases where multiple tasks are happening at once as it
	/// will not accidentally decrease the value, etc.
	///
	/// See the various examples all over this page for more information.
	pub fn increment(&self) { self.inner.increment_n(1); }

	#[inline]
	/// # Increment Done by N.
	///
	/// Increase the completed count by `n`. This is safer to use than `set_done()`
	/// and more efficient than calling `increment()` a million times in a row.
	pub fn increment_n(&self, n: u32) { self.inner.increment_n(n); }

	#[must_use]
	/// # Add (Named) Task.
	///
	/// This method can be used to add an active "task" to the [`Progless`]
	/// output, letting the user know what, specifically, is being worked on
	/// at any given moment.
	///
	/// The "task" is bound to the lifetime of the returned [guard](ProglessTaskGuard).
	/// When (the guard is) dropped, the "task" will automatically vanish from
	/// the [`Progless`] output, and the done count will increase by one.
	///
	/// Multiple active "tasks" can exist simultaneously — parallelization,
	/// etc. — but there has to be enough room on the screen for the set or it
	/// won't be displayed.
	///
	/// In practice, this works best for progressions that step one or a dozen
	/// or so items at a time.
	///
	/// See [`Progless::increment`] as an alternative that avoids the whole
	/// "task" concept.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::Progless;
	/// # fn download(_url: &str) -> Option<String> { todo!() }
	/// # let urls = Vec::<&str>::new();
	///
	/// // Download stuff from the internet?
	/// let pbar = Progless::try_from(urls.len()).unwrap();
	/// for url in &urls {
	///     // Add the URL to the progress output so the user knows who to
	///     // blame for major slowdowns, etc.
	///     let task = pbar.task(url);
    ///
    ///     // Do some work that might take a while.
    ///     if let Some(raw) = download(url) {
    ///         // …
    ///     }
    ///
    ///     // One down, N to go!
    ///     drop(task);
	/// }
	///
	/// // Progress is done!
	/// pbar.finish();
	/// ```
	pub fn task<S>(&self, txt: S) -> Option<ProglessTaskGuard<'_>>
	where S: AsRef<str> {
		self.inner.add_guard(txt.as_ref())
			.map(|task| ProglessTaskGuard::from_parts(
				task,
				self.inner.cycle(),
				&self.inner,
			))
	}

	#[inline]
	/// # Push Message.
	///
	/// "Insert" (print) a line (to STDERR) above the running progress bar,
	/// useful for realtime debug logs, warnings, etc., that would otherwise
	/// have to wait for the [`Progless`] instance to finish hogging the
	/// display.
	///
	/// ## Errors
	///
	/// In practice this should never fail, but if for some reason STDERR is
	/// tied up the original message is passed back as an error in case you
	/// want to try to deal with it yourself.
	pub fn push_msg(&self, msg: Msg) -> Result<(), Msg> { self.inner.push_msg(msg) }

	/// # Reset.
	///
	/// Stop the current run (if any), clear the done/doing metrics, and assign
	/// a new total so you can re-use the [`Progless`] instance for a new set
	/// of tasks.
	///
	/// Note: the start/elapsed times for a given [`Progless`] instance are
	/// _continuous_. If you need the time counter to reset to `[00:00:00]`,
	/// you need start a brand new instance instead of resetting an existing
	/// one.
	pub fn reset(&self, total: NonZeroU32) {
		self.inner.reset(total);
		self.steady.start(Arc::clone(&self.inner));
	}

	/// # Reset (Fallible).
	///
	/// Same as [`Progless::reset`], but will fail if `total` is zero.
	///
	/// ## Errors
	///
	/// This will return an error if the new total is zero.
	pub fn try_reset(&self, total: u32) -> Result<(), ProglessError> {
		let total = NonZeroU32::new(total).ok_or(ProglessError::EmptyTotal)?;
		self.reset(total);
		Ok(())
	}

	#[inline]
	/// # Set Done.
	///
	/// Set the done count to a specific (absolute) value.
	///
	/// In general, relative adjustments should be preferred for consistency.
	/// Consider [`Progless::increment`] or [`Progless::task`] instead.
	pub fn set_done(&self, done: u32) { self.inner.set_done(done); }

	#[inline]
	/// # Set Title.
	///
	/// Give the progress bar a title, which will be shown above the progress
	/// bits while progress is progressing, and removed afterward with
	/// everything else.
	///
	/// See [`Progless::with_title`] for more details.
	pub fn set_title<S>(&self, title: Option<S>)
	where S: Into<Msg> {
		let title = title.and_then(|m| {
			let m = m.into();
			if m.is_empty() { None }
			else { Some(m) }
		});
		self.inner.set_title(title);
	}

	#[inline]
	/// # Set Title Message.
	///
	/// Change (only) the message part — what follows the prefix — of the title
	/// to something else.
	///
	/// Note: this has no effect for instances without a title component.
	pub fn set_title_msg(&self, msg: &str) { self.inner.set_title_msg(msg); }

	#[inline]
	/// # Set Title As X: Reticulating Splines…
	///
	/// This is simply shorthand for generating a "Reticulating Splines…"
	/// title, where X is the value passed in (usually the app name).
	///
	/// It's a sort of default…
	pub fn set_reticulating_splines<S>(&self, app: S)
	where S: AsRef<str> {
		self.inner.set_title(Some(Msg::new(
			(app.as_ref(), AnsiColor::Misc199),
			"Reticulating splines\u{2026}"
		)));
	}
}



/// # Sanitize [`Progless`] Task.
///
/// This method strips ANSI sequences and normalizes whitespace, returning the
/// result if non-empty.
fn progless_task(src: &str) -> Option<String> {
	let src = src.trim_end();
	if src.is_empty() { return None; }

	let mut out = String::with_capacity(src.len());
	for c in NoAnsi::new(src) {
		// Convert all whitespace to regular spaces.
		if c.is_whitespace() { out.push(' '); }
		// Keep all other non-control characters as-are.
		else if ! c.is_control() { out.push(c); }
	}

	if out.is_empty() { None }
	else { Some(out) }
}

#[cfg(unix)]
#[must_use]
#[inline]
/// # Term Width and Height.
///
/// Return the width and height of the terminal attached to STDERR, if any,
/// less one to help smooth scroll weirdness.
fn term_size() -> Option<(NonZeroU8, NonZeroU8)> {
	use terminal_size::{Height, Width};
	let (Width(w), Height(h)) = terminal_size::terminal_size_of(std::io::stderr())?;
	let w = NonZeroU8::new(u8::saturating_from(w.saturating_sub(1)))?;
	let h = NonZeroU8::new(u8::saturating_from(h).saturating_sub(1))?;
	Some((w, h))
}

#[cfg(not(unix))]
#[must_use]
#[inline]
/// # Term Width and Height.
///
/// Return the width and height of the terminal attached to STDERR, if any,
/// less one to help smooth scroll weirdness.
fn term_size() -> Option<(NonZeroU8, NonZeroU8)> {
	use terminal_size::{Height, Width};
	let (Width(w), Height(h)) = terminal_size::terminal_size()?;
	let w = NonZeroU8::new(u8::saturating_from(w.saturating_sub(1)))?;
	let h = NonZeroU8::new(u8::saturating_from(h).saturating_sub(1))?;
	Some((w, h))
}



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn t_done_total() {
		/// # Split Done/Total.
		const fn split_done_total(done_total: u64) -> (u64, u64) {
			(done!(done_total), total!(done_total))
		}

		// Test a total-only initial set.
		let done_total = AtomicU64::new(55);
		assert_eq!(split_done_total(done_total.load(SeqCst)), (0, 55));

		// Test setting a done, and extracting non-zero done/total.
		done_total.store(done_total!(32, 55), SeqCst);
		assert_eq!(split_done_total(done_total.load(SeqCst)), (32, 55));

		// Verify our mask is the right size.
		assert_eq!(0xFFFF_FFFF_u64, u64::from(u32::MAX));
	}
}
