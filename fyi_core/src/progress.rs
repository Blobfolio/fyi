/*!
# FYI Core: Progress Bar

This is a very simple thread-capable CLI progress indicator.
*/

#[cfg(feature = "witcher")]
use crate::witcher::formats::FYIFormats;

use ansi_term::{
	Colour,
	Style,
};
use crate::misc::{
	cli,
	strings,
	time,
};
use crate::msg::Msg;
use crate::prefix::Prefix;
use std::sync::atomic::{
	AtomicBool,
	AtomicU8,
	AtomicU64,
	Ordering,
};
use std::sync::{
	Arc,
	Mutex,
};
use std::thread::{
	self,
	JoinHandle,
};
use std::time::{
	Duration,
	Instant,
};
use std::path::{
	Path,
	PathBuf,
};



// Progress bar characters.
pub const CHR_DONE: &str = "◼";
pub const CHR_THREADS: &str = "▭";
pub const CHR_PENDING: &str = "-";

#[derive(Debug, Defaults)]
/// Progress.
pub struct Progress {
	#[def = "Mutex::new(Instant::now())"]
	/// Start time.
	time: Mutex<Instant>,

	#[def = "Arc::new(AtomicU8::new(0))"]
	/// Flags.
	flags: Arc<AtomicU8>,

	#[def = "Arc::new(AtomicU8::new(0))"]
	/// Lines last printed.
	last_lines: Arc<AtomicU8>,

	#[def = "Arc::new(AtomicBool::new(false))"]
	/// Running?
	running: Arc<AtomicBool>,

	#[def = "Mutex::new(String::new())"]
	/// A message to accompany progress.
	msg: Mutex<String>,

	#[def = "Arc::new(AtomicU64::new(0))"]
	/// The total done.
	done: Arc<AtomicU64>,

	#[def = "Arc::new(AtomicU64::new(0))"]
	/// The total in-progress threads happening.
	threads: Arc<AtomicU64>,

	#[def = "Arc::new(AtomicU64::new(0))"]
	/// The total total.
	total: Arc<AtomicU64>,
}

/// Main methods.
impl Progress {
	/// New.
	pub fn new<S> (msg: S, total: u64, flags: u8) -> Arc<Mutex<Self>>
	where S: Into<String> {
		Arc::new(Mutex::new(Progress {
			flags: Arc::new(AtomicU8::new(flags)),
			running: Arc::new(AtomicBool::new(0 < total)),
			msg: Mutex::new(msg.into()),
			total: Arc::new(AtomicU64::new(total)),
			..Progress::default()
		}))
	}

	/// Replace.
	///
	/// Re-use the Arc/Mutex with new data instead of creating a new
	/// one.
	pub fn replace<S> (&self, msg: S, total: u64, flags: u8)
	where S: Into<String> {
		{
			let mut ptr = self.time.lock().expect("Failed to acquire lock: Progress.time");
			*ptr = Instant::now();
		}

		self.flags.store(flags, Ordering::SeqCst);
		self.last_lines.store(0, Ordering::SeqCst);
		self.running.store(0 < total, Ordering::SeqCst);
		self.set_msg(msg);
		self.done.store(0, Ordering::SeqCst);
		self.total.store(total, Ordering::SeqCst);
	}

	/// Is Running
	pub fn is_running(&self) -> bool {
		self.running.load(Ordering::SeqCst)
	}

	/// Tick state.
	pub fn progress(&self) -> (u64, u64, u64) {
		(self.done(), self.threads(), self.total())
	}

	/// Set Threads.
	pub fn close_threads(&self, mut interval: u64) {
		let old = self.threads();
		if interval > old {
			interval = old;
		}

		if 0 != interval {
			self.set_threads(old - interval);
		}
	}

	/// Finish.
	pub fn finish(&self) {
		if self.is_running() {
			self.stop();
		}

		// We're done.
		if 0 != (crate::PROGRESS_CLEAR_ON_FINISH & self.flags()) {
			self.print(String::new());
			return;
		}

		// Come up with a message.
		let msg: String = {
			let ptr = self.time.lock().expect("Failed to acquire lock: Progress.time");
			let msg: Msg = Msg::msg_finished_in(*ptr);
			msg.to_string()
		};

		// Print it!
		self.print(&msg);
	}

