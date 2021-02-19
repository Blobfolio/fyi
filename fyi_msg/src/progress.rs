/*!
# FYI Msg - Progless

[`Progless`] is a simple, thread-safe CLI progress bar that can be used to
entertain users while long jobs are running.

There are two main ways to use it: manually or steady-ticked.

Manual in this case means your code says when to increment the "done" count,
and when to "tick" (possibly render output). This works for both single- and
multi-threaded tasks like so:

```no_run
use fyi_msg::Progless;

// Initialize with a `u32` total. Note, this variable does not need to be
// mutable.
let pbar = Progless::new(1001_u32);

// Iterate your taskwork or whatever.
for i in 0..1001 {
    // Do some work.
    // ...

    // Increment the done count.
    pbar.increment();

    // Call "tick" to render the change(s), if any.
    pbar.tick();
}

// Close it off and receive the elapsed time as a [`dactyl::NiceElapsed`],
// which provides both [`dactyl::NiceElapsed::as_bytes`] and [`dactyl::NiceElapsed::as_str`]
// methods for whatever you may want to do with it.
let elapsed = pbar.finish();
```

Manual ticking is fine, but if tasks take a long time to complete, particularly
in serial iterators, the elapsed time may appear frozen. To fix that and remove
the need to tick yourself, you can use the steady-tick version:

```no_run
use fyi_msg::Progless;

// Same as before, but using the "steady()" method.
let pbar = Progless::steady(1001_u32);

// Iterate your taskwork or whatever.
for i in 0..1001 {
    // Do some work.
    // ...

    // You still need to increment the done count when you've finished a cycle.
    pbar.increment();
}

// And again, same as before.
let elapsed = pbar.finish();
```

[`Progless`] is thread-safe so can be called from parallel iterators like those
from [`rayon`](https://crates.io/crates/rayon) without any special fuss.

When doing parallel work, many tasks might be "in progress" simultaneously. To
that end, you may wish to use the [`Progless::add`] and [`Progless::remove`]
methods at the start and end of each iteration instead of manually incrementing
the counts.

Doing this, a list of active tasks will be maintained and printed along with
the progress. Removing a task automatically increments the done count.

```no_run
use fyi_msg::Progless;
use rayon::prelude::*;

// Same as before, but using the "steady()" method.
let pbar = Progless::steady(1001_u32);

// Iterate.
for i in (0..1001).par_iter() {
    let task: String = format!("Task #{}.", i);
    pbar.add(&task);

    // Do some work.

    pbar.remove(&task);
}

let elapsed = pbar.finish();
```

*/

use ahash::AHashSet;
use crate::{
	Msg,
	MsgBuffer,
	BUFFER8,
};
use dactyl::{
	NiceElapsed,
	NicePercent,
	NiceU32,
	write_time,
};
use std::{
	cmp::Ordering,
	sync::{
		Arc,
		Mutex,
		atomic::{
			AtomicBool,
			AtomicU16,
			AtomicU32,
			AtomicU64,
			AtomicU8,
			Ordering::SeqCst,
		},
	},
	thread::JoinHandle,
	time::{
		Instant,
		Duration,
	},
};



/// # (Not) Random State.
///
/// Using a fixed seed value for `AHashSet` drops a few dependencies and
/// stops Valgrind from complaining about 64 lingering bytes from the runtime
/// static that would be used otherwise.
///
/// For our purposes, the variability of truly random keys isn't really needed.
const AHASH_STATE: ahash::RandomState = ahash::RandomState::with_seeds(13, 19, 23, 71);



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
const MIN_BARS_WIDTH: u32 = 10;
const MIN_DRAW_WIDTH: u32 = 40;



/// # Helper: Mutex Unlock.
///
/// This just moves tedious code out of the way.
macro_rules! mutex_ptr {
	($mutex:expr) => (
		$mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
	);
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

	last_hash: AtomicU64,
	last_lines: AtomicU16,
	last_time: Mutex<Instant>,
	last_width: AtomicU32,

	started: Mutex<Instant>,
	title: Mutex<Msg>,
	elapsed: AtomicU32,
	done: AtomicU32,
	doing: Mutex<AHashSet<Msg>>,
	total: AtomicU32,
}

impl Default for ProglessInner {
	fn default() -> Self {
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
			flags: AtomicU8::new(0),

			last_hash: AtomicU64::new(0),
			last_lines: AtomicU16::new(0),
			last_time: Mutex::new(Instant::now()),
			last_width: AtomicU32::new(0),

			started: Mutex::new(Instant::now()),
			title: Mutex::new(Msg::default()),
			elapsed: AtomicU32::new(0),
			done: AtomicU32::new(0),
			doing: Mutex::new(AHashSet::with_hasher(AHASH_STATE)),
			total: AtomicU32::new(0),
		}
	}
}

