/*!
# FYI Msg - Progless
*/

use ahash::RandomState;
use atomic::Atomic;
use crate::{
	BUFFER8,
	fitted,
	Msg,
	MsgBuffer,
	MsgKind,
};
use dactyl::{
	NiceElapsed,
	NicePercent,
	NiceU32,
	traits::SaturatingFrom,
	write_time,
};

#[cfg(feature = "parking_lot_mutex")]
use parking_lot::Mutex;

use std::{
	borrow::Borrow,
	cmp::Ordering,
	collections::HashSet,
	fmt,
	hash::{
		Hash,
		Hasher,
	},
	num::{
		NonZeroU32,
		NonZeroU64,
	},
	sync::{
		Arc,
		atomic::{
			AtomicBool,
			AtomicU8,
			AtomicU32,
			AtomicU64,
			Ordering::SeqCst,
		},
	},
	thread::JoinHandle,
	time::{
		Instant,
		Duration,
	},
};

#[cfg(not(feature = "parking_lot_mutex"))]
use std::sync::Mutex;



/// # (Not) Random State.
///
/// Using a fixed seed value for `AHashSet` drops a few dependencies and
/// stops Valgrind from complaining about 64 lingering bytes from the runtime
/// static that would be used otherwise.
///
/// For our purposes, the variability of truly random keys isn't really needed.
const AHASH_STATE: RandomState = RandomState::with_seeds(13, 19, 23, 71);



/// # Tick Flags.
///
/// These flags indicate whether or not a given component has changed since the
/// last tick, saving the overhead of recalculating the buffer values each time
/// a value changes. (Instead they're only recalculated at most once per tick.)
const TICK_NEW: u8 =     0b0110_0001;
const TICK_RESIZED: u8 = 0b0001_0011;

const TICK_BAR: u8 =     0b0000_0001;
const TICK_DOING: u8 =   0b0000_0010;
const TICK_DONE: u8 =    0b0000_0100;
const TICK_PERCENT: u8 = 0b0000_1000;
const TICK_TITLE: u8 =   0b0001_0000;
const TICK_TOTAL: u8 =   0b0010_0000;

const TICKING: u8 =      0b0100_0000;



/// # Buffer Indexes.
///
/// The start and end points of the malleable progress components are stored in
/// an array for easy access. These are their indexes.
const PART_TITLE: usize = 0;
const PART_ELAPSED: usize = 1;
const PART_BAR_DONE: usize = 2;
const PART_BAR_UNDONE: usize = 3;
const PART_DONE: usize = 4;
const PART_TOTAL: usize = 5;
const PART_PERCENT: usize = 6;
const PART_DOING: usize = 7;



/// # Misc Variables.
const MIN_BARS_WIDTH: u8 = 10;
const MIN_DRAW_WIDTH: u8 = 40;

// This translates to:          •   •   •   •   ↳             •
const TASK_PREFIX: &[u8; 8] = &[32, 32, 32, 32, 226, 134, 179, 32];



#[cfg(not(feature = "parking_lot_mutex"))]
/// # Helper: Mutex Unlock.
///
/// This just moves tedious code out of the way.
macro_rules! mutex {
	($var:expr) => ($var.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
}

#[cfg(feature = "parking_lot_mutex")]
/// # Helper: Mutex Unlock.
///
/// This just moves tedious code out of the way.
macro_rules! mutex { ($var:expr) => ($var.lock()); }



#[derive(Debug, Copy, Clone)]
/// # Obligatory error type.
pub enum ProglessError {
	/// # Empty task.
	EmptyTask,
	/// # Length (task) overflow.
	TaskOverflow,
	/// # Length (total) must be non-zero.
	EmptyTotal,
	/// # Length (total) overflow.
	TotalOverflow,
}

impl fmt::Display for ProglessError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl std::error::Error for ProglessError {}

impl ProglessError {
	#[must_use]
	/// # As Str.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::EmptyTask => "Task names cannot be empty.",
			Self::TaskOverflow => "Task names cannot exceed 65,535 bytes.",
			Self::EmptyTotal => "At least one task is required.",
			Self::TotalOverflow => "The total number of tasks cannot exceed 4,294,967,295.",
		}
	}
}