	/// Set Done.
	pub fn increment(&self, interval: u64) {
		self.set_done(self.done() + interval);
	}

	/// Increment and decrease thread count.
	pub fn increment_and_close_threads(&self, interval: u64, thread_interval: u64) {
		self.increment(interval);
		self.close_threads(thread_interval);
	}

	/// Set Threads.
	pub fn open_threads(&self, interval: u64) {
		self.set_threads(self.threads() + interval);
	}

	/// Set Done.
	pub fn set_done(&self, mut done: u64) {
		let (old, _, total) = self.progress();
		if total <= done {
			done = total;
		}

		if done != old {
			self.done.store(done, Ordering::SeqCst);

			if done == total {
				self.stop();
			}
		}
	}

	/// Set Thread Count.
	pub fn set_threads(&self, threads: u64) {
		self.threads.store(threads, Ordering::SeqCst);
	}

	/// Set Message.
	pub fn set_msg<S> (&self, msg: S)
	where S: Into<String> {
		let mut ptr = self.msg.lock().expect("Failed to acquire lock: Progress.msg");
		let msg = msg.into();
		if msg != *ptr {
			*ptr = msg;
		}
	}

	/// Set Message as Path
	#[cfg(feature = "witcher")]
	pub fn set_path<P> (&self, path: P)
	where P: AsRef<Path> {
		let path: PathBuf = path.as_ref().fyi_to_path_buf_abs();
		let msg: Msg = Msg::new(path.to_str().unwrap_or(""))
			.with_prefix(Prefix::Custom("Path", 199));

		self.set_msg(msg.to_string());
	}

	/// Tick.
	pub fn tick(&self) {
		// Build the new message.
		let width = cli::term_width();
		let (done, threads, total) = self.progress();

		// Start building the second line first.
		let p_count = self.part_count(done, total);
		let p_percent = self.part_percent(done, total);
		let p_elapsed = self.part_elapsed();
		let mut p_eta = self.part_eta(done, total);

		// How much space have we used?
		let p_space = strings::stripped_len(&p_elapsed) +
			strings::stripped_len(&p_count) +
			strings::stripped_len(&p_percent) +
			3;

		// The bar can have the remaining space.
		let p_bar_len = {
			let mut count = width - p_space;
			if count > 60 {
				count = 60;
			}

			// Adjust the ETA again.
			let eta_len = strings::stripped_len(&p_eta);
			if 0 != eta_len && p_space + count + eta_len + 1 <= width {
				p_eta = format!(
					"{}{}",
					strings::whitespace(width - eta_len - count - p_space),
					&p_eta
				);
			}
			else if 0 != eta_len {
				p_eta = String::new();
			}

			count
		};
		let p_bar = self.part_bar(done, threads, total, p_bar_len);

		// Let's go ahead and make this line.
		let mut out: String = format!(
			"{} {} {} {}{}",
			&p_elapsed,
			&p_bar,
			&p_count,
			&p_percent,
			&p_eta
		).to_string();

		// Is there a message to add to it?
		let p_msg = self.part_msg(width - 6);
		if false == p_msg.is_empty() {
			out = format!(
				"{}\n    {} {}",
				out,
				Colour::Cyan.dimmed().paint("↳"),
				&p_msg
			);
		}

		// Send it to the printer!
		self.print(out);
	}



	// -----------------------------------------------------------------
	// Internal Getters
	// -----------------------------------------------------------------

	/// Get done.
	fn done(&self) -> u64 {
		self.done.load(Ordering::SeqCst)
	}

	/// Get flags.
	fn flags(&self) -> u8 {
		self.flags.load(Ordering::SeqCst)
	}

	/// Get last.
	fn last_lines(&self) -> u8 {
		self.last_lines.load(Ordering::SeqCst)
	}

	/// Get threads.
	fn threads(&self) -> u64 {
		self.threads.load(Ordering::SeqCst)
	}

	/// Get total.
	fn total(&self) -> u64 {
		self.total.load(Ordering::SeqCst)
	}


	// -----------------------------------------------------------------
	// Internal Ops
	// -----------------------------------------------------------------

