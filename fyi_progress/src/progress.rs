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
	chopped_len,
	human_elapsed,
	int_as_bytes,
	secs_chunks,
	term_width,
};
use fyi_msg::{
	Msg,
	MsgBuf,
	utility::time_format_dd,
};
use indexmap::set::IndexSet;
use memchr::Memchr;
use std::{
	borrow::Borrow,
	cmp::Ordering,
	io,
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
	// const IDX_ELAPSED_POST: usize = 3;
	const IDX_BAR: usize = 4;
	// const IDX_DONE_PRE: usize = 5;
	const IDX_DONE: usize = 6;
	// const IDX_DONE_POST: usize = 7;
	const IDX_TOTAL: usize = 8;
	// const IDX_TOTAL_POST: usize = 9;
	const IDX_PERCENT: usize = 10;
	const IDX_PERCENT_POST: usize = 11;
	const IDX_TASKS: usize = 12;



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
					&[27, 91, 48, 109, 10],
					// Tasks.
					&[],
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
				Ordering::Equal => { ProgressInner::print(CLS10); },
				Ordering::Less => {
					let end: usize = 10 + 14 * self.last_lines;
					ProgressInner::print(&CLS10[0..end]);
				},
				Ordering::Greater => {
					ProgressInner::print(
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
			ProgressInner::print(&Msg::crunched([
				"Finished in ",
				unsafe { std::str::from_utf8_unchecked(&human_elapsed(self.last_secs)) },
				".\n",
			].concat()));
		}
	}

	/// Maybe Print?
	///
	/// After a tick has made its changes, check and see if it is worth
	/// painting, and if so, make it happen.
	pub fn maybe_print(&mut self, width: usize) {
		// Check the hash and width if we've done stuff before.
		let hash: u64 = seahash::hash(&self.buf);
		if hash == self.last_hash && width == self.last_width {
			return;
		}

		// Otherwise go ahead and update those things.
		self.last_width = width;
		self.last_hash = hash;

		// Do we need to clear anything?
		if self.last_lines > 0 {
			self.cls();
		}

		self.print_chopped(width);
	}

	/// Print Chopped.
	///
	/// It is surprisingly difficult to chop strings to fit a given "width".
	pub fn print_chopped(&mut self, width: usize) {
		use io::Write;

		if self.buf.is_empty() {
			return;
		}

		#[cfg(not(feature = "bench_sink"))] let writer = io::stderr();
		#[cfg(not(feature = "bench_sink"))] let mut handle = writer.lock();
		#[cfg(feature = "bench_sink")] let mut handle = io::sink();

		// If there is a title, we might have to crunch it.
		let title: &[u8] = self.buf.get_part(ProgressInner::IDX_TITLE);
		if ! title.is_empty() {
			let line_len: usize = title.len();
			// It fits just fine.
			if line_len <= width {
				handle.write_all(title).unwrap();
			}
			// It might fit, but we have to calculate.
			else {
				let fit_len: usize = chopped_len(title, width);
				if fit_len == line_len {
					handle.write_all(title).unwrap();
				}
				else {
					handle.write_all(&title[..fit_len]).unwrap();
					handle.write_all(&[27, 91, 48, 109, 10]).unwrap();
				}
			}

			self.last_lines = 1;
		}

		// Go ahead and write the progress bits. We've already sized those.
		handle.write_all(self.buf.get_parts(
			ProgressInner::IDX_ELAPSED_PRE,
			ProgressInner::IDX_PERCENT_POST,
		)).unwrap();
		self.last_lines += 1;

		// Tasks are the worst. Haha.
		if ! self.tasks.is_empty() {
			let tasks: &[u8] = self.buf.get_part(ProgressInner::IDX_TASKS);
			let max_idx: usize = tasks.len();
			let mut last_idx: usize = 0;
			self.last_lines += self.tasks.len();

			for idx in Memchr::new(10_u8, tasks) {
				let line_len: usize = idx - last_idx;

				// It fits just fine.
				if line_len <= width {
					let next_idx = usize::min(max_idx, idx + 1);
					handle.write_all(&tasks[last_idx..next_idx]).unwrap();
					last_idx = next_idx;
				}
				else {
					let fit_len: usize = chopped_len(&tasks[last_idx..idx], width);

					// It all fits.
					if fit_len + 1 == line_len {
						let next_idx = usize::min(max_idx, idx + 1);
						handle.write_all(&tasks[last_idx..next_idx]).unwrap();
						last_idx = next_idx;
					}
					// We need to chop between fit-len and the end.
					else {
						handle.write_all(&tasks[last_idx..fit_len]).unwrap();

						// We need to keep the last five bytes of the line.
						let next_idx = usize::min(max_idx, idx + 1);
						let last_idx2 = next_idx - 5;
						handle.write_all(&tasks[last_idx2..next_idx]).unwrap();
						last_idx = next_idx;
					}
				}

				if last_idx == max_idx { break; }
			}
		}

		// Flush our work and call it a day!
		handle.flush().unwrap();
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

		// Update our elapsed time.
		let secs: u32 = u32::min(86400, self.time.elapsed().as_secs() as u32);
		if secs != self.last_secs {
			self.last_secs = secs;
			self.redraw_elapsed();
		}

		// Update our done amount.
		if self.flags.contains(ProgressFlags::TICK_DONE) {
			self.buf.replace_part(ProgressInner::IDX_DONE, &int_as_bytes(self.done)[..]);
			self.flags &= !ProgressFlags::TICK_DONE;
		}

		// Did the total change? Probably not, but just in case...
		if self.flags.contains(ProgressFlags::TICK_TOTAL) {
			self.buf.replace_part(ProgressInner::IDX_TOTAL, &int_as_bytes(self.total)[..]);
			self.flags &= !ProgressFlags::TICK_TOTAL;
		}

		// The percent?
		if self.flags.contains(ProgressFlags::TICK_PERCENT) {
			self.buf.replace_part(
				ProgressInner::IDX_PERCENT,
				format!("{:>3.*}%", 2, self.percent() * 100.0).as_bytes(),
			);
			self.flags &= !ProgressFlags::TICK_PERCENT;
		}

		// Update our tasks.
		if self.flags.contains(ProgressFlags::TICK_TASKS) {
			self.redraw_tasks();
			self.flags &= !ProgressFlags::TICK_TASKS;
		}

		// The bar might independently need redrawing if the width has changed.
		let width: usize = term_width();
		if width != self.last_width || self.flags.contains(ProgressFlags::TICK_BAR) {
			self.redraw_bar(width);
			self.flags &= !ProgressFlags::TICK_BAR;
		}

		// If anything changed, print it!
		self.maybe_print(width);
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
			self.buf.get_part_len(ProgressInner::IDX_ELAPSED) +
			self.buf.get_part_len(ProgressInner::IDX_DONE) +
			self.buf.get_part_len(ProgressInner::IDX_TOTAL) +
			self.buf.get_part_len(ProgressInner::IDX_PERCENT)
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
	fn redraw_bar(&mut self, width: usize) {
		// The bar bits.
		// The "done" portion is in range 14..269.
		// The "undone" portion is in range 276..531.
		static BAR: &[u8] = &[27, 91, 50, 109, 91, 27, 91, 48, 59, 57, 54, 59, 49, 109, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 61, 27, 91, 48, 59, 51, 52, 109, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 45, 27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32];

		// We don't have room for it.
		let bar_len: usize = self.bar_space(width);
		if bar_len < 10 {
			self.buf.clear_part(ProgressInner::IDX_BAR);
		}
		else {
			let mut tmp: Vec<u8> = BAR.to_vec();
			let done_width: usize = f64::floor(self.percent() * bar_len as f64) as usize;

			// Chop out the parts we don't need.
			let mut chop: usize = 276 + bar_len - done_width;
			tmp.drain(chop..531);
			chop = 14 + done_width;
			tmp.drain(chop..269);

			// Copy the completed bar on over.
			self.buf.replace_part(ProgressInner::IDX_BAR, &tmp);
		}
	}

	/// Redraw Elapsed
	///
	/// This method updates the "elapsed" time slice.
	fn redraw_elapsed(&mut self) {
		if self.last_secs < 86400 {
			let c = secs_chunks(self.last_secs);
			let mut buf: [u8; 8] = [48, 48, 58, 48, 48, 58, 48, 48];
			buf[..2].copy_from_slice(time_format_dd(c[0]));
			buf[3..5].copy_from_slice(time_format_dd(c[1]));
			buf[6..].copy_from_slice(time_format_dd(c[2]));
			self.buf.replace_part(ProgressInner::IDX_ELAPSED, &buf);
		}
		else {
			self.buf.replace_part(ProgressInner::IDX_ELAPSED, b"23:59:59");
		}
	}

	/// Redraw Tasks
	///
	/// This method updates the "tasks" slice.
	fn redraw_tasks(&mut self) {
		if self.tasks.is_empty() {
			self.buf.clear_part(ProgressInner::IDX_TASKS);
		}
		else {
			self.buf.replace_part(
				ProgressInner::IDX_TASKS,
				&self.tasks.iter()
					.flat_map(|s|
						[32, 32, 32, 32, 27, 91, 51, 53, 109, 226, 134, 179, 32].iter()
							.chain(s.as_bytes())
							.chain(&[27, 91, 48, 109, 10])
							.copied()
					)
					.collect::<Vec<u8>>()
			);
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
			Progress(Mutex::new(inner))
		}
		else {
			Progress(Mutex::new(ProgressInner::new(total)))
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
	pub fn steady_tick(me: &Arc<Progress>, rate: Option<u64>) -> JoinHandle<()> {
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
		let ptr = self.0.lock().unwrap();
		ptr.done
	}

	#[must_use]
	/// Is Running.
	///
	/// Whether or not progress is underway. For the purposes of this library,
	/// that means done is less than total.
	pub fn is_running(&self) -> bool {
		let ptr = self.0.lock().unwrap();
		ptr.is_running()
	}

	#[must_use]
	/// Percent done.
	///
	/// This is literally an expression of `done / total`, returned as a float
	/// between 0.0 and 1.0.
	pub fn percent(&self) -> f64 {
		let ptr = self.0.lock().unwrap();
		ptr.percent()
	}

	/// Get time.
	///
	/// Return the `Instant` the `Progress` struct was instantiated.
	pub fn time(&self) -> Instant {
		let ptr = self.0.lock().unwrap();
		ptr.time
	}

	/// Get total.
	///
	/// Return the total.
	pub fn total(&self) -> u64 {
		let ptr = self.0.lock().unwrap();
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
		let mut ptr = self.0.lock().unwrap();
		ptr.add_task(task)
	}

	/// Finished In
	///
	/// This method can be used to print a quick "Finished in XYZ" summary
	/// message after progress has finished.
	pub fn finished_in(&self) {
		let ptr = self.0.lock().unwrap();
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
		let mut ptr = self.0.lock().unwrap();
		ptr.increment(num)
	}

	/// Remove task.
	///
	/// If you're thinking of tasks as a list of "this is happening now" stuff,
	/// `remove_task()` is the conclusion to `add_task()`.
	pub fn remove_task<T: Borrow<str>> (&self, task: T) {
		let mut ptr = self.0.lock().unwrap();
		ptr.remove_task(task)
	}

	/// Set done.
	///
	/// Set the "done" amount to this absolute value. See also `increment()`,
	/// which instead works relative to the current value.
	pub fn set_done(&self, done: u64) {
		let mut ptr = self.0.lock().unwrap();
		ptr.set_done(done)
	}

	/// Set Title.
	///
	/// Update or unset the message, which if present, prints pinned above the
	/// progress line.
	pub fn set_title<T: Borrow<str>> (&self, title: T) {
		let mut ptr = self.0.lock().unwrap();
		ptr.set_title(title)
	}

	/// Set total.
	pub fn set_total(&self, total: u64) {
		let mut ptr = self.0.lock().unwrap();
		ptr.set_total(total)
	}

	/// Stop
	///
	/// Stop the progress bar, setting done to total, clearing tasks, emptying
	/// the message, and clearing the last bit that was printed to the terminal
	/// if any.
	pub fn stop(&self) {
		let mut ptr = self.0.lock().unwrap();
		ptr.stop()
	}

	/// Tick.
	pub fn tick(&self) {
		let mut ptr = self.0.lock().unwrap();
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
		let mut ptr = self.0.lock().unwrap();
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
