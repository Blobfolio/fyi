/*!
# FYI Witcher: `Witching`

`Witching` is a progress bar wrapper built around a collection of paths. Each
(parallel) iteration of the set results in a tick, providing a nice little
ASCII animation to follow while data is being processed.

Compared with more general-purpose libraries like `indicatif`, `Witching` is
incredibly lightweight and efficient, but it also lacks much in the way of
customizability.

All progress bars include an elapsed time and progress shown as a ratio and
percent. If the window is large enough, an actual "bar" is displayed as well.
`Witching`s can optionally include a title and a list of active tasks.

That's it! Short and sweet.
*/



use ahash::{
	AHasher,
	AHashSet,
};
use crate::{
	NiceElapsed,
	NiceInt,
	traits::{
		FittedRange,
		FittedRangeMut,
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
use rayon::prelude::*;
use std::{
	cmp::Ordering,
	ffi::OsStr,
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
		atomic::{
			AtomicBool,
			Ordering::SeqCst,
		},
		Mutex,
	},
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

/// Helper: Pass through a getter to the `WitchingInner`.
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



/// Witching Flags: Diff
///
/// When summarizing, track the before and after sizes of the files in the set
/// to see how many bytes were "saved".
pub const WITCHING_DIFF: u8 =      0b0001;

/// Witching Flags: Quiet
///
/// Run silently, i.e. without any progress bar. Kinda pointless, but it allows
/// for code re-use in libraries that offer the option.
pub const WITCHING_QUIET: u8 =     0b0010;

/// Witching Flags: Summarize
///
/// Summarize results at the end of the run.
pub const WITCHING_SUMMARIZE: u8 = 0b0100;



/// Tick Flags.
///
/// These flags indicate whether or not a given component has changed since the
/// last tick, saving the overhead of recalculating the buffer values each time
/// a value changes. (Instead they're only recalculated at most once per tick.)
const TICK_ALL: u8 =     0b0111_1111;
const TICK_DEFAULT: u8 = 0b0000_0001;
const TICK_NEW: u8 =     0b0110_0001;
const TICK_RESIZED: u8 = 0b0001_0011;

const TICK_BAR: u8 =     0b0000_0001;
const TICK_DOING: u8 =   0b0000_0010;
const TICK_DONE: u8 =    0b0000_0100;
const TICK_PERCENT: u8 = 0b0000_1000;
const TICK_TITLE: u8 =   0b0001_0000;
const TICK_TOTAL: u8 =   0b0010_0000;

const TICKING: u8 =      0b0100_0000;



/// Buffer Indexes.
///
/// The start and end points of the malleable progress components are stored in
/// an array for easy access. These are their indexes.
const PART_TITLE: usize = 0;
const PART_ELAPSED: usize = 1;
const PART_BAR_DONE: usize = 2;
const PART_BAR_DOING: usize = 3;
const PART_BAR_UNDONE: usize = 4;
const PART_DONE: usize = 5;
const PART_TOTAL: usize = 6;
const PART_PERCENT: usize = 7;
const PART_DOING: usize = 8;



/// Misc Variables.
const MIN_BARS_WIDTH: usize = 10;
const MIN_DRAW_WIDTH: usize = 40;



#[derive(Debug)]
/// Inner Witching.
struct WitchingInner {
	buf: Vec<u8>,
	toc: [BufRange; 9],
	elapsed: u32,
	last_hash: u64,
	last_lines: usize,
	last_time: u128,
	last_width: usize,

	doing: AHashSet<Vec<u8>>,
	done: u32,
	flags: u8,
	started: Instant,
	title: Vec<u8>,
	total: u32,
}

impl Default for WitchingInner {
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

			//  \e   [   2    m   [  \e   [   0   ;   1   ;   9   6    m
				27, 91, 50, 109, 91, 27, 91, 48, 59, 49, 59, 57, 54, 109,

			//  Bar Done would go here.

			//  \e   [   0   ;   1   ;   9   5    m
				27, 91, 48, 59, 49, 59, 57, 53, 109,

			//  Bar Doing would go here.

			//  \e   [   0   ;   1   ;   3   4    m
				27, 91, 48, 59, 49, 59, 51, 52, 109,

			//  Bar Undone would go here.

			//  \e   [   0   ;   2    m   ]  \e   [   0    m   •   •
				27, 91, 48, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32,

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
				BufRange::new(46, 46), // Bar Done.
				BufRange::new(55, 55), // Bar Doing.
				BufRange::new(64, 64), // Bar Undone.
				BufRange::new(84, 85), // Done.
				BufRange::new(101, 102), // Total.
				BufRange::new(110, 115), // Percent.
				BufRange::new(120, 120), // Current Tasks.
			],
			doing: AHashSet::new(),
			done: 0,
			elapsed: 0,
			flags: TICK_DEFAULT,
			last_hash: 0,
			last_lines: 0,
			last_time: 0,
			last_width: 0,
			started: Instant::now(),
			title: Vec::new(),
			total: 0,
		}
	}
}

