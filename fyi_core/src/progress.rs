/*!
# FYI Core: Progress Bar

This is a very simple thread-capable CLI progress indicator.
*/

use crate::{
	Msg,
	PRINT_COMPACT,
	PRINT_NEWLINE,
	PRINT_STDERR,
	PROGRESSING,
	traits::str::FYIStringFormat,
	util::{
		cli,
		strings,
		time,
	},
};
use std::{
	borrow::Cow,
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
	last: Cow<'pi, str>,
	msg: Cow<'pi, str>,
	tasks: HashSet<String>,
	time: Instant,
	total: u64,
}

impl<'pi> Default for ProgressInner<'pi> {
	/// Default.
	fn default() -> Self {
		Self {
			tasks: HashSet::new(),
			done: 0,
			flags: 0,
			last: Cow::Borrowed(""),
			msg: Cow::Borrowed(""),
			time: Instant::now(),
			total: 0,
		}
	}
}

impl<'pi> ProgressInner<'pi> {
	// -------------------------------------------------------------
	// Getters
	// -------------------------------------------------------------

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



	// -------------------------------------------------------------
	// Setters
	// -------------------------------------------------------------

	/// Push Active.
	pub fn add_tasks(&mut self, task: String) {
		if self.is_running() {
			self.tasks.insert(task);
		}
	}

	/// Remove Active.
	pub fn remove_tasks(&mut self, task: String) {
		if self.is_running() {
			self.tasks.remove(&task);
		}
	}

	/// Increment Done.
	pub fn increment(&mut self, num: u64) {
		if self.is_running() {
			self.set_done(self.done + num);
		}
	}

	/// Finished.
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

	/// Msg.
	pub fn set_msg(&mut self, msg: Cow<'pi, str>) {
		if self.is_running() {
			self.msg = msg;
		}
	}



	// -------------------------------------------------------------
	// Summaries
	// -------------------------------------------------------------

	/// Crunched in X.
	pub fn crunched_in(&mut self, saved: Option<(u64, u64)>) {
		if ! self.is_running() {
			self.print(Cow::Owned(
				Msg::crunched_in(self.total, self.time, saved).to_string()
			));
		}
	}

	/// Finished in X.
	pub fn finished_in(&mut self) {
		if ! self.is_running() {
			self.print(Cow::Owned(
				Msg::finished_in(self.time).to_string()
			));
		}
	}



	// -------------------------------------------------------------
	// Misc Operations
	// -------------------------------------------------------------

	/// Is Running?
	pub fn is_running(&self) -> bool {
		(self.flags & PROGRESSING) != 0
	}

	/// Print Whatever.
	pub fn print(&mut self, msg: Cow<'pi, str>) {
		// No change.
		if self.last == msg {
			return;
		}

		// We might need to clear the previous entry.
		let lines = self.last.fyi_lines_len() + 1;
		if 1 < lines {
			for i in 0..lines {
				if i > 0 {
					// Move the cursor up, left, and clear to end of line.
					cli::print("\x1B[1A\x1B[1000D\x1B[K", PRINT_STDERR);
				}
				else {
					// Move the cursor left and clear to end of line.
					cli::print("\x1B[1000D\x1B[K", PRINT_STDERR);
				}
			}
		}

		// Anything doing?
		if msg.is_empty() {
			self.last = Cow::Borrowed("");
			return;
		}

		self.last = msg.clone();
		cli::print(msg, PRINT_NEWLINE | PRINT_STDERR | self.flags);
	}

	/// Wrap it up.
	pub fn stop(&mut self) {
		if self.is_running() {
			self.done = self.total;
			self.flags &= !PROGRESSING;
			self.tasks.clear();
			self.msg = Cow::Borrowed("");
			self.print(Cow::Borrowed(""));
		}
	}

	/// Tick.
	pub fn tick(&mut self) {
		if ! self.is_running() {
			return;
		}

		let width: usize = cli::term_width();
		let p_elapsed = self.part_elapsed();
		let p_progress = self.part_progress();

		// Space the bar can't have.
		let p_space = p_elapsed.fyi_width() + p_progress.fyi_width() + 2;
		if width < p_space + 10 {
			return;
		}

		// Space the bar can have.
		let mut p_eta = self.part_eta();
		let p_bar_len: usize = {
			let mut size = width - p_space;
			if size > 60 {
				size = 60;
			}

			// Adjust the ETA maybe.
			let eta_len = p_eta.fyi_width();
			if 0 != eta_len && p_space + size + eta_len + 1 <= width {
				p_eta = Cow::Owned([
					strings::whitespace(width - eta_len - size - p_space),
					p_eta,
				].concat());
			}
			else if 0 != eta_len {
				p_eta = Cow::Borrowed("");
			}

			size
		};
		let p_bar = self.part_bar(p_bar_len);

		let p_tasks = self.part_tasks(width);
		let has_msg: bool = ! self.msg.is_empty();
		let has_tasks = ! p_tasks.is_empty();
		let out = Cow::Owned(if ! has_msg && ! has_tasks {
			[
				&p_elapsed,
				" ",
				&p_bar,
				" ",
				&p_progress,
				&p_eta,
			].concat()
		}
		else if has_msg && ! has_tasks {
			[
				&self.msg.clone(),
				"\n",
				&p_elapsed,
				" ",
				&p_bar,
				" ",
				&p_progress,
				&p_eta,
			].concat()
		}
		else {
			[
				&self.msg.clone(),
				"\n",
				&p_elapsed,
				" ",
				&p_bar,
				" ",
				&p_progress,
				&p_eta,
				"\n",
				&p_tasks,
			].concat()
		});

		self.print(out);
	}



