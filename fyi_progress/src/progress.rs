/*!
# FYI Progress

This is a simple, fast, thread-safe progress bar built around `fyi_msg`.

It is fast and simple because it does not allow undue user fiddling over
design, formatting, etc. It is intended for those times where an app just
needs _a normal damn progress bar_.

Which is not to say there aren't other times. For those, check out `indicatif`;
it does just about everything imaginable. Haha.

## Display:

This progress bar has three parts, two of which are optional:
1. A message (`fyi_msg::Msg`), pinned above the bar if provided.
2. The "progress" line, which includes elapsed time, an ASCII-art "bar", and human-readable progress.
3. Any number of "tasks", pinned below the bar, which can be used to indicate what a particular thread happens to be working on _right now_, etc.

#2 is always present, and the rest is there if you find it useful.

All together, it looks something like this, but in color:

```text
Optional: Message Goes Here
[00:13:25]  [==========-----] 10/15 66.6%
    ↳ Some arbitrary task (optional)
    ↳ Some other thing (optional)
```

The progress line flexes to fill the terminal width, expanding the "bar" bit as
needed. The bar has a maximum width of 255, constraining the amount of flex to
"within reason".

## Usage:

The method `tick()` is used to manually calculate and paint the current state
to the terminal. If you want to handle the timing, etc., manually, simply call
that method when appropriate, like:

```no_run
let bar = fyi_progress::Progress(None, 1000);
bar.tick();
...do some stuff...
bar.update(1, None, None);
bar.tick();
```

Alternatively, you can use the struct's `steady_tick()` method to handle the
ticking automatically, regularly in its own thread. Take a look at the included
"progress" example for an example.

## Example:

```no_run
use fyi_msg::Msg;
use fyi_progress::Progress;

// Start a bar.
let bar = Progress::new(
    Some(Msg::info("Example message/title/whatever.")),
    1000,
);
assert_eq!(0, bar.done());

bar.update(1, None, None);
assert_eq!(1, bar.done());

bar.update(10, None, None);
assert_eq!(11, bar.done());

bar.stop();
assert_eq!(1000, bar.done());
```
*/

use crate::{
	NiceElapsed,
	NiceInt,
	traits::FittedRange,
	utility::{
		secs_chunks,
		term_width,
	},
};
use fyi_msg::{
	Msg,
	MsgBuf,
	utility::time_format_dd,
};
use indexmap::set::IndexSet;
use std::{
	borrow::Borrow,
	cmp::Ordering,
	io,
	ops::Deref,
	sync::{
		Arc,
		Mutex,
	},
	thread::{
		self,
		JoinHandle,
	},
	time::{
		Duration,
		Instant,
	},
};



/// Helper: Unlock the inner Mutex, handling poisonings inasmuch as is
/// possible.
macro_rules! mutex_ptr {
	($lhs:ident) => (
		match $lhs.0.lock() {
			Ok(guard) => guard,
			Err(poisoned) => poisoned.into_inner(),
		}
	);
}

/// Helper: Push chunk to `ProgressBar`.
macro_rules! pb_push {
	($lhs:ident, $len:expr, $buf:expr) => {{
		let end: usize = $lhs.len + $len;
		$lhs.inner[$lhs.len..end].copy_from_slice($buf);
		$lhs.len = end;
	}};
}



static DONE:   &[u8; 255] = &[61; 255];
static UNDONE: &[u8; 255] = &[45; 255];

const IDX_TITLE: usize = 1;
// const IDX_ELAPSED_PRE: usize = 2;
const IDX_ELAPSED: usize = 3;
// const IDX_ELAPSED_POST: usize = 4;
const IDX_BAR: usize = 5;
// const IDX_DONE_PRE: usize = 6;
const IDX_DONE: usize = 7;
// const IDX_DONE_POST: usize = 8;
const IDX_TOTAL: usize = 9;
// const IDX_TOTAL_POST: usize = 10;
const IDX_PERCENT: usize = 11;
// const IDX_PERCENT_POST: usize = 12;
const IDX_TASKS: usize = 13;



