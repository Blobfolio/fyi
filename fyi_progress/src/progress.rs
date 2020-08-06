/*!
# FYI Progress

The `Progress` struct is a simple wrapper that prints an animated progress bar
to `Stderr` during iteration over a supported set (`Vec<&str>`, `Vec<String>`,
or `Vec<PathBuf>`).

It is thread-safe and applies the specified callback against each entry in
parallel.

It is relatively lightweight compared to crates like `indicatif`, but this
largely due to its lack of customizability. All progress bars display elapsed
time, running counts of done over total, and a percentage. If space allows, an
ASCII art progress bar is also printed. An optional title, if present, is
printed before the main progress line, and if executing tasks in parallel, a
list of the currently running tasks is printed after the main line.

## Examples:

```no_run
use fyi_msg::MsgKind;
use fyi_progress::Progress;
use fyi_progress::ProgressParallelism;

// An example callback function.
fn tally_food(food: &str) { ... }

let p1 = Progress::from(vec!["Apples", "Bananas", "Carrots"]);
p1.run(|f| tally_food(f));
p1.print_summary();

// Progress bars can also have a title:
let p2 = Progress::new(
    vec!["Apples", "Bananas", "Carrots"],
    MsgKind::Info.into_msg("Checking out the produce."),
);

// Parallelism can be disabled or modified:
let p3 = Progress::from(vec!["Apples", "Bananas", "Carrots"])
    .with_threads(ProgressParallelism::Heavy);

// If you want to iterate a collection *without* a progress bar, but using the
// built-in parallelism, you could use this method instead of `run()`:
p3.silent(|f| tally_food(f));
```
*/

use ahash::AHasher;
use crate::{
	NiceElapsed,
	NiceInt,
	traits::{
		FittedRange,
		FittedRangeMut,
	},
	utility,
};
use fyi_msg::{
	Msg,
	MsgKind,
	utility::{
		grow_buffer_mid,
		time_format_dd,
	},
};
use rayon::prelude::*;
use std::{
	cmp::{
		Ordering,
		PartialEq,
	},
	ffi::OsStr,
	hash::{
		Hash,
		Hasher,
	},
	io::{
		self,
		Write,
	},
	ops::{
		Deref,
		Range,
	},
	path::PathBuf,
	sync::{
		Arc,
		Mutex,
	},
	thread,
	time::{
		Duration,
		Instant,
	},
};



/// Helper: Unlock the inner Mutex, handling poisonings inasmuch as is
/// possible.
macro_rules! mutex_ptr {
	($mutex:expr) => (
		match $mutex.lock() {
			Ok(guard) => guard,
			Err(poisoned) => poisoned.into_inner(),
		}
	);
}



/// Progress Bar Flags.
///
/// Rather than rewrite the buffer on each value change, change states are
/// tracked with these flags. If a flag is on during tick time, then the
/// corresponding buffer is updated.
const FLAG_ALL: u8 =          0b0111_1111;
const FLAG_RESIZED: u8 =      0b0001_0011;
const FLAG_START_FROM: u8 =   0b0110_0001;
const FLAG_START_NEW: u8 =    0b0111_0001;

const FLAG_RUNNING: u8 =      0b0100_0000;

const FLAG_TICK_BAR: u8 =     0b0000_0001;
const FLAG_TICK_DOING: u8 =   0b0000_0010;
const FLAG_TICK_DONE: u8 =    0b0000_0100;
const FLAG_TICK_PERCENT: u8 = 0b0000_1000;
const FLAG_TICK_TITLE: u8 =   0b0001_0000;
const FLAG_TICK_TOTAL: u8 =   0b0010_0000;

/// Buffer Indexes.
///
/// The `ProgressBuffer` stores the entire output as a single byte stream. The
/// start and end points of the malleable components are stored separately to
/// make it easier to surgically modify the buffer. The indexes of those parts
/// are as follows.
const PART_TITLE: usize = 0;
const PART_ELAPSED: usize = 1;
const PART_BARS: usize = 2;
const PART_DONE: usize = 3;
const PART_TOTAL: usize = 4;
const PART_PERCENT: usize = 5;
const PART_DOING: usize = 6;



#[derive(Debug, Clone, Copy, Default, Hash, PartialEq)]
/// Progress Buffer Range.
///
/// This is essentially a copyable `Range<usize>`, used to store the
/// (inclusive) start and (exclusive) end points of malleable buffer parts like
/// the title and elapsed times.
struct ProgressBufferRange {
	/// The start index, inclusive.
	pub start: usize,
	/// The end index, exclusive.
	pub end: usize,
}

impl ProgressBufferRange {
	/// New.
	///
	/// Create a new range from `start` to `end`.
	///
	/// Note: this method is `const` and therefore cannot explicitly assert,
	/// however `start` must be less than or equal to `end`. The struct is
	/// private, so this is more a Note-to-Self than anything.
	pub const fn new(start: usize, end: usize) -> Self {
		Self {
			start,
			end,
		}
	}