#[derive(Debug, Clone)]
/// # A Task.
///
/// This holds a boxed slice and the pre-calculated display width of said
/// slice. Though stored as raw bytes, the value is valid UTF-8.
struct ProglessTask {
	task: Box<[u8]>,
	width: u16,
}

impl TryFrom<&[u8]> for ProglessTask {
	type Error = ProglessError;

	fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
		// It has to fit in a u16.
		if src.is_empty() { Err(ProglessError::EmptyTask) }
		else {
			Ok(Self {
				task: Box::from(src),
				width: u16::try_from(fitted::width(src)).map_err(|_| ProglessError::TaskOverflow)?,
			})
		}
	}
}

impl Borrow<[u8]> for ProglessTask {
	#[inline]
	fn borrow(&self) -> &[u8] { &self.task }
}

impl Eq for ProglessTask {}

impl Hash for ProglessTask {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) { self.task.hash(state); }
}

impl PartialEq for ProglessTask {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.task == other.task }
}

impl ProglessTask {
	/// # Push To.
	///
	/// Push this task to the vector buffer, ensuring it fits the specified
	/// width.
	fn push_to(&self, buf: &mut Vec<u8>, width: u8) {
		let avail = width.saturating_sub(6);
		if self.width > u16::from(avail) {
			let end = fitted::length_width(&self.task, usize::from(avail));
			if end > 0 {
				buf.extend_from_slice(TASK_PREFIX);
				buf.extend_from_slice(&self.task[..end]);
				buf.push(b'\n');
			}
		}
		else {
			buf.extend_from_slice(TASK_PREFIX);
			buf.extend_from_slice(&self.task);
			buf.push(b'\n');
		}
	}
}



#[derive(Debug)]
/// # Progless Inner Data.
///
/// This holds most of the actual progress state information. The public
/// struct holds an instance of this behind an [`std::sync::Arc`] for easier
/// thread-sharing.
struct ProglessInner {
	buf: Mutex<MsgBuffer<BUFFER8>>,
	flags: AtomicU8,

	// A hash of what was last printed. Saves redundant work in cases where
	// nothing has changed since the last print.
	last_hash: AtomicU64,

	// The number of lines last printed. Before printing new output, this many
	// lines must be "erased".
	last_lines: AtomicU8,

	// The screen width from the last print. If this changes, all buffer parts
	// are recalculated (even if their values haven't changed) to ensure they
	// fit the new width.
	last_width: AtomicU8,

	// The instant the object was first created. All timings are derived from
	// this value.
	started: Atomic<Instant>,

	// This is the number of elapsed milliseconds as of the last tick. This
	// gives us a reference to throttle back-to-back ticks as well as a cache
	// of the seconds written to the `[00:00:00]` portion of the buffer.
	elapsed: AtomicU32,

	title: Mutex<Option<Msg>>,
	done: AtomicU32,
	doing: Mutex<HashSet<ProglessTask, RandomState>>,
	total: Atomic<NonZeroU32>,
}

