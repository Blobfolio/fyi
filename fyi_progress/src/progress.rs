/*!
# FYI Progress
*/

use ahash::AHasher;
use crate::{
	NiceElapsed,
	NiceInt,
	utility,
};
use fyi_msg::{
	Msg,
	MsgKind,
	utility::time_format_dd,
};
use rayon::prelude::*;
use std::{
	cmp::{
		Ordering,
		PartialEq,
	},
	ffi::OsStr,
	hash::Hasher,
	io::{
		self,
		Write,
	},
	ops::Deref,
	path::PathBuf,
	sync::{
		Arc,
		Mutex,
	},
	thread,
	time::{
		Duration,
		Instant,
	},
};



/// Helper: Unlock the inner Mutex, handling poisonings inasmuch as is
/// possible.
macro_rules! mutex_ptr {
	($mutex:expr) => (
		match $mutex.lock() {
			Ok(guard) => guard,
			Err(poisoned) => poisoned.into_inner(),
		}
	);
}



/// Progress Bar Flags.
///
/// These are used to keep track of changed (raw) values so that when tick time
/// comes around, the corresponding buffers can be redrawn.
const FLAG_ALL: u8 =          0b0111_1111;
const FLAG_MOVED: u8 =        0b0000_1111;
const FLAG_RESIZED: u8 =      0b0001_0011;

const FLAG_RUNNING: u8 =      0b0100_0000;

const FLAG_TICK_BAR: u8 =     0b0000_0001;
const FLAG_TICK_DOING: u8 =   0b0000_0010;
const FLAG_TICK_DONE: u8 =    0b0000_0100;
const FLAG_TICK_PERCENT: u8 = 0b0000_1000;
const FLAG_TICK_TITLE: u8 =   0b0001_0000;
const FLAG_TICK_TOTAL: u8 =   0b0010_0000;

/// Buffer Indexes.
///
/// These are the indexes of the individual buffer pieces. Every other entry is
/// constant, but commented below for reference.
const IDX_TITLE: usize = 0;
// const IDX_ELAPSED_PRE: usize = 1;
const IDX_ELAPSED: usize = 2;
// const IDX_ELAPSED_POST: usize = 3;
const IDX_BAR: usize = 4;
// const IDX_DONE_PRE: usize = 5;
const IDX_DONE: usize = 6;
// const IDX_DONE_POST: usize = 7;
const IDX_TOTAL: usize = 8;
// const IDX_TOTAL_POST: usize = 9;
const IDX_PERCENT: usize = 10;
// const IDX_PERCENT_POST: usize = 11;
const IDX_DOING: usize = 12;



#[derive(Debug, Clone)]
/// Inner Progress.
///
/// This struct holds the "stateful" data for a `Progress`.
struct ProgressInner<T>
where T: ProgressTask + PartialEq + Clone {
	/// Current tasks.
	doing: Vec<T>,
	/// Amount complete.
	done: u32,
	/// Total amount.
	total: u32,
	/// The initiation time.
	time: Instant,
	/// Progress bar title.
	title: Option<Msg>,
	/// Formatted progress bar components.
	///
	/// Each section of the progress bar is stored in its own array slot where
	/// it can be edited independently of the others. Printing still requires
	/// concatenation, but this lets us rest halfway.
	buf: [Vec<u8>; 13],
	/// Flags.
	///
	/// These flags mostly track state changes by field since the last tick.
	/// This way, the buffer need not be rewritten on each individual update,
	/// but once — for changed fields only — during the global `tick()` call.
	flags: u8,
	/// Hash.
	///
	/// This is a hash of the last buffer sent for print, allowing us to avoid
	/// duplicate consecutive print jobs.
	last_hash: u64,
	/// Lines Printed.
	///
	/// This keeps track of the number of lines in the last print job so that
	/// as many lines can be erased before starting the next print job.
	last_lines: u8,
	/// Elapsed Time.
	///
	/// This stores the number of seconds elapsed at the time of the last
	/// print. This is compared against the current elapsed seconds from `time`
	/// to see if the buffer requires an update.
	last_secs: u32,
	/// Screen Width.
	///
	/// This keeps track of the terminal width from the last print as a change
	/// could require redrawing portions of the progress bar whose values may
	/// remain the same.
	last_width: u32,
}

