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
	Flags,
	Msg,
	PrinterKind,
	PrintFlags,
	traits::GirthExt,
	utility,
};
use indexmap::map::IndexMap;
use smallvec::SmallVec;
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



#[derive(Debug, Clone)]
/// Progress Bar (Internal)
struct ProgressInner {
	/// A leading message, optional.
	msg: Vec<u8>,
	/// The creation time.
	time: Instant,
	/// The amount "done".
	done: u64,
	/// The "total" amount.
	total: u64,
	/// Tasks, e.g. a description of what is being worked on now, optional.
	tasks: IndexMap<u64, Vec<u8>>,
	/// A hash of the previous print buffer to avoid needless refreshing.
	last_hash: u64,
	/// The number of lines previously printed so we can clear before printing
	/// something new.
	last_lines: usize,
}

impl Default for ProgressInner {
	/// Default.
	fn default() -> Self {
		ProgressInner {
			msg: Vec::new(),
			time: Instant::now(),
			done: 0,
			total: 0,
			tasks: IndexMap::new(),
			last_hash: 0,
			last_lines: 0,
		}
	}
}

/// Inner Progress Bar
///
/// All of the state details for `Progress` are kept here (and locked behind
/// a Mutex-guard).
impl ProgressInner {
	/// Add task.
	///
	/// Tasks are an arbitrary list of strings that will print beneath the
	/// progress line. The main idea is to communicate ideas like, "Such and
	/// such is being actively worked on in Thread #2" or whatever, but there
	/// are no hard and fast rules.
	pub fn add_task<T: Borrow<str>> (&mut self, task: T) {
		// Each task line starts with: "\n    ↳ "
		static INTRO: &[u8] = &[10, 32, 32, 32, 32, 27, 91, 51, 53, 109, 226, 134, 179, 32];

		// Prioritizing tick speed, we're building and storing a value ready
		// for print. While we might be able to alter the struct's design to
		// take advantage of IndexSet's built-in hash keys, instead we're
		// recording our own to make association-by-original-value possible.
		let task = task.borrow().as_bytes();
		self.tasks.insert(
			seahash::hash(task),
			[
				INTRO,
				task,
				b"\x1B[0m",
			].concat()
		);
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
		self.tasks.remove(&seahash::hash(task.borrow().as_bytes()));
	}

	/// Set done.
	///
	/// Set the "done" amount to this absolute value. See also `increment()`,
	/// which instead works relative to the current value.
	pub fn set_done(&mut self, done: u64) {
		if done >= self.total {
			if self.done != self.total {
				self.done = self.total;
				self.stop();
			}
		}
		else if done != self.done {
			self.done = done;
		}
	}