bitflags::bitflags! {
	/// Progress Bar flags.
	///
	/// These flags describe progress bar elements that have changed since the
	/// last tick and need to be redrawn.
	///
	/// These are handled automatically.
	struct ProgressFlags: u32 {
		const NONE =         0b0000_0000;
		const ALL =          0b0111_1111;
		const PROGRESSED =   0b0000_0111;
		const RESIZED    =   0b0011_0001;

		const TICK_BAR =     0b0000_0001;
		const TICK_DONE =    0b0000_0010;
		const TICK_PERCENT = 0b0000_0100;
		const TICK_TASKS =   0b0001_0000;
		const TICK_TITLE =   0b0010_0000;
		const TICK_TOTAL =   0b0100_0000;
	}
}

impl Default for ProgressFlags {
	/// Default.
	fn default() -> Self {
		Self::NONE
	}
}



#[derive(Copy, Clone)]
/// The progress bar is big and variable compared to the rest of the
/// `ProgressInner` pieces. It is faster to handle its slicing in a specialized
/// struct with a fixed-length buffer.
struct ProgressBar {
	inner: [u8; 289],
	len: usize,
}

impl Default for ProgressBar {
	#[inline]
	fn default() -> Self {
		Self {
			inner: [0; 289],
			len: 0,
		}
	}
}

impl Deref for ProgressBar {
	type Target = [u8];

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.inner[..self.len]
	}
}

impl ProgressBar {
	/// New
	pub fn new(done: u64, total: u64, width: usize) -> Self {
		if done == 0 { Self::new_unstarted(width) }
		else if done == total { Self::new_finished(width) }
		else {
			let done_width: usize = f64::floor((done as f64 / total as f64) * width as f64) as usize;
			let undone_width: usize = width - done_width;
			let mut out = Self::default();

			// Opener.
			pb_push!(
				out, 14,
				//\e   [   2    m   [  \e   [   0   ;   9   6   ;   1    m
				&[27, 91, 50, 109, 91, 27, 91, 48, 59, 57, 54, 59, 49, 109]
			);

			// Done.
			pb_push!(out, done_width, &DONE[..done_width]);

			// The bit in between.
			pb_push!(
				out, 7,
				//\e   [   0   ;   3   4    m
				&[27, 91, 48, 59, 51, 52, 109]
			);

			// Undone.
			pb_push!(out, undone_width, &UNDONE[..undone_width]);

			// Close it.
			pb_push!(
				out, 13,
				//\e   [   0   ;   2    m   ]  \e   [   0    m   •   •
				&[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32]
			);

			out
		}
	}

	/// New, only undone.
	fn new_unstarted(width: usize) -> Self {
		let mut out = Self::default();

		// Opener.
		pb_push!(
			out, 12,
			//\e   [   2    m   [  \e   [   0   ;   3   4    m
			&[27, 91, 50, 109, 91, 27, 91, 48, 59, 51, 52, 109]
		);

		// Undone.
		pb_push!(out, width, &UNDONE[..width]);

		// Close it.
		pb_push!(
			out, 13,
			//\e   [   0   ;   2    m   ]  \e   [   0    m   •   •
			&[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32]
		);

		out
	}

	/// New, only done.
	fn new_finished(width: usize) -> Self {
		let mut out = Self::default();

		// Opener.
		pb_push!(
			out, 14,
			//\e   [   2    m   [  \e   [   0   ;   9   6   ;   1    m
			&[27, 91, 50, 109, 91, 27, 91, 48, 59, 57, 54, 59, 49, 109]
		);

		// Done.
		pb_push!(out, width, &DONE[..width]);

		// Close it.
		pb_push!(
			out, 13,
			//\e   [   0   ;   2    m   ]  \e   [   0    m   •   •
			&[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32]
		);

		out
	}
}



#[derive(Debug, Clone)]
/// Progress Bar (Internal)
///
/// The state data is maintained here. The main `Progress` object locks this
/// behind a `MutexGuard` so it can be shared across threads.
struct ProgressInner {
	/// Buffer.
	buf: MsgBuf,
	/// The title.
	title: String,
	/// The creation time.
	time: Instant,
	/// The amount "done".
	done: u64,
	/// The "total" amount.
	total: u64,
	/// Tasks, e.g. brief descriptions of what is being worked on now, optional.
	tasks: IndexSet<String>,
	/// The flags help keep track of the components that need redrawing at the
	/// next tick.
	flags: ProgressFlags,
	/// The hash of the last buffer printed. This and the other "last_" items
	/// allow for potential repaint throttling, etc.
	last_hash: u64,
	/// The number of lines that buffer had.
	last_lines: usize,
	/// The seconds elapsed at that time.
	last_secs: u32,
	/// The terminal width that existed then.
	last_width: usize,
}

