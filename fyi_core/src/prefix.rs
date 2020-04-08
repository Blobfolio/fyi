/*!
# FYI Core: Prefix
*/

use std::borrow::Cow;



#[derive(Debug, Clone, Copy, PartialEq)]
/// Generic message.
pub enum Prefix<'b> {
	/// Custom.
	Custom(&'b str, u8),
	/// Debug.
	Debug,
	/// Error.
	Error,
	/// Info.
	Info,
	/// Notice.
	Notice,
	/// Success.
	Success,
	/// Warning.
	Warning,
	/// None.
	None,
}

impl Default for Prefix<'_> {
	/// Default.
	fn default() -> Self {
		Prefix::None
	}
}

impl std::fmt::Display for Prefix<'_> {
	/// Display.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.prefix())
	}
}

impl<'b> Prefix<'b> {
	/// Happy or sad?
	pub fn happy(&self) -> bool {
		match *self {
			Self::Error | Self::Warning => false,
			_ => true,
		}
	}

	/// Prefix (Colored).
	pub fn prefix(&self) -> Cow<'_, str> {
		match *self {
			Self::Custom(ref p, c) => match p.is_empty() {
				true => Cow::Borrowed(""),
				false => Cow::Owned([
					"\u{1B}[1;38;5;",
					c.to_string().as_str(),
					"m",
					p,
					":\u{1B}[0m ",
				].concat()),
			},
			Self::Debug => Cow::Borrowed("\u{1B}[96;1mDebug:\u{1B}[0m "),
			Self::Error => Cow::Borrowed("\u{1B}[91;1mError:\u{1B}[0m "),
			Self::Info => Cow::Borrowed("\u{1B}[96;1mInfo:\u{1B}[0m "),
			Self::Notice => Cow::Borrowed("\u{1B}[95;1mNotice:\u{1B}[0m "),
			Self::Success => Cow::Borrowed("\u{1B}[92;1mSuccess:\u{1B}[0m "),
			Self::Warning => Cow::Borrowed("\u{1B}[93;1mWarning:\u{1B}[0m "),
			_ => Cow::Borrowed(""),
		}
	}
}
