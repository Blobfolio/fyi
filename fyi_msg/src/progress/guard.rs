/*!
# FYI Msg - Progless Increment Guard.
*/

use std::sync::Arc;
use super::ProglessInner;

#[expect(unused_imports, reason = "For docs.")]
use super::Progless;



#[derive(Debug)]
/// # Progless Task Guard.
///
/// This guard is returned by [`Progless::task`].
///
/// When dropped, the task will automatically be removed from the [`Progless`]
/// output, and its done count will be incremented by one.
///
/// To drop it _without_ incrementing the done count, use
/// [`ProglessTaskGuard::cancel`].
pub struct ProglessTaskGuard {
	/// # Task Name.
	task: String,

	/// # Increment on Drop?
	inc: bool,

	/// # Generation.
	///
	/// This is a simple sanity control to help prevent a guard from one
	/// progress cycle affecting the totals of a subsequent one (i.e. after
	/// [`Progless::reset`])
	cycle: u8,

	/// # Linked [`Progless`] Instance.
	progress: Arc<ProglessInner>,
}

impl Drop for ProglessTaskGuard {
	#[inline]
	fn drop(&mut self) {
		self.progress.remove_guard(self.task.as_str(), self.cycle, self.inc);
	}
}

impl ProglessTaskGuard {
	/// # From Parts.
	pub(super) const fn from_parts(
		task: String,
		cycle: u8,
		progress: Arc<ProglessInner>,
	) -> Self {
		Self { task, inc: true, cycle, progress }
	}

	#[inline]
	/// # Cancel Guard.
	///
	/// Remove the task from the [`Progless`] output _without_ incrementing
	/// the done count.
	pub fn cancel(mut self) {
		self.inc = false;
		drop(self);
	}
}