impl Default for ProgressInner {
	/// Default.
	fn default() -> Self {
		Self {
			buf: <MsgBuf as From<&[&[u8]; 13]>>::from(&[
				// Title.
				&[],

				// Elapsed.
				//\e   [   2    m   [   \e  [   0   ;   1    m
				&[27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 109],
				// 0   0   :   0   0   :   0   0
				&[48, 48, 58, 48, 48, 58, 48, 48],
				//\e   [   0   ;   2    m   ]  \e   [   0    m   •   •
				&[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32],

				// Bar.
				&[],

				// Done.
				//\e   [   1   ;   9   6    m
				&[27, 91, 49, 59, 57, 54, 109],
				// 0
				&[48],

				// The slash between Done and Total.
				//\e   [   0   ;   2    m   /  \e   [   0   ;   1   ;   3   4    m
				&[27, 91, 48, 59, 50, 109, 47, 27, 91, 48, 59, 49, 59, 51, 52, 109],

				// Total.
				// 0
				&[48],

				// The bit between Total and Percent.
				//\e   [   0   ;   1    m   •   •
				&[27, 91, 48, 59, 49, 109, 32, 32],

				// Percent.
				// 0   .   0   0   %
				&[48, 46, 48, 48, 37],
				//\e   [   0    m  \n
				&[27, 91, 48, 109, 10],

				// Tasks.
				&[],
			]),
			title: String::new(),
			time: Instant::now(),
			done: 0,
			total: 0,
			tasks: IndexSet::new(),
			flags: ProgressFlags::NONE,
			last_hash: 0,
			last_lines: 0,
			last_secs: 0,
			last_width: 0,
		}
	}
}

impl ProgressInner {
	// ------------------------------------------------------------------------
	// Public Static Methods
	// ------------------------------------------------------------------------

	#[must_use]
	/// New.
	///
	/// Start a new progress bar for `total` items.
	pub fn new(total: u64) -> Self {
		if 0 == total {
			Self::default()
		}
		else {
			Self {
				buf: <MsgBuf as From<&[&[u8]; 13]>>::from(&[
					// Title.
					&[],

					// Elapsed.
					//\e   [   2    m   [   \e  [   0   ;   1    m
					&[27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 109],
					// 0   0   :   0   0   :   0   0
					&[48, 48, 58, 48, 48, 58, 48, 48],
					//\e   [   0   ;   2    m   ]  \e   [   0    m   •   •
					&[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32],

					// Bar.
					&[],

					// Done.
					//\e   [   1   ;   9   6    m
					&[27, 91, 49, 59, 57, 54, 109],
					// 0
					&[48],

					// The slash between Done and Total.
					//\e   [   0   ;   2    m   /  \e   [   0   ;   1   ;   3   4    m
					&[27, 91, 48, 59, 50, 109, 47, 27, 91, 48, 59, 49, 59, 51, 52, 109],

					// Total.
					&*NiceInt::from(total),

					// The bit between Total and Percent.
					//\e   [   0   ;   1    m   •   •
					&[27, 91, 48, 59, 49, 109, 32, 32],

					// Percent.
					// 0   .   0   0   %
					&[48, 46, 48, 48, 37],
					//\e   [   0    m  \n
					&[27, 91, 48, 109, 10],

					// Tasks.
					&[],
				]),
				total,
				flags: ProgressFlags::PROGRESSED,
				..Self::default()
			}
		}
	}



	// ------------------------------------------------------------------------
	// Public Methods
	// ------------------------------------------------------------------------

	#[must_use]
	/// Is running?
	pub const fn is_running(&self) -> bool {
		self.done < self.total
	}

	#[must_use]
	/// Percent done.
	///
	/// This is literally an expression of `done / total`, returned as a float
	/// between 0.0 and 1.0.
	pub fn percent(&self) -> f64 {
		if self.total > 0 {
			if self.total > self.done {
				self.done as f64 / self.total as f64
			}
			else { 1.0 }
		}
		else { 0.0 }
	}



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	/// Add task.
	///
	/// Tasks are an arbitrary list of strings that will print beneath the
	/// progress line. The main idea is to communicate ideas like, "Such and
	/// such is being actively worked on in Thread #2" or whatever, but there
	/// are no hard and fast rules.
	pub fn add_task<T: Borrow<str>> (&mut self, task: T) {
		if self.tasks.insert(task.borrow().into()) {
			self.flags |= ProgressFlags::TICK_TASKS;
		}
	}

