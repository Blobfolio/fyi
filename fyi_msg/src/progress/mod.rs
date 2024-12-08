/*!
# FYI Msg - Progless
*/

pub(super) mod ba;
pub(super) mod error;
mod steady;
mod task;



use crate::{
	Msg,
	MsgKind,
	ProglessError,
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
			AtomicU32,
			Ordering::SeqCst,
		},
	},
	time::{
		Duration,
		Instant,
	},
};
use steady::ProglessSteady;
use task::ProglessTask;



/// # Bar Filler (Done).
static BAR_DONE:   [u8; 256] = [b'#'; 256];

/// # Dash Filler (TBD).
static BAR_UNDONE: [u8; 256] = [b'-'; 256];

/// # Twenty Line Clears.
static CLS20: [u8; 280] = *b"\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
	\x1b[1A\x1b[1000D\x1b[K\
";

/// # Helper: Mutex Unlock.
///
/// This just moves tedious code out of the way.
macro_rules! mutex {
	($m:expr) => ($m.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
}

use mutex;



// Tick Flags.
// These flags indicate whether or not a given component has changed since the
// last tick, saving the overhead of recalculating the buffer values each time
// a value changes. (Instead they're only recalculated at most once per tick.)

/// # Flag: Initial State.
const TICK_NEW: u8 =
	TICK_BAR | TICK_TOTAL | TICKING;

/// # Flag: Reset.
const TICK_RESET: u8 =
	TICK_BAR | TICK_DOING | TICK_DONE | TICK_TOTAL | TICKING;

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

/// # Flag: SIGINT Received?
const SIGINT: u8 =       0b0100_0000;

/// # Minimum Bar Width.
const MIN_BARS_WIDTH: u8 = 10;

/// # Minimum Draw Width.
const MIN_DRAW_WIDTH: u8 = 40;



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

	/// # Last Printed Line Count.
	///
	/// The number of lines last printed. Before printing new output, this many
	/// lines must be "erased".
	last_lines: AtomicU8,

	/// # Last Width.
	///
	/// The screen width from the last print. If this changes, all buffer parts
	/// are recalculated (even if their values haven't changed) to ensure they
	/// fit the new width.
	last_width: AtomicU8,

	/// # Start Time.
	///
	/// The instant the object was first created. All timings are derived from
	/// this value.
	started: Instant,

	/// # Elapsed Seconds.
	///
	/// This is the number of elapsed milliseconds as of the last tick. This
	/// gives us a reference to throttle back-to-back ticks as well as a cache
	/// of the seconds written to the `[00:00:00]` portion of the buffer.
	elapsed: AtomicU32,

	/// # Title.
	title: Mutex<Option<Msg>>,

	/// # Finished Tasks.
	done: AtomicU32,

	/// # Total Tasks.
	total: AtomicU32,

	/// # Active Task List.
	doing: Mutex<BTreeSet<ProglessTask>>,
}

impl Default for ProglessInner {
	#[inline]
	fn default() -> Self {
		Self {
			buf: Mutex::new(ProglessBuffer::DEFAULT),
			flags: AtomicU8::new(0),

			last_lines: AtomicU8::new(0),
			last_width: AtomicU8::new(0),

			started: Instant::now(),
			elapsed: AtomicU32::new(0),

			title: Mutex::new(None),
			done: AtomicU32::new(0),
			total: AtomicU32::new(1),
			doing: Mutex::new(BTreeSet::default()),
		}
	}
}

impl From<NonZeroU32> for ProglessInner {
	#[inline]
	fn from(total: NonZeroU32) -> Self {
		Self {
			flags: AtomicU8::new(TICK_NEW),
			total: AtomicU32::new(total.get()),
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
					total: AtomicU32::new(u32::from(total.get())),
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
			#[expect(clippy::cast_possible_truncation, reason = "We're checking for fit.")]
			fn try_from(total: $ty) -> Result<Self, Self::Error> {
				let total = total.get();
				if total <= 4_294_967_295 {
					Ok(Self {
						flags: AtomicU8::new(TICK_NEW),
						total: AtomicU32::new(total as u32),
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
		if TICKING == self.flags.swap(0, SeqCst) & TICKING {
			// Acquire the lock a little early just in case there is a
			// final in-progress tick.
			let mut handle = std::io::stderr().lock();

			self.done.store(self.total(), SeqCst);
			self.elapsed.store(
				u32::saturating_from(self.started.elapsed().as_millis()),
				SeqCst
			);
			mutex!(self.doing).clear();

			// Clear the screen one last time.
			self.print_cls(&mut handle);
		}
	}
}

/// # Getters.
impl ProglessInner {
	#[inline]
	/// # Done.
	///
	/// The number of completed tasks.
	fn done(&self) -> u32 { self.done.load(SeqCst) }

	#[inline]
	/// # Is Ticking.
	///
	/// This is `true` so long as `done` does not equal `total`, and `total`
	/// is greater than `0`. Otherwise it is `false`.
	///
	/// For the most part, this struct's setter methods only work while
	/// progress is happening; after that they're frozen.
	fn running(&self) -> bool { 0 != self.flags.load(SeqCst) & TICKING }

	#[inline]
	/// # Total.
	///
	/// The total number of tasks.
	fn total(&self) -> u32 { self.total.load(SeqCst) }
}

/// # Setters.
impl ProglessInner {
	/// # Add a task.
	///
	/// The progress bar can optionally keep track of tasks that are actively
	/// "in progress", which can be particularly useful when operating in
	/// parallel.
	///
	/// Returns `true` if the task was accepted.
	fn add(&self, txt: &str) -> bool {
		if
			self.running() &&
			ProglessTask::new(txt).is_some_and(|m| mutex!(self.doing).insert(m))
		{
			self.flags.fetch_or(TICK_DOING, SeqCst);
			true
		}
		else { false }
	}

	#[inline]
	/// # Increment Done.
	///
	/// Increase the completed count by exactly one. This is safer to use than
	/// `set_done()` in cases where multiple tasks are happening at once as it
	/// will not accidentally decrease the value, etc.
	fn increment(&self) {
		if self.running() {
			let done = self.done.fetch_add(1, SeqCst) + 1;
			if done >= self.total() { self.stop() }
			else {
				self.flags.fetch_or(TICK_DONE | TICK_BAR, SeqCst);
			}
		}
	}

	#[inline]
	/// # Increment Done by N.
	///
	/// Increase the completed count by `n`. This is safer to use than `set_done()`
	/// and more efficient than calling `increment()` a million times in a row.
	fn increment_n(&self, n: u32) {
		if n != 0 && self.running() {
			let done = self.done.fetch_add(n, SeqCst) + n;
			if done >= self.total() { self.stop() }
			else {
				self.flags.fetch_or(TICK_DONE | TICK_BAR, SeqCst);
			}
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
			self.print_cls(&mut handle);
			let res = handle.write_all(msg.as_bytes())
				.and_then(|()| handle.flush())
				.is_err();
			drop(handle);

			// To complete the illusion, restore the progress bits.
			self.tick();

			// This shouldn't happen.
			if res { return Err(msg); }
		}
		// Otherwise we can just print it directly.
		else { msg.eprint(); }

		Ok(())
	}

	/// # Remove a task.
	///
	/// This is the equal and opposite companion to `add`. Calling this will
	/// automatically increment the done count by one, so should not be used
	/// in cases where you're triggering done changes manually.
	fn remove(&self, txt: &str) {
		if self.running() {
			// Try to remove the task.
			let removed: bool = {
				let txt = txt.trim_end();
				let mut ptr = mutex!(self.doing);

				// Check for a direct hit first as it is relatively unlikely
				// the label would have been reformatted for storage.
				ptr.remove(txt.as_bytes()) ||
				// Then again, maybe it was…
				ProglessTask::new(txt).is_some_and(|task|
					task != *txt && ptr.remove(&task)
				)
			};

			// If we removed an entry, set the tick flag and increment.
			if removed {
				self.flags.fetch_or(TICK_DOING, SeqCst);
				self.increment();
			}
		}
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
	///
	/// ## Errors
	///
	/// This will return an error if the new total is zero.
	fn reset(&self, total: u32) -> Result<(), ProglessError> {
		self.stop();
		if 0 == total { Err(ProglessError::EmptyTotal) }
		else {
			self.total.store(total, SeqCst);
			self.done.store(0, SeqCst);
			self.flags.store(TICK_RESET, SeqCst);
			Ok(())
		}
	}

	/// # Set Done.
	///
	/// Set the done count to a specific value. Be careful in cases where
	/// things are happening in parallel; in such cases `increment` is probably
	/// better.
	fn set_done(&self, done: u32) {
		if self.running() && done != self.done.swap(done, SeqCst) {
			if done >= self.total() { self.stop(); }
			else {
				self.flags.fetch_or(TICK_DONE | TICK_BAR, SeqCst);
			}
		}
	}

	/// # Set Title.
	///
	/// Give the progress bar a title, which will be shown above the progress
	/// bits while progress is progressing, and removed afterward with
	/// everything else.
	fn set_title(&self, title: Option<Msg>) {
		if self.running() {
			*mutex!(self.title) = title.map(|m| m.with_newline(true));
			self.flags.fetch_or(TICK_TITLE, SeqCst);
		}
	}

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
	fn sigint(&self) {
		let flags = self.flags.load(SeqCst);
		if TICKING == flags & (SIGINT | TICKING) {
			mutex!(self.title).replace(Msg::warning("Early shutdown in progress."));
			self.flags.fetch_or(SIGINT | TICK_TITLE, SeqCst);
		}
	}
}

/// # Render.
impl ProglessInner {
	/// # Erase Output.
	///
	/// This method "erases" any prior output so that new output can be written
	/// in the same place.
	///
	/// (This would be a lot easier if we had only a single line, but that's
	/// CLI animation for you. Haha.)
	fn print_cls(&self, handle: &mut StderrLock<'static>) {
		// We might not need to do anything.
		let mut last_lines = usize::from(self.last_lines.swap(0, SeqCst));
		if 0 != last_lines {
			// Clear the current line.
			let _res = handle.write_all(b"\x1b[1000D\x1b[K");

			// Now move the cursor up the appropriate number of lines, clearing
			// each as we go.
			loop {
				// We can handle up to twenty lines at a time.
				let chunk = usize::min(last_lines, 20);
				let _res = handle.write_all(&CLS20[..14 * chunk]);
				last_lines -= chunk;
				if last_lines == 0 { break; }
			}

			// Don't forget to flush!
			let _res = handle.flush();
		}
	}
}

/// # Ticks.
impl ProglessInner {
	#[inline]
	/// # Tick Flag Toggle.
	///
	/// If a flag is set, unset it and return true. Otherwise false.
	fn flag_unset(&self, flag: u8) -> bool {
		0 != self.flags.fetch_and(! flag, SeqCst) & flag
	}

	#[expect(clippy::cast_possible_truncation, reason = "It is what it is.")]
	/// # Tick.
	///
	/// Ticking takes all of the changed values (since the last tick), updates
	/// their corresponding parts in the buffer, and prints the result, if any.
	///
	/// To help keep repeated calls to this from overloading the system, work
	/// only takes place if it has been at least 60ms from the last tick.
	fn tick(&self) -> bool {
		// We aren't running!
		if ! self.running() { return false; }

		// Lock STDERR as early as possible to keep the state as consistent as
		// possible, even though we may well not end up using it.
		let mut handle = std::io::stderr().lock();

		// If there's not enough room for a progress bar, just clear the
		// previous output, if any.
		let width = self.tick_set_width();
		if width < MIN_DRAW_WIDTH {
			self.print_cls(&mut handle);
		}
		// If something drawable changed, we need a complete refresh.
		else if self.tick_set_secs() || 0 != self.flags.load(SeqCst) & TICK_DRAWABLE {
			// Update the buffer bits.
			let mut buf = mutex!(self.buf);

			// Let's start with the numbers since they affect multiple pieces.
			let ticked = self.flags.fetch_and(! (TICK_DONE | TICK_TOTAL | TICK_BAR), SeqCst);
			if ticked != 0 {
				let done = self.done();
				let total = self.total();
				if TICK_DONE == ticked & TICK_DONE { buf.done.replace(done); }
				if TICK_TOTAL == ticked & TICK_TOTAL { buf.total.replace(total); }

				// If either number changed, we need to update the percentage.
				if 0 != ticked & (TICK_DONE | TICK_TOTAL) {
					let percent =
						if done == 0 || total == 0 { 0.0 }
						else if done >= total { 1.0 }
						else { (f64::from(done) / f64::from(total)) as f32 };
					buf.percent.replace(percent);
				}

				// All three conditions independently require a bar update.
				buf.set_bars(width, done, total);
			}

			// Update the tasks?
			if self.flag_unset(TICK_DOING) {
				buf.set_doing(&mutex!(self.doing), width);
			}

			// Update the title?
			if self.flag_unset(TICK_TITLE) {
				buf.set_title(mutex!(self.title).as_ref(), width);
			}

			// Clear the previous output.
			self.print_cls(&mut handle);

			// Print the updated progress details and update the line count.
			let lines = buf.print(&mut handle);
			drop(buf);
			if let Some(lines) = lines {
				self.last_lines.store(lines, SeqCst);
			}
		}

		true
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
	fn tick_set_secs(&self) -> bool {
		let now: u32 = u32::saturating_from(self.started.elapsed().as_millis());
		let before: u32 = self.elapsed.load(SeqCst);

		let secs: u32 = now.wrapping_div(1000);
		self.elapsed.store(now, SeqCst);

		// No change to the seconds bit.
		if secs == before.wrapping_div(1000) { false }
		else {
			mutex!(self.buf).elapsed.replace(secs);
			true
		}
	}

	/// # Tick Width.
	///
	/// Check to see if the terminal width has changed since the last run and
	/// update values — i.e. the relevant tick flags — as necessary.
	fn tick_set_width(&self) -> u8 {
		let width = term_width();
		if width != self.last_width.swap(width, SeqCst) {
			self.flags.fetch_or(TICK_RESIZED, SeqCst);
		}
		width
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

	/// # Title Lines.
	lines_title: u8,
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
		lines_title: 0,
	};
}

impl ProglessBuffer {
	#[expect(clippy::cast_possible_truncation, reason = "False positive.")]
	/// # Line Count.
	///
	/// One line is always assumed for the time/bar/totals, but the title and
	/// task lists can add more.
	const fn lines(&self) -> u8 {
		// Scale up the addition to prevent overflow, however unlikely.
		let lines: u16 = self.lines_doing as u16 + self.lines_title as u16 + 1;

		// Scale back down, saturating as necessary.
		if lines <= u8::MAX as u16 { lines as u8 }
		else { u8::MAX }
	}

	/// # Write It!
	///
	/// This writes the fully-formatted progress data to STDERR, returning the
	/// (precalculated) line count.
	fn print(&self, handle: &mut StderrLock<'static>) -> Option<u8> {
		use std::io::ErrorKind;
		use std::io::Write;

		/// # Write All Vectored.
		///
		/// TODO: remove once `Write::write_all_vectored` is stable.
		fn write_all_vectored(
			mut bufs: &mut [IoSlice<'_>],
			handle: &mut StderrLock<'static>,
		) -> bool {
			// Make sure we have something to print.
			IoSlice::advance_slices(&mut bufs, 0);
			if bufs.is_empty() { true }
			else {
				// Write it all!
				loop {
					match handle.write_vectored(bufs) {
						Ok(0) => return false,
						Ok(n) => IoSlice::advance_slices(&mut bufs, n),
						Err(e) =>
							if e.kind() == ErrorKind::Interrupted {} // Keep trying.
							else { return false; },
					}
					if bufs.is_empty() { break; }
				}
				handle.flush().is_ok()
			}
		}

		// We're discontiguous enough, I think.
		let parts = &mut [
			// Title.
			IoSlice::new(&self.title),

			// Elapsed.
			IoSlice::new(b"\x1b[2m[\x1b[0;1m"),
			IoSlice::new(self.elapsed.as_bytes()),
			IoSlice::new(b"\x1b[0;2m]\x1b[0m  "),

			// Bars.
			IoSlice::new(b"\x1b[2m[\x1b[0;1;96m"),
			IoSlice::new(self.bar_done),
			IoSlice::new(b"\x1b[0;1;34m"),
			IoSlice::new(self.bar_undone),
			IoSlice::new(b"\x1b[0;2m]\x1b[0;1;96m  "),

			// Done/total.
			IoSlice::new(self.done.as_bytes()),
			IoSlice::new(b"\x1b[0;2m/\x1b[0;1;34m"),
			IoSlice::new(self.total.as_bytes()),

			// Percent.
			IoSlice::new(b"\x1b[0;1m  "),
			IoSlice::new(self.percent.as_bytes()),

			// Tasks.
			IoSlice::new(b"\x1b[0;35m"),
			IoSlice::new(&self.doing),

			// The end!
			IoSlice::new(b"\x1b[0m\n"),
		];

		// Write and return the line count!
		if write_all_vectored(parts.as_mut_slice(), handle) { Some(self.lines()) }
		else { None }
	}
}

impl ProglessBuffer {
	/// # Set Bars.
	fn set_bars(&mut self, width: u8, done: u32, total: u32) {
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
		let space: u8 = width.saturating_sub(u8::saturating_from(
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
	fn set_doing(&mut self, doing: &BTreeSet<ProglessTask>, width: u8) {
		/// # Task Prefix.
		///
		/// This translates to:           •   •   •   •   ↳             •
		const PREFIX: &[u8; 9] = &[b'\n', 32, 32, 32, 32, 226, 134, 179, 32];

		// Reset.
		self.doing.truncate(0);

		// The actual width we can work with is minus six for padding, six for
		// the prefix.
		let width = usize::from(width.saturating_sub(12));

		// Add each task as its own line, assuming we have the room.
		self.lines_doing = 0;
		if 2 <= width {
			for line in doing.iter().filter_map(|line| line.fitted(width)).take(255) {
				self.doing.extend_from_slice(PREFIX);
				self.doing.extend_from_slice(line);
				self.lines_doing += 1;
			}
		}
	}

	/// # Update Title.
	fn set_title(&mut self, title: Option<&Msg>, width: u8) {
		if let Some(title) = title {
			title.fitted(usize::from(width)).as_ref().clone_into(&mut self.title);
			self.lines_title = u8::try_from(bytecount::count(title, b'\n'))
				.unwrap_or(u8::MAX);
		}
		else {
			self.title.truncate(0);
			self.lines_title = 0;
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
/// To that end, you may wish to use the [`Progless::add`] and [`Progless::remove`]
/// methods at the start and end of each iteration instead of manually
/// incrementing the counts.
///
/// Doing this, a list of active tasks will be maintained and printed along
/// with the progress. Removing a task automatically increments the done count,
/// so if you're tracking tasks, you should *not* call [`Progless::increment`].
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
///     let task: String = format!("Task #{}.", i);
///     pbar.add(&task);
///
///     // Do some work.
///
///     pbar.remove(&task);
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
			inner
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
					inner
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
	/// directly, or something that implements `AsRef<str>` or `Borrow<str>`.
	///
	/// As this takes an `Option`, you can pass `None` to unset the title
	/// entirely.
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
		Msg::new(kind, format!(
			"{} in {}.",
			self.inner.done().nice_inflect(singular.as_ref(), plural.as_ref()),
			NiceElapsed::from(self.inner.started),
		))
			.with_newline(true)
	}
}

/// # Passthrough Setters.
impl Progless {
	#[inline]
	/// # Add a task.
	///
	/// The progress bar can optionally keep track of tasks that are actively
	/// "in progress", which can be particularly useful when operating in
	/// parallel.
	///
	/// Any `AsRef<str>` value will do. See the module documentation for
	/// example usage.
	///
	/// Returns `true` if the task was accepted. (If `false`, you should use
	/// [`Progless::increment`] to mark the task as done instead of
	/// [`Progless::remove`].)
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
	///     let task: String = format!("Task #{}.", i);
    ///     pbar.add(&task);
    ///
    ///     // Do some work.
    ///
    ///     pbar.remove(&task);
	/// }
	///
	/// pbar.finish();
	/// ```
	pub fn add<S>(&self, txt: S) -> bool
	where S: AsRef<str> { self.inner.add(txt.as_ref()) }

	#[inline]
	/// # Increment Done.
	///
	/// Increase the completed count by exactly one. This is safer to use than
	/// `set_done()` in cases where multiple tasks are happening at once as it
	/// will not accidentally decrease the value, etc.
	///
	/// See the various examples all over this page for more information.
	pub fn increment(&self) { self.inner.increment(); }

	#[inline]
	/// # Increment Done by N.
	///
	/// Increase the completed count by `n`. This is safer to use than `set_done()`
	/// and more efficient than calling `increment()` a million times in a row.
	pub fn increment_n(&self, n: u32) { self.inner.increment_n(n); }

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

	#[inline]
	/// # Remove a task.
	///
	/// This is the equal and opposite companion to [`Progless::add`]. Calling
	/// this will automatically increment the done count by one, so should not
	/// be used in cases where you're triggering done changes manually.
	///
	/// See [`Progless::add`] for more details. If you use one, you must use
	/// both.
	pub fn remove<S>(&self, txt: S)
	where S: AsRef<str> { self.inner.remove(txt.as_ref()); }

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
	///
	/// ## Errors
	///
	/// This will return an error if the new total is zero.
	pub fn reset(&self, total: u32) -> Result<(), ProglessError> {
		self.inner.reset(total)?;
		self.steady.start(Arc::clone(&self.inner));
		Ok(())
	}

	#[inline]
	/// # Set Done.
	///
	/// Set the done count to a specific value.
	///
	/// In general, you should either use [`Progless::add`]/[`Progless::remove`]
	/// or [`Progless::increment`] rather than this method, as they ensure any
	/// changes made are *relative*.
	///
	/// This method *overrides* the done value instead, so can cause
	/// regressions if you're doing task work in parallel and one thread
	/// finishes before another, etc.
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
	/// # Set Title As X: Reticulating Splines…
	///
	/// This is simply shorthand for generating a "Reticulating Splines…"
	/// title, where X is the value passed in (usually the app name).
	///
	/// It's a sort of default…
	pub fn set_reticulating_splines<S>(&self, app: S)
	where S: AsRef<str> {
		self.inner.set_title(Some(Msg::custom(
			app.as_ref(),
			199,
			"Reticulating splines\u{2026}"
		)));
	}

	#[inline]
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
	pub fn sigint(&self) { self.inner.sigint(); }
}



#[cfg(unix)]
#[must_use]
#[inline]
/// # Term Width.
///
/// Return the column width of STDERR, if any, minus one to mitigate any
/// whitespace weirdness at the edge.
fn term_width() -> u8 {
	use terminal_size::Width;
	terminal_size::terminal_size_of(std::io::stderr()).map_or(
		0,
		|(Width(w), _)| u8::saturating_from(w.saturating_sub(1))
	)
}

#[cfg(not(unix))]
#[must_use]
#[inline]
/// # Term Width.
///
/// Return the terminal column width, if any, minus one to mitigate any
/// whitespace weirdness at the edge.
fn term_width() -> u8 {
	use terminal_size::Width;
	terminal_size::terminal_size().map_or(
		0,
		|(Width(w), _)| u8::saturating_from(w.saturating_sub(1))
	)
}