	/// Range.
	pub const fn as_range(&self) -> Range<usize> {
		Range {
			start: self.start,
			end: self.end,
		}
	}

	/// Decrement.
	///
	/// Decrease both `start` and `end` by `adj`.
	pub fn decrement(&mut self, adj: usize) {
		self.start -= adj;
		self.end -= adj;
	}

	/// Increment.
	///
	/// Increase both `start` and `end` by `adj`.
	pub fn increment(&mut self, adj: usize) {
		self.start += adj;
		self.end += adj;
	}

	/// Grow.
	///
	/// Increase `end` by `adj`.
	pub fn grow(&mut self, adj: usize) {
		self.end += adj;
	}

	/// Is Empty.
	///
	/// Returns true if the range is empty.
	pub const fn is_empty(&self) -> bool {
		self.end == self.start
	}

	/// Length.
	///
	/// Returns the length of the range.
	pub const fn len(&self) -> usize {
		self.end - self.start
	}

	/// Shrink.
	///
	/// Decrease `end` by `adj`.
	pub fn shrink(&mut self, adj: usize) {
		self.end -= adj;
	}
}



#[derive(Debug, Clone)]
/// Progress Buffer.
///
/// To cut down on rewrites and allocations, the components of the progress bar
/// are stored in a single `Vec<u8>` buffer. A partition table keeps track of
/// the start end end bounds of the malleable bits so that they can be updated
/// in-place.
struct ProgressBuffer {
	buf: Vec<u8>,
	parts: [ProgressBufferRange; 7],
}

impl Default for ProgressBuffer {
	fn default() -> Self {
		Self {
			buf: vec![
			//  Title would go here.

			//  \e   [   2    m   [   \e  [   0   ;   1    m
				27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 109,
			//   0   0   :   0   0   :   0   0
				48, 48, 58, 48, 48, 58, 48, 48,
			//  \e   [   0   ;   2    m   ]  \e   [   0    m   •   •
				27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32,

			//  Bar would go here.

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
			parts: [
				ProgressBufferRange::new(0, 0),   // Title.
				ProgressBufferRange::new(11, 19), // Elapsed.
				ProgressBufferRange::new(32, 32), // Bar(s).
				ProgressBufferRange::new(39, 40), // Done.
				ProgressBufferRange::new(56, 57), // Total.
				ProgressBufferRange::new(65, 70), // Percent.
				ProgressBufferRange::new(75, 75), // Current Tasks.
			],
		}
	}
}

impl Deref for ProgressBuffer {
	type Target = [u8];

	/// Deref to Slice.
	fn deref(&self) -> &Self::Target { self.as_bytes() }
}

impl Hash for ProgressBuffer {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.buf.hash(state);
	}
}

impl ProgressBuffer {
	/// As Bytes.
	pub fn as_bytes(&self) -> &[u8] { &self.buf }

	/// Get an `AHash`.
	pub fn calculate_hash(&self) -> u64 {
		let mut hasher = AHasher::default();
		self.hash(&mut hasher);
		hasher.finish()
	}

	/// Part Length.
	///
	/// Get the length of a given part.
	///
	/// Panics if `idx` is out of range.
	pub const fn part_len(&self, idx: usize) -> usize {
		self.parts[idx].len()
	}

	/// Part Mut.
	///
	/// Return a mutable buffer slice corresponding to a given part.
	///
	/// Panics if `idx` is out of range.
	pub fn part_mut(&mut self, idx: usize) -> &mut [u8] {
		assert!(idx < 7);

		let r = self.parts[idx].as_range();
		&mut self.buf[r]
	}

	/// Replace Part.
	///
	/// Replace a part. The new value can be of any size, including no size.
	/// The buffer and partition table will be updated accordingly.
	///
	/// Panics if `idx` is out of range.
	pub fn replace_part(&mut self, idx: usize, buf: &[u8]) {
		self.resize_part(idx, buf.len());
		if ! buf.is_empty() {
			self.part_mut(idx).copy_from_slice(buf);
		}
	}

	#[allow(clippy::comparison_chain)] // We're only matching two arms.
	/// Resize Part.
	///
	/// This is a helper method to resize the part (without writing any
	/// particular data to it).
	///
	/// If the new length is shorter, the part will be shrunk; if it is bigger,
	/// it will be expanded. When expanding from the middle, data at the split
	/// is copied to the right. No guarantees are made about the content in the
	/// newly created space. It might reflect old values or zeroes. Either way
	/// it will need to be written to after this action.
	fn resize_part(&mut self, idx: usize, len: usize) {
		let old_len: usize = self.parts[idx].len();

		// Shrink it.
		if len < old_len {
			self.resize_part_shrink(idx, old_len - len);
		}
		// Grow it.
		else if old_len < len {
			self.resize_part_grow(idx, len - old_len);
		}
	}