	// -------------------------------------------------------------
	// Build-a-Bar
	// -------------------------------------------------------------

	/// Active paths!
	fn part_tasks(&self, width: usize) -> Cow<'_, str> {
		if self.tasks.is_empty() {
			Cow::Borrowed("")
		}
		else {
			let mut out: Vec<String> = self.tasks.iter()
				.cloned()
				.map(|ref x| {
					let out: String = [
						"    \x1B[35m↳ ",
						&x,
						"\x1B[0m",
					].concat();

					out.fyi_shorten(width).to_string()
				})
				.collect();

			out.sort();
			Cow::Owned(out.join("\n"))
		}
	}

	/// Bar!
	fn part_bar(&self, mut width: usize) -> Cow<'_, str> {
		// We need at least 10 chars.
		if width < 10 {
			return Cow::Borrowed("");
		}

		// Reserve two characters for brackets.
		width -= 2;

		let done_len: usize = f64::floor(self.percent() * width as f64) as usize;
		let undone_len: usize = width - done_len;

		let done: String = match done_len {
			0 => "".to_string(),
			x => "◼".to_string().repeat(x),
		};
		let undone: String = match undone_len {
			0 => "".to_string(),
			x => String::from_utf8(vec![b'-'; x]).unwrap(),
		};

		Cow::Owned([
			"\x1B[2m[\x1B[0m\x1B[96;1m",
			&done,
			"\x1B[0m\x1B[36m",
			&undone,
			"\x1B[0m\x1B[2m]\x1B[0m",
		].concat())
	}

	/// Elapsed.
	fn part_elapsed(&self) -> Cow<'_, str> {
		Cow::Owned([
			"\x1B[2m[\x1B[0m\x1B[1m",
			&time::human_elapsed(
				self.time.elapsed().as_secs() as usize,
				PRINT_COMPACT
			),
			"\x1B[0m\x1B[2m]\x1B[0m",
		].concat())
	}

	/// ETA.
	fn part_eta(&self) -> Cow<'_, str> {
		// Don't bother printing an ETA if we haven't gotten far
		// enough along to have good math.
		let percent: f64 = self.percent();
		if percent < 0.1 || percent == 1.0 {
			return Cow::Borrowed("");
		}

		// And abort if we haven't been at it for at least 10s.
		let elapsed: f64 = self.time.elapsed().as_secs_f64();
		if elapsed < 10.0 {
			return Cow::Borrowed("");
		}

		let s_per: f64 = elapsed / self.done as f64;
		Cow::Owned([
			"\x1B[35mETA: \x1B[0m\x1B[95;1m",
			&time::human_elapsed(
				f64::ceil(s_per * (self.total - self.done) as f64) as usize,
				PRINT_COMPACT
			),
			"\x1B[0m",
		].concat())
	}

	/// Progress bits (count, percent).
	fn part_progress(&self) -> Cow<'_, str> {
		Cow::Owned([
			"\x1B[96;1m",
			self.done.to_string().as_str(),
			"\x1B[0m\x1B[2m/\x1B[0m\x1B[36m",
			self.total.to_string().as_str(),
			"\x1B[0m \x1B[1m",
			format!("{:>3.*}%", 2, self.percent() * 100.0).as_str(),
			"\x1B[0m"
		].concat())
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
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
		ptr.tasks.insert(task.into());
	}

	/// Crunched in X.
	pub fn crunched_in(&self, saved: Option<(u64, u64)>) {
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
		ptr.crunched_in(saved);
	}

	/// Finished in X.
	pub fn finished_in(&self) {
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
		ptr.finished_in();
	}

	/// Is Running?
	pub fn is_running(&self) -> bool {
		let ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
		ptr.is_running()
	}

	/// Increment.
	pub fn increment<S>(&self, num: u64) {
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
		ptr.increment(num);
	}

	/// Add Task.
	pub fn remove_task<S>(&self, task: S)
	where S: Into<String> {
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
		ptr.tasks.remove(&task.into());
	}

	/// Set Message.
	pub fn set_msg<S>(&self, msg: S)
	where S: Into<Cow<'static, str>> {
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
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
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
		ptr.stop();
	}

	/// Tick.
	pub fn tick(&self) {
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
		ptr.tick();
	}

	/// Update one-liner.
	pub fn update(&self, increment: u64, msg: Option<String>, task: Option<String>) {
		let mut ptr = self.0.lock().expect("Failed to acquire lock: Progress.0");
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