impl<T> Default for ProgressInner<T>
where T: ProgressTask + PartialEq + Clone {
	/// Default.
	///
	/// The default is empty with a few of the constant (formatting-related)
	/// pieces pre-entered.
	fn default() -> Self {
		Self {
			doing: Vec::new(),
			done: 0,
			total: 0,
			time: Instant::now(),
			title: None,
			buf: [
				// Title.
				Vec::new(),

				// Elapsed.
				//   \e   [   2    m   [   \e  [   0   ;   1    m
				vec![27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 109],
				//    0   0   :   0   0   :   0   0
				vec![48, 48, 58, 48, 48, 58, 48, 48],
				//   \e   [   0   ;   2    m   ]  \e   [   0    m   •   •
				vec![27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32],

				// Bar. This is a hugely complicated part, so we'll handle it
				// separately.
				Vec::new(),

				// Done.
				//   \e   [   1   ;   9   6    m
				vec![27, 91, 49, 59, 57, 54, 109],
				//    0
				vec![48],

				// The slash between Done and Total.
				//   \e   [   0   ;   2    m   /  \e   [   0   ;   1   ;   3   4    m
				vec![27, 91, 48, 59, 50, 109, 47, 27, 91, 48, 59, 49, 59, 51, 52, 109],

				// Total.
				//    0
				vec![48],

				// The bit between Total and Percent.
				//   \e   [   0   ;   1    m   •   •
				vec![27, 91, 48, 59, 49, 109, 32, 32],

				// Percent.
				//    0   .   0   0   %
				vec![48, 46, 48, 48, 37],
				//   \e   [   0    m  \n
				vec![27, 91, 48, 109, 10],

				// Tasks.
				Vec::new(),
			],
			flags: 0,
			last_hash: 0,
			last_lines: 0,
			last_secs: 0,
			last_width: 0,
		}
	}
}

