/*!
# FYI Core: Prefix
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

use ansi_term::Colour;



#[derive(Debug, Clone)]
/// Generic message.
pub enum Prefix {
	/// Custom.
	Custom(String, u8),
	/// Debug.
	Debug,
	/// Error.
	Error,
	/// Notice.
	Notice,
	/// Success.
	Success,
	/// Warning.
	Warning,
}

impl std::fmt::Display for Prefix {
	/// Display.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let prefix = self.prefix();
		match prefix.is_empty() {
			true => f.write_str(""),
			false => f.write_str(&prefix),
		}
	}
}

impl Prefix {
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
			Self::Notice => format!("{} ", Colour::Purple.bold().paint("Notice:")),
			Self::Success => format!("{} ", Colour::Green.bold().paint("Success:")),
			Self::Warning => format!("{} ", Colour::Yellow.bold().paint("Warning:")),
		}
	}
}