	/// Resize Part: Grow.
	fn resize_part_grow(&mut self, idx: usize, adj: usize) {
		// Add extra bytes to the end.
		grow_buffer_mid(&mut self.buf, self.parts[idx].end, adj);

		// Adjust the parts table.
		self.parts[idx].grow(adj);
		self.parts.iter_mut()
			.skip(idx + 1)
			.for_each(|x| x.increment(adj));
	}

	/// Resize Part: Shrink.
	fn resize_part_shrink(&mut self, idx: usize, adj: usize) {
		// Truncate the end.
		if idx ==  PART_DOING {
			self.buf.truncate(self.buf.len() - adj);
		}
		// Snip out the range.
		else {
			self.buf.drain(self.parts[idx].end - adj..self.parts[idx].end);
		}

		// Adjust the parts table.
		self.parts[idx].shrink(adj);
		self.parts.iter_mut()
			.skip(idx + 1)
			.for_each(|x| x.decrement(adj));
	}

	/// Write Bars.
	///
	/// Update the bar component of the buffer. There are actually three bars:
	/// completed tasks; in-progress tasks; and remaining tasks. Each is
	/// represented by a width relative to the total number of tasks, and
	/// color-coded for beauty.
	pub fn write_bars(&mut self, done: usize, doing: usize, undone: usize) {
		static BAR: &[u8; 255] = &[b'#'; 255];

		// No bar.
		if done + doing + undone == 0 {
			self.resize_part(PART_BARS, 0);
		}
		// Some bar.
		else {
			self.replace_part(PART_BARS, &[
				b"\x1b[2m[\x1b[0;1;96m",
				&BAR[0..done],
				b"\x1b[0;1;95m",
				&BAR[0..doing],
				b"\x1b[0;1;34m",
				&BAR[0..undone],
				b"\x1b[0;2m]\x1b[0m  ",
			].concat());
		}
	}

	/// Write Doing.
	///
	/// Update the in-progress task portion of the buffer, which prints after
	/// the main line.
	///
	/// Tasks are not outputted when running in single-threaded mode as that
	/// would require ticking twice per run, which is a bit much.
	pub fn write_doing<T> (&mut self, doing: &[T], width: usize)
	where T: ProgressTask + PartialEq + Clone {
		if doing.is_empty() {
			self.resize_part(PART_DOING, 0);
		}
		else {
			self.replace_part(
				PART_DOING,
				&doing.iter()
					.flat_map(|x| x.task_line(width))
					.collect::<Vec<u8>>()
			);
		}
	}

	/// Write Done.
	///
	/// Update the "done" portion of the buffer. This is just a number, but
	/// optimized table-based byte conversions are done instead of calling
	/// `to_string()`.
	pub fn write_done(&mut self, done: u32) {
		self.replace_part(PART_DONE, &*NiceInt::from(done));
	}

	/// Write Elapsed.
	///
	/// Update the elapsed time portion of the buffer. This is displayed in
	/// `hh:mm:ss` format. It is assumed no single progress will be running for
	/// longer than a day, but if that should happen, the value will remain
	/// fixed at `23:59:59`.
	pub fn write_elapsed(&mut self, secs: u32) {
		// The value is capped at 86400, i.e. one day.
		if secs == 86400 {
			self.part_mut(PART_ELAPSED).copy_from_slice(b"23:59:59");
		}
		// For everything else, we need to parse it into bigger units.
		else {
			let c = utility::secs_chunks(secs);
			let buf = self.part_mut(PART_ELAPSED);
			buf[..2].copy_from_slice(time_format_dd(c[0]));
			buf[3..5].copy_from_slice(time_format_dd(c[1]));
			buf[6..].copy_from_slice(time_format_dd(c[2]));
		}
	}

	/// Write Percent.
	///
	/// Update the "percent" portion of the buffer. Values are displayed to two
	/// decimal places.
	pub fn write_percent(&mut self, percent: f64) {
		self.replace_part(
			PART_PERCENT,
			format!("{:>3.*}%", 2, percent * 100.0).as_bytes()
		);
	}

	/// Write Title.
	///
	/// Update the "title" portion of the buffer, which, if present, is printed
	/// before the main line.
	pub fn write_title(&mut self, title: Option<&Msg>, width: usize) {
		match title {
			Some(m) => {
				let mut m = m.to_vec();
				m.fit_to_range(width - 1);
				m.push(b'\n');

				// Write it!
				self.replace_part(PART_TITLE, &m);
			},
			None =>
				if ! self.parts[PART_TITLE].is_empty() {
					self.resize_part(PART_TITLE, 0);
				},
		}
	}

	/// Write Total.
	///
	/// Update the "total" portion of the buffer. This is just a number, but
	/// optimized table-based byte conversions are done instead of calling
	/// `to_string()`.
	pub fn write_total(&mut self, total: u32) {
		self.replace_part(PART_TOTAL, &*NiceInt::from(total));
	}
}