	/// Increment Done.
	///
	/// Increment the "done" count by this amount. It is recommended to prefer
	/// this method over `set_done()`, which instead takes an absolute value.
	/// Particularly in multi-threaded scenarios, one cannot be certain the
	/// order in which instructions will arrive, but if both are just "add
	/// one to the current", it all works out.
	pub fn increment(&mut self, num: u64) {
		self.set_done(self.done + num);
	}

	/// Remove task.
	///
	/// If you're thinking of tasks as a list of "this is happening now" stuff,
	/// `remove_task()` is the conclusion to `add_task()`.
	pub fn remove_task<T: Borrow<str>> (&mut self, task: T) {
		if self.tasks.shift_remove(task.borrow()) {
			self.flags |= ProgressFlags::TICK_TASKS;
		}
	}

	/// Set done.
	///
	/// Set the "done" amount to this absolute value. See also `increment()`,
	/// which instead works relative to the current value.
	pub fn set_done(&mut self, done: u64) {
		if done >= self.total {
			if self.done != self.total {
				self.done = self.total;
				self.flags |= ProgressFlags::PROGRESSED;
				self.stop();
			}
		}
		else if done != self.done {
			self.done = done;
			self.flags |= ProgressFlags::PROGRESSED;
		}
	}

	/// Set msg.
	///
	/// Update or unset the message, which if present, prints pinned above the
	/// progress line.
	pub fn set_title<T: Borrow<str>> (&mut self, title: T) {
		let title = title.borrow();
		if title != self.title {
			self.title = title.to_string();
			self.flags |= ProgressFlags::TICK_TITLE;
		}
	}

	/// Set total.
	///
	/// Totals are meant to be set at instantiation, but they can be changed
	/// mid-go if you're doing something weird.
	pub fn set_total(&mut self, total: u64) {
		if total != self.total {
			self.total = total;
			self.flags |= ProgressFlags::TICK_TOTAL | ProgressFlags::PROGRESSED;

			// Total must still be bigger than done.
			if self.total < self.done {
				self.set_done(self.total);
			}
		}
	}



	// ------------------------------------------------------------------------
	// Display
	// ------------------------------------------------------------------------