/// # Construction/Destruction.
impl ProglessInner {
	/// # New.
	///
	/// Create a new instance with the specified total.
	fn new(total: u32) -> Self {
		Self {
			total: AtomicU32::new(total),
			flags: AtomicU8::new(
				if total > 0 { TICK_NEW }
				else { 0 }
			),
			..Self::default()
		}
	}

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
				mutex_ptr!(self.started).elapsed().as_secs() as u32,
				SeqCst
			);
			mutex_ptr!(self.doing).clear();
			self.print_blank();
		}
	}
}

/// # Getters.
impl ProglessInner {
	/// # Done.
	///
	/// The number of completed tasks.
	fn done(&self) -> u32 { self.done.load(SeqCst) }

	/// # Elapsed.
	///
	/// The elapsed time, in seconds, as it was when last updated. Dynamic
	/// calculations just look at `started` directly.
	fn elapsed(&self) -> u32 { self.elapsed.load(SeqCst) }

	/// # Last Width.
	///
	/// The CLI screen width as it was when last checked. If this value
	/// happens to change between ticks, it will force redraw the content to
	/// make sure it fits correctly.
	fn last_width(&self) -> u32 { self.last_width.load(SeqCst) }

	/// # Percent.
	///
	/// Return the value of `done / total`. The value will always be between
	/// `0.0..=1.0`.
	fn percent(&self) -> f32 {
		let done = self.done.load(SeqCst);
		let total = self.total.load(SeqCst);

		if total == 0 || done == 0 { 0.0 }
		else if done == total { 1.0 }
		else {
			done as f32 / total as f32
		}
	}

	/// # Is Ticking.
	///
	/// This is `true` so long as `done` does not equal `total`, and `total`
	/// is greater than `0`. Otherwise it is `false`.
	///
	/// For the most part, this struct's setter methods only work while
	/// progress is happening; after that they're frozen.
	fn running(&self) -> bool {
		0 != self.flags.load(SeqCst) & TICKING
	}

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
	/// Any `AsRef<str>` value will do. See the module documentation for
	/// example usage.
	fn add<S>(&self, txt: S)
	where S: AsRef<str> {
		if self.running() {
			if let Some(m) = task_msg(txt) {
				if mutex_ptr!(self.doing).insert(m)	{
					self.flags.fetch_or(TICK_DOING | TICK_BAR, SeqCst);
				}
			}
		}
	}

	/// # Increment Done.
	///
	/// Increase the completed count by exactly one. This is safer to use than
	/// `set_done()` in cases where multiple tasks are happening at once as it
	/// will not accidentally decrease the value, etc.
	fn increment(&self) {
		self.set_done(self.done() + 1);
	}

