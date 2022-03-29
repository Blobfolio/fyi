/*!
# FYI Msg - Progless Tasks
*/

use crate::{
	fitted,
	ProglessError,
};
use std::{
	borrow::Borrow,
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

impl TryFrom<&[u8]> for ProglessTask {
	type Error = ProglessError;

	fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
		// It has to fit in a u16.
		if src.is_empty() { Err(ProglessError::EmptyTask) }
		else {
			Ok(Self {
				task: Box::from(src),
				width: u16::try_from(fitted::width(src)).map_err(|_| ProglessError::TaskOverflow)?,
			})
		}
	}
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

impl PartialEq for ProglessTask {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.task == other.task }
}

impl ProglessTask {
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