#[derive(Debug, Clone)]
/// Inner Progress.
///
/// This struct holds the "stateful" data for a `Progress`. This includes all
/// of the progress-related values, the buffer, and information about the last
/// print job.
///
/// None of these values are themselves thread-safe, but the `Progress` struct
/// holds an `Arc<Mutex<ProgressInner>>`, which serves the same purpose with
/// less overhead.
struct ProgressInner<T>
where T: ProgressTask + PartialEq + Clone {
	/// Current tasks.
	doing: Vec<T>,
	/// Amount complete.
	done: u32,
	/// Total amount.
	total: u32,
	/// The initiation time.
	time: Instant,
	/// Progress bar title.
	title: Option<Msg>,
	/// Formatted progress bar components.
	///
	/// Each section of the progress bar is stored in its own array slot where
	/// it can be edited independently of the others. Printing still requires
	/// concatenation, but this lets us rest halfway.
	buf: ProgressBuffer,
	/// Flags.
	///
	/// These flags mostly track state changes by field since the last tick.
	/// This way, the buffer need not be rewritten on each individual update,
	/// but once — for changed fields only — during the global `tick()` call.
	flags: u8,
	/// Hash.
	///
	/// This is a hash of the last buffer sent for print, allowing us to avoid
	/// duplicate consecutive print jobs.
	last_hash: u64,
	/// Lines Printed.
	///
	/// This keeps track of the number of lines in the last print job so that
	/// as many lines can be erased before starting the next print job.
	last_lines: usize,
	/// Elapsed Time.
	///
	/// This stores the number of seconds elapsed at the time of the last
	/// print. This is compared against the current elapsed seconds from `time`
	/// to see if the buffer requires an update.
	last_secs: u32,
	/// Screen Width.
	///
	/// This keeps track of the terminal width from the last print as a change
	/// could require redrawing portions of the progress bar whose values may
	/// remain the same.
	last_width: usize,
}

impl<T> Default for ProgressInner<T>
where T: ProgressTask + PartialEq + Clone {
	/// Default.
	///
	/// The default is empty with a few of the constant (formatting-related)
	/// pieces pre-entered.
	fn default() -> Self {
		Self {
			doing: Vec::new(),
			done: 0,
			total: 0,
			time: Instant::now(),
			title: None,
			buf: ProgressBuffer::default(),
			flags: 0,
			last_hash: 0,
			last_lines: 0,
			last_secs: 0,
			last_width: 0,
		}
	}
}

