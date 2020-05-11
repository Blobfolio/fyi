/*!
# FYI Progress

This is a simple, thread-safe progress bar built around `fyi_msg`. It is a
performant altnerative to crates like `indicatif`, but only because it lacks
all but the most basic of configuration options.

In other words, if you want just want a damn progress bar, it makes one. If you
want different colors or the ability to decide which bit goes where, there are
much more flexible alternatives. Haha.
*/

use crate::lapsed;
use fyi_msg::{
	Flags,
	Msg,
	print_to,
	traits::GirthExt,
	utility,
};
use indexmap::map::IndexMap;
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
	msg: Vec<u8>,
	time: Instant,
	done: u64,
	total: u64,
	tasks: IndexMap<u64, Vec<u8>>,
	last_hash: u64,
	last_lines: usize,
	tick_lock: bool,
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
			tick_lock: false,
		}
	}
}

impl ProgressInner {
	#[must_use]
	/// Is Running.
	pub fn is_running(&self) -> bool {
		self.done < self.total
	}

	/// Add task.
	pub fn add_task<T: Borrow<str>> (&mut self, task: T) {
		// Each task line starts with: "\n    ↳ "
		static INTRO: &[u8] = &[10, 32, 32, 32, 32, 27, 91, 51, 53, 109, 226, 134, 179, 32];

		let task = task.borrow().as_bytes();
		self.tasks.insert(
			utility::hash(task),
			[
				INTRO,
				task,
				b"\x1B[0m",
			].concat()
		);
	}

	/// Increment Done.
	pub fn increment(&mut self, num: u64) {
		self.set_done(self.done + num);
	}

	#[must_use]
	/// Percent done.
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
	pub fn remove_task<T: Borrow<str>> (&mut self, task: T) {
		self.tasks.remove(&utility::hash(task.borrow().as_bytes()));
	}

	/// Set done.
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
	pub fn set_msg(&mut self, msg: &Msg) {
		if self.msg != &**msg {
			self.msg.clear();
			if ! msg.is_empty() {
				self.msg.extend_from_slice(&**msg);
				self.msg.push(b'\n');
			}
		}
	}

	/// Wrap it up.
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
	pub fn tick(&mut self) {
		// Easy bail.
		if self.tick_lock || ! self.is_running() {
			return;
		}
		self.tick_lock = true;

		// There's a message.
		if ! self.msg.is_empty() {
			// Message and bar.
			if self.tasks.is_empty() {
				self._print(&[
					&*self.msg,
					&*self._tick_bar(),
				].concat());
			}
			// All three.
			else {
				self._print(&[
					&*self.msg,
					&*self._tick_bar(),
					&*self._tick_tasks(),
				].concat());
			}
		}
		// Just the bar.
		else if self.tasks.is_empty() {
			self._print(&self._tick_bar());
		}
		// Bar and tasks.
		else {
			self._print(&[
				self._tick_bar(),
				self._tick_tasks(),
			].concat());
		}

		self.tick_lock = false;
	}

	/// Print.
	fn _print(&mut self, text: &[u8]) {
		// We don't need to reprint if nothing's changed since last time.
		let hash: u64 = utility::hash(text);
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
			print_to(
				&mut handle,
				text,
				Flags::NONE
			);
		}
	}

	/// Tick bar.
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
	fn _tick_label_bits(&self) -> (Vec<u8>, usize, usize, usize) {
		let mut buf: Vec<u8> = Vec::with_capacity(32);

		itoa::write(&mut buf, self.done).unwrap();
		let done_end: usize = buf.len();

		itoa::write(&mut buf, self.total).unwrap();
		let total_end: usize = buf.len();

		buf.extend_from_slice(format!("{:>3.*}%", 2, self.percent() * 100.0).as_bytes());
		let percent_end: usize = buf.len();

		(buf, done_end, total_end, percent_end)
	}

	/// Tick tasks.
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
	pub fn add_task<T: Borrow<str>> (&self, task: T) {
		let mut ptr = self.0.lock().unwrap();
		ptr.add_task(task)
	}

	/// Finished in.
	pub fn finished_in(&self) {
		use fyi_msg::traits::Printable;

		let ptr = self.0.lock().unwrap();
		if ! ptr.is_running() {
			Msg::crunched([
				"Finished in ",
				unsafe { std::str::from_utf8_unchecked(&lapsed::full(
					ptr.time.elapsed().as_secs() as u32
				))},
				".",
			].concat())
				.print(0, Flags::TO_STDERR);
		}
	}

	/// Increment Done.
	pub fn increment(&self, num: u64) {
		let mut ptr = self.0.lock().unwrap();
		ptr.increment(num)
	}

	#[must_use]
	/// Is Running.
	pub fn is_running(&self) -> bool {
		let ptr = self.0.lock().unwrap();
		ptr.is_running()
	}

	#[must_use]
	/// Percent done.
	pub fn percent(&self) -> f64 {
		let ptr = self.0.lock().unwrap();
		ptr.percent()
	}

	/// Remove task.
	pub fn remove_task<T: Borrow<str>> (&self, task: T) {
		let mut ptr = self.0.lock().unwrap();
		ptr.remove_task(task)
	}

	/// Set done.
	pub fn set_done(&self, done: u64) {
		let mut ptr = self.0.lock().unwrap();
		ptr.set_done(done)
	}

	/// Set msg.
	pub fn set_msg(&self, msg: &Msg) {
		let mut ptr = self.0.lock().unwrap();
		ptr.set_msg(msg)
	}

	#[must_use]
	/// Steady tick.
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

	/// Stop.
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
			print_to(
				&mut handle,
				CLEAR[num],
				Flags::NO_LINE
			);
		}
	}
	else {
		unsafe {
			print_to(
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