	/// Clear Screen
	pub fn cls(&mut self) {
		// Buffer 10 Line Clears
		static CLS10: &[u8] = &[27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75];
		// 0..10 moves the cursor left. This is done only once per reset.
		// 14 is the length of each subsequent command, which moves the cursor up.
		// To clear "n" lines, then, slice [0..(10 + 14 * n)].

		if self.last_lines > 0 {
			match self.last_lines.cmp(&10) {
				Ordering::Equal => { Self::print(CLS10); },
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

			// Having cleared whatever it was, there were no last_lines.
			self.last_lines = 0;
		}
	}

	/// Finished In
	///
	/// This method can be used to print a quick "Finished in XYZ" summary
	/// message after progress has finished.
	pub fn finished_in(&self) {
		if ! self.is_running() {
			Self::print(&Msg::crunched(format!(
				"Finished in {}.\n",
				unsafe { std::str::from_utf8_unchecked(&*NiceElapsed::from(self.last_secs)) },
			)));
		}
	}

	/// Stop
	///
	/// Stop the progress bar, setting done to total, clearing tasks, emptying
	/// the message, and clearing the last bit that was printed to the terminal
	/// if any.
	pub fn stop(&mut self) {
		if self.done != self.total {
			self.done = self.total;
		}
		if ! self.tasks.is_empty() {
			self.tasks.clear();
		}
		self.flags = ProgressFlags::NONE;
		self.last_secs = u32::min(86400, self.time.elapsed().as_secs() as u32);
		self.cls();
	}

	/// Tick.
	///
	/// Calculate progress and print it to the terminal, if necessary.
	pub fn tick(&mut self) {
		// Easy bail.
		if ! self.is_running() { return; }

		// Some things depend on the width, which might be changing.
		let width: usize = term_width();
		if width != self.last_width {
			self.flags |= ProgressFlags::RESIZED;
			self.last_width = width;
		}

		// Update our elapsed time.
		let secs: u32 = u32::min(86400, self.time.elapsed().as_secs() as u32);
		if secs != self.last_secs {
			self.last_secs = secs;
			self.redraw_elapsed();
		}

		// Update our done amount.
		if self.flags.contains(ProgressFlags::TICK_DONE) {
			self.buf.replace_part(IDX_DONE, &*NiceInt::from(self.done));
			self.flags &= !ProgressFlags::TICK_DONE;
		}

		// Did the total change? Probably not, but just in case...
		if self.flags.contains(ProgressFlags::TICK_TOTAL) {
			self.buf.replace_part(IDX_TOTAL, &*NiceInt::from(self.total));
			self.flags &= !ProgressFlags::TICK_TOTAL;
		}

		// The percent?
		if self.flags.contains(ProgressFlags::TICK_PERCENT) {
			self.buf.replace_part(
				IDX_PERCENT,
				format!("{:>3.*}%", 2, self.percent() * 100.0).as_bytes(),
			);
			self.flags &= !ProgressFlags::TICK_PERCENT;
		}

		// The title changed?
		if self.flags.contains(ProgressFlags::TICK_TITLE) {
			self.redraw_title();
			self.flags &= !ProgressFlags::TICK_TITLE;
		}

		// Update our tasks.
		if self.flags.contains(ProgressFlags::TICK_TASKS) {
			self.redraw_tasks();
			self.flags &= !ProgressFlags::TICK_TASKS;
		}

		// The bar might independently need redrawing if the width has changed.
		if self.flags.contains(ProgressFlags::TICK_BAR) {
			self.redraw_bar();
			self.flags &= !ProgressFlags::TICK_BAR;
		}

		// Check the hash and see if we did something worth printing!
		let hash: u64 = seahash::hash(&self.buf);
		if hash == self.last_hash {
			return;
		}
		self.last_hash = hash;

		// Do we need to clear anything?
		if self.last_lines > 0 {
			self.cls();
		}

		// How many lines do we have now?
		self.last_lines = 1 + self.tasks.len();
		if ! self.title.is_empty() {
			self.last_lines += 1;
		}

		Self::print(&self.buf);
	}



	// ------------------------------------------------------------------------
	// Internal Helpers
	// ------------------------------------------------------------------------

	/// Calculate Available Bar Space
	///
	/// The bar stretches to fill the available space on the progress line,
	/// which requires we know how much displayable width the progress line
	/// has.
	///
	/// We don't want to print a bar narrower than 10 characters, and we need
	/// to reserve an additional 4 characters for the braces and spaces, so
	/// if the result is less than 14, 0 will be returned.
	fn bar_space(&self, width: usize) -> usize {
		// The magic "11" is made up of the following hard-coded pieces:
		// 2: braces around elapsed time;
		// 2: spaces after elapsed time;
		// 1: the "/" between done and total;
		// 2: the spaces after total;
		// 2: the braces around the bar itself (should there be one);
		// 2: the spaces after the bar itself (should there be one);
		let total: usize = usize::min(255, width.saturating_sub(
			11 +
			self.buf.part_len(IDX_ELAPSED) +
			self.buf.part_len(IDX_DONE) +
			self.buf.part_len(IDX_TOTAL) +
			self.buf.part_len(IDX_PERCENT)
		));

		if total >= 10 { total }
		else { 0 }
	}

	/// Print!
	///
	/// Print some arbitrary data to the write place.
	fn print(buf: &[u8]) {
		use io::Write;

		#[cfg(not(feature = "bench_sink"))] let writer = io::stderr();
		#[cfg(not(feature = "bench_sink"))] let mut handle = writer.lock();
		#[cfg(feature = "bench_sink")] let mut handle = io::sink();

		handle.write_all(buf).unwrap();
		handle.flush().unwrap();
	}

	/// Redraw Bar
	///
	/// This method updates the "bar" slice.
	fn redraw_bar(&mut self) {
		// We don't have room for it.
		let bar_len: usize = self.bar_space(self.last_width);
		if bar_len < 10 {
			self.buf.clear_part(IDX_BAR);
		}
		else {
			self.buf.replace_part(
				IDX_BAR,
				&*ProgressBar::new(self.done, self.total, bar_len)
			);
		}
	}

	/// Redraw Elapsed
	///
	/// This method updates the "elapsed" time slice.
	fn redraw_elapsed(&mut self) {
		if self.last_secs < 86400 {
			let c = secs_chunks(self.last_secs);
			let buf = &mut self.buf[IDX_ELAPSED];
			buf[..2].copy_from_slice(time_format_dd(c[0]));
			buf[3..5].copy_from_slice(time_format_dd(c[1]));
			buf[6..].copy_from_slice(time_format_dd(c[2]));
		}
		else {
			//                                       2   3   :   5   9   :   5   9
			self.buf[IDX_ELAPSED].copy_from_slice(&[50, 51, 58, 53, 57, 58, 53, 57]);
		}
	}

	/// Redraw Tasks
	///
	/// This method updates the "tasks" slice.
	fn redraw_tasks(&mut self) {
		if self.tasks.is_empty() {
			self.buf.clear_part(IDX_TASKS);
		}
		else {
			// Our formatting eats up 7 printable spaces; the task can use up
			// whatever else.
			let width: usize = self.last_width - 7;
			self.buf.replace_part(
				IDX_TASKS,
				&self.tasks.iter()
					.flat_map(|s|
						//•   •   •   •  \e   [	  3   5    m    ↳ ---  ---   •
						[32, 32, 32, 32, 27, 91, 51, 53, 109, 226, 134, 179, 32].iter()
							.chain(&s.as_bytes()[s.fitted_range(width)])
							//       \e   [   0    m  \n
							.chain(&[27, 91, 48, 109, 10])
							.copied()
					)
					.collect::<Vec<u8>>()
			);
		}
	}

	/// Redraw Title
	///
	/// This method updates the "title" slice.
	fn redraw_title(&mut self) {
		if self.title.is_empty() {
			self.buf.clear_part(IDX_TITLE);
		}
		else {
			let fit = self.title.fitted_range(self.last_width);

			// The whole thing fits; just add a line break.
			if fit.end == self.title.len() {
				self.buf.replace_part(
					IDX_TITLE,
					&self.title.as_bytes().iter()
						//       \n
						.chain(&[10])
						.copied()
						.collect::<Vec<u8>>()
				);
			}
			// It has to be chopped; add an ANSI clear in addition to the line
			// break to be safe.
			else {
				self.buf.replace_part(
					IDX_TITLE,
					&self.title.as_bytes()[fit].iter()
						//       \e   [   0    m  \n
						.chain(&[27, 91, 48, 109, 10])
						.copied()
						.collect::<Vec<u8>>()
				);
			}
		}
	}
}



#[derive(Debug, Default)]
/// Progress Bar!
pub struct Progress(Mutex<ProgressInner>);

impl Progress {
	// ------------------------------------------------------------------------
	// Public Static Methods
	// ------------------------------------------------------------------------

