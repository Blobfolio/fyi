/*!
# FYI Core: Progress Bar

This is a very simple thread-capable CLI progress indicator.
*/

use ansi_term::{
	Colour,
	Style,
};
use crate::misc::{
	cli,
	strings::{
		self,
		FYIStringFormat,
	},
	time,
};
use crate::msg::Msg;
use std::{
	borrow::Cow,
	collections::HashSet,
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		Mutex,
		atomic::{
			AtomicBool,
			AtomicU8,
			AtomicU64,
			Ordering,
		},
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



// Progress bar characters.
pub const CHR_DONE: &str = "◼";
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
	/// The total total.
	total: Arc<AtomicU64>,

	#[def = "Mutex::new(HashSet::new())"]
	/// Working Paths.
	working: Mutex<HashSet<PathBuf>>,
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

		{
			let mut ptr = self.working.lock().expect("Failed to acquire lock: Progress.working");
			ptr.clear();
		}

		self.flags.store(flags, Ordering::SeqCst);
		self.last_lines.store(0, Ordering::SeqCst);
		self.running.store(0 < total, Ordering::SeqCst);
		self.set_msg(msg);
		self.done.store(0, Ordering::SeqCst);
		self.total.store(total, Ordering::SeqCst);
	}



	// -----------------------------------------------------------------
	// Public Getters
	// -----------------------------------------------------------------

	/// Is Running
	pub fn is_running(&self) -> bool {
		self.running.load(Ordering::SeqCst)
	}

	/// Tick state.
	pub fn progress(&self) -> (u64, u64) {
		(self.done(), self.total())
	}



	// -----------------------------------------------------------------
	// Ops
	// -----------------------------------------------------------------

	/// Remove working path.
	pub fn add_working<P> (&self, path: P)
	where P: AsRef<Path> {
		let path: PathBuf = if cfg!(feature = "witcher") {
			use crate::witcher::formats::FYIPathFormat;
			path.as_ref().fyi_to_path_buf_abs()
		} else {
			path.as_ref().to_path_buf()
		};

		let mut ptr = self.working.lock().expect("Failed to acquire lock: Progress.working");
		ptr.insert(path);
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

	/// Remove working path.
	pub fn remove_working<P> (&self, path: P)
	where P: AsRef<Path> {
		let path: PathBuf = if cfg!(feature = "witcher") {
			use crate::witcher::formats::FYIPathFormat;
			path.as_ref().fyi_to_path_buf_abs()
		} else {
			path.as_ref().to_path_buf()
		};

		let mut ptr = self.working.lock().expect("Failed to acquire lock: Progress.working");
		ptr.remove(&path);
	}

	/// Set Done.
	pub fn set_done(&self, mut done: u64) {
		let total = self.total();
		if total <= done {
			done = total;
		}

		if done != self.done() {
			self.done.store(done, Ordering::SeqCst);

			if done == total {
				self.stop();
			}
		}
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

	/// Tick.
	pub fn tick(&self) {
		// Build the new message.
		let width = cli::term_width();
		let done = self.done();
		let total = self.total();

		// Start building the second line first.
		let p_count = self.part_count(done, total);
		let p_percent = self.part_percent(done, total);
		let p_elapsed = self.part_elapsed();
		let mut p_eta = self.part_eta(done, total);

		// How much space have we used?
		let p_space = p_elapsed.fyi_width() +
			p_count.fyi_width() +
			&p_percent.fyi_width() +
			3;

		// The bar can have the remaining space.
		let p_bar_len = {
			let mut count = width - p_space;
			if count > 60 {
				count = 60;
			}

			// Adjust the ETA again.
			let eta_len = p_eta.fyi_width();
			if 0 != eta_len && p_space + count + eta_len + 1 <= width {
				*(p_eta.to_mut()) = format!(
					"{}{}",
					strings::whitespace(width - eta_len - count - p_space),
					&p_eta
				);
			}
			else if 0 != eta_len {
				*(p_eta.to_mut()) = String::new();
			}

			count
		};
		let p_bar = self.part_bar(done, total, p_bar_len);

		// Let's go ahead and make this line.
		let mut out: String = [
			&p_elapsed,
			" ",
			&p_bar,
			" ",
			&p_count,
			" ",
			&p_percent,
			&p_eta
		].concat();

		// Is there a message to prepend to it?
		let p_msg = self.part_msg(width);
		if false == p_msg.is_empty() {
			out = [
				&p_msg,
				"\n",
				&out
			].concat();
		}

		// How about working paths to add to it?
		let p_working = self.part_working(width);
		if false == p_working.is_empty() {
			out = [
				&out,
				"\n",
				&p_working
			].concat();
		}

		// Send it to the printer!
		self.print(out);
	}

	/// Increment, set message, remove working path.
	pub fn update(
		&self,
		interval: u64,
		msg: Option<String>,
		working: Option<PathBuf>
	) {
		self.set_done(self.done() + interval);
		if let Some(s) = msg {
			self.set_msg(s);
		}
		if let Some(w) = working {
			self.remove_working(w);
		}
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

		let nlines: u8 = msg.fyi_lines_len() as u8;
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
	fn part_bar(&self, done: u64, total: u64, mut width: usize) -> Cow<'_, str> {
		// Early abort.
		if 10 > width {
			return Cow::Borrowed("");
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

		// Now we can do the lengths!
		let done_len: usize = f64::floor(done_percent * width as f64) as usize;
		let pending_len: usize = width - done_len;

		// And now we can build the base strings.
		let done_str: String = match done_len {
			0 => String::new(),
			x => CHR_DONE.repeat(x),
		};
		let pending_str: String = match pending_len {
			0 => String::new(),
			x => CHR_PENDING.repeat(x),
		};

		// And now we can send pretty stuff back.
		Cow::Owned(format!(
			"{}{}{}{}",
			Style::new().dimmed().paint("["),
			Colour::Cyan.bold().paint(&done_str),
			Colour::Cyan.dimmed().paint(&pending_str),
			Style::new().dimmed().paint("]"),
		))
	}

	/// Tick count.
	fn part_count(&self, done: u64, total: u64) -> Cow<'_, str> {
		if 0 == total {
			Cow::Borrowed("")
		}
		else {
			Cow::Owned(format!(
				"{}{}{}",
				Colour::Cyan.bold().paint(done.to_string()),
				Style::new().dimmed().paint("/"),
				Colour::Cyan.dimmed().paint(total.to_string())
			))
		}
	}

	/// Tick elapsed.
	fn part_elapsed(&self) -> Cow<'_, str> {
		let ptr = self.time.lock().expect("Failed to acquire lock: Progress.time");
		let elapsed = time::human_elapsed(
			ptr.elapsed().as_secs() as usize,
			crate::PRINT_COMPACT
		);

		Cow::Owned(format!(
			"{}{}{}",
			Style::new().dimmed().paint("["),
			Style::new().bold().paint(&elapsed.to_string()),
			Style::new().dimmed().paint("]"),
		))
	}

	/// ETA.
	fn part_eta(&self, done: u64, total: u64) -> Cow<'_, str> {
		let done: f64 = done as f64;
		let total: f64 = total as f64;

		// Abort if no progress has been made.
		if done < 2.0 || done >= total {
			return Cow::Borrowed("");
		}

		let elapsed: f64 = {
			let ptr = self.time.lock().expect("Failed to acquire lock: Progress.time");
			ptr.elapsed().as_secs_f64()
		};

		// Abort if we haven't spent ten seconds doing anything yet.
		if elapsed < 10.0 {
			return Cow::Borrowed("");
		}

		let s_per: f64 = elapsed / done;
		let remaining = time::human_elapsed(
			f64::ceil(s_per * (total - done)) as usize,
			crate::PRINT_COMPACT
		);

		Cow::Owned(format!(
			"{} {}",
			Colour::Purple.dimmed().paint("ETA:"),
			Colour::Purple.bold().paint(&remaining.to_string()),
		))
	}

	/// Tick message.
	fn part_msg(&self, width: usize) -> Cow<'_, str> {
		let ptr = self.msg.lock().expect("Failed to acquire lock: Progress.msg");
		if ptr.is_empty() {
			Cow::Borrowed("")
		}
		else {
			let tmp: String = ptr.fyi_shorten(width).into();
			Cow::Owned(tmp)
		}
	}

	/// Tick percent.
	fn part_percent(&self, done: u64, total: u64) -> Cow<'_, str> {
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

		Cow::Owned(format!(
			"{}",
			Style::new().bold().paint(format!("{:>3.*}%", 2, percent * 100.0))
		))
	}

	/// Tick working.
	fn part_working(&self, width: usize) -> Cow<'_, str> {
		let ptr = self.working.lock().expect("Failed to acquire lock: Progress.working");
		if ptr.is_empty() {
			return Cow::Borrowed("")
		}

		let mut out: Vec<String> = ptr.iter()
			.cloned()
			.map(|ref x| {
				let out: String = format!(
					"    {} {}",
					Colour::Purple.dimmed().paint("↳"),
					Colour::Purple.dimmed().paint(x.to_str().unwrap_or("")),
				);
				out.fyi_shorten(width).to_string()
			})
			.collect();

		out.sort();
		Cow::Owned(out.join("\n"))
	}
}



/// Arc wrappers.
pub mod arc {
	use super::*;

	/// Remove working path.
	pub fn add_working<P> (progress: &Arc<Mutex<Progress>>, path: P)
	where P: AsRef<Path> {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.add_working(path)
	}

	/// Finish.
	pub fn finish(progress: &Arc<Mutex<Progress>>) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.finish()
	}

	/// Increment Done.
	pub fn increment(progress: &Arc<Mutex<Progress>>, interval: u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.increment(interval)
	}

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

	/// Tick progress.
	pub fn progress(progress: &Arc<Mutex<Progress>>) -> (u64, u64) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.progress()
	}

	/// Remove working path.
	pub fn remove_working<P> (progress: &Arc<Mutex<Progress>>, path: P)
	where P: AsRef<Path> {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.remove_working(&path)
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

	/// Tick.
	pub fn tick(progress: &Arc<Mutex<Progress>>) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.tick()
	}

	/// Increment and set message.
	pub fn update(
		progress: &Arc<Mutex<Progress>>,
		interval: u64,
		msg: Option<String>,
		working: Option<PathBuf>
	) {
		let ptr = progress.lock().expect("Failed to acquire lock: Progress");
		ptr.update(interval, msg, working)
	}
}
