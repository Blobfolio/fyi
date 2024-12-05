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
	hash,
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



/// # Bar Filler: Done.
static BAR_DONE:   [u8; 256] = [b'#'; 256];

/// # Dash Filler: TBD.
static BAR_UNDONE: [u8; 256] = [b'-'; 256];

/// # Double-Digit Times.
///
/// This holds pre-asciified double-digit numbers up to sixty for use by the
/// `write_time` method. It doesn't need to hold anything larger than that.
static DD: [[u8; 2]; 60] = [
	[48, 48], [48, 49], [48, 50], [48, 51], [48, 52], [48, 53], [48, 54], [48, 55], [48, 56], [48, 57],
	[49, 48], [49, 49], [49, 50], [49, 51], [49, 52], [49, 53], [49, 54], [49, 55], [49, 56], [49, 57],
	[50, 48], [50, 49], [50, 50], [50, 51], [50, 52], [50, 53], [50, 54], [50, 55], [50, 56], [50, 57],
	[51, 48], [51, 49], [51, 50], [51, 51], [51, 52], [51, 53], [51, 54], [51, 55], [51, 56], [51, 57],
	[52, 48], [52, 49], [52, 50], [52, 51], [52, 52], [52, 53], [52, 54], [52, 55], [52, 56], [52, 57],
	[53, 48], [53, 49], [53, 50], [53, 51], [53, 52], [53, 53], [53, 54], [53, 55], [53, 56], [53, 57],
];

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
	TICK_BAR | TICK_DOING | TICK_DONE | TICK_PERCENT | TICK_TOTAL | TICKING;

/// # Flag: Terminal Resized.
const TICK_RESIZED: u8 =
	TICK_BAR | TICK_DOING | TICK_TITLE;

/// # Flag: Drawables.
const TICK_DRAWABLE: u8 =
	TICK_BAR | TICK_DOING | TICK_DONE | TICK_PERCENT | TICK_TITLE | TICK_TOTAL;

/// # Flag: Repaint Bar.
const TICK_BAR: u8 =     0b0000_0001;

/// # Flag: Repaint Task List.
const TICK_DOING: u8 =   0b0000_0010;

/// # Flag: Repaint Done Value.
const TICK_DONE: u8 =    0b0000_0100;

/// # Flag: Repaint Percent.
const TICK_PERCENT: u8 = 0b0000_1000;

/// # Flag: Repaint Title.
const TICK_TITLE: u8 =   0b0001_0000;

/// # Flag: Repaint Total Value.
const TICK_TOTAL: u8 =   0b0010_0000;

/// # Flag: Is Ticking?
const TICKING: u8 =      0b0100_0000;

/// # Flag: SIGINT Received?
const SIGINT: u8 =       0b1000_0000;

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

	/// # Active Task List.
	doing: Mutex<BTreeSet<ProglessTask>>,

	/// # Total Tasks.
	total: AtomicU32,
}

