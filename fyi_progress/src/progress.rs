/*!
# FYI Progress

The `Progress` struct is a simple wrapper that prints an animated progress bar
to `Stderr` during iteration over a supported set (`Vec<&str>`, `Vec<String>`,
or `Vec<PathBuf>`).

It is thread-safe and applies the specified callback against each entry in
parallel.

It is relatively lightweight compared to crates like `indicatif`, but this
largely due to its lack of customizability. All progress bars display elapsed
time, running counts of done over total, and a percentage. If space allows, an
ASCII art progress bar is also printed. An optional title, if present, is
printed before the main progress line, and if executing tasks in parallel, a
list of the currently running tasks is printed after the main line.

## Examples:

```no_run
use fyi_msg::MsgKind;
use fyi_progress::Progress;
use fyi_progress::utility;

// An example callback function.
fn tally_food(food: &str) { ... }

let p1 = Progress::from(vec!["Apples", "Bananas", "Carrots"]);
p1.run(|f| tally_food(f));
p1.print_summary();

// Progress bars can also have a title:
let p2 = Progress::new(
    vec!["Apples", "Bananas", "Carrots"],
    MsgKind::Info.into_msg("Checking out the produce."),
);

// Parallelism can be disabled or modified. Use `0` for auto — number of
// reported threads — or any positive number for that specific number.
let p3 = Progress::from(vec!["Apples", "Bananas", "Carrots"])
    .with_threads(utility::num_threads() * 2);

// If you want to iterate a collection *without* a progress bar, but using the
// built-in parallelism, you could use this method instead of `run()`:
p3.silent(|f| tally_food(f));
```
*/

use ahash::{
	AHasher,
	AHashSet,
};
use crate::{
	NiceElapsed,
	NiceInt,
	traits::{
		FittedRangeMut,
		ProgressTask,
	},
	utility,
};
use fyi_msg::{
	BufRange,
	Msg,
	MsgKind,
	replace_buf_range,
	resize_buf_range,
	utility::time_format_dd,
};
use std::{
	cmp::Ordering,
	hash::{
		Hash,
		Hasher,
	},
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
	time::{
		Duration,
		Instant,
	},
};
use threadpool::ThreadPool;



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

/// Helper: Pass through a getter to the `ProgressInner`.
macro_rules! get_inner {
	($func:ident, $type:ty) => {
		#[must_use]
		/// Wrapper.
		pub fn $func(&self) -> $type {
			let ptr = mutex_ptr!(self.inner);
			ptr.$func()
		}
	};
}



/// Progress Bar Flags.
///
/// Rather than rewrite the buffer on each value change, change states are
/// tracked with these flags. If a flag is on during tick time, then the
/// corresponding buffer is updated.
const FLAGS_ALL: u8 =         0b0111_1111;
const FLAGS_DEFAULT: u8 =     0b0000_0001;
const FLAGS_NEW: u8 =         0b0110_0001;
const FLAGS_RESIZED: u8 =     0b0001_0011;

const FLAG_TICK_BAR: u8 =     0b0000_0001;
const FLAG_TICK_DOING: u8 =   0b0000_0010;
const FLAG_TICK_DONE: u8 =    0b0000_0100;
const FLAG_TICK_PERCENT: u8 = 0b0000_1000;
const FLAG_TICK_TITLE: u8 =   0b0001_0000;
const FLAG_TICK_TOTAL: u8 =   0b0010_0000;

const FLAG_RUNNING: u8 =      0b0100_0000;
const FLAG_SILENT: u8 =       0b1000_0000;

/// Buffer Indexes.
///
/// The start and end points of the malleable progress components are stored in
/// an array for easy access. These are their indexes.
const PART_TITLE: usize = 0;
const PART_ELAPSED: usize = 1;
const PART_BARS: usize = 2;
const PART_DONE: usize = 3;
const PART_TOTAL: usize = 4;
const PART_PERCENT: usize = 5;
const PART_DOING: usize = 6;

/// Misc Variables.
const MIN_BARS_WIDTH: usize = 10;
const MIN_DRAW_WIDTH: usize = 40;