impl From<NonZeroU32> for ProglessInner {
	fn from(total: NonZeroU32) -> Self {
		Self {
			buf: Mutex::new(MsgBuffer::<BUFFER8>::from_raw_parts(
				vec![
					//  Title would go here.

					//  \e   [   2    m   [   \e  [   0   ;   1    m
						27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 109,
					//   0   0   :   0   0   :   0   0
						48, 48, 58, 48, 48, 58, 48, 48,
					//  \e   [   0   ;   2    m   ]  \e   [   0    m   •   •
						27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32,

					//  \e   [   2    m   [  \e   [   0   ;   1   ;   9   6    m
						27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 59, 57, 54, 109,

					//  Bar Done would go here.

					//  \e   [   0   ;   1   ;   3   4    m
						27, 91, 48, 59, 49, 59, 51, 52, 109,

					//  Bar Undone would go here.

					//  \e   [   0   ;   2    m   ]  \e   [   0    m   •   •
						27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32,

					//  Done.
					//  \e   [   1   ;   9   6    m
						27, 91, 49, 59, 57, 54, 109,
					//   0
						48,

					//  The slash between Done and Total.
					//  \e   [   0   ;   2    m   /  \e   [   0   ;   1   ;   3   4    m
						27, 91, 48, 59, 50, 109, 47, 27, 91, 48, 59, 49, 59, 51, 52, 109,

					//  Total.
					//   0
						48,

					//  The bit between Total and Percent.
					//  \e   [   0   ;   1    m   •   •
						27, 91, 48, 59, 49, 109, 32, 32,

					//  Percent.
					//   0   .   0   0   %
						48, 46, 48, 48, 37,
					//  \e   [   0    m  \n
						27, 91, 48, 109, 10,

				//  Doing would go here.
				],
				[
					0, 0,     // Title.
					11, 19,   // Elapsed.
					46, 46,   // Bar Done.
					55, 55,   // Bar Undone.
					75, 76,   // Done.
					92, 93,   // Total.
					101, 106, // Percent.
					111, 111, // Current Tasks.
				]
			)),
			flags: AtomicU8::new(TICK_NEW),

			last_hash: AtomicU64::new(0),
			last_lines: AtomicU8::new(0),
			last_width: AtomicU8::new(0),

			started: Atomic::new(Instant::now()),
			elapsed: AtomicU32::new(0),

			title: Mutex::new(None),
			done: AtomicU32::new(0),
			doing: Mutex::new(HashSet::with_hasher(AHASH_STATE)),
			total: Atomic::new(total),
		}
	}
}

impl TryFrom<u32> for ProglessInner {
	type Error = ProglessError;

	fn try_from(total: u32) -> Result<Self, Self::Error> {
		Ok(Self::from(NonZeroU32::new(total).ok_or(ProglessError::EmptyTotal)?))
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
				u32::saturating_from(self.started.load(SeqCst).elapsed().as_millis()),
				SeqCst
			);
			mutex!(self.doing).clear();
			self.print_blank();
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
	/// # Last Width.
	///
	/// The CLI screen width as it was when last checked. If this value
	/// happens to change between ticks, it will force redraw the content to
	/// make sure it fits correctly.
	fn last_width(&self) -> u8 { self.last_width.load(SeqCst) }

	/// # Percent.
	///
	/// Return the value of `done / total`. The value will always be between
	/// `0.0..=1.0`.
	fn percent(&self) -> f64 {
		let done = self.done.load(SeqCst);
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
	fn total(&self) -> u32 { self.total.load(SeqCst).get() }
}

/// # Setters.
impl ProglessInner {
	/// # Add a task.
	///
	/// The progress bar can optionally keep track of tasks that are actively
	/// "in progress", which can be particularly useful when operating in
	/// parallel.
	///
	/// Any `AsRef<str>` value will do. See the module documentation for
	/// example usage.
	fn add<S>(&self, txt: S)
	where S: AsRef<str> {
		if self.running() {
			if let Ok(m) = ProglessTask::try_from(txt.as_ref().as_bytes()) {
				if mutex!(self.doing).insert(m)	{
					self.flags.fetch_or(TICK_DOING, SeqCst);
				}
			}
		}
	}

	#[inline]
	/// # Increment Done.
	///
	/// Increase the completed count by exactly one. This is safer to use than
	/// `set_done()` in cases where multiple tasks are happening at once as it
	/// will not accidentally decrease the value, etc.
	fn increment(&self) { self.set_done(self.done() + 1); }

	/// # Remove a task.
	///
	/// This is the equal and opposite companion to `add`. Calling this will
	/// automatically increment the done count by one, so should not be used
	/// in cases where you're triggering done changes manually.
	fn remove<S>(&self, txt: S)
	where S: AsRef<str> {
		if self.running() && mutex!(self.doing).remove(txt.as_ref().as_bytes())	{
			self.flags.fetch_or(TICK_DOING, SeqCst);
			self.increment();
		}
	}

