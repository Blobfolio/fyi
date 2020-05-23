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

use crate::utility::{
	int_as_bytes,
	term_width,
};
use fyi_msg::MsgBuf;
use indexmap::set::IndexSet;
use std::borrow::Borrow;
use std::sync::Mutex;
use std::time::Instant;



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
		const PROGRESSED =   0b0001_0001;

		const TICK_BAR =     0b0000_0001;
		const TICK_DONE =    0b0000_0010;
		const TICK_ELAPSED = 0b0000_0100;
		const TICK_MSG =     0b0000_1000;
		const TICK_PERCENT = 0b0001_0000;
		const TICK_TASKS =   0b0010_0000;
		const TICK_TOTAL =   0b0100_0000;
	}
}

impl Default for ProgressFlags {
	/// Default.
	fn default() -> Self {
		ProgressFlags::NONE
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
		ProgressInner {
			buf: MsgBuf::from_many(&[
				// Title.
				&[],
				// Elapsed.
				&[27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 109],
				&[48, 48, 58, 48, 48, 58, 48, 48],
				&[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32],
				// Bar.
				&[],
				// Done.
				&[27, 91, 49, 59, 57, 54, 109],
				&[48],
				// The slash between Done and Total.
				&[27, 91, 48, 59, 50, 109, 47, 27, 91, 48, 59, 49, 59, 51, 52, 109],
				// Total.
				&[48],
				// The bit between Total and Percent.
				&[27, 91, 48, 59, 49, 109, 32, 32],
				// Percent.
				&[48, 46, 48, 48, 37],
				&[27, 91, 48, 109],
				// Tasks.
				&[],
				// Trailing line break.
				&[10],
			]),
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
	const IDX_TITLE: usize = 0;
	const IDX_ELAPSED_PRE: usize = 1;
	const IDX_ELAPSED: usize = 2;
	const IDX_ELAPSED_POST: usize = 3;
	const IDX_BAR: usize = 4;
	const IDX_DONE_PRE: usize = 5;
	const IDX_DONE: usize = 6;
	const IDX_DONE_POST: usize = 7;
	const IDX_TOTAL: usize = 8;
	const IDX_TOTAL_POST: usize = 9;
	const IDX_PERCENT: usize = 10;
	const IDX_PERCENT_POST: usize = 11;
	const IDX_TASKS: usize = 12;
	const IDX_LINE: usize = 13;



	// ------------------------------------------------------------------------
	// Public Static Methods
	// ------------------------------------------------------------------------

	#[must_use]
	/// New.
	///
	/// Start a new progress bar for `total` items.
	pub fn new(total: u64) -> Self {
		if 0 == total {
			ProgressInner::default()
		}
		else {
			ProgressInner {
				buf: MsgBuf::from_many(&[
					// Title.
					&[],
					// Elapsed.
					&[27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 109],
					&[48, 48, 58, 48, 48, 58, 48, 48],
					&[27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32],
					// Bar.
					&[],
					// Done.
					&[27, 91, 49, 59, 57, 54, 109],
					&[48],
					// The slash between Done and Total.
					&[27, 91, 48, 59, 50, 109, 47, 27, 91, 48, 59, 49, 59, 51, 52, 109],
					// Total.
					&int_as_bytes(total)[..],
					// The bit between Total and Percent.
					&[27, 91, 48, 59, 49, 109, 32, 32],
					// Percent.
					&[48, 46, 48, 48, 37],
					&[27, 91, 48, 109],
					// Tasks.
					&[],
					// Trailing line break.
					&[10],
				]),
				total,
				flags: ProgressFlags::PROGRESSED,
				..ProgressInner::default()
			}
		}
	}



	// ------------------------------------------------------------------------
	// Public Methods
	// ------------------------------------------------------------------------

	#[must_use]
	/// Is running?
	pub fn is_running(&self) -> bool {
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
				self.flags |= ProgressFlags::TICK_DONE | ProgressFlags::PROGRESSED;
				// TODO: self.stop();
			}
		}
		else if done != self.done {
			self.done = done;
			self.flags |= ProgressFlags::TICK_DONE | ProgressFlags::PROGRESSED;
		}
	}

	/// Set msg.
	///
	/// Update or unset the message, which if present, prints pinned above the
	/// progress line.
	pub fn set_title<T: Borrow<str>> (&mut self, title: T) {
		let title = title.borrow();

		// Empty message?
		if title.is_empty() {
			self.buf.clear_part(ProgressInner::IDX_TITLE);
		}
		else {
			self.buf.replace_part(
				ProgressInner::IDX_TITLE,
				&title.as_bytes().iter()
					.chain(&[10])
					.copied()
					.collect::<Vec<u8>>(),
			);
		}
	}



	// ------------------------------------------------------------------------
	// Display
	// ------------------------------------------------------------------------

	// clear
	// finished_in
	// maybe_print
	// print
	// stop
	// tick



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
	fn bar_space(&self) -> usize {
		// The magic "5" comes from Elapsed's 2 trailing spaces, total's 2
		// trailing spaces, and the slash between done/total. (Those values are
		// hard-coded into the formatting parts.)
		let total: usize = 5 +
			self.buf.get_part_len(ProgressInner::IDX_ELAPSED) +
			self.buf.get_part_len(ProgressInner::IDX_DONE) +
			self.buf.get_part_len(ProgressInner::IDX_TOTAL) +
			self.buf.get_part_len(ProgressInner::IDX_PERCENT);

		if total >= 14 { total }
		else { 0 }
	}

	// write_bar
	// write_tasks
}



#[derive(Debug)]
/// Progress Bar!
pub struct Progress(Mutex<ProgressInner>);