	/// Print.
	fn print<S> (&self, msg: S)
	where S: Into<String> {
		let msg = msg.into();
		let lines: u8 = self.last_lines();
		if 0 < lines {
			cli::print(
				&format!("{}", ansi_escapes::EraseLines(lines as u16 + 1)),
				crate::PRINT_STDERR
			);
		}

		let nlines: u8 = strings::lines(&msg) as u8;
		if nlines != lines {
			self.last_lines.store(nlines, Ordering::SeqCst);
		}

		if 0 < nlines {
			cli::print(msg, crate::PRINT_NEWLINE | crate::PRINT_STDERR | self.flags());
		}
	}

	/// Stop
	fn stop(&self) {
		self.running.store(false, Ordering::SeqCst);
		self.done.store(self.total(), Ordering::SeqCst);
		let mut ptr = self.msg.lock().expect("Failed to acquire lock: Progress.msg");
		*ptr = String::new();
	}



	// -----------------------------------------------------------------
	// Bar Parts
	// -----------------------------------------------------------------

	/// Part: Bar
	fn part_bar(&self, done: u64, threads: u64, total: u64, mut width: usize) -> String {
		// Early abort.
		if 10 > width {
			return String::new();
		}
		width = width - 2;

		// Calculate the percentage done.
		let done_percent: f64 = {
			let mut tmp: f64 = 0.0;
			if 0 < total {
				if total <= done {
					tmp = 1.0;
				}
				else {
					tmp = done as f64 / total as f64;
				}
			}
			tmp
		};

		// And calculate the percentage in-progress.
		let threads_percent: f64 = {
			let mut tmp: f64 = 0.0;
			if 0 < threads {
				if total <= done + threads {
					tmp = 1.0;
				}
				else {
					tmp = threads as f64 / total as f64;
				}
			}

			if done_percent + tmp > 1.0 {
				tmp = 1.0 - done_percent;
			}

			tmp
		};

		// Now we can do the lengths!
		let done_len: usize = f64::floor(done_percent * width as f64) as usize;
		let threads_len: usize = {
			let mut tmp: usize = f64::floor(threads_percent * width as f64) as usize;
			if done_len + tmp > width {
				tmp = width - done_len;
			}
			tmp
		};
		let pending_len: usize = width - threads_len - done_len;

		// And now we can build the base strings.
		let done_str: String = match done_len {
			0 => String::new(),
			x => CHR_DONE.repeat(x),
		};
		let threads_str: String = match threads_len {
			0 => String::new(),
			x => CHR_THREADS.repeat(x),
		};
		let pending_str: String = match pending_len {
			0 => String::new(),
			x => CHR_PENDING.repeat(x),
		};

		// And now we can send pretty stuff back.
		format!(
			"{}{}{}{}{}",
			Style::new().dimmed().paint("["),
			Colour::Cyan.bold().paint(&done_str),
			Colour::Cyan.dimmed().paint(&threads_str),
			Colour::Cyan.dimmed().paint(&pending_str),
			Style::new().dimmed().paint("]"),
		).to_string()
	}

	/// Tick count.
	fn part_count(&self, done: u64, total: u64) -> String {
		if 0 == total {
			String::new()
		}
		else {
			format!(
				"{}{}{}",
				Colour::Cyan.bold().paint(format!("{}", done)),
				Style::new().dimmed().paint("/"),
				Colour::Cyan.dimmed().paint(format!("{}", total))
			)
		}
	}

	/// Tick elapsed.
	fn part_elapsed(&self) -> String {
		let ptr = self.time.lock().expect("Failed to acquire lock: Progress.time");
		let elapsed: String = time::human_elapsed(
			ptr.elapsed().as_secs() as usize,
			crate::PRINT_COMPACT
		);

		format!(
			"{}{}{}",
			Style::new().dimmed().paint("["),
			Style::new().bold().paint(&elapsed),
			Style::new().dimmed().paint("]"),
		).to_string()
	}

	/// Tick message.
	fn part_msg(&self, width: usize) -> String {
		let ptr = self.msg.lock().expect("Failed to acquire lock: Progress.msg");
		if ptr.is_empty() {
			return String::new();
		}

		let msg: String = ptr.clone();
		strings::shorten_right(msg, width)
	}