impl<T> ProgressInner<T>
where T: ProgressTask + PartialEq + Clone {
	// ------------------------------------------------------------------------
	// Getters
	// ------------------------------------------------------------------------

	/// Number Done.
	pub fn done(&self) -> u32 { self.done }

	/// Elapsed (Seconds).
	pub fn elapsed(&self) -> u32 { 86400.min(self.time.elapsed().as_secs() as u32) }

	/// Percent Done.
	pub fn percent(&self) -> f64 {
		if self.total == 0 || self.done == 0 { 0.0 }
		else if self.done == self.total { 1.0 }
		else { f64::from(self.done) / f64::from(self.total) }
	}

	/// Total.
	pub fn total(&self) -> u32 { self.total }



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	/// Increment Done.
	///
	/// Increase the number of tasks completed by one.
	pub fn increment(&mut self) {
		self.set_done(self.done + 1);
	}

	/// Remove A Task.
	///
	/// Remove a task from the currently-running list, and increment `done` by
	/// one.
	pub fn remove_doing(&mut self, task: &T) {
		if let Some(idx) = self.doing.iter().position(|x| x == task) {
			self.flags |= FLAG_TICK_DOING | FLAG_TICK_BAR;
			self.doing.remove(idx);
			self.increment();
		}
	}

	/// Add A Task.
	///
	/// A new task to the currently-running list.
	pub fn set_doing(&mut self, task: T) {
		if ! self.doing.contains(&task) {
			self.flags |= FLAG_TICK_DOING | FLAG_TICK_BAR;
			self.doing.push(task);
		}
	}

	/// Set Done.
	///
	/// Update the amount done. Once this value reaches or exceeds the total,
	/// the progress bar is stopped.
	fn set_done(&mut self, done: u32) {
		let done: u32 = self.total.min(done);
		if done != self.done {
			// We're done!
			if done == self.total {
				self.stop();
			}
			// We just moved a bit.
			else {
				self.flags |= FLAG_TICK_DONE | FLAG_TICK_PERCENT | FLAG_TICK_BAR;
				self.done = done;
			}
		}
	}

	/// Update the title.
	pub fn set_title(&mut self, title: Option<Msg>) {
		if self.title != title {
			self.flags |= FLAG_TICK_TITLE;
			self.title = title;
		}
	}

	/// Stop Progress.
	///
	/// This operation disables the running state. More specifically, it will
	/// reset the flags, set `done` equal to `total`, clear the title and
	/// tasks, and erase any output from the screen.
	pub fn stop(&mut self) {
		self.flags = FLAG_ALL & ! FLAG_RUNNING;
		self.done = self.total;
		self.doing.truncate(0);
		self.title = None;

		self.print_blank();
	}



	// ------------------------------------------------------------------------
	// Tick
	// ------------------------------------------------------------------------

	/// Tick.
	///
	/// This method will rewrite and print the buffer (e.g. progress bar) if
	/// any of that data changed since the last call.
	///
	/// If the progress bar is inactive, no action is taken.
	///
	/// This method returns `true` if a print was at least considered, `false`
	/// if the instance is inactive.
	pub fn tick(&mut self) -> bool {
		// We aren't running!
		if 0 == self.flags & FLAG_RUNNING {
			return false;
		}

		// Check the current terminal width first as that affects a lot of what
		// follows.
		self.tick_set_width();

		// We can't really draw anything meaningful in small spaces.
		if self.last_width < 40 {
			self.flags = FLAG_RUNNING;
			self.print_blank();
			return true;
		}

		// If the time hasn't changed, and nothing else has changed, we can
		// abort without all the tedious checking.
		if ! self.tick_set_secs() && self.flags == FLAG_RUNNING {
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

	/// Tick Bar Dimensions.
	///
	/// This calculates the available widths for each of the three progress
	/// bars (done, doing, remaining).
	///
	/// If the total available space winds up being less than 10, all three
	/// values are set to zero, indicating this component should be removed.
	fn tick_bar_widths(&self) -> (usize, usize, usize) {
		// The magic "11" is made up of the following hard-coded pieces:
		// 2: braces around elapsed time;
		// 2: spaces after elapsed time;
		// 1: the "/" between done and total;
		// 2: the spaces after total;
		// 2: the braces around the bar itself (should there be one);
		// 2: the spaces after the bar itself (should there be one);
		let space: usize = 255_usize.min(self.last_width.saturating_sub(
			11 +
			self.buf.part_len(PART_ELAPSED) +
			self.buf.part_len(PART_DONE) +
			self.buf.part_len(PART_TOTAL) +
			self.buf.part_len(PART_PERCENT)
		));

		// Insufficient space!
		if space < 10 || 0 == self.total { (0, 0, 0) }
		// Done!
		else if self.done == self.total { (space, 0, 0) }
		// Working on it!
		else {
			// Done and doing are both floored to prevent rounding-related
			// overflow. Any remaining space will be counted as "pending".
			let done: usize = num_integer::div_floor(
				self.done as usize * space,
				self.total as usize
			);
			let doing: usize = num_integer::div_floor(
				self.doing.len() * space,
				self.total as usize
			);
			(done, doing, space - doing - done)
		}
	}

	/// Tick Bar.
	///
	/// This redraws the actual progress *bar* portion of the buffer, which is
	/// actually three different bars squished together: Done, Doing, and
	/// Pending.
	///
	/// The combined width of the `###` will never exceed 255, and will never
	/// be less than 10.
	fn tick_set_bar(&mut self) {
		if 0 != self.flags & FLAG_TICK_BAR {
			self.flags &= ! FLAG_TICK_BAR;
			let (done, doing, undone) = self.tick_bar_widths();
			self.buf.write_bars(done, doing, undone);
		}
	}

	/// Tick Doing.
	///
	/// Update the task list portion of the buffer. This is triggered both by
	/// changes to the task list as well as resoluation changes (as long values
	/// may require lazy cropping).
	fn tick_set_doing(&mut self) {
		if 0 != self.flags & FLAG_TICK_DOING {
			self.flags &= ! FLAG_TICK_DOING;
			self.buf.write_doing(&self.doing, self.last_width);
		}
	}

	/// Tick Done.
	///
	/// This updates the "done" portion of the buffer as needed.
	fn tick_set_done(&mut self) {
		if 0 != self.flags & FLAG_TICK_DONE {
			self.flags &= ! FLAG_TICK_DONE;
			self.buf.write_done(self.done);
		}
	}

	/// Tick Percent.
	///
	/// This updates the "percent" portion of the buffer as needed.
	fn tick_set_percent(&mut self) {
		if 0 != self.flags & FLAG_TICK_PERCENT {
			self.flags &= ! FLAG_TICK_PERCENT;
			self.buf.write_percent(self.percent());
		}
	}

	/// Tick Elapsed Seconds.
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
	fn tick_set_secs(&mut self) -> bool {
		let secs: u32 = self.elapsed();
		if secs == self.last_secs { false }
		else {
			self.last_secs = secs;
			self.buf.write_elapsed(secs);
			true
		}
	}

	/// Tick Title.
	///
	/// The title needs to be rewritten both on direct change and resolution
	/// change. Long titles are lazy-cropped as needed.
	fn tick_set_title(&mut self) {
		if 0 != self.flags & FLAG_TICK_TITLE {
			self.flags &= ! FLAG_TICK_TITLE;
			self.buf.write_title(self.title.as_ref(), self.last_width);
		}
	}

	/// Tick Total.
	///
	/// This updates the "total" portion of the buffer as needed.
	fn tick_set_total(&mut self) {
		if 0 != self.flags & FLAG_TICK_TOTAL {
			self.flags &= ! FLAG_TICK_TOTAL;
			self.buf.write_total(self.total);
		}
	}

	/// Tick Width.
	///
	/// Check to see if the terminal width has changed since the last run and
	/// update values — i.e. the relevant tick flags — as necessary.
	fn tick_set_width(&mut self) {
		let width = utility::term_width();
		if width != self.last_width {
			self.flags |= FLAG_RESIZED;
			self.last_width = width;
		}
	}



	// ------------------------------------------------------------------------
	// Render
	// ------------------------------------------------------------------------

	/// Preprint.
	///
	/// This method accepts a completed buffer ready for printing, hashing it
	/// for comparison with the last job. If unique, the previous output is
	/// erased and replaced with the new output.
	fn preprint(&mut self) {
		if self.buf.is_empty() {
			self.print_blank();
			return;
		}

		// Make sure the content is unique, otherwise we can leave the old bits
		// up.
		let hash = self.buf.calculate_hash();
		if hash == self.last_hash {
			return;
		}
		self.last_hash = hash;

		// Erase old lines if needed.
		self.print_cls();

		// Update the line count and print!
		self.last_lines = 1 + bytecount::count(&self.buf, b'\n');
		Self::print(&self.buf);
	}

	/// Print Blank.
	fn print_blank(&mut self) {
		if self.last_hash != 0 {
			self.last_hash = 0;
		}

		self.print_cls();
	}

	/// Print!
	///
	/// Print some arbitrary data to the write place. Haha.
	///
	/// `Stderr` is used as the output device in production, but if the
	/// `bench_sink` feature is enabled, output will be sent to `io::sink()`
	/// instead. As the feature name suggests, this is only really useful for
	/// measuring timings.
	fn print(buf: &[u8]) {
		#[cfg(not(feature = "bench_sink"))] let writer = io::stderr();
		#[cfg(not(feature = "bench_sink"))] let mut handle = writer.lock();
		#[cfg(feature = "bench_sink")] let mut handle = io::sink();

		handle.write_all(buf).unwrap();
		handle.flush().unwrap();
	}

	/// Erase Output.
	///
	/// This method "erases" any prior output so that new output can be written
	/// in the same place. That's animation, folks!
	fn print_cls(&mut self) {
		// Buffer 10 Line Clears.
		// 0..10 moves the cursor left. This is done only once per reset.
		// 14 is the length of each subsequent command, which moves the cursor up.
		// To clear "n" lines, then, slice [0..(10 + 14 * n)].
		static CLS10: [u8; 150] = [27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75];

		if self.last_lines > 0 {
			// Figure out how to slice our `CLS10` buffer.
			match self.last_lines.cmp(&10) {
				Ordering::Equal => { Self::print(&CLS10[..]); },
				Ordering::Less => {
					let end: usize = 10 + 14 * self.last_lines;
					Self::print(&CLS10[0..end]);
				},
				// To clear more lines, print our pre-calculated buffer (which
				// covers the first 10), and duplicate the line-up chunk (n-10)
				// times to cover the rest.
				Ordering::Greater => {
					Self::print(
						&CLS10.iter()
							.chain(&CLS10[14..28].repeat(self.last_lines - 10))
							.copied()
							.collect::<Vec<u8>>()
					);
				},
			}

			// Having cleared whatever it was, there are now no last_lines.
			self.last_lines = 0;
		}
	}

	/// Print Generic Finish Message.
	///
	/// This method can be called after a progress bar has finished to
	/// summarize what went down.
	///
	/// If the total is zero, a warning is printed instead.
	pub fn print_summary<S> (&mut self, one: S, many: S)
	where S: AsRef<str> {
		if 0 == self.flags & FLAG_RUNNING {
			// Print a warning.
			if self.total == 0 {
				Msg::from([
					b"No ",
					many.as_ref().as_bytes(),
					b" were found.\n",
				].concat())
					.with_prefix(MsgKind::Warning)
			}
			else {
				Msg::from([
					&utility::inflect(u64::from(self.total), one, many),
					&b" in "[..],
					&*NiceElapsed::from(self.elapsed()),
					&[46, 10],
				].concat())
					.with_prefix(MsgKind::Done)
			}
			.eprint()
		}
	}
}



#[derive(Debug, Copy, Clone, Hash, PartialEq)]
/// Degree of Parallelism.
///
/// By default, one parallel thread will be spawned for each (reported) CPU
/// core. Depending on the types of workload, `Light` or `Heavy` parallelism
/// could be a better choice.
///
/// Parallelism can be explicitly disabled using `None`.
pub enum ProgressParallelism {
	/// One thread.
	None,
	/// Half cores.
	Light,
	/// Leave one core open.
	Reserve,
	/// Use all cores.
	Default,
	/// Double cores.
	Heavy,
}

impl Default for ProgressParallelism {
	fn default() -> Self { Self::Default }
}

impl Eq for ProgressParallelism {}

impl Ord for ProgressParallelism {
	fn cmp(&self, other: &Self) -> Ordering {
		self.threads().cmp(&other.threads())
	}
}

impl PartialOrd for ProgressParallelism {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl ProgressParallelism {
	#[must_use]
	/// Number of Threads.
	pub fn threads(self) -> usize {
		match self {
			Self::None => 1,
			Self::Light => 1.max(num_integer::div_floor(num_cpus::get(), 2)),
			Self::Reserve => 1.max(num_cpus::get() - 1),
			Self::Default => num_cpus::get(),
			Self::Heavy => num_cpus::get() * 2,
		}
	}
}



#[derive(Debug)]
/// Progress Bar.
///
/// This is it! The whole point of the crate! See the library documentation for
/// more information.
pub struct Progress<T>
where T: ProgressTask + PartialEq + Clone {
	/// The set to progress through.
	set: Vec<T>,
	/// Thread handling.
	threads: ProgressParallelism,
	/// The stateful data.
	inner: Arc<Mutex<ProgressInner<T>>>,
}

impl<T> Default for Progress<T>
where T: ProgressTask + PartialEq + Clone {
	fn default() -> Self {
		Self {
			set: Vec::new(),
			threads: ProgressParallelism::Default,
			inner: Arc::new(Mutex::new(ProgressInner::<T>::default())),
		}
	}
}

impl<T> From<Vec<T>> for Progress<T>
where T: ProgressTask + PartialEq + Clone {
	fn from(src: Vec<T>) -> Self {
		let total: u32 = src.len() as u32;
		if total == 0 {
			Self::default()
		}
		else {
			Self {
				set: src,
				threads: ProgressParallelism::default(),
				inner: Arc::new(Mutex::new(ProgressInner::<T> {
					total,
					flags: FLAG_START_FROM,
					..ProgressInner::<T>::default()
				})),
			}
		}
	}
}

impl<T> Deref for Progress<T>
where T: ProgressTask + PartialEq + Clone {
	type Target = [T];

	/// Deref to Slice.
	fn deref(&self) -> &Self::Target { &self.set }
}

impl<T> Progress<T>
where T: ProgressTask + PartialEq + Clone + Sync + Send + 'static {
	#[must_use]
	/// New Progress.
	///
	/// Start a new progress bar with a dataset and title. If you don't need a
	/// title, just use `Progress::from()` instead.
	pub fn new(src: Vec<T>, title: Msg) -> Self {
		let total: u32 = src.len() as u32;
		if total == 0 {
			Self::default()
		}
		else {
			Self {
				set: src,
				threads: ProgressParallelism::default(),
				inner: Arc::new(Mutex::new(ProgressInner::<T> {
					total,
					title: Some(title),
					flags: FLAG_START_NEW,
					..ProgressInner::<T>::default()
				})),
			}
		}
	}

	/// Set Threads.
	pub fn set_threads(&mut self, threads: ProgressParallelism) {
		self.threads = threads;
	}

	#[must_use]
	/// With Threads.
	pub fn with_threads(mut self, threads: ProgressParallelism) -> Self {
		self.threads = threads;
		self
	}

	/// Run!
	///
	/// This here is the whole point. Iterate through the set in parallel with
	/// a progress bar, while executing a custom callback on each entry.
	///
	/// When parallelism is such that only one thread is to be used, the
	/// progress portion will run without a steady tick and without displaying
	/// the current task information.
	///
	/// When parallelism is more than one thread, tasks will be executed in
	/// parallel using that many threads, plus one extra thread to steadily
	/// tick the timer, ensuring the elapsed time updates at least once per
	/// second.
	pub fn run<F>(&self, cb: F)
	where F: Fn(&T) + Copy + Send + Sync {
		if ! self.set.is_empty() {
			match self.threads.threads() {
				1 => self.run_single(cb),
				t => self.run_parallel(t + 1, cb),
			}
		}
	}

	/// Run Multi.
	fn run_parallel<F>(&self, threads: usize, cb: F)
	where F: Fn(&T) + Copy + Send + Sync {
		let pool = rayon::ThreadPoolBuilder::new()
			.num_threads(threads)
			.build()
			.unwrap();

		// This extra process gives us a steady tick, ensuring slow tasks
		// don't make the user think everything's crashed.
		let inner = self.inner.clone();
		pool.spawn(move || Self::steady_tick(&inner));

		// Iterate!
		pool.install(|| self.set.par_iter().for_each(|x| {
			// Mark the task as currently running.
			self.set_doing(x.clone());

			// Do whatever.
			cb(x);

			// Mark the task as complete.
			self.remove_doing(x);
		}));
	}

	/// Run Single.
	fn run_single<F>(&self, cb: F)
	where F: Fn(&T) + Copy + Send + Sync {
		self.set.iter().for_each(|x| {
			cb(x);
			self.increment();
			Self::tick(&self.inner);
		});
	}

	/// Silent Run.
	///
	/// This is a convenience function for looping the set *without* any
	/// progress bar output, but with the built-in parallel iterators. Being
	/// that no progress output is happening, no progress-related data changes
	/// are triggered by the run. In the end, `done` will still be `0`, etc.
	pub fn silent<F>(&self, cb: F)
	where F: Fn(&T) + Copy + Send + Sync {
		match self.threads.threads() {
			1 => { self.set.iter().for_each(cb); },
			t => {
				let pool = rayon::ThreadPoolBuilder::new()
					.num_threads(t)
					.build()
					.unwrap();

				pool.install(|| self.set.par_iter().for_each(cb));
			},
		}
	}

	/// Steady Tick.
	///
	/// This is a simple endless timer thread used to auto-tick the progress
	/// bar once every 60ms. As ticks otherwise only occur at the completion of
	/// a task, this ensures independent values like the time elapsed are
	/// repainted consistently.
	///
	/// Note: As steady ticking requires its own thread — to avoid blocking the
	/// actual taskwork! — it is only available for parallel requests. In
	/// single-threaded mode it is disabled.
	fn steady_tick(inner: &Arc<Mutex<ProgressInner<T>>>) {
		let sleep = Duration::from_millis(60);
		loop {
			if ! Self::tick(inner) {
				break;
			}
			thread::sleep(sleep);
		}
	}



	// ------------------------------------------------------------------------
	// `ProgressInner` Wrappers
	// ------------------------------------------------------------------------

	#[must_use]
	/// Get Doing.
	pub fn doing(&self) -> u32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.doing.len() as u32
	}

	#[must_use]
	/// Get Done.
	///
	/// Wrapper for `ProgressInner::done()`.
	pub fn done(&self) -> u32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.done()
	}

	#[must_use]
	/// Get Elapsed.
	///
	/// Wrapper for `ProgressInner::elapsed()`.
	pub fn elapsed(&self) -> u32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.elapsed()
	}

	/// Increment.
	///
	/// Wrapper for `ProgressInner::increment()`.
	fn increment(&self) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.increment();
	}

	#[must_use]
	/// Get Percent.
	///
	/// Wrapper for `ProgressInner::percent()`.
	pub fn percent(&self) -> f64 {
		let ptr = mutex_ptr!(self.inner);
		ptr.percent()
	}

	/// Print Finish Message.
	///
	/// Wrapper for `ProgressInner::print_summary()`.
	pub fn print_summary<S> (&self, one: S, many: S)
	where S: AsRef<str> {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.print_summary(one, many);
	}

	/// Remove a Task.
	///
	/// Wrapper for `ProgressInner::remove_doing()`.
	fn remove_doing(&self, task: &T) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.remove_doing(task);
	}

	/// Add a Task.
	///
	/// Wrapper for `ProgressInner::set_doing()`.
	fn set_doing(&self, task: T) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.set_doing(task);
	}

	/// Set Title.
	///
	/// Wrapper for `ProgressInner::set_title()`.
	pub fn set_title(&self, title: Option<Msg>) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.set_title(title);
	}

	/// Steady Tick.
	///
	/// Wrapper for `ProgressInner::tick()`.
	fn tick(inner: &Arc<Mutex<ProgressInner<T>>>) -> bool {
		let mut ptr = mutex_ptr!(inner);
		ptr.tick()
	}

	#[must_use]
	/// Get Total.
	///
	/// Wrapper for `ProgressInner::total()`.
	pub fn total(&self) -> u32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.total()
	}
}



