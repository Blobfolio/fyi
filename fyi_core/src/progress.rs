/*!
# FYI Core: Progress Bar

This is a very simple thread-capable CLI progress indicator.
*/

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
			done: 0,
			flags: 0,
			last: Cow::Borrowed(""),
			msg: Cow::Borrowed(""),
			tasks: HashSet::new(),
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
				cli::print(CLEAR[lines], PRINT_STDERR);
			}
			else {
				cli::print([
					CLEAR[9],
					CLEAR_MORE.repeat(lines - 9).as_str(),
				].concat(), PRINT_STDERR);
			}

			// If there's no next message, replace last and leave.
			if msg.is_empty() {
				self.last = Cow::Borrowed("");
				return;
			}
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
		let p_space = p_elapsed.width() + p_progress.width() + 2;
		if width < p_space + 10 {
			return;
		}

		// Space the bar can have.
		let p_bar_len: usize = cmp::min(60, width - p_space);
		let p_bar = self.part_bar(p_bar_len);

		// Gather up the rest.
		let p_eta = self.part_eta(width - p_bar_len - p_space);
		let p_tasks = self.part_tasks(width);

		let out = Cow::Owned({
			let has_eta: bool = ! p_eta.is_empty();
			let has_msg: bool = ! self.msg.is_empty();
			let has_tasks: bool = ! p_tasks.is_empty();

			let mut total_len: usize = p_elapsed.len() + p_bar.len() + p_progress.len() + 2;
			if has_eta {
				total_len += p_eta.len();
			}
			if has_msg {
				total_len += self.msg.len() + 1;
			}
			if has_tasks {
				total_len += p_tasks.len() + 1;
			}

			let mut p: String = String::with_capacity(total_len);

			if has_msg {
				p.push_str(&self.msg);
				p.push('\n');
			}

			p.push_str(&p_elapsed);
			p.push(' ');
			p.push_str(&p_bar);
			p.push(' ');
			p.push_str(&p_progress);

			if has_eta {
				p.push_str(&p_eta);
			}

			if has_tasks {
				p.push('\n');
				p.push_str(&p_tasks);
			}

			p
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
				.map(|ref x| {
					[
						"    \x1B[35m↳ ",
						x,
						"\x1B[0m",
					].concat()
					.shorten(width)
					.to_string()
				})
				.collect();

			out.sort();
			Cow::Owned(out.join("\n"))
		}
	}

	/// Bar!
	fn part_bar(&self, mut width: usize) -> Cow<'_, str> {
		lazy_static::lazy_static! {
			// Precompute each bar to its maximum possible length (58);
			// it is cheaper to shrink than to grow.
			static ref DONE: Cow<'static, str> = Cow::Owned("==========================================================".to_string());
			static ref UNDONE: Cow<'static, str> = Cow::Owned("----------------------------------------------------------".to_string());
		}

		// We need at least 10 chars.
		if width < 10 {
			return Cow::Borrowed("");
		}

		// Reserve two characters for brackets.
		width -= 2;

		let done_len: usize = f64::floor(self.percent() * width as f64) as usize;
		// No progress.
		if 0 == done_len {
			Cow::Owned([
				"\x1B[2m[\x1B[0m\x1B[36m",
				&UNDONE[0..width],
				"\x1B[0m\x1B[2m]\x1B[0m",
			].concat())
		}
		// Total progress.
		else if done_len == width {
			Cow::Owned([
				"\x1B[2m[\x1B[0m\x1B[96;1m",
				&DONE[0..width],
				"\x1B[0m\x1B[2m]\x1B[0m",
			].concat())
		}
		// Mixed progress.
		else {
			let undone_len: usize = width - done_len;
			Cow::Owned([
				"\x1B[2m[\x1B[0m\x1B[96;1m",
				&DONE[0..done_len],
				"\x1B[0m\x1B[36m",
				&UNDONE[0..undone_len],
				"\x1B[0m\x1B[2m]\x1B[0m",
			].concat())
		}
	}

	/// Elapsed.
	fn part_elapsed(&self) -> Cow<'_, str> {
		Cow::Owned([
			"\x1B[2m[\x1B[0m\x1B[1m",
			&self.time.elapsed().as_secs().elapsed_short(),
			"\x1B[0m\x1B[2m]\x1B[0m",
		].concat())
	}

	/// ETA.
	fn part_eta(&self, width: usize) -> Cow<'_, str> {
		if width < 16 {
			return Cow::Borrowed("");
		}

		let percent: f64 = self.percent();
		if percent < 0.1 || percent == 1.0 {
			return Cow::Borrowed("");
		}

		// And abort if we haven't been at it for at least 10s.
		let elapsed: f64 = self.time.elapsed().as_secs_f64();
		if elapsed < 10.0 {
			return Cow::Borrowed("");
		}

		let eta = (f64::ceil(
			elapsed / self.done as f64 * (self.total - self.done) as f64
		) as usize).elapsed_short();
		let eta_width: usize = eta.width() + 5;

		// Last abort check.
		if eta_width >= width {
			return Cow::Borrowed("");
		}

		Cow::Owned({
			let mut p: String = String::with_capacity(eta.len() + 25 + (width - eta_width));

			p.push_str(&strings::whitespace(width - eta_width));
			p.push_str("\x1B[35mETA: \x1B[0m\x1B[95;1m");
			p.push_str(&eta);
			p.push_str("\x1B[0m");

			p
		})
	}

	/// Progress bits (count, percent).
	fn part_progress(&self) -> Cow<'_, str> {
		Cow::Owned([
			"\x1B[96;1m",
			&self.done.to_string(),
			"\x1B[0m\x1B[2m/\x1B[0m\x1B[36m",
			&self.total.to_string(),
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
	pub fn increment(&self, num: u64) {
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
