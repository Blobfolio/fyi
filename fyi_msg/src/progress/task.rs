/*!
# FYI Msg - Progless Tasks
*/

use crate::fitted;
use std::{
	borrow::Borrow,
	cmp::Ordering,
	hash::{
		Hash,
		Hasher,
	},
};
use super::TASK_PREFIX;



#[derive(Debug, Clone)]
/// # A Task.
///
/// This holds a boxed slice and the pre-calculated display width of said
/// slice. Though stored as raw bytes, the value is valid UTF-8.
pub(super) struct ProglessTask {
	task: Box<[u8]>,
	width: u16,
}

impl Borrow<[u8]> for ProglessTask {
	#[inline]
	fn borrow(&self) -> &[u8] { &self.task }
}

impl Eq for ProglessTask {}

impl Hash for ProglessTask {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) { self.task.hash(state); }
}

impl Ord for ProglessTask {
	fn cmp(&self, other: &Self) -> Ordering { self.task.cmp(&other.task) }
}

impl PartialEq for ProglessTask {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.task == other.task }
}

impl PartialOrd for ProglessTask {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl ProglessTask {
	/// # New.
	///
	/// Create a new task from raw bytes.
	///
	/// This will return `None` if the source is empty or larger than `u16`.
	pub(super) fn new<B>(src: B) -> Option<Self>
	where B: AsRef<[u8]> {
		let src = src.as_ref();
		if src.is_empty() { None }
		else if let Ok(width) = u16::try_from(fitted::width(src)) {
			Some(Self {
				task: Box::from(src),
				width,
			})
		}
		else { None }
	}

	/// # Push To.
	///
	/// Push this task to the vector buffer, ensuring it fits the specified
	/// width.
	pub(super) fn push_to(&self, buf: &mut Vec<u8>, width: u8) {
		let avail = width.saturating_sub(6);
		if self.width > u16::from(avail) {
			let end = fitted::length_width(&self.task, usize::from(avail));
			if end > 0 {
				buf.extend_from_slice(TASK_PREFIX);
				buf.extend_from_slice(&self.task[..end]);
				buf.push(b'\n');
			}
		}
		else {
			buf.extend_from_slice(TASK_PREFIX);
			buf.extend_from_slice(&self.task);
			buf.push(b'\n');
		}
	}
}