#[derive(Debug)]
struct ProgressInner<T>
where T: ProgressTask {
	buf: Vec<u8>,
	toc: [BufRange; 7],
	elapsed: u32,
	last_hash: u64,
	last_lines: usize,
	last_time: u128,
	last_width: usize,

	doing: AHashSet<T>,
	done: u32,
	flags: u8,
	threads: usize,
	started: Instant,
	title: Vec<u8>,
	total: u32,
}

impl<T> Default for ProgressInner<T>
where T: ProgressTask {
	fn default() -> Self {
		Self {
			buf: vec![
			//  Title would go here.

			//  \e   [   2    m   [   \e  [   0   ;   1    m
				27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 109,
			//   0   0   :   0   0   :   0   0
				48, 48, 58, 48, 48, 58, 48, 48,
			//  \e   [   0   ;   2    m   ]  \e   [   0    m   •   •
				27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32,

			//  Bar would go here.

			//  Done.
			//  \e   [   1   ;   9   6    m
				27, 91, 49, 59, 57, 54, 109,
			//   0
				48,

			//  The slash between Done and Total.
			//  \e   [   0   ;   2    m   /  \e   [   0   ;   1   ;   3   4    m
				27, 91, 48, 59, 50, 109, 47, 27, 91, 48, 59, 49, 59, 51, 52, 109,

			//  Total.
			//   0
				48,

			//  The bit between Total and Percent.
			//  \e   [   0   ;   1    m   •   •
				27, 91, 48, 59, 49, 109, 32, 32,

			//  Percent.
			//   0   .   0   0   %
				48, 46, 48, 48, 37,
			//  \e   [   0    m  \n
				27, 91, 48, 109, 10,

			//  Doing would go here.
			],
			toc: [
				BufRange::new(0, 0),   // Title.
				BufRange::new(11, 19), // Elapsed.
				BufRange::new(32, 32), // Bar(s).
				BufRange::new(39, 40), // Done.
				BufRange::new(56, 57), // Total.
				BufRange::new(65, 70), // Percent.
				BufRange::new(75, 75), // Current Tasks.
			],
			doing: AHashSet::new(),
			done: 0,
			elapsed: 0,
			flags: FLAGS_DEFAULT,
			last_hash: 0,
			last_lines: 0,
			last_time: 0,
			last_width: 0,
			started: Instant::now(),
			threads: utility::num_threads(),
			title: Vec::new(),
			total: 0,
		}
	}
}

