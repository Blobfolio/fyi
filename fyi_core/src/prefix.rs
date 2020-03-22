/*!
# FYI Core: Prefix
*/

use ansi_term::Colour;



#[derive(Debug, Clone, Copy)]
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
	/// Prefix (Colored).
	pub fn prefix(&self) -> String {
		match *self {
			Self::Custom(ref x, c) => match x.is_empty() {
				true => "".to_string(),
				false => format!(
					"{}{} ",
					Colour::Fixed(c).bold().paint(x.clone()),
					Colour::Fixed(c).bold().paint(":".to_string())
				),
			},
			Self::Debug => format!("{} ", Colour::Cyan.bold().paint("Debug:")),
			Self::Error => format!("{} ", Colour::Red.bold().paint("Error:")),
			Self::Info => format!("{} ", Colour::Cyan.bold().paint("Info:")),
			Self::Notice => format!("{} ", Colour::Purple.bold().paint("Notice:")),
			Self::Success => format!("{} ", Colour::Green.bold().paint("Success:")),
			Self::Warning => format!("{} ", Colour::Yellow.bold().paint("Warning:")),
			_ => "".to_string(),
		}
	}
}
