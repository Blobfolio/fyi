/*!
# FYI Core: Progress Bar

This is a very simple thread-capable CLI progress indicator.
*/

use bytes::{
	BytesMut,
	BufMut
};
use crate::{
	Msg,
	PRINT_NEWLINE,
	PRINT_STDERR,
	PROGRESSING,
	traits::{
		AnsiBitsy,
		Elapsed,
		Shorty,
	},
	util::{
		cli,
		strings,
	},
};
use std::{
	borrow::Cow,
	cmp,
	collections::HashSet,
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



#[derive(Debug, Clone)]
/// Progress Bar (Internal)
pub struct ProgressInner<'pi> {
	done: u64,
	flags: u8,
	last: BytesMut,
	msg: Cow<'pi, str>,
	tasks: HashSet<String>,
	time: Instant,
	total: u64,
}

impl<'pi> Default for ProgressInner<'pi> {
	/// Default.
	fn default() -> Self {
		Self {
			done: 0,
			flags: PRINT_STDERR,
			last: BytesMut::with_capacity(0),
			msg: Cow::Borrowed(""),
			tasks: HashSet::new(),
			time: Instant::now(),
			total: 0,
		}
	}
}

impl<'pi> ProgressInner<'pi> {
	// -----------------------------------------------------------------
	// Status
	// -----------------------------------------------------------------

	/// Is Running?
	pub fn is_running(&self) -> bool {
		(self.flags & PROGRESSING) != 0
	}

	/// Wrap it up.
	pub fn stop(&mut self) {
		if self.is_running() {
			self.done = self.total;
			self.flags &= !PROGRESSING;
			self.tasks.clear();
			self.msg = Cow::Borrowed("");
			self.print(BytesMut::default());
		}
	}

	/// Tick.
	pub fn tick(&mut self) {
		if ! self.is_running() {
			return;
		}

		let width: usize = cli::term_width();
		let mut buf: BytesMut = BytesMut::with_capacity(256);

		// If we have a message to add, we need to record the length
		// afterwards so we can ignore the slice when building the bar
		// line.
		let start: usize = if ! self.msg.is_empty() {
			self._msg_put_msg(&mut buf, width);
			buf.len()
		} else {
			0
		};

		// These always happen.
		self._msg_put_elapsed(&mut buf);

		// Switch to a different buffer for the second part of the line,
		// which we always want.
		let mut buf2: BytesMut = BytesMut::with_capacity(64);
		self._msg_put_label(&mut buf2);

		// Add an ASCII bar if we have the room. The magic numbers break
		// down as follows:
		//   14: 10 bar, 2[], 2 spaces (the minimum we want for display)
		//   14: the width of an ETA with one space.
		// We want the bar to fill all available space (up to a max of
		// 62 chars with spaces and brackets), but if it needs to be a
		// bit smaller to fit the ETA, we'll go with that.
		let cur_width: usize = buf[start..].width() + buf2.width();
		if width >= cur_width + 14 + 14 {
			self._msg_put_bar(&mut buf, width - cur_width - 14);
		}
		else if width >= cur_width + 14 {
			self._msg_put_bar(&mut buf, width - cur_width);
		}

		// Attach the label to our main buffer.
		buf.put(buf2);

		// Tack on an ETA.
		let cur_width: usize = buf[start..].width();
		if width >= cur_width + 14 {
			self._msg_put_eta(&mut buf, width - cur_width);
		}

		// And lastly, the tasks, if any.
		if ! self.tasks.is_empty() {
			self._msg_put_tasks(&mut buf, width);
		}

		self.print(buf);
	}

	/// Msg.
	fn _msg_put_msg(&self, buf: &mut BytesMut, width: usize) {
		buf.put(self.msg.shorten(width).as_bytes());
		buf.put_u8(b'\n');
	}

	/// Elapsed.
	fn _msg_put_elapsed(&self, buf: &mut BytesMut) {
		buf.extend_from_slice(b"\x1B[2m[\x1B[0m\x1B[1m");
		buf.put(self.time.elapsed().as_secs().elapsed_short().as_bytes());
		buf.extend_from_slice(b"\x1B[0m\x1B[2m]\x1B[0m  ");
	}

