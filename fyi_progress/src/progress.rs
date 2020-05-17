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

use crate::lapsed;
use fyi_msg::{
	Msg,
	PrintBuf,
	PrinterKind,
	PrintFlags,
};
use indexmap::set::IndexSet;
use std::{
	borrow::Borrow,
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
	}
};



bitflags::bitflags! {
	/// Progress Bar flags.
	///
	/// These flags describe progress bar elements that have changed since the
	/// last tick and need to be redrawn.
	///
	/// These are handled automatically.
	struct ProgressFlags: u32 {
		const NONE = 0b0000_0000;
		const ALL = 0b0111_1111;
		const TICK_PROGRESSED = 0b0001_0001;

		const TICK_BAR = 0b0000_0001;
		const TICK_DONE = 0b0000_0010;
		const TICK_ELAPSED = 0b0000_0100;
		const TICK_MSG = 0b0000_1000;
		const TICK_PERCENT = 0b0001_0000;
		const TICK_TASKS = 0b0010_0000;
		const TICK_TOTAL = 0b0100_0000;
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
struct ProgressInner {
	/// Buffer.
	buf: PrintBuf,
	/// The creation time.
	time: Instant,
	/// The amount "done".
	done: u64,
	/// The "total" amount.
	total: u64,
	/// Tasks, e.g. a description of what is being worked on now, optional.
	tasks: IndexSet<String>,
	/// Flags.
	flags: ProgressFlags,
	/// Last Elapsed.
	last_secs: u32,
}

impl Default for ProgressInner {
	/// Default.
	fn default() -> Self {
		let mut buf: PrintBuf = PrintBuf::from_parts(&[
			"", // 0 Message + \n.
			"\x1B[2m[\x1B[0;1m",
			"00:00:00", // 2 Elapsed.
			"\x1B[0;2m]\x1B[0m  ",
			"", // 4 Bar + space space.
			"\x1B[96;1m",
			"0", // 6 Done.
			"\x1B[0;2m/\x1B[0;34m",
			"0", // 8 Total.
			"\x1B[0;1m  ",
			"0.00%", // 10 Percent.
			"\x1B[0m",
			"", // 12 Tasks.
		]);

		#[cfg(feature = "bench_sink")] buf.set_printer(PrinterKind::Sink);
		#[cfg(not(feature = "bench_sink"))] buf.set_printer(PrinterKind::Stderr);

		ProgressInner {
			buf,
			time: Instant::now(),
			done: 0,
			total: 0,
			tasks: IndexSet::new(),
			flags: ProgressFlags::NONE,
			last_secs: 0,
		}
	}
}

/// Inner Progress Bar
///
/// All of the state details for `Progress` are kept here (and locked behind
/// a Mutex-guard).
impl ProgressInner {
	const IDX_MSG: usize = 0;
	const IDX_ELAPSED: usize = 2;
	const IDX_BAR: usize = 4;
	const IDX_DONE: usize = 6;
	const IDX_TOTAL: usize = 8;
	const IDX_PERCENT: usize = 10;
	const IDX_TASKS: usize = 12;

	/// New.
	pub fn new(msg: Option<Msg>, total: u64) -> Self {
		if total > 0 {
			let mut pi = ProgressInner {
				total,
				flags: ProgressFlags::TICK_TOTAL | ProgressFlags::TICK_PROGRESSED,
				..ProgressInner::default()
			};

			if let Some(msg) = msg {
				pi.set_msg(&msg);
			}

			pi
		}
		else { ProgressInner::default() }
	}

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

	#[must_use]
	/// Is Running.
	///
	/// Whether or not progress is underway. For the purposes of this library,
	/// that means done is less than total.
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
			else {
				1.0
			}
		}
		else {
			0.0
		}
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
				self.flags |= ProgressFlags::TICK_DONE | ProgressFlags::TICK_PROGRESSED;
				self.stop();
			}
		}
		else if done != self.done {
			self.done = done;
			self.flags |= ProgressFlags::TICK_DONE | ProgressFlags::TICK_PROGRESSED;
		}
	}

	/// Set msg.
	///
	/// Update or unset the message, which if present, prints pinned above the
	/// progress line.
	pub fn set_msg(&mut self, msg: &Msg) {
		// Empty message?
		if msg.is_empty() {
			// Was it not empty before?
			if ! self.buf.get_part(ProgressInner::IDX_MSG).is_empty() {
				self.buf.replace_part(ProgressInner::IDX_MSG, "");
			}
		}
		else {
			let mut old_range = self.buf.get_part_range(ProgressInner::IDX_MSG);
			old_range.1 = old_range.1.saturating_sub(1);
			if self.buf[old_range.0..old_range.1] != msg[..] {
				unsafe {
					self.buf.replace_part_unchecked(
						ProgressInner::IDX_MSG,
						&[&msg[..], &[10_u8]].concat()
					);
				}
			}
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
		self.last_secs = self.time.elapsed().as_secs() as u32;
		self.buf.print_erase();
	}

	/// Tick.
	///
	/// Calculate progress and print it to the terminal, if necessary.
	pub fn tick(&mut self) {
		// Easy bail.
		if ! self.is_running() {
			return;
		}

		// Update our elapsed time.
		let secs: u32 = self.time.elapsed().as_secs() as u32;
		if secs != self.last_secs {
			self.last_secs = secs;
			unsafe { self.buf.replace_part_unchecked(ProgressInner::IDX_ELAPSED, lapsed::compact(secs).as_ref()); }
			self.flags &= !ProgressFlags::TICK_ELAPSED;
		}

		// The done amount changed?
		if self.flags.contains(ProgressFlags::TICK_DONE) {
			let mut tmp: Vec<u8> = Vec::with_capacity(16);
			itoa::write(&mut tmp, self.done).unwrap();
			unsafe { self.buf.replace_part_unchecked(ProgressInner::IDX_DONE, &tmp[..]); }
			self.flags &= !ProgressFlags::TICK_DONE;
		}

		// The total amount changed?
		if self.flags.contains(ProgressFlags::TICK_TOTAL) {
			let mut tmp: Vec<u8> = Vec::with_capacity(16);
			itoa::write(&mut tmp, self.total).unwrap();
			unsafe { self.buf.replace_part_unchecked(ProgressInner::IDX_TOTAL, &tmp[..]); }
			self.flags &= !ProgressFlags::TICK_TOTAL;
		}

		// The percent changed?
		if self.flags.contains(ProgressFlags::TICK_PERCENT) {
			unsafe {
				self.buf.replace_part_unchecked(
					ProgressInner::IDX_PERCENT,
					format!("{:>3.*}%", 2, self.percent() * 100.0).as_bytes(),
				);
			}
			self.flags &= !ProgressFlags::TICK_PERCENT;
		}

		// Did the tasks change?
		if self.flags.contains(ProgressFlags::TICK_TASKS) {
			unsafe {
				self.buf.replace_part_unchecked(
					ProgressInner::IDX_TASKS,
					&self._tick_tasks(),
				);
			}
			self.flags &= !ProgressFlags::TICK_TASKS;
		}

		// Even if bar wasn't explicitly changed, we'll need to redraw it if
		// the terminal width did.
		let widths = self.buf.term_widths();
		if widths.0 != widths.1 || self.flags.contains(ProgressFlags::TICK_BAR) {
			unsafe {
				self.buf.replace_part_unchecked(
					ProgressInner::IDX_BAR,
					&self._tick_bar(widths.0),
				);
			}
			self.flags &= !ProgressFlags::TICK_BAR;
		}

		self.buf.print(PrintFlags::CHOPPED | PrintFlags::REPRINT);
	}

	/// Tick Tasks.
	fn _tick_tasks(&self) -> Vec<u8> {
		self.tasks.iter()
			.cloned()
			.flat_map(|s| {
				[
					// "\n    ↳ ".
					&[10, 32, 32, 32, 32, 27, 91, 51, 53, 109, 226, 134, 179, 32],
					s.as_bytes(),
					b"\x1B[0m",
				].concat()
			})
			.collect()
	}

	/// Tick bar.
	///
	/// A not-insubstantial amount of calculation goes into building the bar.
	/// This internal method does all of that work and delivers what the
	/// content would be so that the main `tick()` method can set it.
	fn _tick_bar(&self, width: usize) -> Vec<u8> {
		// The bar bits.
		static DONE: &[u8] = &[b'='; 255];
		static UNDONE: &[u8] = &[b'-'; 255];

		// First, find out how much *display* space is being used by the other
		// bar elements. The magic number comes from:
		// IDX_ELAPSED (12) + / (1) + 2 spaces between done/total and percent.
		let width_used: usize = 15 +
			self.buf.get_part_len(ProgressInner::IDX_DONE) +
			self.buf.get_part_len(ProgressInner::IDX_TOTAL) +
			self.buf.get_part_len(ProgressInner::IDX_PERCENT);

		// The bar itself needs to reserve 2 slots for trailing spaces and 2
		// slots for brackets. But because we're only caching 255 chars for
		// each style, the maximum width is capped at 255.
		let width_available: usize = usize::min(
			width.saturating_sub(width_used + 4),
			255
		);

		// We don't have enough room for a bar. Boo.
		if width_available < 10 {
			Vec::new()
		}
		// There is no progress at all.
		else if 0 == self.done {
			[
				b"\x1B[2m[\x1B[0;34m",
				&UNDONE[0..width_available],
				b"\x1B[0;2m]\x1B[0m  ",
			].concat()
		}
		// We're totally done!
		else if self.done == self.total {
			[
				b"\x1B[2m[\x1B[0;96;1m",
				&DONE[0..width_available],
				b"\x1B[0;2m]\x1B[0m  ",
			].concat()
		}
		// We've got a mixture.
		else {
			let done_width: usize = f64::floor(self.percent() * width_available as f64) as usize;
			let undone_width: usize = width_available - done_width;

			[
				b"\x1B[2m[\x1B[0;96;1m",
				&DONE[0..done_width],
				b"\x1B[0;34m",
				&UNDONE[0..undone_width],
				b"\x1B[0;2m]\x1B[0m  ",
			].concat()
		}
	}
}