impl Default for ProglessInner {
	fn default() -> Self {
		Self {
			buf: Mutex::new(ProglessBuffer::default()),
			flags: AtomicU8::new(0),

			last_lines: AtomicU8::new(0),
			last_width: AtomicU8::new(0),

			started: Instant::now(),
			elapsed: AtomicU32::new(0),

			title: Mutex::new(None),
			done: AtomicU32::new(0),
			doing: Mutex::new(BTreeSet::default()),
			total: AtomicU32::new(1),
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
		if self.running() {
			self.flags.store(0, SeqCst);
			self.done.store(self.total(), SeqCst);
			self.elapsed.store(
				u32::saturating_from(self.started.elapsed().as_millis()),
				SeqCst
			);
			mutex!(self.doing).clear();
			self.print_cls();
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

	/// # Percent.
	///
	/// Return the value of `done / total`. The value will always be between
	/// `0.0..=1.0`.
	fn percent(&self) -> f64 {
		let done = self.done();
		let total = self.total();

		if done == 0 { 0.0 }
		else if done == total { 1.0 }
		else {
			f64::from(done) / f64::from(total)
		}
	}

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
	fn add(&self, txt: &str) {
		if
			self.running() &&
			ProglessTask::new(txt).is_some_and(|m| mutex!(self.doing).insert(m))
		{
			self.flags.fetch_or(TICK_DOING, SeqCst);
		}
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
				self.flags.fetch_or(TICK_DONE | TICK_PERCENT | TICK_BAR, SeqCst);
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
				self.flags.fetch_or(TICK_DONE | TICK_PERCENT | TICK_BAR, SeqCst);
			}
		}
	}

	/// # Push Message.
	///
	/// "Insert" (print) a line before the running progress bar, useful for
	/// debug logs, warnings, etc., that would otherwise have to wait for the
	/// [`Progless`] instance to finish hogging the display.
	///
	/// Note: This will add a `\n` to the end of the string.
	///
	/// The message will be printed to STDERR if `stderr`, otherwise STDOUT.
	fn push_msg(&self, msg: Msg, stderr: bool) {
		self.print_cls();

		let msg = msg.with_newline(true);
		if stderr { msg.eprint(); }
		else { msg.print(); }

		if self.running() { self.tick(true); }
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
				self.flags.fetch_or(TICK_DONE | TICK_PERCENT | TICK_BAR, SeqCst);
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
			if let Some(title) = title {
				mutex!(self.title).replace(title.with_newline(true));
			}
			else {
				mutex!(self.title).take();
			}

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
			self.flags.fetch_or(SIGINT, SeqCst);
			self.set_title(Some(Msg::warning("Early shutdown in progress.")));
		}
	}
}

/// # Render.
impl ProglessInner {
	/// # Preprint.
	///
	/// This method accepts a completed buffer ready for printing, hashing it
	/// for comparison with the last job. If unique, the previous output is
	/// erased and replaced with the new output.
	fn preprint(&self) {
		// Erase old lines if needed.
		self.print_cls();

		// Print and update the line count.
		let lines = mutex!(self.buf).print();
		if let Ok(lines) = lines {
			self.last_lines.store(lines, SeqCst);
		}
	}