	/// Bar label (X/Y Z%).
	fn _msg_put_label(&self, buf: &mut BytesMut) {
		buf.extend_from_slice(b"\x1B[96;1m");
		itoa::fmt(&mut *buf, self.done).unwrap();
		buf.extend_from_slice(b"\x1B[0m\x1B[2m/\x1B[0m\x1B[36m");
		itoa::fmt(&mut *buf, self.total).unwrap();
		buf.extend_from_slice(b"\x1B[0m \x1B[1m");
		buf.put(format!("{:>3.*}%", 2, self.percent() * 100.0).as_bytes());
		buf.extend_from_slice(b"\x1B[0m");
	}

	/// The Bar.
	fn _msg_put_bar(&self, buf: &mut BytesMut, width: usize) {
		lazy_static::lazy_static! {
			// Precompute each bar to its maximum possible length (58);
			// it is cheaper to shrink than to grow.
			static ref DONE: Cow<'static, [u8]> = Cow::Owned(vec![b'='; 58]);
			static ref UNDONE: Cow<'static, [u8]> = Cow::Owned(vec![b'-'; 58]);
		}

		buf.extend_from_slice(b"\x1B[2m[\x1B[0m\x1B[96;1m");

		// Reserve two chars for the brackets and two for spaces.
		let width: usize = cmp::min(60, width) - 4;

		let done_len: usize = f64::floor(self.percent() * width as f64) as usize;
		// No progress.
		if 0 == done_len {
			buf.put(&UNDONE[0..width]);
		}
		// Only progress.
		else if done_len == width {
			buf.put(&DONE[0..width]);
		}
		// A mixture.
		else {
			let undone_len: usize = width - done_len;
			buf.put(&DONE[0..done_len]);
			buf.extend_from_slice(b"\x1B[0m\x1B[36m");
			buf.put(&UNDONE[0..undone_len]);
		}

		buf.extend_from_slice(b"\x1B[0m\x1B[2m]\x1B[0m  ");
	}

	/// The ETA.
	fn _msg_put_eta(&self, buf: &mut BytesMut, width: usize) {
		// To make a worthwhile estimate, we need to be somewhere in the
		// middle.
		let percent: f64 = self.percent();
		if percent < 0.1 || percent == 1.0 {
			return;
		}

		// We also need some amount of time to have elapsed.
		let elapsed: f64 = self.time.elapsed().as_secs_f64();
		if elapsed < 10.0 {
			return;
		}

		let eta = (f64::ceil(
			elapsed / self.done as f64 * (self.total - self.done) as f64
		) as usize).elapsed_short();

		buf.put(strings::whitespace_bytes(width - 13).as_ref());
		buf.extend_from_slice(b"\x1B[35mETA: \x1B[0m\x1B[95;1m");
		buf.put(eta.as_bytes());
		buf.extend_from_slice(b"\x1B[0m");
	}

	/// The tasks.
	fn _msg_put_tasks(&self, buf: &mut BytesMut, width: usize) {
		for i in &self.tasks {
			buf.put("\n    \x1B[35m↳ ".as_bytes());
			buf.put(i.shorten(width).as_bytes());
			buf.extend_from_slice(b"\x1B[0m");
		}
	}



	// -----------------------------------------------------------------
	// Getters
	// -----------------------------------------------------------------

	/// Percent done.
	pub fn percent(&self) -> f64 {
		if self.total > 0 {
			match self.total > self.done {
				true => self.done as f64 / self.total as f64,
				false => 1.0,
			}
		}
		else {
			0.0
		}
	}



	// -----------------------------------------------------------------
	// Setters
	// -----------------------------------------------------------------

	/// Increment Done.
	pub fn increment(&mut self, num: u64) {
		self.set_done(self.done + num);
	}

	/// Set Done.
	pub fn set_done(&mut self, done: u64) {
		if self.is_running() {
			if done >= self.total {
				self.stop();
			}
			else {
				self.done = done;
			}
		}
	}



	// -------------------------------------------------------------
	// Display
	// -------------------------------------------------------------

	/// Crunched in X.
	pub fn crunched_in(&mut self, saved: Option<(u64, u64)>) {
		if ! self.is_running() {
			self.print(BytesMut::from(
				Msg::crunched_in(self.total, self.time, saved).to_string().as_bytes()
			));
		}
	}