	/// Tick percent.
	fn part_percent(&self, done: u64, total: u64) -> String {
		let percent: f64 = {
			let mut tmp: f64 = 0.0;
			if 0 < total {
				if total <= done {
					tmp = 1.0;
				}
				else {
					tmp = done as f64 / total as f64;
				}
			}
			tmp
		};

		format!(
			"{}",
			Style::new().bold().paint(format!("{:>3.*}%", 2, percent * 100.0))
		).to_string()
	}

	/// Tick elapsed.
	fn part_eta(&self, done: u64, total: u64) -> String {
		let done: f64 = done as f64;
		let total: f64 = total as f64;

		// Abort if no progress has been made.
		if done < 2.0 || done >= total {
			return String::new();
		}

		let elapsed: f64 = {
			let ptr = self.time.lock().expect("Failed to acquire lock: Progress.time");
			ptr.elapsed().as_secs() as f64
		};

		// Abort if we haven't spent ten seconds doing anything yet.
		if elapsed < 10.0 {
			return String::new();
		}

		let s_per: f64 = elapsed / done;
		let remaining: String = time::human_elapsed(
			f64::ceil(s_per * (total - done)) as usize,
			crate::PRINT_COMPACT
		);

		format!(
			"{} {}",
			Colour::Purple.dimmed().paint("ETA:"),
			Colour::Purple.bold().paint(&remaining),
		).to_string()
	}
}



/// Arc wrappers.
pub mod arc {
	use super::*;

	/// Is Running
	pub fn is_running(progress: &Arc<Mutex<Progress>>) -> bool {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.is_running()
	}

	/// Event loop.
	pub fn looper(progress: &Arc<Mutex<Progress>>, interval: u64) -> JoinHandle<()> {
		let pclone = progress.clone();

		std::thread::spawn(move || {
			// Ping every 150ms.
			let sleep = Duration::from_millis(interval);
			loop {
				tick(&pclone);

				thread::sleep(sleep);

				// Are we done?
				if ! is_running(&pclone) {
					break;
				}
			}

			// And finish up.
			finish(&pclone);
		})
	}

	/// Set Threads.
	pub fn close_threads(progress: &Arc<Mutex<Progress>>, interval: u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.close_threads(interval)
	}

	/// Increment Done.
	pub fn increment(progress: &Arc<Mutex<Progress>>, interval: u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.increment(interval)
	}

	/// Increment and decrease thread count.
	pub fn increment_and_close_threads(progress: &Arc<Mutex<Progress>>, interval: u64, thread_interval: u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.increment_and_close_threads(interval, thread_interval)
	}

	/// Set Threads.
	pub fn open_threads(progress: &Arc<Mutex<Progress>>, interval: u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.open_threads(interval);
	}

	/// Tick progress.
	pub fn progress(progress: &Arc<Mutex<Progress>>) -> (u64, u64, u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.progress()
	}

	/// Replace.
	///
	/// Re-use the Arc/Mutex with new data instead of creating a new
	/// one.
	pub fn replace<S> (progress: &Arc<Mutex<Progress>>, msg: S, total: u64, flags: u8)
	where S: Into<String> {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.replace(msg, total, flags)
	}

	/// Set Done.
	pub fn set_done(progress: &Arc<Mutex<Progress>>, done: u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.set_done(done)
	}

	/// Set Message.
	pub fn set_msg<S> (progress: &Arc<Mutex<Progress>>, msg: S)
	where S: Into<String> {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.set_msg(msg)
	}

	/// Set Path as Message
	#[cfg(feature = "witcher")]
	pub fn set_path<P> (progress: &Arc<Mutex<Progress>>, path: P)
	where P: AsRef<Path> {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.set_path(path.as_ref())
	}

	/// Set Thread Count.
	pub fn set_threads(progress: &Arc<Mutex<Progress>>, threads: u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.set_threads(threads);
	}

	/// Finish.
	pub fn finish(progress: &Arc<Mutex<Progress>>) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.finish()
	}

	/// Tick.
	pub fn tick(progress: &Arc<Mutex<Progress>>) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.tick()
	}
}
