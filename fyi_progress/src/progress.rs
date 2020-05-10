/*!
# FYI Progress

This is a simple, thread-safe progress bar built around `fyi_msg`. It is a
performant altnerative to crates like `indicatif`, but only because it lacks
all but the most basic of configuration options.

In other words, if you want just want a damn progress bar, it makes one. If you
want different colors or the ability to decide which bit goes where, there are
much more flexible alternatives. Haha.
*/

use fyi_msg::{
	Flags,
	Msg,
	print_to,
	term_width,
	traits::GirthExt,
};
use indexmap::map::IndexMap;
use std::cmp::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;



#[derive(Debug, Clone)]
/// Progress Bar (Internal)
struct ProgressInner {
	done: u64,
	total: u64,
	time: Instant,
	msg: Msg,
	msg_w: usize,
	tasks: IndexMap<String, usize>,
	last: Vec<u8>,
}

impl Default for ProgressInner {
	/// Default.
	fn default() -> Self {
		ProgressInner {
			done: 0,
			total: 0,
			time: Instant::now(),
			msg: Msg::default(),
			msg_w: 0,
			tasks: IndexMap::new(),
			last: Vec::new(),
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
	pub fn add_task<T: Into<String>> (&mut self, task: T) {
		let task = task.into();
		let width: usize = task.count_width();
		self.tasks.insert(task, width);
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
	pub fn remove_task<T: AsRef<str>> (&mut self, task: T) {
		self.tasks.remove(task.as_ref());
	}

	/// Set done.
	pub fn set_done(&mut self, done: u64) {
		if done >= self.total {
			self.done = self.total;
		}
		else if done != self.done {
			self.done = done;
		}
	}

	/// Set msg.
	pub fn set_msg(&mut self, msg: Msg) {
		if self.msg != msg {
			self.msg_w = msg.count_width();
			self.msg = msg;
		}
	}

	/// Wrap it up.
	pub fn stop(&mut self) {
		if self.is_running() {
			self.done = self.total;
			self.tasks.clear();
			self.set_msg(Msg::default());
			self.print(&[]);
		}
	}

	/// Tick.
	pub fn tick(&mut self) {
		// Easy bail.
		if ! self.is_running() {
			return;
		}

		// How much terminal we got?
		let width: usize = term_width();
		let mut buf: Vec<u8> = Vec::with_capacity(usize::min(self.last.len(), 256));

		// Msg.
		// Elapsed.
		// Bar bit.
		// Progress bit.
		// ETA.
		// Tasks.
	}

	/// Print.
	fn print(&mut self, text: &[u8]) {
		// No change?
		if self.last == text {
			return;
		}

		// Did we do something last time?
		if ! self.last.is_empty() {
			cls(self.last.count_lines());

			// If there's no message, we're done!
			if text.is_empty() {
				self.last.clear();
				return;
			}
		}

		// Let's try to merge it in.
		let old_len: usize = self.last.len();
		let new_len: usize = text.len();

		match old_len.cmp(&new_len) {
			Ordering::Greater => self.last.truncate(new_len),
			Ordering::Less => self.last.reserve(new_len - old_len),
			_ => {},
		}

		// Update our cached copy.
		self.last.copy_from_slice(text);

		// We'll be sending to `Stderr`.
		let writer = std::io::stderr();
		let mut handle = writer.lock();
		unsafe {
			print_to(
				&mut handle,
				&self.last,
				Flags::NONE
			);
		}
	}
}



/// Progress Bar
#[derive(Debug, Default)]
pub struct Progress(Mutex<ProgressInner>);

impl Progress {
	#[must_use]
	/// Is Running.
	pub fn is_running(&self) -> bool {
		let ptr = self.0.lock().unwrap();
		ptr.is_running()
	}

	/// Add task.
	pub fn add_task<T: Into<String>> (&self, task: T) {
		let mut ptr = self.0.lock().unwrap();
		ptr.add_task(task)
	}

	/// Increment Done.
	pub fn increment(&self, num: u64) {
		let mut ptr = self.0.lock().unwrap();
		ptr.increment(num)
	}

	#[must_use]
	/// Percent done.
	pub fn percent(&self) -> f64 {
		let ptr = self.0.lock().unwrap();
		ptr.percent()
	}

	/// Remove task.
	pub fn remove_task<T: AsRef<str>> (&self, task: T) {
		let mut ptr = self.0.lock().unwrap();
		ptr.remove_task(task)
	}

	/// Set done.
	pub fn set_done(&self, done: u64) {
		let mut ptr = self.0.lock().unwrap();
		ptr.set_done(done)
	}

	/// Set msg.
	pub fn set_msg(&self, msg: Msg) {
		let mut ptr = self.0.lock().unwrap();
		ptr.set_msg(msg)
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
	pub fn update<T: AsRef<str>> (&self, num: u64, msg: Option<Msg>, task: Option<T>) {
		let mut ptr = self.0.lock().unwrap();
		if num > 0 {
			ptr.increment(num);
		}
		if let Some(msg) = msg {
			ptr.set_msg(msg);
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
	let writer = std::io::stderr();
	let mut handle = writer.lock();

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