	/// Finished in X.
	pub fn finished_in(&mut self) {
		if ! self.is_running() {
			self.print(BytesMut::from(
				Msg::finished_in(self.time).to_string().as_bytes()
			));
		}
	}

	/// Print Whatever.
	pub fn print(&mut self, msg: BytesMut) {
		lazy_static::lazy_static! {
			// Pre-compute line clearings. Ten'll do for most 2020 use cases.
			static ref CLEAR: [&'static str; 10] = [
				"\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
				"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			];

			static ref CLEAR_MORE: Cow<'static, str> = Cow::Borrowed("\x1B[1A\x1B[1000D\x1B[K");
		}

		// No change.
		if self.last == msg {
			return;
		}

		// We might need to clear the previous entry.
		if false == self.last.is_empty() {
			let lines = self.last.lines_len();

			// The count starts at zero for the purposes of CLEAR.
			if lines <= 9 {
				cli::print(CLEAR[lines], self.flags);
			}
			else {
				cli::print([
					CLEAR[9],
					CLEAR_MORE.repeat(lines - 9).as_str(),
				].concat(), self.flags);
			}

			// If there's no next message, replace last and leave.
			if msg.is_empty() {
				self.last.clear();
				return;
			}
		}

		self.last.clear();
		self.last.put(msg);
		cli::print(
			unsafe { String::from_utf8_unchecked(self.last.to_vec()) },
			PRINT_NEWLINE | self.flags
		);
	}
}



#[derive(Debug)]
/// Progress Bar!
pub struct Progress<'p>(Mutex<ProgressInner<'p>>);

impl<'p> Progress<'p> {
	/// New.
	pub fn new<S>(msg: S, total: u64, mut flags: u8) -> Self
	where S: Into<Cow<'static, str>> {
		if total > 0 {
			flags |= PROGRESSING;
		}
		flags |= PRINT_STDERR;
		flags &= !PRINT_NEWLINE;

		Self(Mutex::new(ProgressInner{
			msg: msg.into(),
			total: total,
			flags: flags,
			..ProgressInner::default()
		}))
	}

	/// Add Task.
	pub fn add_task<S>(&self, task: S)
	where S: Into<String> {
		let mut ptr = self.0.lock().unwrap();
		ptr.tasks.insert(task.into());
	}

	/// Crunched in X.
	pub fn crunched_in(&self, saved: Option<(u64, u64)>) {
		let mut ptr = self.0.lock().unwrap();
		ptr.crunched_in(saved);
	}

	/// Finished in X.
	pub fn finished_in(&self) {
		let mut ptr = self.0.lock().unwrap();
		ptr.finished_in();
	}

	/// Is Running?
	pub fn is_running(&self) -> bool {
		let ptr = self.0.lock().unwrap();
		ptr.is_running()
	}

	/// Increment.
	pub fn increment(&self, num: u64) {
		let mut ptr = self.0.lock().unwrap();
		ptr.increment(num);
	}

	/// Percent done.
	pub fn percent(&self) -> f64 {
		let ptr = self.0.lock().unwrap();
		ptr.percent()
	}

	/// Add Task.
	pub fn remove_task<S>(&self, task: S)
	where S: Into<String> {
		let mut ptr = self.0.lock().unwrap();
		ptr.tasks.remove(&task.into());
	}

	/// Set Message.
	pub fn set_msg<S>(&self, msg: S)
	where S: Into<Cow<'static, str>> {
		let mut ptr = self.0.lock().unwrap();
		ptr.msg = msg.into();
	}

	/// Steady tick.
	pub fn steady_tick(me: &Arc<Progress<'static>>, rate: Option<u64>) -> JoinHandle<()> {
		let sleep = Duration::from_millis(match rate {
			Some(r) => r,
			_ => 60,
		});

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
		ptr.stop();
	}

	/// Tick.
	pub fn tick(&self) {
		let mut ptr = self.0.lock().unwrap();
		ptr.tick();
	}

	/// Update one-liner.
	pub fn update(&self, increment: u64, msg: Option<String>, task: Option<String>) {
		let mut ptr = self.0.lock().unwrap();
		if increment > 0 {
			ptr.increment(increment);
		}
		if let Some(msg) = msg {
			ptr.msg = Cow::Owned(msg);
		}
		if let Some(task) = task {
			ptr.tasks.remove(&task);
		}
	}
}