impl<T> ProgressInner<T>
where T: ProgressTask {
	// ------------------------------------------------------------------------
	// Getters
	// ------------------------------------------------------------------------

	/// Done.
	pub fn done(&self) -> u32 { self.done }

	/// Elapsed (Seconds).
	pub fn elapsed(&self) -> u32 {
		86400.min(self.started.elapsed().as_secs() as u32)
	}

	/// Percent.
	pub fn percent(&self) -> f64 {
		if self.total == 0 || self.done == 0 { 0.0 }
		else if self.done == self.total { 1.0 }
		else {
			f64::from(self.done) / f64::from(self.total)
		}
	}

	/// Is Running?
	pub fn is_running(&self) -> bool { 0 != self.flags & FLAG_RUNNING }

	/// Is Silent?
	pub fn is_silent(&self) -> bool { 0 != self.flags & FLAG_SILENT }

	/// Threads.
	pub fn threads(&self) -> usize { self.threads }

	/// Total.
	pub fn total(&self) -> u32 { self.total }



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	/// End Task.
	///
	/// Remove a task from the currently-running list and increment `done` by
	/// one.
	pub fn end_task(&mut self, task: &T) {
		if self.doing.remove(task) {
			self.flags |= FLAG_TICK_DOING | FLAG_TICK_BAR;
			self.increment();
		}
	}

	/// Increment.
	///
	/// Increment `done` by one. If this reaches the total, it will
	/// automatically trigger a stop.
	pub fn increment(&mut self) {
		let new_done = self.total.min(self.done + 1);
		if new_done != self.done {
			if new_done == self.total { self.stop(); }
			else {
				self.flags |= FLAG_TICK_DONE | FLAG_TICK_PERCENT | FLAG_TICK_BAR;
				self.done = new_done;
			}
		}
	}

	/// Set Threads.
	///
	/// The number of threads to use for iteration. A value of `0` implies
	/// "auto", which defaults to the number of available threads. Any other
	/// value indicates exactly that number of threads.
	///
	/// To not run anything in parallel, use a value of `1`.
	pub fn set_threads(&mut self, threads: usize) {
		self.threads = match threads {
			0 => utility::num_threads(),
			x => x,
		};
	}

	/// Set Title.
	///
	/// To remove a title, pass an empty string.
	pub fn set_title<S> (&mut self, title: S)
	where S: AsRef<str> {
		let title: &[u8] = title.as_ref().as_bytes();
		if self.title.ne(&title) {
			self.title.truncate(0);
			if ! title.is_empty() {
				self.title.extend_from_slice(title);
			}

			self.flags |= FLAG_TICK_TITLE;
		}
	}

	/// Start Task.
	///
	/// Add a task to the currently-running list.
	pub fn start_task(&mut self, task: T) {
		if self.doing.insert(task) {
			self.flags |= FLAG_TICK_DOING | FLAG_TICK_BAR;
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
	fn preprint(&mut self) {
		if self.buf.is_empty() {
			self.print_blank();
			return;
		}

		// Make sure the content is unique, otherwise we can leave the old bits
		// up.
		let hash = {
			let mut hasher = AHasher::default();
			self.buf.hash(&mut hasher);
			hasher.finish()
		};
		if hash == self.last_hash {
			return;
		}
		self.last_hash = hash;

		// Erase old lines if needed.
		self.print_cls();

		// Update the line count and print!
		self.last_lines = bytecount::count(&self.buf, b'\n');
		Self::print(&self.buf);
	}

	/// Print Blank.
	///
	/// This simply resets the hash and clears any prior output.
	fn print_blank(&mut self) {
		if self.last_hash != 0 {
			self.last_hash = 0;
		}

		self.print_cls();
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
					let end: usize = 10 + 14 * self.last_lines;
					Self::print(&CLS10[0..end]);
				},
				// To clear more lines, print our pre-calculated buffer (which
				// covers the first 10), and duplicate the line-up chunk (n-10)
				// times to cover the rest.
				Ordering::Greater => {
					Self::print(
						&CLS10.iter()
							.chain(&CLS10[14..28].repeat(self.last_lines - 10))
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
		if ! self.is_running() {
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
					&*NiceElapsed::from(self.elapsed()),
					&[46, 10],
				].concat())
					.with_prefix(MsgKind::Done)
			}
			.eprint()
		}
	}

	/// Stop.
	pub fn stop(&mut self) {
		self.flags = FLAGS_ALL & ! FLAG_RUNNING;
		self.done = self.total;
		self.doing.clear();
		self.print_blank();
	}

	/// Tick.
	///
	/// Ticking takes all of the changed values (since the last tick), updates
	/// their corresponding parts in the buffer, and prints the result, if any.
	pub fn tick(&mut self) -> bool {
		// We aren't running!
		if ! self.is_running() {
			return false;
		}
		// We aren't ticking!
		else if 0 != self.flags & FLAG_SILENT {
			return true;
		}

		// We don't want to tick too often... that will just look bad.
		let ms = self.started.elapsed().as_millis();
		if ms.saturating_sub(self.last_time) < 60 {
			return true;
		}
		self.last_time = ms;

		// Check the terminal width first because it affects most of what
		// follows.
		self.tick_set_width();
		if self.last_width < MIN_DRAW_WIDTH {
			self.flags = FLAG_RUNNING;
			self.print_blank();
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
		// elements sharing its line so it needs to go last.
		self.tick_set_bar();

		// Maybe we're printing, maybe we're not!
		self.preprint();

		true
	}

	/// Tick Bar Dimensions.
	///
	/// This calculates the available widths for each of the three progress
	/// bars (done, doing, remaining).
	///
	/// If the total available space winds up being less than 10, all three
	/// values are set to zero, indicating this component should be removed.
	fn tick_bar_widths(&self) -> (usize, usize, usize) {
		// The magic "11" is made up of the following hard-coded pieces:
		// 2: braces around elapsed time;
		// 2: spaces after elapsed time;
		// 1: the "/" between done and total;
		// 2: the spaces after total;
		// 2: the braces around the bar itself (should there be one);
		// 2: the spaces after the bar itself (should there be one);
		let space: usize = 255_usize.min(self.last_width.saturating_sub(
			11 +
			self.toc[PART_ELAPSED].len() +
			self.toc[PART_DONE].len() +
			self.toc[PART_TOTAL].len() +
			self.toc[PART_PERCENT].len()
		));

		// Insufficient space!
		if space < MIN_BARS_WIDTH || 0 == self.total { (0, 0, 0) }
		// Done!
		else if self.done == self.total { (space, 0, 0) }
		// Working on it!
		else {
			// Done and doing are both floored to prevent rounding-related
			// overflow. Any remaining space will be counted as "pending".
			let done: usize = num_integer::div_floor(
				self.done as usize * space,
				self.total as usize
			);
			let doing: usize = num_integer::div_floor(
				self.doing.len() * space,
				self.total as usize
			);
			(done, doing, space - doing - done)
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
		static BAR: &[u8; 255] = &[b'#'; 255];

		if 0 != self.flags & FLAG_TICK_BAR {
			self.flags &= ! FLAG_TICK_BAR;
			match self.tick_bar_widths() {
				// No bars.
				(0, 0, 0) => {
					resize_buf_range(
						&mut self.buf,
						&mut self.toc,
						PART_BARS,
						0
					);
				},
				// Skip active tasks.
				(done, 0, undone) => {
					replace_buf_range(
						&mut self.buf,
						&mut self.toc,
						PART_BARS,
						&[
							b"\x1b[2m[\x1b[0;1;96m",
							&BAR[0..done],
							b"\x1b[0;1;34m",
							&BAR[0..undone],
							b"\x1b[0;2m]\x1b[0m  ",
						].concat()
					);
				},
				// Do all three.
				(done, doing, undone) => {
					replace_buf_range(
						&mut self.buf,
						&mut self.toc,
						PART_BARS,
						&[
							b"\x1b[2m[\x1b[0;1;96m",
							&BAR[0..done],
							b"\x1b[0;1;95m",
							&BAR[0..doing],
							b"\x1b[0;1;34m",
							&BAR[0..undone],
							b"\x1b[0;2m]\x1b[0m  ",
						].concat()
					);
				}
			}
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
			if self.doing.is_empty() {
				resize_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_DOING,
					0
				);
			}
			else {
				let tasks: &[u8] = &self.doing.iter()
					.flat_map(|x| x.task_line(self.last_width))
					.collect::<Vec<u8>>();

				replace_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_DOING,
					tasks
				);
			}
		}
	}

	/// Tick Done.
	///
	/// This updates the "done" portion of the buffer as needed.
	fn tick_set_done(&mut self) {
		if 0 != self.flags & FLAG_TICK_DONE {
			self.flags &= ! FLAG_TICK_DONE;
			replace_buf_range(
				&mut self.buf,
				&mut self.toc,
				PART_DONE,
				&*NiceInt::from(self.done)
			);
		}
	}

	/// Tick Percent.
	///
	/// This updates the "percent" portion of the buffer as needed.
	fn tick_set_percent(&mut self) {
		if 0 != self.flags & FLAG_TICK_PERCENT {
			self.flags &= ! FLAG_TICK_PERCENT;
			let p: String = format!("{:>3.*}%", 2, self.percent() * 100.0);
			replace_buf_range(
				&mut self.buf,
				&mut self.toc,
				PART_PERCENT,
				p.as_bytes(),
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
	/// progress, it has no corresponding tick flag.
	///
	/// A value of `true` is returned if one or more seconds has elapsed since
	/// the last tick, otherwise `false` is returned.
	fn tick_set_secs(&mut self) -> bool {
		let secs: u32 = self.elapsed();
		if secs == self.elapsed { false }
		else {
			self.elapsed = secs;

			if secs == 86400 {
				replace_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_ELAPSED,
					b"23:59:59"
				);
			}
			else {
				let c = utility::secs_chunks(secs);
				let rgs: usize = self.toc[PART_ELAPSED].start();
				self.buf[rgs..rgs + 2].copy_from_slice(time_format_dd(c[0]));
				self.buf[rgs + 3..rgs + 5].copy_from_slice(time_format_dd(c[1]));
				self.buf[rgs + 6..rgs + 8].copy_from_slice(time_format_dd(c[2]));
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
			if self.title.is_empty() {
				resize_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_TITLE,
					0
				);
			}
			else {
				replace_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_TITLE,
					&{
						let mut m = self.title.clone();
						m.fit_to_range(self.last_width - 1);
						m.push(b'\n');
						m
					}
				);
			}
		}
	}

	/// Tick Total.
	///
	/// This updates the "total" portion of the buffer as needed.
	fn tick_set_total(&mut self) {
		if 0 != self.flags & FLAG_TICK_TOTAL {
			self.flags &= ! FLAG_TICK_TOTAL;
			replace_buf_range(
				&mut self.buf,
				&mut self.toc,
				PART_TOTAL,
				&*NiceInt::from(self.total)
			);
		}
	}

	/// Tick Width.
	///
	/// Check to see if the terminal width has changed since the last run and
	/// update values — i.e. the relevant tick flags — as necessary.
	fn tick_set_width(&mut self) {
		let width = utility::term_width();
		if width != self.last_width {
			self.flags |= FLAGS_RESIZED;
			self.last_width = width;
		}
	}
}



#[derive(Debug)]
/// Progress Bar.
///
/// This is it! The whole point of the crate! See the library documentation for
/// more information.
pub struct Progress<T>
where T: ProgressTask {
	/// The set to progress through.
	set: Vec<T>,
	/// The stateful data.
	inner: Arc<Mutex<ProgressInner<T>>>,
}

impl<T> Default for Progress<T>
where T: ProgressTask {
	fn default() -> Self {
		Self {
			set: Vec::new(),
			inner: Arc::new(Mutex::new(ProgressInner::<T>::default())),
		}
	}
}

impl<T> From<Vec<T>> for Progress<T>
where T: ProgressTask {
	fn from(src: Vec<T>) -> Self {
		let total: u32 = src.len() as u32;
		if total == 0 { Self::default() }
		else {
			Self {
				set: src,
				inner: Arc::new(Mutex::new(ProgressInner::<T> {
					total,
					flags: FLAGS_NEW,
					..ProgressInner::<T>::default()
				})),
			}
		}
	}
}

impl<T> Deref for Progress<T>
where T: ProgressTask {
	type Target = [T];
	fn deref(&self) -> &Self::Target { &self.set }
}

impl<T> Progress<T>
where T: ProgressTask + Sync + Send + 'static {
	// ------------------------------------------------------------------------
	// Setup
	// ------------------------------------------------------------------------

	#[must_use]
	/// With Threads.
	pub fn with_threads(self, threads: usize) -> Self {
		self.set_threads(threads);
		self
	}

	#[must_use]
	/// With Title.
	pub fn with_title<S> (self, title: S) -> Self
	where S: AsRef<str> {
		self.set_title(title);
		self
	}

	#[must_use]
	/// Toggle Progress Barness.
	pub fn with_display(self, on: bool) -> Self {
		self.set_display(on);
		self
	}

	/// Run!
	///
	/// This here is the whole point. Iterate through the set in parallel with
	/// a progress bar, while executing a custom callback on each entry.
	///
	/// When parallelism is such that only one thread is to be used, the
	/// progress portion will run without a steady tick and without displaying
	/// the current task information, but will otherwise still produce a
	/// progress bar to watch.
	///
	/// When parallelism is more than one thread, tasks will be executed in
	/// parallel using that many threads, with the display updated at a steady
	/// pace (60ms) throughout.
	pub fn run<F>(&self, cb: F)
	where F: Fn(&T) + Copy + Send + Sync + 'static {
		if ! self.set.is_empty() {
			match (self.is_silent(), self.threads()) {
				(false, 1) => self.set.iter().for_each(|x| {
					cb(x);
					self.increment();
					progress_tick(&self.inner);
				}),
				(true, 1) => self.set.iter().for_each(|x| {
					cb(x);
					self.increment();
				}),
				(true, x) => self.run_parallel_silent(x, cb),
				(false, x) => self.run_parallel(x, cb),
			}
		}
	}

	/// Loop in Parallel!
	fn run_parallel<F>(&self, threads: usize, cb: F)
	where F: Fn(&T) + Copy + Send + Sync + 'static {
		// The thread pool.
		let pool = ThreadPool::new(threads);
		let (tx, rx) = crossbeam_channel::bounded(self.set.len());

		// Do the main loop!
		self.set.iter().cloned().for_each(|x| {
			let tx = tx.clone();
			let inner = self.inner.clone();
			pool.execute(move|| {
				progress_start(&inner, x.clone());
				cb(&x);
				progress_end(&inner, &x);
				tx.send(()).unwrap();
			});
		});

		// Run steady tick until we're out of tasks.
		let ticker = crossbeam_channel::tick(Duration::from_millis(60));
		loop {
			ticker.recv().unwrap();
			if ! progress_tick(&self.inner) {
				drop(ticker);
				break;
			}
		}

		// The ticker loop should be blocking, but let's make sure we've
		// consumed all of the signals before leaving.
		drop(tx);
		let _: Vec<_> = rx.iter().collect();
	}

	/// Loop (Silently) in Parallel!
	///
	/// This is just like `run_parallel()`, except it avoids calls to tick-
	/// related methods as there is no ticking needed.
	fn run_parallel_silent<F>(&self, threads: usize, cb: F)
	where F: Fn(&T) + Copy + Send + Sync + 'static {
		// The thread pool.
		let pool = ThreadPool::new(threads);
		let (tx, rx) = crossbeam_channel::bounded(self.set.len());

		// Do the main loop!
		self.set.iter().cloned().for_each(|x| {
			let tx = tx.clone();
			let inner = self.inner.clone();
			pool.execute(move|| {
				cb(&x);
				progress_increment(&inner);
				tx.send(()).unwrap();
			});
		});

		// Run steady tick until we're out of tasks.
		let ticker = crossbeam_channel::tick(Duration::from_millis(30));
		loop {
			ticker.recv().unwrap();
			if ! progress_is_running(&self.inner) {
				drop(ticker);
				break;
			}
		}

		// The ticker loop should be blocking, but let's make sure we've
		// consumed all of the signals before leaving.
		drop(tx);
		let _: Vec<_> = rx.iter().collect();
	}



	// ------------------------------------------------------------------------
	// Breakdown
	// ------------------------------------------------------------------------

	#[must_use]
	/// Consume.
	///
	/// Return the loopable vector.
	pub fn consume(self) -> Vec<T> { self.set }

	/// Reset the Counts.
	///
	/// This resets the totals so the source can be looped a second time.
	pub fn reset(&self) {
		if ! self.set.is_empty() && ! self.is_running() {
			let mut ptr = mutex_ptr!(self.inner);

			// Reset the tickable values.
			ptr.doing.clear();
			ptr.done = 0;
			ptr.elapsed = 0;
			ptr.started = Instant::now();

			// Reset the flags and hashes.
			ptr.flags = FLAGS_ALL;
			ptr.last_hash = 0;
			ptr.last_lines = 0;
			ptr.last_time = 0;
			ptr.last_width = 0;
		}
	}



	// ------------------------------------------------------------------------
	// `ProgressInner` Wrappers
	// ------------------------------------------------------------------------

	// These just return the inner values.
	get_inner!(done, u32);
	get_inner!(elapsed, u32);
	get_inner!(percent, f64);
	get_inner!(threads, usize);
	get_inner!(total, u32);
	get_inner!(is_running, bool);
	get_inner!(is_silent, bool);

	#[must_use]
	/// Get Doing.
	pub fn doing(&self) -> u32 {
		let ptr = mutex_ptr!(self.inner);
		ptr.doing.len() as u32
	}

	/// Increment.
	///
	/// Wrapper for `ProgressInner::increment()`.
	fn increment(&self) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.increment();
	}

	/// Print Finish Message.
	///
	/// Wrapper for `ProgressInner::print_summary()`.
	pub fn print_summary<S> (&self, one: S, many: S)
	where S: AsRef<str> {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.print_summary(one, many);
	}

	/// Toggle Progress Barness.
	pub fn set_display(&self, on: bool) {
		let mut ptr = mutex_ptr!(self.inner);
		if on { ptr.flags &= ! FLAG_SILENT; }
		else { ptr.flags |= FLAG_SILENT; }
	}

	/// With Threads.
	pub fn set_threads(&self, threads: usize) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.set_threads(threads);
	}

	/// Set Title.
	///
	/// Wrapper for `ProgressInner::set_title()`.
	pub fn set_title<S> (&self, title: S)
	where S: AsRef<str> {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.set_title(title);
	}
}

