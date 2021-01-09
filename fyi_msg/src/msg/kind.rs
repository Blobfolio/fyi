use super::Msg;
use std::{
	fmt,
	ops::Deref,
};



#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
/// # Message Kind.
pub enum MsgKind {
	/// None.
	None,
	/// Confirm.
	Confirm,
	/// Crunched.
	Crunched,
	/// Debug.
	Debug,
	/// Done.
	Done,
	/// Error.
	Error,
	/// Info.
	Info,
	/// Notice.
	Notice,
	/// Success.
	Success,
	/// Task.
	Task,
	/// Warning.
	Warning,
}

impl AsRef<str> for MsgKind {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl Default for MsgKind {
	#[inline]
	fn default() -> Self { Self::None }
}

impl Deref for MsgKind {
	type Target = [u8];
	fn deref(&self) -> &Self::Target { self.as_bytes() }
}

impl fmt::Display for MsgKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<&str> for MsgKind {
	fn from(txt: &str) -> Self {
		match txt {
			"confirm" | "prompt" => Self::Confirm,
			"crunched" => Self::Crunched,
			"debug" => Self::Debug,
			"done" => Self::Done,
			"error" => Self::Error,
			"info" => Self::Info,
			"notice" => Self::Notice,
			"success" => Self::Success,
			"task" => Self::Task,
			"warning" => Self::Warning,
			_ => Self::None,
		}
	}
}

/// ## Details.
impl MsgKind {
	#[must_use]
	/// # Is Empty?
	pub const fn is_empty(self) -> bool {
		matches!(self, Self::None)
	}

	#[must_use]
	/// # Length.
	pub const fn len(self) -> usize {
		match self {
			Self::None => 0,
			Self::Confirm => 26,
			Self::Crunched => 21,
			Self::Done | Self::Info => 17,
			Self::Debug | Self::Error => 18,
			Self::Notice => 19,
			Self::Success | Self::Warning => 20,
			Self::Task => 23,
		}
	}
}

/// ## Conversion.
impl MsgKind {
	#[must_use]
	/// # As Bytes.
	pub const fn as_bytes(self) -> &'static [u8] {
		match self {
			Self::None => &[],
			Self::Confirm => b"\x1b[1;38;5;208mConfirm:\x1b[0m ",
			Self::Crunched => b"\x1b[92;1mCrunched:\x1b[0m ",
			Self::Debug => b"\x1b[96;1mDebug:\x1b[0m ",
			Self::Done => b"\x1b[92;1mDone:\x1b[0m ",
			Self::Error => b"\x1b[91;1mError:\x1b[0m ",
			Self::Info => b"\x1b[95;1mInfo:\x1b[0m ",
			Self::Notice => b"\x1b[95;1mNotice:\x1b[0m ",
			Self::Success => b"\x1b[92;1mSuccess:\x1b[0m ",
			Self::Task => b"\x1b[1;38;5;199mTask:\x1b[0m ",
			Self::Warning => b"\x1b[93;1mWarning:\x1b[0m ",
		}
	}

	#[allow(clippy::missing_const_for_fn)] // Can't const unsafe.
	#[must_use]
	/// # As Str.
	pub fn as_str(self) -> &'static str {
		unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
	}

	/// # Into Message.
	pub fn into_msg<S>(self, msg: S) -> Msg
	where S: AsRef<str> {
		Msg::new(self, msg)
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_len() {
		for p in &[
			MsgKind::Confirm,
			MsgKind::Crunched,
			MsgKind::Debug,
			MsgKind::Done,
			MsgKind::Error,
			MsgKind::Info,
			MsgKind::None,
			MsgKind::Notice,
			MsgKind::Success,
			MsgKind::Task,
			MsgKind::Warning,
		] {
			assert_eq!(p.len(), p.as_bytes().len());
			assert_eq!(p.is_empty(), p.as_bytes().is_empty());
		}
	}
}