impl<T> ProgressInner<T>
where T: ProgressTask + PartialEq + Clone {
	// ------------------------------------------------------------------------
	// Getters
	// ------------------------------------------------------------------------

	/// Number Done.
	pub fn done(&self) -> u32 { self.done }

	/// Elapsed (Seconds).
	pub fn elapsed(&self) -> u64 { self.time.elapsed().as_secs() }

	/// Percent Done.
	pub fn percent(&self) -> f32 {
		if self.total == 0 || self.done == 0 { 0.0 }
		else if self.done == self.total { 1.0 }
		else { self.done as f32 / self.total as f32 }
	}

	/// Total.
	pub fn total(&self) -> u32 { self.total }



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	/// Remove A Task.
	///
	/// Remove a task from the currently-running list, and increment `done` by
	/// one.
	pub fn remove_doing(&mut self, task: &T) {
		if let Some(idx) = self.doing.iter().position(|x| x == task) {
			self.flags |= FLAG_MOVED;
			self.doing.remove(idx);
			self.set_done(self.done + 1);
		}
	}

	/// Add A Task.
	///
	/// A new task to the currently-running list.
	pub fn set_doing(&mut self, task: T) {
		if ! self.doing.contains(&task) {
			self.flags |= FLAG_MOVED;
			self.doing.push(task);
		}
	}

	/// Set Done.
	///
	/// Update the amount done. Once this value reaches or exceeds the total,
	/// the progress bar is stopped.
	fn set_done(&mut self, done: u32) {
		let done: u32 = self.total.min(done);
		if done != self.done {
			if done == self.total {
				self.stop();
			}
			else {
				self.flags |= FLAG_MOVED;
				self.done = done;
			}
		}
	}

	/// Update the title.
	pub fn set_title(&mut self, title: Option<Msg>) {
		self.flags |= FLAG_TICK_TITLE;
		self.title = title;
	}

	/// Stop Progress.
	///
	/// This operation disables the running state. More specifically, it will
	/// reset the flags, set `done` equal to `total`, clear the title and
	/// tasks, and erase any output from the screen.
	pub fn stop(&mut self) {
		self.flags = FLAG_ALL & ! FLAG_RUNNING;
		self.done = self.total;
		self.doing.truncate(0);
		self.title = None;

		self.preprint(&[]);
	}



	// ------------------------------------------------------------------------
	// Tick
	// ------------------------------------------------------------------------

	/// Tick.
	///
	/// This method will rewrite and print the buffer (e.g. progress bar) if
	/// any of that data changed since the last call.
	///
	/// If the progress bar is inactive, no action is taken.
	///
	/// This method returns `true` if a print was at least considered, `false`
	/// if the instance is inactive.
	pub fn tick(&mut self) -> bool {
		// We aren't running!
		if 0 == self.flags & FLAG_RUNNING {
			return false;
		}

		// Check the current terminal width first as that affects a lot of what
		// follows.
		self.tick_set_width();

		// We can't really draw anything meaningful in small spaces.
		if self.last_width < 40 {
			self.flags = FLAG_RUNNING;
			self.preprint(&[]);
			return true;
		}

		// If the time hasn't changed, and nothing else has changed, we can
		// abort without all the tedious checking.
		if ! self.tick_set_secs() && self.flags == FLAG_RUNNING {
			return true;
		}

		// Handle the rest!
		self.tick_set_doing();
		self.tick_set_done();
		self.tick_set_percent();
		self.tick_set_title();
		self.tick_set_total();

		// The bar's width depends on how much space remains after the other
		// elements sharing its line. That's why it goes last.
		self.tick_set_bar();

		// Maybe we're printing, maybe we're not!
		self.preprint(&self.buf.concat());

		true
	}

	/// Tick Bar Dimensions.
	///
	/// This calculates the available widths for each of the three progress
	/// bars (done, doing, remaining).
	///
	/// If the total available space winds up being less than 10, all three
	/// values are set to zero, indicating this component should be removed.
	fn tick_bar_widths(&self) -> (u8, u8, u8) {
		// The magic "11" is made up of the following hard-coded pieces:
		// 2: braces around elapsed time;
		// 2: spaces after elapsed time;
		// 1: the "/" between done and total;
		// 2: the spaces after total;
		// 2: the braces around the bar itself (should there be one);
		// 2: the spaces after the bar itself (should there be one);
		let space: u8 = 255_usize.min((self.last_width as usize).saturating_sub(
			11 +
			self.buf[IDX_ELAPSED].len() +
			self.buf[IDX_DONE].len() +
			self.buf[IDX_TOTAL].len() +
			self.buf[IDX_PERCENT].len()
		)) as u8;

		// Insufficient space!
		if space < 10 { (0, 0, 0) }
		// Done!
		else if self.done == self.total { (space, 0, 0) }
		// Working on it!
		else {
			let done: u8 = f64::floor(f64::from(self.done) / f64::from(self.total) * f64::from(space)) as u8;
			let doing: u8 = f64::floor(self.doing.len() as f64 / f64::from(self.total) * f64::from(space)) as u8;
			let undone: u8 = space - doing - done;
			(done, doing, undone)
		}
	}

	/// Tick Bar.
	///
	/// This redraws the actual progress *bar* portion of the buffer, which is
	/// actually three different bars squished together: Done, Doing, and
	/// Pending.
	///
	/// The combined width of the `###` will never exceed 255, and will never
	/// be less than 10.
	fn tick_set_bar(&mut self) {
		if 0 != self.flags & FLAG_TICK_BAR {
			self.flags &= ! FLAG_TICK_BAR;
			self.buf[IDX_BAR].truncate(0);
			let (done, doing, undone) = self.tick_bar_widths();
			if done + doing + undone == 0 { return; }

			if done == 0 { self.buf[IDX_BAR].extend_from_slice(b"\x1b[2m["); }
			else {
				self.buf[IDX_BAR].extend_from_slice(b"\x1b[2m[\x1b[0;1;96m");
				self.buf[IDX_BAR].resize(self.buf[IDX_BAR].len() + done as usize, b'#');
			}

			if doing != 0 {
				self.buf[IDX_BAR].extend_from_slice(b"\x1b[0;1;95m");
				self.buf[IDX_BAR].resize(self.buf[IDX_BAR].len() + doing as usize, b'#');
			}

			if undone != 0 {
				self.buf[IDX_BAR].extend_from_slice(b"\x1b[0;1;34m");
				self.buf[IDX_BAR].resize(self.buf[IDX_BAR].len() + undone as usize, b'#');
			}

			// Always close it off.
			self.buf[IDX_BAR].extend_from_slice(b"\x1b[0;2m]\x1b[0m  ");
		}
	}

	/// Tick Doing.
	///
	/// Update the task list portion of the buffer. This is triggered both by
	/// changes to the task list as well as resoluation changes (as long values
	/// may require lazy cropping).
	fn tick_set_doing(&mut self) {
		if 0 != self.flags & FLAG_TICK_DOING {
			self.flags &= ! FLAG_TICK_DOING;
			self.buf[IDX_DOING].truncate(0);
			let w: usize = self.last_width as usize;
			self.buf[IDX_DOING].extend(self.doing.iter().flat_map(|x| x.task_line(w)));
		}
	}

	/// Tick Done.
	///
	/// This updates the "done" portion of the buffer as needed.
	fn tick_set_done(&mut self) {
		if 0 != self.flags & FLAG_TICK_DONE {
			self.flags &= ! FLAG_TICK_DONE;
			self.buf[IDX_DONE].truncate(0);
			self.buf[IDX_DONE].extend_from_slice(&*NiceInt::from(self.done));
		}
	}

	/// Tick Percent.
	///
	/// This updates the "percent" portion of the buffer as needed.
	fn tick_set_percent(&mut self) {
		if 0 != self.flags & FLAG_TICK_PERCENT {
			self.flags &= ! FLAG_TICK_PERCENT;
			self.buf[IDX_PERCENT].truncate(0);
			self.buf[IDX_PERCENT].extend_from_slice(
				format!("{:>3.*}%", 2, self.percent() * 100.0).as_bytes()
			);
		}
	}

	/// Tick Elapsed Seconds.
	///
	/// The precision of `Instant` is greater than we need for printing
	/// purposes; here we're just looking to see if one or more seconds have
	/// elapsed since the last tick.
	///
	/// Because this is relative to the tick rather than the overall state of
	/// progress, it has no corresponding tick flag. It simply always runs.
	fn tick_set_secs(&mut self) -> bool {
		let secs: u32 = 86400.min(self.elapsed() as u32);
		if secs == self.last_secs { false }
		else {
			self.last_secs = secs;
			// The value is capped at 86400, i.e. one day.
			if secs == 86400 {
				self.buf[IDX_ELAPSED].copy_from_slice(b"23:59:59");
			}
			// For everything else, we need to parse it into bigger units.
			else {
				let c = utility::secs_chunks(secs);
				self.buf[IDX_ELAPSED][..2].copy_from_slice(time_format_dd(c[0]));
				self.buf[IDX_ELAPSED][3..5].copy_from_slice(time_format_dd(c[1]));
				self.buf[IDX_ELAPSED][6..].copy_from_slice(time_format_dd(c[2]));
			}

			true
		}
	}

	/// Tick Title.
	///
	/// The title needs to be rewritten both on direct change and resolution
	/// change. Long titles are lazy-cropped as needed.
	fn tick_set_title(&mut self) {
		if 0 != self.flags & FLAG_TICK_TITLE {
			self.flags &= ! FLAG_TICK_TITLE;
			self.buf[IDX_TITLE].truncate(0);

			if let Some(t) = &self.title {
				let mut vt: Vec<u8> = t.to_vec();

				if self.last_width as usize <= vt.len() {
					// An easy, if inefficient, way to locate the last allowable
					// char boundary.
					let tmp: String = unsafe { String::from_utf8_unchecked(vt.clone()) }
						.chars()
						.take(self.last_width as usize - 1)
						.collect();

					vt.truncate(tmp.len());
				}

				// Add a line break.
				vt.push(10);

				// Update the buffer.
				self.buf[IDX_TITLE].extend(vt);
			}
		}
	}

	/// Tick Total.
	///
	/// This updates the "total" portion of the buffer as needed.
	fn tick_set_total(&mut self) {
		if 0 != self.flags & FLAG_TICK_TOTAL {
			self.flags &= ! FLAG_TICK_TOTAL;
			self.buf[IDX_TOTAL].truncate(0);
			self.buf[IDX_TOTAL].extend_from_slice(&*NiceInt::from(self.total));
		}
	}

	/// Tick Width.
	///
	/// Check to see if the terminal width has changed since the last run and
	/// update values — i.e. the relevant tick flags — as necessary.
	fn tick_set_width(&mut self) {
		let width: u32 = utility::term_width() as u32;
		if width != self.last_width {
			self.flags |= FLAG_RESIZED;
			self.last_width = width;
		}
	}



	// ------------------------------------------------------------------------
	// Render
	// ------------------------------------------------------------------------

	/// Preprint.
	///
	/// This method accepts a completed buffer ready for printing, hashing it
	/// for comparison with the last job. If unique, the previous output is
	/// erased and replaced with the new output.
	fn preprint(&mut self, buf: &[u8]) {
		// Make sure the content is unique, otherwise we can leave the old bits
		// up.
		let hash: u64 = {
			let mut hasher = AHasher::default();
			hasher.write(buf);
			hasher.finish()
		};
		if hash == self.last_hash {
			return;
		}
		self.last_hash = hash;

		// Erase old lines if needed.
		self.print_cls();

		// Update the line count and print!
		if ! buf.is_empty() {
			self.last_lines = 1_u8.saturating_add(bytecount::count(buf, b'\n') as u8);
			Self::print(buf);
		}
	}

	/// Print!
	///
	/// Print some arbitrary data to the write place. Haha.
	///
	/// `Stderr` is used as the output device in production, but if the
	/// `bench_sink` feature is enabled, output will be sent to `io::sink()`
	/// instead. As the feature name suggests, this is only really useful for
	/// measuring timings.
	fn print(buf: &[u8]) {
		#[cfg(not(feature = "bench_sink"))] let writer = io::stderr();
		#[cfg(not(feature = "bench_sink"))] let mut handle = writer.lock();
		#[cfg(feature = "bench_sink")] let mut handle = io::sink();

		handle.write_all(buf).unwrap();
		handle.flush().unwrap();
	}

	/// Erase Output.
	///
	/// This method "erases" any prior output so that new output can be written
	/// in the same place. That's animation, folks!
	fn print_cls(&mut self) {
		// Buffer 10 Line Clears.
		// 0..10 moves the cursor left. This is done only once per reset.
		// 14 is the length of each subsequent command, which moves the cursor up.
		// To clear "n" lines, then, slice [0..(10 + 14 * n)].
		static CLS10: [u8; 150] = [27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75, 27, 91, 49, 65, 27, 91, 49, 48, 48, 48, 68, 27, 91, 75];

		if self.last_lines > 0 {
			// Figure out how to slice our `CLS10` buffer.
			match self.last_lines.cmp(&10) {
				Ordering::Equal => { Self::print(&CLS10[..]); },
				Ordering::Less => {
					let end: usize = 10 + 14 * self.last_lines as usize;
					Self::print(&CLS10[0..end]);
				},
				// To clear more lines, print our pre-calculated buffer (which
				// covers the first 10), and duplicate the line-up chunk (n-10)
				// times to cover the rest.
				Ordering::Greater => {
					Self::print(
						&CLS10.iter()
							.chain(&CLS10[14..28].repeat(self.last_lines as usize - 10))
							.copied()
							.collect::<Vec<u8>>()
					);
				},
			}

			// Having cleared whatever it was, there are now no last_lines.
			self.last_lines = 0;
		}
	}

	/// Print Generic Finish Message.
	///
	/// This method can be called after a progress bar has finished to
	/// summarize what went down.
	///
	/// If the total is zero, a warning is printed instead.
	pub fn print_summary<S> (&mut self, one: S, many: S)
	where S: AsRef<str> {
		if 0 == self.flags & FLAG_RUNNING {
			// Print a warning.
			if self.total == 0 {
				Msg::from([
					b"No ",
					many.as_ref().as_bytes(),
					b" were found.\n",
				].concat())
					.with_prefix(MsgKind::Warning)
			}
			else {
				Msg::from([
					&utility::inflect(u64::from(self.total), one, many),
					&b" in "[..],
					&*NiceElapsed::from(self.elapsed() as u32),
					&[46, 10],
				].concat())
					.with_prefix(MsgKind::Done)
			}
			.eprint()
		}
	}
}