/// Progressable Type.
///
/// Types with this trait may be used to seed a `Progress`. Right now that
/// just means strings and paths.
///
/// The only required method is `task_name()`, which must convert the value
/// into a meaningful byte slice.
///
/// Anything convertable to bytes could implement this trait; we've just
/// started with the types we're actually using. If you would like to see
/// other `std` types added, just open a request ticket.
pub trait ProgressTask {
	/// Task Name.
	fn task_name(&self) -> &[u8];

	/// The Full Task Line.
	///
	/// This combines an indented arrow with the task name, lazy chopping long
	/// values as needed.
	fn task_line(&self, width: usize) -> Vec<u8> {
		let name: &[u8] = self.task_name();

		[
			// •   •   •   •  \e   [   3   5    m   ↳  ---  ---   •
			&[32, 32, 32, 32, 27, 91, 51, 53, 109, 226, 134, 179, 32][..],
			&name[name.fitted_range(width.saturating_sub(6))],
			b"\x1b[0m\n",
		].concat()
	}
}

impl ProgressTask for PathBuf {
	#[allow(trivial_casts)]
	/// Task Name.
	fn task_name(&self) -> &[u8] {
		unsafe { &*(self.as_os_str() as *const OsStr as *const [u8]) }
	}
}

impl ProgressTask for &str {
	/// Task Name.
	fn task_name(&self) -> &[u8] { self.as_bytes() }
}

impl ProgressTask for String {
	/// Task Name.
	fn task_name(&self) -> &[u8] { self.as_bytes() }
}
