/*!
# FYI Core: Prefix
*/

use ansi_term::{
	Colour,
	Style,
};



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
		let prefix = self.prefix();
		match prefix.is_empty() {
			true => f.write_str(""),
			false => f.write_str(&prefix),
		}
	}
}

impl<'b> Prefix<'b> {
	/// Color.
	pub fn color(&self) -> Style {
		match *self {
			Self::Custom(ref x, c) => match x.is_empty() {
				true => Style::new(),
				false => Colour::Fixed(c).bold(),
			},
			Self::Debug => Colour::Cyan.bold(),
			Self::Error => Colour::Red.bold(),
			Self::Info => Colour::Cyan.bold(),
			Self::Notice => Colour::Purple.bold(),
			Self::Success => Colour::Green.bold(),
			Self::Warning => Colour::Yellow.bold(),
			_ => Style::new(),
		}
	}

	/// Happy or sad?
	pub fn happy(&self) -> bool {
		match *self {
			Self::Error | Self::Warning => false,
			_ => true,
		}
	}

	/// Label.
	pub fn label(&self) -> &'b str {
		match *self {
			Self::Custom(ref x, _) => match x.is_empty() {
				true => "",
				false => x,
			},
			Self::Debug => "Debug",
			Self::Error => "Error",
			Self::Info => "Info",
			Self::Notice => "Notice",
			Self::Success => "Success",
			Self::Warning => "Warning",
			_ => "",
		}
	}

	/// Prefix (Colored).
	pub fn prefix(&self) -> String {
		let label = self.label();

		match label.is_empty() {
			true => String::new(),
			false => format!("{} ", self.color().paint([&label, ":"].concat()))
				.to_string(),
		}
	}
}
