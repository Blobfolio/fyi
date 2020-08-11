/*!
# FYI Progress: Traits: Progress Tasks

This trait provides a method for converting a value into a line item for the
progress iterator.
*/

use crate::traits::FittedRange;
use std::{
	ffi::{
		OsStr,
		OsString,
	},
	os::unix::ffi::OsStrExt,
	path::PathBuf,
};



/// Progressable Type.
///
/// Types with this trait may be used to seed a `Progress`. Right now that
/// just means strings and paths.
///
/// The only required method is `task_name()`, which must convert the value
/// into a meaningful byte slice.
///
/// Anything convertable to bytes could implement this trait; we've just
/// started with the types we're actually using. If you would like to see
/// other `std` types added, just open a request ticket.
pub trait ProgressTask
where Self: Clone + Eq + std::hash::Hash + PartialEq + Sized {
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
			&name[name.fitted_range(width.saturating_sub(6))],
			b"\x1b[0m\n",
		].concat()
	}
}

impl ProgressTask for &OsStr {
	/// Task Name.
	fn task_name(&self) -> &[u8] { self.as_bytes() }
}

impl ProgressTask for OsString {
	#[allow(trivial_casts)]
	/// Task Name.
	fn task_name(&self) -> &[u8] {
		unsafe { &*(self.as_os_str() as *const OsStr as *const [u8]) }
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