	/// # Set Done.
	///
	/// Set the done count to a specific value. Be careful in cases where
	/// things are happening in parallel; in such cases `increment` is probably
	/// better.
	fn set_done(&self, mut done: u32) {
		if self.running() {
			let total = self.total();

			done = total.min(done);
			if done != self.done() {
				if done == total { self.stop(); }
				else {
					self.done.store(done, SeqCst);
					self.flags.fetch_or(TICK_DONE | TICK_PERCENT | TICK_BAR, SeqCst);
				}
			}
		}
	}

	#[allow(clippy::option_if_let_else)] // This is better.
	/// # Set Title.
	///
	/// Give the progress bar a title, which will be shown above the progress
	/// bits while progress is progressing, and removed afterward with
	/// everything else.
	fn set_title<S>(&self, title: Option<S>)
	where S: Into<Msg> {
		if self.running() {
			if let Some(title) = title.map(Into::into).filter(|x| ! x.is_empty()) {
				mutex!(self.title).replace(title.with_newline(true));
			}
			else {
				mutex!(self.title).take();
			}

			self.flags.fetch_or(TICK_TITLE, SeqCst);
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
		let buf = mutex!(self.buf);
		if 0 == buf.total_len() {
			self.print_blank();
			return;
		}

		// Make sure the content is unique, otherwise we can leave the old bits
		// up.
		let hash = hash64(&*buf);
		if hash == self.last_hash.swap(hash, SeqCst) {
			return;
		}

		// Erase old lines if needed.
		self.print_cls();

		// Update the line count and print!
		self.last_lines.store(u8::saturating_from(bytecount::count(&*buf, b'\n')), SeqCst);
		Self::print(&*buf);
	}

	/// # Print Blank.
	///
	/// This simply resets the last-print hash and clears any prior output.
	fn print_blank(&self) {
		self.last_hash.store(0, SeqCst);
		self.print_cls();
	}

	/// # Print!
	///
	/// Print some arbitrary data to the write place. Haha.
	fn print(buf: &[u8]) {
		use std::io::Write;

		let writer = std::io::stderr();
		let mut handle = writer.lock();
		let _res = handle.write_all(buf).and_then(|_| handle.flush());
	}

	/// # Erase Output.
	///
	/// This method "erases" any prior output so that new output can be written
	/// in the same place. That's CLI animation, folks!
	fn print_cls(&self) {
		// Buffer 10 Line Clears.
		// 0..10 moves the cursor left. This is done only once per reset.
		// 14 is the length of each subsequent command, which moves the cursor up.
		// To clear "n" lines, then, slice [0..(10 + 14 * n)].
		static CLS10: [u8; 150] = [27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75];

		let last_lines = self.last_lines.swap(0, SeqCst);
		if last_lines > 0 {
			// Figure out how to slice our `CLS10` buffer.
			match last_lines.cmp(&10) {
				Ordering::Equal => { Self::print(&CLS10[..]); },
				Ordering::Less => {
					Self::print(&CLS10[0..10 + 14 * usize::from(last_lines)]);
				},
				// To clear more lines, print our pre-calculated buffer (which
				// covers the first 10), and duplicate the line-up chunk (n-10)
				// times to cover the rest.
				Ordering::Greater => {
					Self::print(&[
						&CLS10[..],
						&CLS10[14..28].repeat(usize::from(last_lines - 10)),
					].concat());
				},
			}
		}
	}
}

/// # Ticks.
impl ProglessInner {
	/// # Tick Flag Toggle.
	///
	/// If a flag is set, unset it and return true. Otherwise false.
	fn flag_toggle(&self, flag: u8) -> bool {
		let flags = self.flags.load(SeqCst);
		if 0 == flags & flag { false }
		else {
			self.flags.store(flags & ! flag, SeqCst);
			true
		}
	}