#[derive(Debug, Default)]
/// Progress Bar.
///
/// This is a very crude wrapper around a `Vec` of progressable items and a
/// thread-safe `ProgressInner`, the latter stored behind an `Arc<Mutex>>`.
pub struct Progress<T>
where T: ProgressTask + PartialEq + Clone {
	/// The set to progress through.
	set: Vec<T>,
	/// The stateful data.
	inner: Arc<Mutex<ProgressInner<T>>>,
}

impl<T> From<Vec<T>> for Progress<T>
where T: ProgressTask + PartialEq + Clone {
	fn from(src: Vec<T>) -> Self {
		let total: u32 = src.len() as u32;
		let mut flags: u8 = FLAG_ALL;
		if total == 0 {
			flags &= ! FLAG_RUNNING;
		}

		Self {
			set: src,
			inner: Arc::new(Mutex::new(ProgressInner::<T> {
				total,
				flags,
				..ProgressInner::<T>::default()
			})),
		}
	}
}

impl<T> Deref for Progress<T>
where T: ProgressTask + PartialEq + Clone {
	type Target = [T];

	/// Deref to Slice.
	fn deref(&self) -> &Self::Target { &self.set }
}

impl<T> Progress<T>
where T: ProgressTask + PartialEq + Clone + Sync + Send + 'static {
	#[must_use]
	/// New Progress.
	///
	/// Start a new progress bar with a dataset and title. If you don't need a
	/// title, just use `Progress::from()` instead.
	pub fn new(src: Vec<T>, title: Msg) -> Self {
		let total: u32 = src.len() as u32;
		let mut flags: u8 = FLAG_ALL;
		if total == 0 {
			flags &= ! FLAG_RUNNING;
		}

		Self {
			set: src,
			inner: Arc::new(Mutex::new(ProgressInner::<T> {
				total,
				flags,
				title: Some(title),
				..ProgressInner::<T>::default()
			})),
		}
	}

	/// Run!
	///
	/// This here is the whole point. Iterate through the set in parallel with
	/// a progress bar, while executing a custom callback.
	pub fn run<F>(&self, cb: F)
	where F: Fn(&T) + Copy + Send + Sync {
		// If the set is empty, skip all this nonsense.
		if self.set.is_empty() { return; }

		{
			// This extra process gives us a steady tick, ensuring slow tasks
			// don't make the user think everything's crashed.
			let inner = self.inner.clone();
			rayon::spawn(move || {
				let sleep = Duration::from_millis(60);
				loop {
					if ! Self::tick(&inner) {
						break;
					}
					thread::sleep(sleep);
				}
			});

			// Iterate!
			self.set.par_iter().for_each(|x| {
				// Mark the task as currently running.
				self.set_doing(x.clone());

				// Do whatever.
				cb(x);

				// Mark the task as complete.
				self.remove_doing(x);
				Self::tick(&self.inner);
			});
		}
	}

	/// Silent Run.
	///
	/// This simply loops the dataset in parallel, applying the custom callback
	/// for each.
	///
	/// It utterly defeats the purpose of having a progress bar (since none is
	/// shown), but potentially saves the implementing library having to list
	/// `rayon` as a direct dependency.
	pub fn silent<F>(&self, cb: F)
	where F: Fn(&T) + Copy + Send + Sync {
		self.set.par_iter().for_each(cb);
	}



	// ------------------------------------------------------------------------
	// `ProgressInner` Wrappers
	// ------------------------------------------------------------------------

	#[must_use]
	/// Get Doing.
	pub fn doing(&self) -> u32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.doing.len() as u32
	}

	#[must_use]
	/// Get Done.
	///
	/// Wrapper for `ProgressInner::done()`.
	pub fn done(&self) -> u32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.done()
	}

	#[must_use]
	/// Get Elapsed.
	///
	/// Wrapper for `ProgressInner::elapsed()`.
	pub fn elapsed(&self) -> u64 {
		let ptr = mutex_ptr!(self.inner);
		ptr.elapsed()
	}

	#[must_use]
	/// Get Percent.
	///
	/// Wrapper for `ProgressInner::percent()`.
	pub fn percent(&self) -> f32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.percent()
	}

	/// Print Finish Message.
	///
	/// Wrapper for `ProgressInner::print_summary()`.
	pub fn print_summary<S> (&self, one: S, many: S)
	where S: AsRef<str> {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.print_summary(one, many);
	}

	/// Remove a Task.
	///
	/// Wrapper for `ProgressInner::remove_doing()`.
	fn remove_doing(&self, task: &T) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.remove_doing(task);
	}

	/// Add a Task.
	///
	/// Wrapper for `ProgressInner::set_doing()`.
	fn set_doing(&self, task: T) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.set_doing(task);
	}

	/// Set Title.
	///
	/// Wrapper for `ProgressInner::set_title()`.
	pub fn set_title(&self, title: Option<Msg>) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.set_title(title);
	}

	/// Steady Tick.
	///
	/// Wrapper for `ProgressInner::tick()`.
	fn tick(inner: &Arc<Mutex<ProgressInner<T>>>) -> bool {
		let mut ptr = mutex_ptr!(inner);
		ptr.tick()
	}

	#[must_use]
	/// Get Total.
	///
	/// Wrapper for `ProgressInner::total()`.
	pub fn total(&self) -> u32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.total()
	}
}



/// Progressable Type.
///
/// Types with this trait may be used to seed a `Progress`. Right now that
/// means strings and paths.
pub trait ProgressTask {
	/// Task Name.
	fn task_name(&self) -> &[u8];

	/// The Full Task Line.
	///
	/// This combines an indented arrow with the task name, lazy chopping long
	/// values as needed.
	fn task_line(&self, width: usize) -> Vec<u8> {
		let name: &[u8] = self.task_name();
		[
			// •   •   •   •  \e   [   3   5    m   ↳  ---  ---   •
			&[32, 32, 32, 32, 27, 91, 51, 53, 109, 226, 134, 179, 32][..],
			&name[0..name.len().min(width.saturating_sub(9))],
			b"\x1b[0m\n",
		].concat()
	}
}

impl ProgressTask for PathBuf {
	#[allow(trivial_casts)]
	/// Task Name.
	fn task_name(&self) -> &[u8] {
		unsafe { &*(self.as_os_str() as *const OsStr as *const [u8]) }
	}
}

impl ProgressTask for &str {
	/// Task Name.
	fn task_name(&self) -> &[u8] { self.as_bytes() }
}

impl ProgressTask for String {
	/// Task Name.
	fn task_name(&self) -> &[u8] { self.as_bytes() }
}