/// Progress Bar
#[derive(Debug, Default)]
pub struct Progress(Mutex<ProgressInner>);

impl Progress {
	#[must_use]
	/// New Progress!
	///
	/// Start a new progress bar, optionally with a message.
	pub fn new(msg: Option<Msg>, total: u64) -> Self {
		Progress(Mutex::new(ProgressInner::new(
			msg,
			total,
		)))
	}

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

	/// Get done.
	///
	/// Return the amount done.
	pub fn done(&self) -> u64 {
		let ptr = self.0.lock().unwrap();
		ptr.done
	}

	/// Finished in.
	///
	/// This is a convenience method for printing a simple "Finished in X"
	/// message, to be used after the bar has `.stop()`ed.
	///
	/// The timing begins with the moment of the `Progress` struct's
	/// instantiation, so don't dally or cache the object or anything or
	/// the timing won't make any sense.
	pub fn finished_in(&self) {
		let ptr = self.0.lock().unwrap();
		if ! ptr.is_running() {
			let mut msg = Msg::crunched([
				"Finished in ",
				unsafe { std::str::from_utf8_unchecked(&lapsed::full(
					ptr.time.elapsed().as_secs() as u32
				))},
				".",
			].concat());
			msg.set_printer(PrinterKind::Stderr);
			msg.print(PrintFlags::NONE);
		}
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

	/// Set msg.
	///
	/// Update or unset the message, which if present, prints pinned above the
	/// progress line.
	pub fn set_msg(&self, msg: &Msg) {
		let mut ptr = self.0.lock().unwrap();
		ptr.set_msg(msg)
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
	///
	/// Calculate progress and print it to the terminal, if necessary.
	pub fn tick(&self) {
		let mut ptr = self.0.lock().unwrap();
		ptr.tick()
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

	/// Update.
	///
	/// A one-liner to increment the done count, change the message, and/or
	/// remove a task. All three values are optional and only result in a
	/// change if positive.
	///
	/// To unset a message, pass `Some(Msg::default())` rather than `None`.
	pub fn update<T: Borrow<str>> (&self, num: u64, msg: Option<Msg>, task: Option<T>) {
		let mut ptr = self.0.lock().unwrap();
		if num > 0 {
			ptr.increment(num);
		}
		if let Some(msg) = msg {
			ptr.set_msg(&msg);
		}
		if let Some(task) = task {
			ptr.remove_task(task);
		}
	}
}
