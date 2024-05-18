/*!
# FYI Msg - Progless Tasks
*/

use crate::{
	fitted,
	iter::NoAnsi,
};
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
	inner: Box<[u8]>,
	width: u16,
}

impl Borrow<[u8]> for ProglessTask {
	#[inline]
	fn borrow(&self) -> &[u8] { &self.inner }
}

impl Eq for ProglessTask {}

impl Hash for ProglessTask {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) { self.inner.hash(state); }
}

impl Ord for ProglessTask {
	#[inline]
	fn cmp(&self, other: &Self) -> Ordering { self.inner.cmp(&other.inner) }
}

impl PartialEq for ProglessTask {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl PartialOrd for ProglessTask {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl ProglessTask {
	/// # New.
	///
	/// Create a new task from raw bytes.
	///
	/// This will return `None` if the source is empty or larger than `u16`.
	pub(super) fn new<S>(src: S) -> Option<Self>
	where S: AsRef<str> {
		let inner = Self::fmt(src)?.into_bytes();
		let width = u16::try_from(fitted::width(&inner)).ok()?;

		if width != 0 {
			Some(Self {
				inner: inner.into_boxed_slice(),
				width
			})
		}
		else { None }
	}

	/// # Format String.
	///
	/// This strips ANSI sequences and control characters, and converts new
	/// lines to spaces to improve display consistency.
	///
	/// If the result is empty, `None` is returned.
	pub(super) fn fmt<S>(src: S) -> Option<String>
	where S: AsRef<str> {
		let out = NoAnsi::<char, _>::new(src.as_ref().trim_end().chars())
			.filter_map(|c|
				if c == '\n' { Some(' ') }
				else if c.is_control() { None }
				else { Some(c) }
			)
			.collect::<String>();
		if out.is_empty() { None }
		else { Some(out) }
	}

	/// # Push To.
	///
	/// Push this task to the vector buffer, ensuring it fits the specified
	/// width.
	pub(super) fn push_to(&self, buf: &mut Vec<u8>, width: u8) {
		let avail = width.saturating_sub(6);
		if self.width <= u16::from(avail) {
			buf.extend_from_slice(TASK_PREFIX);
			buf.extend_from_slice(&self.inner);
			buf.push(b'\n');
		}
		else {
			let end = fitted::length_width(&self.inner, usize::from(avail));
			if end != 0 {
				buf.extend_from_slice(TASK_PREFIX);
				buf.extend_from_slice(&self.inner[..end]);
				buf.push(b'\n');
			}
		}
	}
}



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn t_task_fmt() {
		assert_eq!(
			ProglessTask::fmt("\x1b[0mHello\x1b\nWorld!"),
			Some("Hello World!".to_owned()),
		);
	}
}