	/// # Tick.
	///
	/// Ticking takes all of the changed values (since the last tick), updates
	/// their corresponding parts in the buffer, and prints the result, if any.
	///
	/// To help keep repeated calls to this from overloading the system, work
	/// only takes place if it has been at least 60ms from the last tick.
	fn tick(&self) -> bool {
		// We aren't running!
		if ! self.running() {
			return false;
		}

		// We don't want to tick too often... that will just look bad.
		let time_changed: bool = match self.tick_set_secs() {
			None => return true,
			Some(x) => x,
		};

		// Check the terminal width first because it affects most of what
		// follows.
		self.tick_set_width();
		if self.last_width() < MIN_DRAW_WIDTH {
			self.flags.store(TICKING, SeqCst);
			self.print_blank();
			return true;
		}

		// If the time hasn't changed, and nothing else has changed, we can
		// abort without all the tedious checking.
		if ! time_changed && self.flags.load(SeqCst) == TICKING {
			return true;
		}

		// Handle the rest!
		self.tick_set_doing();
		self.tick_set_done();
		self.tick_set_percent();
		self.tick_set_title();
		self.tick_set_total();

		// The bar's width depends on how much space remains after the other
		// elements sharing its line so it needs to go last.
		self.tick_set_bar();

		// Maybe we're printing, maybe we're not!
		self.preprint();

		true
	}

	/// # Tick Bar Dimensions.
	///
	/// This calculates the available widths for each of the three progress
	/// bars (done, doing, remaining).
	///
	/// If the total available space winds up being less than 10, all three
	/// values are set to zero, indicating this component should be removed.
	fn tick_bar_widths(&self) -> (u8, u8) {
		// The magic "11" is made up of the following hard-coded pieces:
		// 2: braces around elapsed time;
		// 2: spaces after elapsed time;
		// 1: the "/" between done and total;
		// 2: the spaces after total;
		// 2: the braces around the bar itself (should there be one);
		// 2: the spaces after the bar itself (should there be one);
		let space: u8 = self.last_width().saturating_sub(u8::saturating_from({
			let buf = mutex!(self.buf);
			11 +
			buf.len(PART_ELAPSED) +
			buf.len(PART_DONE) +
			buf.len(PART_TOTAL) +
			buf.len(PART_PERCENT)
		}));
		if space < MIN_BARS_WIDTH { return (0, 0); }

		let total = self.total();
		if 0 == total { return (0, 0); }

		// Done!
		let done = self.done();
		if done == total { (space, 0) }
		// Working on it!
		else {
			let o_done: u8 = u8::saturating_from((done * u32::from(space)).wrapping_div(total));
			(o_done, space.saturating_sub(o_done))
		}
	}

	#[allow(clippy::cast_possible_truncation)] // These parts are constrained to u8::MAX.
	/// # Tick Bar.
	///
	/// This redraws the actual progress *bar* portion of the buffer, which is
	/// actually three different bars squished together: Done, Doing, and
	/// Pending.
	///
	/// The entire line will never exceed 255 characters. The bars,
	/// conservatively, cannot exceed 244, and will always be at least 10.
	fn tick_set_bar(&self) {
		static BAR: [u8; 244] = [b'#'; 244];
		static DASH: [u8; 244] = [b'-'; 244];

		if self.flag_toggle(TICK_BAR) {
			let (w_done, w_undone) = self.tick_bar_widths();

			// Update the parts!.
			let mut buf = mutex!(self.buf);

			// We're handling undone first — the reverse display order — as it
			// will only ever shrink, leaving that much less to copy-right when
			// extending the done portion.
			if buf.len(PART_BAR_UNDONE) as u8 != w_undone {
				buf.replace(PART_BAR_UNDONE, &DASH[0..usize::from(w_undone)]);
			}

			if buf.len(PART_BAR_DONE) as u8 != w_done {
				buf.replace(PART_BAR_DONE, &BAR[0..usize::from(w_done)]);
			}
		}
	}

	/// # Tick Doing.
	///
	/// Update the task list portion of the buffer. This is triggered both by
	/// changes to the task list as well as resoluation changes (as long values
	/// may require lazy cropping).
	fn tick_set_doing(&self) {
		if self.flag_toggle(TICK_DOING) {
			let doing = mutex!(self.doing);
			if doing.is_empty() {
				mutex!(self.buf).truncate(PART_DOING, 0);
			}
			else {
				let width: u8 = self.last_width().saturating_sub(6);

				let mut tasks = Vec::<u8>::with_capacity(256);
				tasks.extend_from_slice(b"\x1b[35m");
				doing.iter().for_each(|x| x.push_to(&mut tasks, width));
				tasks.extend_from_slice(b"\x1b[0m");

				mutex!(self.buf).replace(PART_DOING, &tasks);
			}
		}
	}