impl Progress<PathBuf> {
	/// Crunch Run.
	///
	/// This is a special version of `run()` for `PathBuf` collections that
	/// compares the before and after sizes, reporting any savings in a summary
	/// at the end.
	///
	/// Note: a warning is printed if the contents of the set are empty.
	///
	/// Note: the summary is printed regardless of whether or not the progress
	/// display has been silenced.
	pub fn crunch<F> (&self, cb: F)
	where F: Fn(&PathBuf) + Copy + Send + Sync + 'static {
		if self.set.is_empty() {
			MsgKind::Warning.into_msg("No matching files were found.\n")
				.eprint();

			return;
		}

		let before: u64 = self.du();
		self.run(cb);

		crunched_in(
			self.total().into(),
			self.elapsed(),
			before,
			self.du(),
		);
	}

	#[must_use]
	/// Total File(s) Size.
	///
	/// Add up the size of all files in a set. Calculations are run in parallel so
	/// should be fairly fast depending on the file system.
	fn du(&self) -> u64 {
		use rayon::prelude::*;
		self.set.par_iter()
			.map(|x| x.metadata().map_or(0, |m| m.len()))
			.sum()
	}
}

/// Tick.
///
/// Wrapper for `ProgressInner::tick()`.
fn progress_tick<T>(inner: &Arc<Mutex<ProgressInner<T>>>) -> bool
where T: ProgressTask + Sync + Send + 'static {
	let mut ptr = mutex_ptr!(inner);
	ptr.tick()
}