	/// # Remove a task.
	///
	/// This is the equal and opposite companion to `add`. Calling this will
	/// automatically increment the done count by one, so should not be used
	/// in cases where you're triggering done changes manually.
	fn remove<S>(&self, txt: S)
	where S: AsRef<str> {
		if self.running() {
			if let Some(m) = task_msg(txt) {
				if mutex_ptr!(self.doing).remove(&m)	{
					self.flags.fetch_or(TICK_DOING | TICK_BAR, SeqCst);
					self.increment();
				}
			}
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

	/// # Set Title.
	///
	/// Give the progress bar a title, which will be shown above the progress
	/// bits while progress is progressing, and removed afterward with
	/// everything else.
	fn set_title<S>(&self, title: S)
	where S: Into<Msg> {
		if self.running() {
			let title: Msg = title.into();
			let mut old_title = mutex_ptr!(self.title);
			if title != *old_title {
				if title.is_empty() {
					*old_title = title;
				}
				else {
					*old_title = title.with_newline(true);
				}

				self.flags.fetch_or(TICK_TITLE, SeqCst);
			}
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
		let buf = mutex_ptr!(self.buf);
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
		self.last_lines.store(bytecount::count(&*buf, b'\n') as u16, SeqCst);
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
						&CLS10[14..28].repeat(usize::from(last_lines) - 10),
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
		{
			let mut last_time = mutex_ptr!(self.last_time);
			if last_time.elapsed().as_millis() < 60 {
				return true;
			}
			*last_time = Instant::now();
		}

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
		if ! self.tick_set_secs() && self.flags.load(SeqCst) == TICKING {
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
	fn tick_bar_widths(&self) -> (u32, u32) {
		// The magic "11" is made up of the following hard-coded pieces:
		// 2: braces around elapsed time;
		// 2: spaces after elapsed time;
		// 1: the "/" between done and total;
		// 2: the spaces after total;
		// 2: the braces around the bar itself (should there be one);
		// 2: the spaces after the bar itself (should there be one);
		let space: u32 = 255_u32.min(self.last_width().saturating_sub({
			let buf = mutex_ptr!(self.buf);
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
			let o_done: u32 = num_integer::div_floor(done * space, total);
			(o_done, space - o_done)
		}
	}

	/// # Tick Bar.
	///
	/// This redraws the actual progress *bar* portion of the buffer, which is
	/// actually three different bars squished together: Done, Doing, and
	/// Pending.
	///
	/// The combined width of the `###` will never exceed 255, and will never
	/// be less than 10.
	fn tick_set_bar(&self) {
		static BAR: [u8; 255] = [b'#'; 255];
		static DASH: [u8; 255] = [b'-'; 255];

		if self.flag_toggle(TICK_BAR) {
			let (w_done, w_undone) = self.tick_bar_widths();

			// Update the parts!.
			let mut buf = mutex_ptr!(self.buf);
			if buf.len(PART_BAR_DONE) != w_done {
				buf.replace(PART_BAR_DONE, &BAR[0..w_done as usize]);
			}
			if buf.len(PART_BAR_UNDONE) != w_undone {
				buf.replace(PART_BAR_UNDONE, &DASH[0..w_undone as usize]);
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
			let doing = mutex_ptr!(self.doing);
			if doing.is_empty() {
				mutex_ptr!(self.buf).truncate(PART_DOING, 0);
			}
			else {
				let width: usize = self.last_width().saturating_sub(6) as usize;
				let tasks: Vec<u8> = {
					let mut v = Vec::with_capacity(256);
					v.extend_from_slice(b"\x1b[35m");
					doing.iter()
						.for_each(|x| v.extend_from_slice(&x.fitted(width)));
					v.extend_from_slice(b"\x1b[0m");
					v
				};
				mutex_ptr!(self.buf).replace(PART_DOING, &tasks);
			}
		}
	}

	/// # Tick Done.
	///
	/// This updates the "done" portion of the buffer as needed.
	fn tick_set_done(&self) {
		if self.flag_toggle(TICK_DONE) {
			mutex_ptr!(self.buf).replace(PART_DONE, &NiceU32::from(self.done()));
		}
	}

	/// # Tick Percent.
	///
	/// This updates the "percent" portion of the buffer as needed.
	fn tick_set_percent(&self) {
		if self.flag_toggle(TICK_PERCENT) {
			mutex_ptr!(self.buf).replace(PART_PERCENT, &NicePercent::from(self.percent()));
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
	fn tick_set_secs(&self) -> bool {
		let secs: u32 = mutex_ptr!(self.started).elapsed().as_secs() as u32;
		if secs == self.elapsed.swap(secs, SeqCst) { false }
		else {
			let [h, m, s] = NiceElapsed::hms(secs);
			unsafe {
				let mut buf = mutex_ptr!(self.buf);
				write_time(
					buf.as_mut_ptr().add(buf.start(PART_ELAPSED) as usize),
					h,
					m,
					s,
				);
			}

			true
		}
	}

	/// # Tick Title.
	///
	/// The title needs to be rewritten both on direct change and resolution
	/// change. Long titles are lazy-cropped as needed.
	fn tick_set_title(&self) {
		if self.flag_toggle(TICK_TITLE) {
			let title = mutex_ptr!(self.title);
			if title.is_empty() {
				mutex_ptr!(self.buf).truncate(PART_TITLE, 0);
			}
			else {
				mutex_ptr!(self.buf).replace(
					PART_TITLE,
					&title.fitted(self.last_width() as usize - 1),
				);
			}
		}
	}

	/// # Tick Total.
	///
	/// This updates the "total" portion of the buffer as needed.
	fn tick_set_total(&self) {
		if self.flag_toggle(TICK_TOTAL) {
			mutex_ptr!(self.buf).replace(PART_TOTAL, &NiceU32::from(self.total()));
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

impl ProglessSteady {
	/// # New (Enabled).
	///
	/// Spawn a steady ticker, provided there is a running progress bar.
	fn new(t_inner: Arc<ProglessInner>) -> Self {
		// The inner has to be running or else there's no point in setting this
		// up.
		if t_inner.running() {
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
		else { Self::default() }
	}

	/// # Stop.
	///
	/// Make sure the steady ticker has actually aborted. This is called
	/// automatically when [`Progless::finish`] is called.
	fn stop(&self) {
		self.enabled.store(false, SeqCst);
		if let Some(handle) = mutex_ptr!(self.ticker).take() {
			handle.join().unwrap();
		}
	}
}



#[derive(Debug, Default, Clone)]
/// # Progless.
///
/// This here is the whole point. See the module documentation for more
/// details.
pub struct Progless {
	steady: Arc<ProglessSteady>,
	inner: Arc<ProglessInner>,
}

/// # Construction/Destruction.
impl Progless {
	#[must_use]
	/// # New.
	///
	/// Create a new, manually-controlled progress bar instance. When made
	/// this way, the implementing code needs to manually call [`Progless::tick`]
	/// at regularish intervals in order for anything to actually display.
	pub fn new(total: u32) -> Self {
		Self {
			steady: Arc::new(ProglessSteady::default()),
			inner: Arc::new(ProglessInner::new(total)),
		}
	}

	#[must_use]
	/// # New Steady.
	///
	/// Create a new steady-ticking progress bar instance. When made this way,
	/// implementing code should *not* call [`Progless::tick`] manually; that
	/// will be handled automatically at regular intervals.
	pub fn steady(total: u32) -> Self {
		if total > 0 {
			let inner = Arc::new(ProglessInner::new(total));
			Self {
				steady: Arc::new(ProglessSteady::new(inner.clone())),
				inner
			}
		}
		else { Self::new(total) }
	}

	/// # With Title.
	///
	/// Add a title to the progress bar.
	pub fn with_title<S>(self, title: S) -> Self
	where S: Into<Msg> {
		self.inner.set_title(title);
		self
	}

	#[must_use]
	/// # Stop.
	///
	/// Finish the progress bar, shut down the steady ticker (if any), and
	/// return the final elapsed count as a [`dactyl::NiceElapsed`]. Do with
	/// it what you will.
	///
	/// Calling this method will also erase any previously-printed progress
	/// information from the CLI screen.
	pub fn finish(&self) -> NiceElapsed {
		self.inner.stop();
		self.steady.stop();
		NiceElapsed::from(self.inner.elapsed())
	}
}

/// # Passthrough Setters.
impl Progless {
	/// # Add a task.
	///
	/// The progress bar can optionally keep track of tasks that are actively
	/// "in progress", which can be particularly useful when operating in
	/// parallel.
	///
	/// Any `AsRef<str>` value will do. See the module documentation for
	/// example usage.
	pub fn add<S>(&self, txt: S)
	where S: AsRef<str> {
		self.inner.add(txt);
	}

	/// # Increment Done.
	///
	/// Increase the completed count by exactly one. This is safer to use than
	/// `set_done()` in cases where multiple tasks are happening at once as it
	/// will not accidentally decrease the value, etc.
	pub fn increment(&self) {
		self.inner.increment();
	}

	/// # Remove a task.
	///
	/// This is the equal and opposite companion to [`Progless::add`]. Calling this
	/// will automatically increment the done count by one, so should not be used
	/// in cases where you're triggering done changes manually.
	pub fn remove<S>(&self, txt: S)
	where S: AsRef<str> {
		self.inner.remove(txt);
	}

	/// # Set Done.
	///
	/// Set the done count to a specific value. Be careful in cases where
	/// things are happening in parallel; in such cases `increment` is probably
	/// better.
	pub fn set_done(&self, done: u32) {
		self.inner.set_done(done);
	}

	/// # Set Title.
	///
	/// Give the progress bar a title, which will be shown above the progress
	/// bits while progress is progressing, and removed afterward with
	/// everything else.
	pub fn set_title<S>(&self, title: S)
	where S: Into<Msg> {
		self.inner.set_title(title);
	}

	/// # Tick.
	///
	/// Manually trigger a tick, which will paint any progress updates to
	/// `STDERR` if the progress bar is running.
	pub fn tick(&self) {
		self.inner.tick();
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
	use std::hash::Hasher;

	let mut hasher = ahash::AHasher::new_with_keys(1319, 2371);
	hasher.write(src);
	hasher.finish()
}

#[inline]
/// # Format Task Into Message.
///
/// This makes the tasks a little prettier for list-like display.
fn task_msg<S>(txt: S) -> Option<Msg>
where S: AsRef<str> {
	let txt = txt.as_ref();
	if txt.is_empty() { None }
	else {
		// This starts with a ↳.
		Some(Msg::custom_unchecked("    \u{21b3} ", txt).with_newline(true))
	}
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
fn term_width() -> u32 {
	term_size::dimensions().map_or(0, |(w, _)| (w as u32).saturating_sub(1))
}