	#[must_use]
	/// New Progress!
	///
	/// Start a new progress bar, optionally with a message.
	pub fn new<T: Borrow<str>> (total: u64, title: Option<T>) -> Self {
		if let Some(title) = title {
			let mut inner = ProgressInner::new(total);
			inner.set_title(title);
			Self(Mutex::new(inner))
		}
		else {
			Self(Mutex::new(ProgressInner::new(total)))
		}
	}

	#[must_use]
	/// Steady tick.
	///
	/// If your `Progress` instance is behind an Arc, you can pass it to this
	/// method to spawn a steady-ticker in its own thread. When you use this,
	/// you do not need to manually call `tick()`, but do need to remember to
	/// join the handle when you're through with your own loop.
	///
	/// See the "progress" example for usage.
	pub fn steady_tick(me: &Arc<Self>, rate: Option<u64>) -> JoinHandle<()> {
		let sleep = Duration::from_millis(u64::max(60, rate.unwrap_or(60)));

		let me2 = me.clone();
		thread::spawn(move || {
			loop {
				me2.clone().tick();
				thread::sleep(sleep);

				// Are we done?
				if ! me2.clone().is_running() {
					break;
				}
			}
		})
	}



	// ------------------------------------------------------------------------
	// Getters
	// ------------------------------------------------------------------------