impl WitchingInner {
	// ------------------------------------------------------------------------
	// Getters
	// ------------------------------------------------------------------------

	/// Doing.
	pub fn doing(&self) -> u32 { self.doing.len() as u32 }

	/// Done.
	pub const fn done(&self) -> u32 { self.done }

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
	pub const fn is_running(&self) -> bool { 0 != self.flags & TICKING }

	/// Total.
	pub const fn total(&self) -> u32 { self.total }



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	#[allow(trivial_casts)] // We need triviality!
	/// End Task.
	///
	/// Remove a task from the currently-running list and increment `done` by
	/// one.
	pub fn end_task(&mut self, task: &PathBuf) {
		if self.doing.remove(unsafe { &*(task.as_os_str() as *const OsStr as *const [u8]) }) {
			self.flags |= TICK_DOING | TICK_BAR;
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
				self.flags |= TICK_DONE | TICK_PERCENT | TICK_BAR;
				self.done = new_done;
			}
		}
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

			self.flags |= TICK_TITLE;
		}
	}

	#[allow(trivial_casts)] // We need triviality!
	/// Start Task.
	///
	/// Add a task to the currently-running list.
	pub fn start_task(&mut self, task: &PathBuf) {
		let task: Vec<u8> = unsafe { &*(task.as_os_str() as *const OsStr as *const [u8]) }.to_vec();
		if self.doing.insert(task) {
			self.flags |= TICK_DOING | TICK_BAR;
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

	/// Stop.
	pub fn stop(&mut self) {
		self.flags |= TICK_ALL;
		self.flags &= ! TICKING;
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
			self.flags = TICKING;
			self.print_blank();
			return true;
		}

		// If the time hasn't changed, and nothing else has changed, we can
		// abort without all the tedious checking.
		if ! self.tick_set_secs() && self.flags == TICKING {
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
		static DASH: &[u8; 255] = &[b'-'; 255];

		if 0 != self.flags & TICK_BAR {
			self.flags &= ! TICK_BAR;
			let (w_done, w_doing, w_undone) = self.tick_bar_widths();

			if self.toc[PART_BAR_DONE].len() != w_done {
				replace_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_BAR_DONE,
					&BAR[0..w_done],
				);
			}

			if self.toc[PART_BAR_DOING].len() != w_doing {
				replace_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_BAR_DOING,
					&DASH[0..w_doing],
				);
			}

			if self.toc[PART_BAR_UNDONE].len() != w_undone {
				replace_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_BAR_UNDONE,
					&DASH[0..w_undone],
				);
			}
		}
	}

	/// Tick Doing.
	///
	/// Update the task list portion of the buffer. This is triggered both by
	/// changes to the task list as well as resoluation changes (as long values
	/// may require lazy cropping).
	fn tick_set_doing(&mut self) {
		if 0 != self.flags & TICK_DOING {
			self.flags &= ! TICK_DOING;
			if self.doing.is_empty() {
				resize_buf_range(
					&mut self.buf,
					&mut self.toc,
					PART_DOING,
					0
				);
			}
			else {
				let width: usize = self.last_width.saturating_sub(6);
				let tasks: &[u8] = &self.doing.iter()
					.flat_map(|x|
						[
							// •   •   •   •  \e   [   3   5    m   ↳  ---  ---   •
							&[32, 32, 32, 32, 27, 91, 51, 53, 109, 226, 134, 179, 32][..],
							&x[x.fitted_range(width)],
							b"\x1b[0m\n",
						].concat()
					)
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
		if 0 != self.flags & TICK_DONE {
			self.flags &= ! TICK_DONE;
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
		if 0 != self.flags & TICK_PERCENT {
			self.flags &= ! TICK_PERCENT;
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
		if 0 != self.flags & TICK_TITLE {
			self.flags &= ! TICK_TITLE;
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
		if 0 != self.flags & TICK_TOTAL {
			self.flags &= ! TICK_TOTAL;
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
			self.flags |= TICK_RESIZED;
			self.last_width = width;
		}
	}
}



#[derive(Debug)]
/// Witching Bar.
///
/// This is it! The whole point of the crate! See the library documentation for
/// more information.
pub struct Witching {
	/// The set to progress through.
	set: Vec<PathBuf>,
	/// The stateful data.
	inner: Arc<Mutex<WitchingInner>>,
	/// Flags.
	flags: u8,
	/// Summary labels.
	one: Vec<u8>,
	many: Vec<u8>,
}

impl Default for Witching {
	fn default() -> Self {
		Self {
			set: Vec::new(),
			inner: Arc::new(Mutex::new(WitchingInner::default())),
			flags: 0,
			one: vec![102, 105, 108, 101],
			many: vec![102, 105, 108, 101, 115],
		}
	}
}

impl From<Vec<PathBuf>> for Witching {
	fn from(src: Vec<PathBuf>) -> Self {
		let total: u32 = src.len() as u32;
		if total == 0 { Self::default() }
		else {
			Self {
				set: src,
				inner: Arc::new(Mutex::new(WitchingInner {
					total,
					flags: TICK_NEW,
					..WitchingInner::default()
				})),
				..Self::default()
			}
		}
	}
}

impl Deref for Witching {
	type Target = [PathBuf];
	fn deref(&self) -> &Self::Target { &self.set }
}

impl Witching {
	// ------------------------------------------------------------------------
	// Setup
	// ------------------------------------------------------------------------

	#[must_use]
	/// With Flags.
	pub fn with_flags(mut self, flags: u8) -> Self {
		self.set_flags(flags);
		self
	}

	#[must_use]
	/// With Labels.
	pub fn with_labels<S>(mut self, one: S, many: S) -> Self
	where S: AsRef<str> {
		self.set_labels(one, many);
		self
	}

	#[must_use]
	/// With Title.
	pub fn with_title<S> (self, title: S) -> Self
	where S: AsRef<str> {
		self.set_title(title);
		self
	}

	/// Set Flags.
	pub fn set_flags(&mut self, flags: u8) { self.flags = flags; }

	/// Set Labels.
	pub fn set_labels<S>(&mut self, one: S, many: S)
	where S: AsRef<str> {
		self.one.truncate(0);
		self.one.extend_from_slice(one.as_ref().as_bytes());

		self.many.truncate(0);
		self.many.extend_from_slice(many.as_ref().as_bytes());
	}

	#[must_use]
	#[allow(clippy::missing_const_for_fn)] // Evidently it can't!
	/// Into Vec.
	///
	/// Consume and return the path collection.
	pub fn into_vec(self) -> Vec<PathBuf> { self.set }



	// ------------------------------------------------------------------------
	// Operations
	// ------------------------------------------------------------------------

	#[must_use]
	/// Total File(s) Size.
	///
	/// Add up the size of all files in a set. Calculations are run in parallel so
	/// should be fairly fast depending on the file system.
	fn du(&self) -> u64 {
		self.set.par_iter()
			.map(|x| x.metadata().map_or(0, |m| m.len()))
			.sum()
	}

	/// Label.
	///
	/// What label should we be using? One or many?
	fn label(&self) -> &[u8] {
		if self.set.len() == 1 { &self.one }
		else { &self.many }
	}

	/// Run!
	pub fn run<F>(&self, cb: F)
	where F: Fn(&PathBuf) + Copy + Send + Sync + 'static {
		// Empty set?
		if self.set.is_empty() {
			if 0 != self.flags & WITCHING_SUMMARIZE {
				self.summarize_empty();
			}
		}
		else {
			// We might need to note our starting size.
			let before: u64 =
				if 0 == self.flags & WITCHING_DIFF { 0 }
				else { self.du() };

			if 0 == self.flags & WITCHING_QUIET {
				self.run_sexy(cb);
			}
			// Quiet iteration.
			else {
				self.set.par_iter().for_each(cb);
				self.stop();
			}

			// Summarize?
			if 0 != self.flags & WITCHING_SUMMARIZE {
				// Just the time.
				if 0 == self.flags & WITCHING_DIFF { self.summarize(); }
				// Time and savings.
				else { self.summarize_diff(before); }
			}
		}
	}

	/// Run!
	fn run_sexy<F>(&self, cb: F)
	where F: Fn(&PathBuf) + Copy + Send + Sync + 'static {
		// Track the run independently of `WitchingInner`, just in case a
		// `Mutex` crashes or something.
		let done = Arc::new(AtomicBool::new(false));

		// Run steady tick until we're out of tasks.
		let t_inner = self.inner.clone();
		let t_sleep = Duration::from_millis(60);
		let t_done = done.clone();
		let t_handle = std::thread::spawn(move|| loop {
			if t_done.load(SeqCst) || ! progress_tick(&t_inner) { break; }
			std::thread::sleep(t_sleep);
		});

		// Do the main loop!
		let inner = self.inner.clone();
		self.set.par_iter().for_each(|x| {
			progress_start(&inner, x);
			cb(x);
			progress_end(&inner, x);
		});

		// The steady tick loop should close on its own, but just in case,
		// let's give it another reason to do so.
		done.store(true, SeqCst);
		t_handle.join().unwrap();
	}

	/// Summary Stub.
	///
	/// Almost all of the summaries start the same way. This is that way.
	fn summary_stub(&self) -> Vec<u8> {
		[
			&*NiceInt::from(u64::from(self.total())),
			b" ",
			self.label(),
			b" in ",
			&*NiceElapsed::from(self.elapsed()),
		].concat()
	}

	/// Summarize.
	fn summarize(&self) {
		Msg::from([
			&self.summary_stub()[..],
			b".\n",
		].concat())
			.with_prefix(MsgKind::Done)
			.eprint();
	}

	/// Summarize (with savings).
	///
	/// This summary compares the before and after bytes.
	fn summarize_diff(&self, before: u64) {
		let after: u64 = self.du();

		// No savings. Boo.
		if 0 == after || before <= after {
			Msg::from([
				&self.summary_stub()[..],
				b", but nothing doing.\n",
			].concat())
				.with_prefix(MsgKind::Crunched)
				.eprint();
		}
		else {
			MsgKind::Crunched.into_msg(format!(
				"{}, saving {} bytes ({:3.*}%).\n",
				unsafe { std::str::from_utf8_unchecked(&self.summary_stub()[..]) },
				NiceInt::from(before - after).as_str(),
				2,
				(1.0 - (after as f64 / before as f64)) * 100.0
			)).eprint();
		}
	}

	/// Summarize empty.
	///
	/// This summary prints when the set is empty.
	fn summarize_empty(&self) {
		Msg::from([
			&b"No "[..],
			&self.many,
			b" were found.\n",
		].concat())
			.with_prefix(MsgKind::Warning)
			.eprint();
	}



	// ------------------------------------------------------------------------
	// `WitchingInner` Wrappers
	// ------------------------------------------------------------------------

	// These just return the inner values.
	get_inner!(doing, u32);
	get_inner!(done, u32);
	get_inner!(elapsed, u32);
	get_inner!(percent, f64);
	get_inner!(total, u32);
	get_inner!(is_running, bool);

	/// Stop.
	///
	/// Wrapper for `WitchingInner::stop()`.
	fn stop(&self) {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.stop();
	}

	/// Set Title.
	///
	/// Wrapper for `WitchingInner::set_title()`.
	pub fn set_title<S> (&self, title: S)
	where S: AsRef<str> {
		let mut ptr = mutex_ptr!(self.inner);
		ptr.set_title(title);
	}
}

/// Tick.
///
/// Wrapper for `WitchingInner::tick()`.
fn progress_tick(inner: &Arc<Mutex<WitchingInner>>) -> bool {
	let mut ptr = mutex_ptr!(inner);
	ptr.tick()
}

/// End Task.
///
/// Wrapper for `WitchingInner::end_task()`.
fn progress_end(inner: &Arc<Mutex<WitchingInner>>, task: &PathBuf) {
	let mut ptr = mutex_ptr!(inner);
	ptr.end_task(task);
}

/// Start Task.
///
/// Wrapper for `WitchingInner::start_task()`.
fn progress_start(inner: &Arc<Mutex<WitchingInner>>, task: &PathBuf) {
	let mut ptr = mutex_ptr!(inner);
	ptr.start_task(task);
}
