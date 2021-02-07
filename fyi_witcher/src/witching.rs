/*!
# FYI Witcher: `Witching`

This struct is only available when the crate feature `witching` is enabled.
*/

use ahash::AHashSet;
use crate::utility;
use fyi_msg::{
	Msg,
	MsgKind,
	MsgBuffer9,
};
use fyi_num::{
	NiceElapsed,
	NicePercent,
	NiceU32,
	NiceU64,
	write_time,
};
use rayon::prelude::*;
use std::{
	cmp::Ordering,
	io::{
		self,
		Write,
	},
	ops::Deref,
	path::PathBuf,
	sync::{
		Arc,
		Mutex,
		atomic::{
			AtomicBool,
			Ordering::SeqCst,
		},
	},
	time::{
		Duration,
		Instant,
	},
};



/// Helper: Pass through a getter to the `WitchingInner`.
macro_rules! get_inner {
	($func:ident, $type:ty) => {
		#[must_use]
		/// Wrapper.
		pub fn $func(&self) -> $type {
			let ptr = crate::mutex_ptr!(self.inner);
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
/// # Inner Witching.
///
/// Most of the stateful data for our [`Witching`] struct lives here so that
/// it can be wrapped up in an `Arc<Mutex>` and passed between threads.
struct WitchingInner {
	buf: MsgBuffer9,
	elapsed: u32,
	last_hash: u64,
	last_lines: usize,
	last_time: u128,
	last_width: usize,

	doing: AHashSet<Msg>,
	done: u32,
	flags: u8,
	started: Instant,
	title: Msg,
	total: u32,
}

impl Default for WitchingInner {
	fn default() -> Self {
		Self {
			buf: MsgBuffer9::from_raw_parts(
				vec![
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
				[
					0, 0,     // Title.
					11, 19,   // Elapsed.
					46, 46,   // Bar Done.
					55, 55,   // Bar Doing.
					64, 64,   // Bar Undone.
					84, 85,   // Done.
					101, 102, // Total.
					110, 115, // Percent.
					120, 120, // Current Tasks.
				]
			),
			doing: AHashSet::new(),
			done: 0,
			elapsed: 0,
			flags: TICK_DEFAULT,
			last_hash: 0,
			last_lines: 0,
			last_time: 0,
			last_width: 0,
			started: Instant::now(),
			title: Msg::default(),
			total: 0,
		}
	}
}

impl WitchingInner {
	// ------------------------------------------------------------------------
	// Getters
	// ------------------------------------------------------------------------

	/// # Doing.
	///
	/// Return the number of active tasks.
	pub(crate) fn doing(&self) -> u32 { self.doing.len() as u32 }

	/// # Done.
	///
	/// Return the number of completed tasks.
	pub(crate) const fn done(&self) -> u32 { self.done }

	/// # Elapsed (Seconds).
	///
	/// Return the elapsed time in seconds.
	pub(crate) fn elapsed(&self) -> u32 {
		86400.min(self.started.elapsed().as_secs()) as u32
	}

	/// # Percent.
	///
	/// Return the percentage of tasks completed, i.e. `done / total`.
	pub(crate) fn percent(&self) -> f64 {
		if self.total == 0 || self.done == 0 { 0.0 }
		else if self.done == self.total { 1.0 }
		else {
			f64::from(self.done) / f64::from(self.total)
		}
	}

	/// # Is Running?
	///
	/// If the amount completed is less than the total amount, this returns
	/// `true`, otherwise `false`.
	pub(crate) const fn is_running(&self) -> bool { 0 != self.flags & TICKING }

	/// # Total.
	///
	/// Return the total number of tasks.
	pub(crate) const fn total(&self) -> u32 { self.total }



	// ------------------------------------------------------------------------
	// Setters
	// ------------------------------------------------------------------------

	/// # End Task.
	///
	/// Remove a task from the currently-running list and increment `done` by
	/// one.
	pub(crate) fn end_task(&mut self, task: &PathBuf) {
		if self.doing.remove(&task_msg(task)) {
			self.flags |= TICK_DOING | TICK_BAR;
			self.increment();
		}
	}

	/// # Increment (Completed).
	///
	/// Increment `done` by one. If this reaches the total, it will
	/// automatically trigger a stop.
	pub(crate) fn increment(&mut self) {
		let new_done = self.total.min(self.done + 1);
		if new_done != self.done {
			if new_done == self.total { self.stop(); }
			else {
				self.flags |= TICK_DONE | TICK_PERCENT | TICK_BAR;
				self.done = new_done;
			}
		}
	}

	/// # Set Title.
	///
	/// This updates the progress bar's title. If an empty string is passed,
	/// the title will be removed.
	pub(crate) fn set_title<S> (&mut self, title: S)
	where S: Into<Msg> {
		let title: Msg = title.into().with_newline(true);
		if self.title != title {
			self.title = title;
			self.flags |= TICK_TITLE;
		}
	}

	/// # Start Task.
	///
	/// Add a task to the currently-running list.
	pub(crate) fn start_task(&mut self, task: &PathBuf) {
		if self.doing.insert(task_msg(task)) {
			self.flags |= TICK_DOING | TICK_BAR;
		}
	}



	// ------------------------------------------------------------------------
	// Render
	// ------------------------------------------------------------------------

	/// # Preprint.
	///
	/// This method accepts a completed buffer ready for printing, hashing it
	/// for comparison with the last job. If unique, the previous output is
	/// erased and replaced with the new output.
	fn preprint(&mut self) {
		if 0 == self.buf.total_len() {
			self.print_blank();
			return;
		}

		// Make sure the content is unique, otherwise we can leave the old bits
		// up.
		let hash = utility::hash64(&self.buf);
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

	/// # Print Blank.
	///
	/// This simply resets the last-print hash and clears any prior output.
	fn print_blank(&mut self) {
		if self.last_hash != 0 {
			self.last_hash = 0;
		}

		self.print_cls();
	}

	/// # Print!
	///
	/// Print some arbitrary data to the write place. Haha.
	fn print(buf: &[u8]) {
		let writer = io::stderr();
		let mut handle = writer.lock();
		let _ = handle.write_all(buf).and_then(|_| handle.flush());
	}

	/// # Erase Output.
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
					Self::print(&CLS10[0..10 + 14 * self.last_lines]);
				},
				// To clear more lines, print our pre-calculated buffer (which
				// covers the first 10), and duplicate the line-up chunk (n-10)
				// times to cover the rest.
				Ordering::Greater => {
					Self::print(&[
						&CLS10[..],
						&CLS10[14..28].repeat(self.last_lines - 10),
					].concat());
				},
			}

			// Having cleared whatever it was, there are now no last_lines.
			self.last_lines = 0;
		}
	}

	/// # Stop.
	///
	/// This method ends all progression, setting `done` to `total` so that
	/// summaries can be generated.
	pub(crate) fn stop(&mut self) {
		self.flags |= TICK_ALL;
		self.flags &= ! TICKING;
		self.done = self.total;
		self.doing.clear();
		self.print_blank();
	}

	/// # Tick.
	///
	/// Ticking takes all of the changed values (since the last tick), updates
	/// their corresponding parts in the buffer, and prints the result, if any.
	pub(crate) fn tick(&mut self) -> bool {
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

	/// # Tick Bar Dimensions.
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
			self.buf.len(PART_ELAPSED) +
			self.buf.len(PART_DONE) +
			self.buf.len(PART_TOTAL) +
			self.buf.len(PART_PERCENT)
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

	/// # Tick Bar.
	///
	/// This redraws the actual progress *bar* portion of the buffer, which is
	/// actually three different bars squished together: Done, Doing, and
	/// Pending.
	///
	/// The combined width of the `###` will never exceed 255, and will never
	/// be less than 10.
	fn tick_set_bar(&mut self) {
		static BAR: [u8; 255] = [b'#'; 255];
		static DASH: [u8; 255] = [b'-'; 255];

		if 0 != self.flags & TICK_BAR {
			self.flags &= ! TICK_BAR;
			let (w_done, w_doing, w_undone) = self.tick_bar_widths();

			// Update the parts!.
			if self.buf.len(PART_BAR_DONE) != w_done {
				self.buf.replace(PART_BAR_DONE, &BAR[0..w_done]);
			}
			if self.buf.len(PART_BAR_DOING) != w_doing {
				self.buf.replace(PART_BAR_DOING, &DASH[0..w_doing]);
			}
			if self.buf.len(PART_BAR_UNDONE) != w_undone {
				self.buf.replace(PART_BAR_UNDONE, &DASH[0..w_undone]);
			}
		}
	}

	/// # Tick Doing.
	///
	/// Update the task list portion of the buffer. This is triggered both by
	/// changes to the task list as well as resoluation changes (as long values
	/// may require lazy cropping).
	fn tick_set_doing(&mut self) {
		if 0 != self.flags & TICK_DOING {
			self.flags &= ! TICK_DOING;
			if self.doing.is_empty() {
				self.buf.truncate(PART_DOING, 0);
			}
			else {
				let width: usize = self.last_width.saturating_sub(6);
				let tasks: Vec<u8> = {
					let mut v = Vec::with_capacity(256);
					v.extend_from_slice(b"\x1b[35m");
					self.doing.iter()
						.for_each(|x| v.extend_from_slice(&x.fitted(width)));
					v.extend_from_slice(b"\x1b[0m");
					v
				};
				self.buf.replace(PART_DOING, &tasks);
			}
		}
	}

	/// # Tick Done.
	///
	/// This updates the "done" portion of the buffer as needed.
	fn tick_set_done(&mut self) {
		if 0 != self.flags & TICK_DONE {
			self.flags &= ! TICK_DONE;
			self.buf.replace(PART_DONE, &NiceU32::from(self.done));
		}
	}

	/// # Tick Percent.
	///
	/// This updates the "percent" portion of the buffer as needed.
	fn tick_set_percent(&mut self) {
		if 0 != self.flags & TICK_PERCENT {
			self.flags &= ! TICK_PERCENT;
			self.buf.replace(PART_PERCENT, &NicePercent::from(self.percent()));
		}
	}

	/// # Tick Elapsed Seconds.
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
			let [h, m, s] = NiceElapsed::hms(secs);
			unsafe {
				write_time(
					self.buf.as_mut_ptr().add(self.buf.start(PART_ELAPSED)),
					h,
					m,
					s,
				);
			}

			true
		}
	}

	/// # Tick Title.
	///
	/// The title needs to be rewritten both on direct change and resolution
	/// change. Long titles are lazy-cropped as needed.
	fn tick_set_title(&mut self) {
		if 0 != self.flags & TICK_TITLE {
			self.flags &= ! TICK_TITLE;
			if self.title.is_empty() {
				self.buf.truncate(PART_TITLE, 0);
			}
			else {
				self.buf.replace(
					PART_TITLE,
					&self.title.fitted(self.last_width - 1),
				);
			}
		}
	}

	/// # Tick Total.
	///
	/// This updates the "total" portion of the buffer as needed.
	fn tick_set_total(&mut self) {
		if 0 != self.flags & TICK_TOTAL {
			self.flags &= ! TICK_TOTAL;
			self.buf.replace(PART_TOTAL, &NiceU32::from(self.total));
		}
	}

	/// # Tick Width.
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
/// `Witching` is a progress bar wrapper built around a collection of paths.
/// Each (parallel) iteration of the set results in a tick, providing a nice
/// little ASCII animation to follow while data is being processed.
///
/// Compared with more general-purpose libraries like [`indicatif`](https://crates.io/crates/indicatif), `Witching`
/// is incredibly lightweight and efficient, but it also lacks much in the way
/// of customizability.
///
/// All progress bars include an elapsed time and progress shown as a ratio and
/// percent. If the window is large enough, an actual "bar" is displayed as well.
/// `Witching`s can optionally include a title and a list of active tasks.
///
/// That's it! Short and sweet.
///
/// ## Examples
///
/// `Witching` is instantiated using a builder pattern. Simple chain the desired
/// `with_*()` methods together along with [`Witching::run`] when you're ready to go!
///
/// ```ignore
/// use fyi_witcher::Witcher;
/// use fyi_witcher::Witching;
///
/// // Find the files you want.
/// let files = Witcher::default()
///     .with_ext(b".jpg")
///     .with_path("/my/dir")
///     .into_witching() // Convert it to a Witching instance.
///     .with_title("My Progress Title") // Set whatever options.
///     .run(|p| { ... }); // Run your magic!
/// ```
pub struct Witching {
	/// The set to progress through.
	set: Vec<PathBuf>,
	/// The stateful data.
	inner: Arc<Mutex<WitchingInner>>,
	/// Flags.
	flags: u8,
	/// Summary labels.
	///
	/// The first byte stores the boundary between the singular and plural
	/// labels, such that `label[1..label[0]]` is singular, and `label[label[0]..]`
	/// is plural.
	label: Vec<u8>,
}

impl Default for Witching {
	fn default() -> Self {
		Self {
			set: Vec::new(),
			inner: Arc::new(Mutex::new(WitchingInner::default())),
			flags: 0,
			// "file" and "files" respectively.
			label: vec![5, 102, 105, 108, 101, 102, 105, 108, 101, 115],
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
	/// # With Flags.
	///
	/// Set the desired operational flags.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use fyi_witcher::{
	///     Witcher,
	///     Witching,
	///     WITCHING_DIFF,
	///     WITCHING_SUMMARIZE,
	/// };
	///
	/// // Find and run!
	/// Witcher::default()
	///     .with_ext(b".jpg")
	///     .with_path("/my/dir")
	///     .into_witching() // Convert it to a Witching instance.
	///     .with_flags(WITCHING_SUMMARIZE | WITCHING_DIFF)
	///     .run(|p| { ... }); // Run your magic!
	/// ```
	pub fn with_flags(mut self, flags: u8) -> Self {
		self.set_flags(flags);
		self
	}

	#[must_use]
	/// # With Labels.
	///
	/// The `Witching` summary will report how many "files" were run. Use this
	/// method to call them "images" or "documents" or whatever else.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use fyi_witcher::Witcher;
	/// use fyi_witcher::Witching;
	///
	/// // Find and run!
	/// Witcher::default()
	///     .with_ext(b".jpg")
	///     .with_path("/my/dir")
	///     .into_witching() // Convert it to a Witching instance.
	///     .with_labels("image", "images")
	///     .run(|p| { ... });
	/// ```
	pub fn with_labels<S>(mut self, one: S, many: S) -> Self
	where S: AsRef<str> {
		self.set_labels(one, many);
		self
	}

	#[must_use]
	/// # With Title.
	///
	/// Progress bars can optionally start with a title line. This method sets
	/// that value.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use fyi_witcher::Witcher;
	/// use fyi_witcher::Witching;
	///
	/// // Find and run!
	/// Witcher::default()
	///     .with_ext(b".jpg")
	///     .with_path("/my/dir")
	///     .into_witching() // Convert it to a Witching instance.
	///     .with_title("My Title")
	///     .run(|p| { ... });
	/// ```
	pub fn with_title<S> (self, title: S) -> Self
	where S: Into<Msg> {
		self.set_title(title);
		self
	}

	/// # Set Flags.
	///
	/// While `Witching` is generally meant to be set up by chaining together
	/// builder methods, you can use this method to set the flags for an
	/// object that has been saved to a variable.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witching;
	///
	/// let mut witch = Witching::default();
	/// witch.set_flags(0);
	/// ```
	pub fn set_flags(&mut self, flags: u8) { self.flags = flags; }

	/// # Set Labels.
	///
	/// While `Witching` is generally meant to be set up by chaining together
	/// builder methods, you can use this method to set the summary labels for
	/// an object that has been saved to a variable.
	///
	/// ## Panics
	///
	/// Panics if either label is empty, or if their combined length is greater
	/// than `255`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_witcher::Witching;
	///
	/// let mut witch = Witching::default();
	/// witch.set_labels("animal", "animals");
	/// ```
	pub fn set_labels<S>(&mut self, one: S, many: S)
	where S: AsRef<str> {
		let one: &[u8] = one.as_ref().as_bytes();
		let many: &[u8] = many.as_ref().as_bytes();

		assert!(! one.is_empty() && ! many.is_empty() && one.len() + many.len() <= 255);

		unsafe { self.set_labels_unchecked(one, many); }
	}

	/// # Set Labels (Unchecked).
	///
	/// This works just like [`Witching::set_labels`], except it accepts bytes
	/// directly.
	///
	/// ## Safety
	///
	/// Both labels must have a length, and their combined length must not
	/// exceed `255`.
	pub unsafe fn set_labels_unchecked(&mut self, one: &[u8], many: &[u8]) {
		// Make sure we start with one spot for the boundary.
		self.label.truncate(0);
		self.label.push(one.len() as u8 + 1);

		// Add the singular.
		self.label.extend_from_slice(one);

		// And add the plural.
		self.label.extend_from_slice(many);
	}

	#[must_use]
	#[allow(clippy::missing_const_for_fn)] // Doesn't work.
	/// # Into Vec.
	///
	/// Consume and return the path collection. This may be useful in cases
	/// where you've run through the set, but need to perform non-progress-related
	/// actions afterwards.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use fyi_witcher::Witcher;
	/// use fyi_witcher::Witching;
	///
	/// // Set up the instance.
	/// let mut witch = Witcher::default()
	///     .with_ext(b".jpg")
	///     .with_path("/my/dir")
	///     .into_witching() // Convert it to a Witching instance.
	///     .with_title("My Title");
	///
	/// // Run your magic.
	/// witch.run(|p| { ... });
	///
	/// // And get the original vector back.
	/// let files: Vec<PathBuf> = witch.into_vec();
	/// ```
	pub fn into_vec(self) -> Vec<PathBuf> { self.set }



	// ------------------------------------------------------------------------
	// Operations
	// ------------------------------------------------------------------------

	#[must_use]
	/// # Total File(s) Size.
	///
	/// Add up the size of all files in a set. Calculations are run in parallel so
	/// should be fairly fast depending on the file system.
	fn du(&self) -> u64 {
		self.set.par_iter()
			.map(|x| x.metadata().map_or(0, |m| m.len()))
			.sum()
	}

	/// # Label.
	///
	/// What label should we be using? One or many?
	fn label(&self) -> &str {
		if self.set.len() == 1 {
			unsafe { std::str::from_utf8_unchecked(&self.label[1..usize::from(self.label[0])]) }
		}
		else {
			unsafe { std::str::from_utf8_unchecked(&self.label[usize::from(self.label[0])..]) }
		}
	}

	/// # Run!
	///
	/// This method is used to start the actual progression! This will iterate
	/// through each path in parallel, sending each to the provided callback.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use fyi_witcher::Witcher;
	/// use fyi_witcher::Witching;
	///
	/// // Find and run!
	/// Witcher::default()
	///     .with_path("/my/dir")
	///     .with_ext(b".jpg")
	///     .into_witching()   // Convert it to a Witching instance.
	///     .run(|p| { ... }); // Here's the magic.
	/// ```
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

	/// # (Actually) Run!
	///
	/// This internal method is used for iterations requiring progress display.
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

	/// # Summary.
	///
	/// This is the base summary, no prefix.
	///
	/// `X files in M minutes and S seconds.`
	fn summary(&self) -> String {
		format!(
			"{} {} in {}.",
			NiceU32::from(self.total()).as_str(),
			self.label(),
			NiceElapsed::from(self.elapsed()).as_str(),
		)
	}

	/// # Summarize.
	///
	/// This prints a simple summary after iteration has completed. It is
	/// enabled using the [`WITCHING_SUMMARIZE`] flag and reads something like:
	///
	/// `Done: 5 files in 3 seconds.`
	fn summarize(&self) {
		fyi_msg::done!(self.summary(), true);
	}

	/// # Summarize (with savings).
	///
	/// This version of the summary compares the before and after bytes and
	/// notes how much space has been saved. It is intended for operations that
	/// minify or compress the file paths in the set.
	///
	/// This is engaged when both [`WITCHING_SUMMARIZE`] and [`WITCHING_DIFF`] flags
	/// are set and will return a message like:
	///
	/// `Crunched: 5 files in 3 seconds, saving 2 bytes. (1.00%)`
	/// `Crunched: 5 files in 3 seconds. (No savings.)`
	fn summarize_diff(&self, before: u64) {
		let after: u64 = self.du();

		if 0 == after || before <= after {
			Msg::new(MsgKind::Crunched, self.summary())
				.with_suffix(" \x1b[2m(No savings.)\x1b[0m")
		}
		else {
			Msg::new(MsgKind::Crunched, self.summary())
				.with_suffix(format!(
					" \x1b[2m(Saved {} bytes, {}.)\x1b[0m",
					NiceU64::from(before - after).as_str(),
					NicePercent::from(1.0 - (after as f64 / before as f64)).as_str(),
				))
		}
			.with_newline(true)
			.eprint();
	}

	/// # Summarize empty.
	///
	/// This summary prints when the set is empty and the instance has the
	/// [`WITCHING_SUMMARIZE`] flag set. It simply reads:
	///
	/// `No files were found.`
	fn summarize_empty(&self) {
		fyi_msg::warning!(format!("No {} were found.", self.label()), true);
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

	/// # Stop.
	///
	/// Wrapper for `WitchingInner::stop()`.
	fn stop(&self) {
		let mut ptr = crate::mutex_ptr!(self.inner);
		ptr.stop();
	}

	/// # Set Title.
	///
	/// Wrapper for `WitchingInner::set_title()`.
	pub fn set_title<S> (&self, title: S)
	where S: Into<Msg> {
		let mut ptr = crate::mutex_ptr!(self.inner);
		ptr.set_title(title);
	}
}

/// # Tick.
///
/// Wrapper for `WitchingInner::tick()`.
fn progress_tick(inner: &Arc<Mutex<WitchingInner>>) -> bool {
	let mut ptr = crate::mutex_ptr!(inner);
	ptr.tick()
}

/// # End Task.
///
/// Wrapper for `WitchingInner::end_task()`.
fn progress_end(inner: &Arc<Mutex<WitchingInner>>, task: &PathBuf) {
	let mut ptr = crate::mutex_ptr!(inner);
	ptr.end_task(task);
}

/// # Start Task.
///
/// Wrapper for `WitchingInner::start_task()`.
fn progress_start(inner: &Arc<Mutex<WitchingInner>>, task: &PathBuf) {
	let mut ptr = crate::mutex_ptr!(inner);
	ptr.start_task(task);
}

#[inline]
/// # Format Task Into Message.
fn task_msg(path: &PathBuf) -> Msg {
	// This starts with a ↳.
	Msg::custom_unchecked("    \u{21b3} ", path.to_str().unwrap_or_default())
		.with_newline(true)
}