	/// Get done.
	///
	/// Return the amount done.
	pub fn done(&self) -> u64 {
		let ptr = mutex_ptr!(self);
		ptr.done
	}

	#[must_use]
	/// Is Running.
	///
	/// Whether or not progress is underway. For the purposes of this library,
	/// that means done is less than total.
	pub fn is_running(&self) -> bool {
		let ptr = mutex_ptr!(self);
		ptr.is_running()
	}

	#[must_use]
	/// Percent done.
	///
	/// This is literally an expression of `done / total`, returned as a float
	/// between 0.0 and 1.0.
	pub fn percent(&self) -> f64 {
		let ptr = mutex_ptr!(self);
		ptr.percent()
	}

	/// Get time.
	///
	/// Return the `Instant` the `Progress` struct was instantiated.
	pub fn time(&self) -> Instant {
		let ptr = mutex_ptr!(self);
		ptr.time
	}

	/// Get total.
	///
	/// Return the total.
	pub fn total(&self) -> u64 {
		let ptr = mutex_ptr!(self);
		ptr.total
	}



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	/// Add task.
	///
	/// Tasks are an arbitrary list of strings that will print beneath the
	/// progress line. The main idea is to communicate ideas like, "Such and
	/// such is being actively worked on in Thread #2" or whatever, but there
	/// are no hard and fast rules.
	pub fn add_task<T: Borrow<str>> (&self, task: T) {
		let mut ptr = mutex_ptr!(self);
		ptr.add_task(task)
	}

	/// Finished In
	///
	/// This method can be used to print a quick "Finished in XYZ" summary
	/// message after progress has finished.
	pub fn finished_in(&self) {
		let ptr = mutex_ptr!(self);
		ptr.finished_in()
	}

	/// Increment Done.
	///
	/// Increment the "done" count by this amount. It is recommended to prefer
	/// this method over `set_done()`, which instead takes an absolute value.
	/// Particularly in multi-threaded scenarios, one cannot be certain the
	/// order in which instructions will arrive, but if both are just "add
	/// one to the current", it all works out.
	pub fn increment(&self, num: u64) {
		let mut ptr = mutex_ptr!(self);
		ptr.increment(num)
	}

	/// Remove task.
	///
	/// If you're thinking of tasks as a list of "this is happening now" stuff,
	/// `remove_task()` is the conclusion to `add_task()`.
	pub fn remove_task<T: Borrow<str>> (&self, task: T) {
		let mut ptr = mutex_ptr!(self);
		ptr.remove_task(task)
	}

	/// Set done.
	///
	/// Set the "done" amount to this absolute value. See also `increment()`,
	/// which instead works relative to the current value.
	pub fn set_done(&self, done: u64) {
		let mut ptr = mutex_ptr!(self);
		ptr.set_done(done)
	}

	/// Set Title.
	///
	/// Update or unset the message, which if present, prints pinned above the
	/// progress line.
	pub fn set_title<T: Borrow<str>> (&self, title: T) {
		let mut ptr = mutex_ptr!(self);
		ptr.set_title(title)
	}

	/// Set total.
	pub fn set_total(&self, total: u64) {
		let mut ptr = mutex_ptr!(self);
		ptr.set_total(total)
	}

	/// Stop
	///
	/// Stop the progress bar, setting done to total, clearing tasks, emptying
	/// the message, and clearing the last bit that was printed to the terminal
	/// if any.
	pub fn stop(&self) {
		let mut ptr = mutex_ptr!(self);
		ptr.stop()
	}

	/// Tick.
	pub fn tick(&self) {
		let mut ptr = mutex_ptr!(self);
		ptr.tick()
	}

	/// Update.
	///
	/// A one-liner to increment the done count, change the message, and/or
	/// remove a task. All three values are optional and only result in a
	/// change if positive.
	///
	/// To unset a message, pass `Some(Msg::default())` rather than `None`.
	pub fn update<T1, T2> (&self, num: u64, title: Option<T1>, task: Option<T2>)
	where
	T1: Borrow<str>,
	T2: Borrow<str> {
		let mut ptr = mutex_ptr!(self);
		if num > 0 {
			ptr.increment(num);
		}
		if let Some(title) = title {
			ptr.set_title(title);
		}
		if let Some(task) = task {
			ptr.remove_task(task);
		}
	}
}