	/// # Tick Done.
	///
	/// This updates the "done" portion of the buffer as needed.
	fn tick_set_done(&self) {
		if self.flag_toggle(TICK_DONE) {
			mutex!(self.buf).replace(PART_DONE, &NiceU32::from(self.done()));
		}
	}

	/// # Tick Percent.
	///
	/// This updates the "percent" portion of the buffer as needed.
	fn tick_set_percent(&self) {
		if self.flag_toggle(TICK_PERCENT) {
			mutex!(self.buf).replace(PART_PERCENT, &NicePercent::from(self.percent()));
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
		let now: u32 = u32::saturating_from(self.started.load(SeqCst).elapsed().as_millis());
		let before: u32 = self.elapsed.load(SeqCst);

		// Throttle back-to-back ticks.
		if now.saturating_sub(before) < 60 { return None; }

		let secs: u32 = now.wrapping_div(1000);
		self.elapsed.store(now, SeqCst);

		// No change to the seconds bit.
		if secs == before.wrapping_div(1000) { Some(false) }
		else {
			let [h, m, s] = NiceElapsed::hms(secs);
			unsafe {
				let mut buf = mutex!(self.buf);
				let start = buf.start(PART_ELAPSED);
				write_time(buf.as_mut_ptr(start), h, m, s);
			}

			Some(true)
		}
	}

	#[allow(clippy::option_if_let_else)] // This is better.
	/// # Tick Title.
	///
	/// The title needs to be rewritten both on direct change and resolution
	/// change. Long titles are lazy-cropped as needed.
	fn tick_set_title(&self) {
		if self.flag_toggle(TICK_TITLE) {
			if let Some(title) = &*mutex!(self.title) {
				mutex!(self.buf).replace(
					PART_TITLE,
					&title.fitted(usize::from(self.last_width().saturating_sub(1))),
				);
			}
			else {
				mutex!(self.buf).truncate(PART_TITLE, 0);
			}
		}
	}

	/// # Tick Total.
	///
	/// This updates the "total" portion of the buffer as needed.
	fn tick_set_total(&self) {
		if self.flag_toggle(TICK_TOTAL) {
			mutex!(self.buf).replace(PART_TOTAL, &NiceU32::from(self.total()));
		}
	}

	/// # Tick Width.
	///
	/// Check to see if the terminal width has changed since the last run and
	/// update values — i.e. the relevant tick flags — as necessary.
	fn tick_set_width(&self) {
		let width = term_width();
		if width != self.last_width.swap(width, SeqCst) {
			self.flags.fetch_or(TICK_RESIZED, SeqCst);
		}
	}
}



#[derive(Debug, Default)]
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
struct ProglessSteady {
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
				if ! t_enabled.load(SeqCst) || ! t_inner.tick() { break; }

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
	fn stop(&self) {
		if let Some(handle) = mutex!(self.ticker).take() {
			self.enabled.store(false, SeqCst);
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



#[derive(Debug, Clone)]
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
/// ```ignore
/// # use fyi_msg::Progless;
/// # use rayon::prelude::*;
///
/// # let pbar = Progless::try_from(1001_u32).unwrap();
///
/// // ... snip
///
/// // Iterate in Parallel.
/// for i in (0..1001).par_iter() {
///     let task: String = format!("Task #{}.", i);
///     pbar.add(&task);
///
///     // Do some work.
///
///     pbar.remove(&task);
/// }
///
/// // ... snip
/// ```
pub struct Progless {
	steady: Arc<ProglessSteady>,
	inner: Arc<ProglessInner>,
}

impl From<NonZeroU32> for Progless {
	#[inline]
	fn from(total: NonZeroU32) -> Self {
		let inner = Arc::new(ProglessInner::from(total));
		Self {
			steady: Arc::new(ProglessSteady::from(inner.clone())),
			inner
		}
	}
}

impl TryFrom<u32> for Progless {
	type Error = ProglessError;

	#[inline]
	fn try_from(total: u32) -> Result<Self, Self::Error> {
		Ok(Self::from(NonZeroU32::new(total).ok_or(ProglessError::EmptyTotal)?))
	}
}

/// # Helper: `TryFrom`
///
/// This will generate `TryFrom` implementations for various integer types, both
/// bigger and smaller than the target `u32`.
macro_rules! impl_tryfrom {
	// These types fit into u32.
	(true, ($($from:ty),+)) => (
		$(
			impl TryFrom<$from> for Progless {
				type Error = ProglessError;

				#[inline]
				fn try_from(total: $from) -> Result<Self, Self::Error> {
					Self::try_from(u32::from(total))
				}
			}
		)+
	);

	// These types don't necessarily fit.
	(false, ($($from:ty),+)) => (
		$(
			impl TryFrom<$from> for Progless {
				type Error = ProglessError;

				#[inline]
				fn try_from(total: $from) -> Result<Self, Self::Error> {
					let total = u32::try_from(total).map_err(|_| ProglessError::TotalOverflow)?;
					Self::try_from(total)
				}
			}
		)+
	);
}

impl_tryfrom!(true, (u8, u16));
impl_tryfrom!(false, (u64, u128, usize));

impl From<Progless> for Msg {
	#[inline]
	/// # Into [`Msg`]
	///
	/// This provides a simple way to convert a (finished) [`Progless`]
	/// instance into a generic summary [`Msg`] that can e.g. be printed.
	///
	/// For a more advanced summary, use the [`Progless::summary`] method.
	fn from(src: Progless) -> Self {
		// The content is all valid UTF-8; this is safe.
		Self::done([
			"Finished in ",
			NiceElapsed::from(src.inner.started.load(SeqCst)).as_str(),
			".",
		].concat())
			.with_newline(true)
	}
}

/// # Construction/Destruction.
impl Progless {
	#[must_use]
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
		self.inner.set_title(title);
		self
	}

	/// # Stop.
	///
	/// Finish the progress bar and shut down the steady ticker.
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
	pub fn finish(&self) {
		self.inner.stop();
		self.steady.stop();
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
		let done = self.inner.done();
		let noun =
			if done == 1 { singular.as_ref() }
			else { plural.as_ref() };

		Msg::new(kind, format!(
			"{} {} in {}.",
			NiceU32::from(done).as_str(),
			noun,
			NiceElapsed::from(self.inner.started.load(SeqCst)).as_str(),
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
	where S: AsRef<str> { self.inner.add(txt); }

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
	/// # Remove a task.
	///
	/// This is the equal and opposite companion to [`Progless::add`]. Calling
	/// this will automatically increment the done count by one, so should not
	/// be used in cases where you're triggering done changes manually.
	///
	/// See [`Progless::add`] for more details. If you use one, you must use
	/// both.
	pub fn remove<S>(&self, txt: S)
	where S: AsRef<str> { self.inner.remove(txt); }

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
	where S: Into<Msg> { self.inner.set_title(title); }
}



#[derive(Debug, Copy, Clone)]
/// # Before and After.
///
/// This is a potentially useful companion to [`Progless`] that tracks an
/// arbitrary non-zero before and after state. It was created to make it easire
/// to track before/after file sizes from minification-type tasks, but it
/// doesn't ascribe any particular meaning to the data it holds.
///
/// ## Examples
///
/// Usage is as simple as:
///
/// ```no_run
/// use fyi_msg::BeforeAfter;
///
/// let mut ba = BeforeAfter::start(123_u64);
///
/// // Do some stuff.
///
/// ba.stop(50_u64);
/// ```
///
/// Once before and after are set, you can use the getter methods [`BeforeAfter::before`]
/// and [`BeforeAfter::after`] to obtain the values.
///
/// For relative changes where `after` is expected to be smaller than `before`,
/// there is [`BeforeAfter::less`] and [`BeforeAfter::less_percent`] to obtain
/// the relative difference.
///
/// For cases where `after` is expected to be larger, use [`BeforeAfter::more`]
/// and [`BeforeAfter::more_percent`] instead.
pub struct BeforeAfter {
	before: Option<NonZeroU64>,
	after: Option<NonZeroU64>,
}

impl From<(u64, u64)> for BeforeAfter {
	fn from(src: (u64, u64)) -> Self {
		Self {
			before: NonZeroU64::new(src.0),
			after: NonZeroU64::new(src.1),
		}
	}
}

impl BeforeAfter {
	#[must_use]
	#[inline]
	/// # New Instance: Set Before.
	///
	/// This creates a new instance with the defined starting point.
	///
	/// A `before` value of `0_u64` is equivalent to `None`. The instance will
	/// still be created, but the difference methods won't return any values.
	pub const fn start(before: u64) -> Self {
		Self {
			before: NonZeroU64::new(before),
			after: None,
		}
	}

	#[inline]
	/// # Finish Instance: Set After.
	///
	/// This sets the `after` value of an existing instance, closing it out.
	///
	/// An `after` value of `0_u64` is equivalent to `None`, meaning the
	/// difference methods won't return any values.
	pub fn stop(&mut self, after: u64) {
		self.after = NonZeroU64::new(after);
	}

	#[must_use]
	#[inline]
	/// # Get Before.
	///
	/// Return the `before` value if non-zero, otherwise `None`.
	pub const fn before(&self) -> Option<NonZeroU64> { self.before }

	#[must_use]
	#[inline]
	/// # Get After.
	///
	/// Return the `after` value if non-zero, otherwise `None`.
	pub const fn after(&self) -> Option<NonZeroU64> { self.after }

	#[must_use]
	/// # Get Difference (After < Before).
	///
	/// If the after state is expected to be smaller than the before state,
	/// return the difference. If either state is unset/zero, or after is
	/// larger, `None` is returned.
	pub fn less(&self) -> Option<NonZeroU64> {
		let b: u64 = self.before?.get();
		let a: u64 = self.after?.get();

		NonZeroU64::new(b.saturating_sub(a))
	}

	#[must_use]
	/// # Percentage Difference (After < Before).
	///
	/// This is the same as [`BeforeAfter::less`], but returns a percentage of
	/// the difference over `before`.
	pub fn less_percent(&self) -> Option<f64> {
		self.less().and_then(|l| dactyl::int_div_float(l.get(), self.before?.get()))
	}

	#[must_use]
	/// # Get Difference (After > Before).
	///
	/// If the after state is expected to be larger than the before state,
	/// return the difference. If either state is unset/zero, or after is
	/// smaller, `None` is returned.
	pub fn more(&self) -> Option<NonZeroU64> {
		let b: u64 = self.before?.get();
		let a: u64 = self.after?.get();

		NonZeroU64::new(a.saturating_sub(b))
	}

	#[must_use]
	/// # Percentage Difference (After > Before).
	///
	/// This is the same as [`BeforeAfter::more`], but returns a percentage of
	/// the difference over `before`.
	pub fn more_percent(&self) -> Option<f64> {
		self.more().and_then(|m| dactyl::int_div_float(m.get(), self.before?.get()))
	}
}



#[must_use]
#[inline]
/// # `AHash` Byte Hash.
///
/// This is a convenience method for quickly hashing bytes using the
/// [`AHash`](https://crates.io/crates/ahash) crate. Check out that project's
/// home page for more details. Otherwise, TL;DR it is very fast.
fn hash64(src: &[u8]) -> u64 {
	let mut hasher = ahash::AHasher::new_with_keys(1319, 2371);
	hasher.write(src);
	hasher.finish()
}

#[must_use]
#[inline]
/// # Term Width.
///
/// This is a simple wrapper around [`term_size::dimensions`] to provide
/// the current terminal column width. We don't have any use for height,
/// so that property is ignored.
///
/// Note: The width returned will be `1` less than the actual value to mitigate
/// any whitespace weirdness that might be lurking at the edge.
fn term_width() -> u8 {
	term_size::dimensions().map_or(
		0,
		|(w, _)| u8::saturating_from(w.saturating_sub(1))
	)
}