	/// Set msg.
	///
	/// Update or unset the message, which if present, prints pinned above the
	/// progress line.
	pub fn set_msg(&mut self, msg: &Msg) {
		let len: usize = self.msg.len().saturating_sub(1);
		if self.msg[..len] != msg[..] {
			self.msg.clear();
			if ! msg.is_empty() {
				self.msg.extend_from_slice(msg);
				self.msg.push(b'\n');
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
		if ! self.msg.is_empty() {
			self.msg.clear();
		}

		self._print(&[]);
	}

	/// Tick.
	///
	/// Calculate progress and print it to the terminal, if necessary.
	pub fn tick(&mut self) {
		// Easy bail.
		if ! self.is_running() {
			return;
		}

		// There's a message.
		if ! self.msg.is_empty() {
			// Message and bar.
			if self.tasks.is_empty() {
				self._print(&[
					&*self.msg,
					&self._tick_bar(),
				].concat())
			}
			// All three.
			else {
				self._print(&[
					&*self.msg,
					&self._tick_bar(),
					&self._tick_tasks(),
				].concat())
			}
		}
		// Just the bar.
		else if self.tasks.is_empty() {
			self._print(&self._tick_bar())
		}
		// Bar and tasks.
		else {
			self._print(&[
				self._tick_bar(),
				self._tick_tasks(),
			].concat())
		}
	}

	/// Print.
	///
	/// Handle the actual printing, clearing the previous buffer as needed.
	fn _print(&mut self, text: &[u8]) {
		// We don't need to reprint if nothing's changed since last time.
		let hash: u64 = seahash::hash(text);
		if self.last_hash == hash {
			return;
		}
		self.last_hash = hash;

		// Clear old lines.
		if 0 != self.last_lines {
			cls(self.last_lines);

			// If there's no message, we're done!
			if text.is_empty() {
				self.last_lines = 0;
				return;
			}
		}
		self.last_lines = text.count_lines();

		// We'll be sending to `Stderr`.
		#[cfg(not(feature = "stdout_sinkhole"))] let writer = std::io::stderr();
		#[cfg(not(feature = "stdout_sinkhole"))] let mut handle = writer.lock();

		// JK, sinkhole is used for benchmarking.
		#[cfg(feature = "stdout_sinkhole")] let mut handle = std::io::sink();

		unsafe {
			utility::print_to(
				&mut handle,
				text,
				Flags::NONE
			);
		}
	}

	/// Tick bar.
	///
	/// A lot of shit goes into building the progress line; this returns a
	/// complete byte string representation.
	fn _tick_bar(&self) -> Vec<u8> {
		// The bar bits.
		static DONE: &[u8] = &[b'='; 255];
		static UNDONE: &[u8] = &[b'-'; 255];

		// This translates to: "\x1B[2m[\x1B[22;1m00:00:00\x1B[22;2m]\x1B[0m  "
		static ELAPSED: &[u8] = &[27, 91, 50, 109, 91, 27, 91, 50, 50, 59, 49, 109, 48, 48, 58, 48, 48, 58, 48, 48, 27, 91, 50, 50, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32];

		// Magic Number: contstant width from elapsed/label.
		// 12 = "[00:00:00]" + 2 trailing spaces
		// 2 = "/" and 1 space that join with the non-number-bits in the label.
		const BASE_WIDTH: usize = 14;

		// Minimum available width to have a bar. This breaks down as 10 for
		// the bar itself, 2 for its [], and 2 additional spaces. It happens to
		// be the same value as BASE_WIDTH, but again, that's coincidental.
		const BAR_MIN_WIDTH: usize = 14;

		// The maximum bar width is 255; we aren't storing any more than that.
		const BAR_MAX_WIDTH: usize = 255;

		// We need to fetch the label details first as those are the variable
		// in size.
		let (label_bits, done_end, total_end, percent_end) = self._tick_label_bits();

		// How much terminal we got?
		let width: usize = utility::term_width();
		let used_width: usize = BASE_WIDTH + percent_end;

		// Build a bar.
		let mut buf: Vec<u8> = if width > used_width + BAR_MIN_WIDTH {
			// Reserve 2 slots for whitespace and 2 for [].
			let bar_width: usize = usize::min(BAR_MAX_WIDTH, width - used_width - 4);

			// No progress.
			if 0 == self.done {
				[
					ELAPSED,
					b"\x1B[2m[\x1B[22;36m",
					&UNDONE[0..bar_width],
					b"\x1B[0;2m]\x1B[22;1;96m  ",
					&label_bits[0..done_end],
					b"\x1B[0;2m/\x1B[22;36m",
					&label_bits[done_end..total_end],
					b"\x1B[39;1m ",
					&label_bits[total_end..],
					b"\x1B[0m",
				].concat()
			}
			// Total progress.
			else if self.done == self.total {
				[
					ELAPSED,
					b"\x1B[2m[\x1B[22;1;96m",
					&DONE[0..bar_width],
					b"\x1B[0;2m]\x1B[22;1;96m  ",
					&label_bits[0..done_end],
					b"\x1B[0;2m/\x1B[22;36m",
					&label_bits[done_end..total_end],
					b"\x1B[39;1m ",
					&label_bits[total_end..],
					b"\x1B[0m",
				].concat()
			}
			// A mixture.
			else {
				let done_width: usize = f64::floor(self.percent() * bar_width as f64) as usize;
				let undone_width: usize = bar_width - done_width;

				[
					ELAPSED,
					b"\x1B[2m[\x1B[22;1;96m",
					&DONE[0..done_width],
					b"\x1B[0;36m",
					&UNDONE[0..undone_width],
					b"\x1B[0;2m]\x1B[22;1;96m  ",
					&label_bits[0..done_end],
					b"\x1B[0;2m/\x1B[22;36m",
					&label_bits[done_end..total_end],
					b"\x1B[39;1m ",
					&label_bits[total_end..],
					b"\x1B[0m",
				].concat()
			}
		}
		// Just print the labels.
		else {
			[
				ELAPSED,
				b"\x1B[96;1m",
				&label_bits[0..done_end],
				b"\x1B[0;2m/\x1B[22;36m",
				&label_bits[done_end..total_end],
				b"\x1B[39;1m ",
				&label_bits[total_end..],
				b"\x1B[0m",
			].concat()
		};

		// Write in the correct time.
		utility::slice_swap(
			&mut buf[12..20],
			&lapsed::compact(self.time.elapsed().as_secs() as u32),
		);

		// And send it off!
		buf
	}

	/// Tick label bits.
	///
	/// Returns done, total, and percentage in a single byte string with their
	/// corresponding ending positions in the buffer.
	fn _tick_label_bits(&self) -> (SmallVec<[u8; 64]>, usize, usize, usize) {
		let mut buf: SmallVec<[u8; 64]> = SmallVec::new();

		itoa::write(&mut buf, self.done).unwrap();
		let done_end: usize = buf.len();

		itoa::write(&mut buf, self.total).unwrap();
		let total_end: usize = buf.len();

		buf.extend_from_slice(format!("{:>3.*}%", 2, self.percent() * 100.0).as_bytes());
		let percent_end: usize = buf.len();

		(buf, done_end, total_end, percent_end)
	}

	/// Tick tasks.
	///
	/// Printing the tasks is relatively simple, but requires a few lines so
	/// is outsourced here.
	fn _tick_tasks(&self) -> Vec<u8> {
		let mut buf: Vec<u8> = Vec::with_capacity(256);
		for i in self.tasks.values() {
			buf.extend_from_slice(&i[..]);
		}
		buf
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
		if 0 == total {
			Progress::default()
		}
		else if let Some(msg) = msg {
			Progress(Mutex::new(ProgressInner {
				msg: [&*msg, b"\n"].concat(),
				total,
				..ProgressInner::default()
			}))
		}
		else {
			Progress(Mutex::new(ProgressInner {
				total,
				..ProgressInner::default()
			}))
		}
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



/// Clear lines.
///
/// This is an optimized display-clearing method, used to wipe the previous
/// progress output before printing the new output. (That's how "animation"
/// happens!)
///
/// The ANSI formatting rules are cached for up to 10 lines, minimizing write
/// instructions for the most common jobs, but it can handle more lines as
/// needed.
fn cls(num: usize) {
	// Pre-compute line clearings. Ten'll do for most 2020 use cases.
	static CLEAR: [&[u8]; 10] = [
		b"\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
	];

	// We'll be sending to `Stderr`.
	#[cfg(not(feature = "stdout_sinkhole"))] let writer = std::io::stderr();
	#[cfg(not(feature = "stdout_sinkhole"))] let mut handle = writer.lock();

	// JK, sinkhole is used for benchmarking.
	#[cfg(feature = "stdout_sinkhole")] let mut handle = std::io::sink();

	if num <= 9 {
		unsafe {
			utility::print_to(
				&mut handle,
				CLEAR[num],
				Flags::NO_LINE
			);
		}
	}
	else {
		unsafe {
			utility::print_to(
				&mut handle,
				&[
					CLEAR[9],
					&CLEAR[1][10..].repeat(num - 9)
				].concat(),
				Flags::NO_LINE
			);
		}
	}
}
