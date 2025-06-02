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
/// output and the done count will be increased by one.
///
/// To drop it _without_ incrementing the done count — because you e.g.
/// changed your mind or need to come back to it later — use
/// [`ProglessTaskGuard::cancel`].
pub struct ProglessTaskGuard<'a> {
	/// # Task Name.
	task: String,

	/// # Increment on Drop?
	inc: bool,

	/// # Cycle Number.
	///
	/// This value is used to prevent a guard from one [`Progless`] cycle
	/// affecting the totals of a subsequent one (after e.g. a reset).
	cycle: u8,

	/// # Linked [`Progless`] Instance.
	progress: &'a Arc<ProglessInner>,
}

impl Drop for ProglessTaskGuard<'_> {
	#[inline]
	fn drop(&mut self) {
		self.progress.remove_guard(self.task.as_str(), self.cycle, self.inc);
	}
}

impl<'a> ProglessTaskGuard<'a> {
	/// # From Parts.
	pub(super) const fn from_parts(
		task: String,
		cycle: u8,
		progress: &'a Arc<ProglessInner>,
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