	/// # Erase Output.
	///
	/// This method "erases" any prior output so that new output can be written
	/// in the same place. That's CLI animation, folks!
	fn print_cls(&self) {
		/// # Ten Line Clears.
		const CLS10: &[u8; 140] = b"\
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

		let mut last_lines = usize::from(self.last_lines.swap(0, SeqCst));
		if 0 != last_lines {
			use std::io::Write;

			let writer = std::io::stderr();
			let mut handle = writer.lock();

			// Clear the current line.
			let _res = handle.write_all(b"\x1b[1000D\x1b[K");

			// Now move the cursor up the appropriate number of lines, clearing
			// each as we go.
			loop {
				// We can handle up to ten lines at a time.
				let chunk = usize::min(last_lines, 10);
				let _res = handle.write_all(&CLS10[..14 * chunk]);
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
	/// # Tick Flag Toggle.
	///
	/// If a flag is set, unset it and return true. Otherwise false.
	fn flag_unset(&self, flag: u8) -> bool {
		let old = self.flags.fetch_and(! flag, SeqCst);
		0 != old & flag
	}

	/// # Tick.
	///
	/// Ticking takes all of the changed values (since the last tick), updates
	/// their corresponding parts in the buffer, and prints the result, if any.
	///
	/// To help keep repeated calls to this from overloading the system, work
	/// only takes place if it has been at least 60ms from the last tick.
	fn tick(&self, force: bool) -> bool {
		// We aren't running!
		if ! self.running() {
			return false;
		}

		// We don't want to tick too often... that will just look bad.
		let time_changed: bool = match self.tick_set_secs() {
			None => if force { true } else { return true; },
			Some(x) => x,
		};

		// Check the terminal width first because it affects most of what
		// follows.
		let width = self.tick_set_width();
		if width < MIN_DRAW_WIDTH {
			self.print_cls();
			return true;
		}

		// If the time hasn't changed, and nothing else has changed, we can
		// abort without all the tedious checking.
		if ! time_changed && (self.flags.load(SeqCst) & TICK_DRAWABLE == 0) {
			return true;
		}

		// Handle the rest!
		self.tick_set_doing(width);
		self.tick_set_done();
		self.tick_set_percent();
		self.tick_set_title(width);
		self.tick_set_total();

		// The bar's width depends on how much space remains after the other
		// elements sharing its line so it needs to go last.
		self.tick_set_bar(width);

		// Maybe we're printing, maybe we're not!
		self.preprint();

		true
	}

	/// # Tick Bar.
	///
	/// This reslices the done/remaining portions of the literal *bar* part of
	/// the progress bar.
	///
	/// The entire line will never exceed 255 characters. The bar portion,
	/// conservatively speaking, cannot exceed 244.
	fn tick_set_bar(&self, width: u8) {
		if self.flag_unset(TICK_BAR) {
			// Assume zero.
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
			let mut buf = mutex!(self.buf);
			let space: u8 = width.saturating_sub(u8::saturating_from(
				19 +
				buf.done.len() +
				buf.total.len() +
				buf.percent.len()
			));

			// If we have any space, divide it up proportionately.
			if MIN_BARS_WIDTH <= space {
				let total = self.total();
				if 0 != total {
					let done = self.done();

					// Nothing is done.
					if done == 0 { w_undone = space; }
					// Everything is done!
					else if done == total { w_done = space; }
					// Working on it!
					else {
						w_done = u8::saturating_from((done * u32::from(space)).wrapping_div(total));
						w_undone = space.saturating_sub(w_done);
					}
				}

				debug_assert_eq!(
					w_done + w_undone,
					space,
					"BUG: bar space was miscalculated."
				);
			}

			// Update the parts!.
			buf.bar_done =     &BAR_DONE[..usize::from(w_done)];
			buf.bar_undone = &BAR_UNDONE[..usize::from(w_undone)];
		}
	}

	/// # Tick Doing.
	///
	/// Update the task list portion of the buffer. This is triggered both by
	/// changes to the task list as well as resoluation changes (as long values
	/// may require lazy cropping).
	fn tick_set_doing(&self, width: u8) {
		if self.flag_unset(TICK_DOING) {
			mutex!(self.buf).set_doing(&mutex!(self.doing), width);
		}
	}

	/// # Tick Done.
	///
	/// This updates the "done" portion of the buffer as needed.
	fn tick_set_done(&self) {
		if self.flag_unset(TICK_DONE) {
			mutex!(self.buf).set_done(NiceU32::from(self.done()));
		}
	}

	/// # Tick Percent.
	///
	/// This updates the "percent" portion of the buffer as needed.
	fn tick_set_percent(&self) {
		if self.flag_unset(TICK_PERCENT) {
			mutex!(self.buf).set_percent(NicePercent::from(self.percent()));
		}
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
	/// A value of `true` is returned if one or more seconds has elapsed since
	/// the last tick, otherwise `false` is returned.
	fn tick_set_secs(&self) -> Option<bool> {
		let now: u32 = u32::saturating_from(self.started.elapsed().as_millis());
		let before: u32 = self.elapsed.load(SeqCst);

		// Try not to exceed the steady tick rate.
		if now.saturating_sub(before) < steady::TICK_RATE { return None; }

		let secs: u32 = now.wrapping_div(1000);
		self.elapsed.store(now, SeqCst);

		// No change to the seconds bit.
		if secs == before.wrapping_div(1000) { Some(false) }
		else {
			let [h, m, s] = NiceElapsed::hms(secs);
			write_time(&mut mutex!(self.buf).elapsed, h, m, s);
			Some(true)
		}
	}

	/// # Tick Title.
	///
	/// The title needs to be rewritten both on direct change and resolution
	/// change. Long titles are lazy-cropped as needed.
	fn tick_set_title(&self, width: u8) {
		if self.flag_unset(TICK_TITLE) {
			mutex!(self.buf).set_title(mutex!(self.title).as_ref(), width);
		}
	}

	/// # Tick Total.
	///
	/// This updates the "total" portion of the buffer as needed.
	fn tick_set_total(&self) {
		if self.flag_unset(TICK_TOTAL) {
			mutex!(self.buf).set_total(NiceU32::from(self.total()));
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
	/// TODO: replace with `NiceClock` after updating dactyl.
	elapsed: [u8; 8],

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

impl Default for ProglessBuffer {
	fn default() -> Self {
		Self {
			title: Vec::new(),
			elapsed: *b"00:00:00",
			bar_done: &[],
			bar_undone: &[],
			done: NiceU32::default(),
			total: NiceU32::default(),
			percent: NicePercent::min(),
			doing: Vec::new(),
			lines_doing: 0,
			lines_title: 0,
		}
	}
}

impl hash::Hash for ProglessBuffer {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
    	state.write(&self.title);
    	state.write(self.elapsed.as_slice());
    	state.write_usize(self.bar_done.len());
    	state.write_usize(self.bar_undone.len());
    	state.write(self.done.as_bytes());
    	state.write(self.total.as_bytes());
    	state.write(self.percent.as_bytes());
    	state.write(&self.doing);
    }
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
	fn print(&self) -> std::io::Result<u8> {
		use std::io::Write;

		let mut w = std::io::stderr().lock();

		// Title.
		w.write_all(&self.title)?;

		// Elapsed.
		w.write_all(b"\x1b[2m[\x1b[0;1m")?;
		w.write_all(self.elapsed.as_slice())?;
		w.write_all(b"\x1b[0;2m]\x1b[0m  ")?;

		// Bars.
		w.write_all(b"\x1b[2m[\x1b[0;1;96m")?;
		w.write_all(self.bar_done)?;
		w.write_all(b"\x1b[0;1;34m")?;
		w.write_all(self.bar_undone)?;
		w.write_all(b"\x1b[0;2m]\x1b[0;1;96m  ")?;

		// Done/total.
		w.write_all(self.done.as_bytes())?;
		w.write_all(b"\x1b[0;2m/\x1b[0;1;34m")?;
		w.write_all(self.total.as_bytes())?;

		// Percent.
		w.write_all(b"\x1b[0;1m  ")?;
		w.write_all(self.percent.as_bytes())?;

		// Tasks.
		if ! self.doing.is_empty() {
			w.write_all(b"\x1b[0;35m")?;
			w.write_all(&self.doing)?;
		}

		// The end!
		w.write_all(b"\x1b[0m\n")?;
		w.flush()?;

		// Return the line count.
		Ok(self.lines())
	}
}

impl ProglessBuffer {
	#[inline]
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

	#[inline]
	/// # Set Done.
	///
	/// TODO: remove and use `NiceU32::replace` after upgrading dactyl.
	fn set_done(&mut self, done: NiceU32) { self.done = done; }

	#[inline]
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

	#[inline]
	/// # Set Total.
	///
	/// TODO: remove and use `NiceU32::replace` after upgrading dactyl.
	fn set_total(&mut self, total: NiceU32) { self.total = total; }

	#[inline]
	/// # Set Percent.
	fn set_percent(&mut self, percent: NicePercent) { self.percent = percent; }
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
	pub fn add<S>(&self, txt: S)
	where S: AsRef<str> { self.inner.add(txt.as_ref()); }

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
	/// "Insert" (print) a line before the running progress bar, useful for
	/// debug logs, warnings, etc., that would otherwise have to wait for the
	/// [`Progless`] instance to finish hogging the display.
	///
	/// Note: This will add a `\n` to the end of the string.
	///
	/// The message will be printed to STDERR if `stderr`, otherwise STDOUT.
	pub fn push_msg<S>(&self, msg: S, stderr: bool)
	where S: Into<Msg> { self.inner.push_msg(msg.into(), stderr); }

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

/// # Write Time.
///
/// This writes HH:MM:SS to the provided pointer.
///
/// ## Panics
///
/// This method is only intended to cover values that fit in a day and will
/// panic if `h`, `m`, or `s` is outside the range of `0..60`.
///
/// ## Safety
///
/// The pointer must have 8 bytes free or undefined things will happen.
fn write_time(buf: &mut [u8; 8], h: u8, m: u8, s: u8) {
	assert!(
		h < 60 && m < 60 && s < 60,
		"BUG: Invalid progress time pieces."
	);

	// Write 'em.
	buf[..2].copy_from_slice(DD[usize::from(h)].as_slice());
	buf[3..5].copy_from_slice(DD[usize::from(m)].as_slice());
	buf[6..8].copy_from_slice(DD[usize::from(s)].as_slice());
}