/// End Task.
///
/// Wrapper for `ProgressInner::end_task()`.
fn progress_end<T>(inner: &Arc<Mutex<ProgressInner<T>>>, task: &T)
where T: ProgressTask + Sync + Send + 'static {
	let mut ptr = mutex_ptr!(inner);
	ptr.end_task(task);
}

/// Start Task.
///
/// Wrapper for `ProgressInner::start_task()`.
fn progress_start<T>(inner: &Arc<Mutex<ProgressInner<T>>>, task: T)
where T: ProgressTask + Sync + Send + 'static {
	let mut ptr = mutex_ptr!(inner);
	ptr.start_task(task);
}

/// Increment.
///
/// Wrapper for `ProgressInner::increment()`.
fn progress_increment<T>(inner: &Arc<Mutex<ProgressInner<T>>>)
where T: ProgressTask + Sync + Send + 'static {
	let mut ptr = mutex_ptr!(inner);
	ptr.increment();
}

/// Is Running?
///
/// Wrapper for `ProgressInner::is_running()`.
fn progress_is_running<T>(inner: &Arc<Mutex<ProgressInner<T>>>) -> bool
where T: ProgressTask + Sync + Send + 'static {
	let ptr = mutex_ptr!(inner);
	ptr.is_running()
}

/// Crunched In Msg
///
/// This is an alternative progress summary that includes the number of bytes
/// saved. It is called after `progress_crunch()`.
fn crunched_in(total: u64, time: u32, before: u64, after: u64) {
	// No savings or weird values.
	if 0 == after || before <= after {
		Msg::from([
			&utility::inflect(total, "file in ", "files in "),
			&*NiceElapsed::from(time),
			b", but nothing doing.\n",
		].concat())
			.with_prefix(MsgKind::Crunched)
			.eprint();
	}
	// Something happened!
	else {
		MsgKind::Crunched.into_msg(format!(
			"{} in {}, saving {} bytes ({:3.*}%).\n",
			unsafe { std::str::from_utf8_unchecked(&utility::inflect(total, "file", "files")) },
			NiceElapsed::from(time).as_str(),
			NiceInt::from(before - after).as_str(),
			2,
			(1.0 - (after as f64 / before as f64)) * 100.0
		)).eprint();
	}
}
