/*!
# FYI Msg - Progless Error
*/

use std::{
	error::Error,
	fmt,
};



#[derive(Debug, Copy, Clone)]
/// # Obligatory error type.
pub enum ProglessError {
	/// # Empty task.
	EmptyTask,
	/// # Length (task) overflow.
	TaskOverflow,
	/// # Length (total) must be non-zero.
	EmptyTotal,
	/// # Length (total) overflow.
	TotalOverflow,
}

impl fmt::Display for ProglessError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl Error for ProglessError {}

impl ProglessError {
	#[must_use]
	/// # As Str.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::EmptyTask => "Task names cannot be empty.",
			Self::TaskOverflow => "Task names cannot exceed 65,535 bytes.",
			Self::EmptyTotal => "At least one task is required.",
			Self::TotalOverflow => "The total number of tasks cannot exceed 4,294,967,295.",
		}
	}
}
