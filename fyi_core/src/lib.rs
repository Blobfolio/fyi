/*!
# FYI Core
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

extern crate ansi_term;
extern crate chrono;
extern crate strip_ansi_escapes;

use ansi_term::{Colour, Style};
use chrono::prelude::*;



#[derive(Debug, Clone)]
/// Generic message.
pub enum Msg {
	/// Custom.
	Custom(String, String),
	/// Debug.
	Debug(String),
	/// Error.
	Error(String),
	/// Notice.
	Notice(String),
	/// Success.
	Success(String),
	/// Warning.
	Warning(String),
}

impl std::fmt::Display for Msg {
	/// Display.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let prefix = self.prefix();
		match prefix.is_empty() {
			true => f.write_str(&self.msg()),
			false => f.write_str(&format!("{} {}", &prefix, self.msg())),
		}
	}
}

impl Msg {
	/// Msg.
	pub fn msg(&self) -> String {
		format!("{}", Style::new().bold().paint(match *self {
			Self::Custom(_, ref x) => x.clone(),
			Self::Debug(ref x) => x.clone(),
			Self::Error(ref x) => x.clone(),
			Self::Notice(ref x) => x.clone(),
			Self::Success(ref x) => x.clone(),
			Self::Warning(ref x) => x.clone(),
		}))
	}

	/// Prefix (Colored).
	pub fn prefix(&self) -> String {
		match *self {
			Self::Custom(ref x, _) => match x.is_empty() {
				true => "".to_string(),
				false => format!(
					"{}{}",
					Colour::Fixed(199).bold().paint(x.clone()),
					Colour::Fixed(199).bold().paint(":".to_string())
				),
			},
			Self::Debug(_) => format!("{}", Colour::Cyan.bold().paint("Debug:")),
			Self::Error(_) => format!("{}", Colour::Red.bold().paint("Error:")),
			Self::Notice(_) => format!("{}", Colour::Purple.bold().paint("Notice:")),
			Self::Success(_) => format!("{}", Colour::Green.bold().paint("Success:")),
			Self::Warning(_) => format!("{}", Colour::Yellow.bold().paint("Warning:")),
		}
	}

	/// Print it!
	pub fn print(&self) {
		self.print_special(true, 0, false);
	}

	/// Print it with options.
	pub fn print_special(&self, color: bool, indent: u64, timestamp: bool) {
		let mut msg = format!("{}{}", indentation(indent), self.to_string());

		// Append timestamp?
		if true == timestamp {
			let timebit: String = format!(
				"{}[{}]",
				indentation(indent),
				Style::new().dimmed().paint(format!(
					"{}",
					Local::now().format("%F %T"),
				))
			);

			msg = format!("{}\n{}", timebit, msg);
		}

		// Remove color?
		if false == color {
			msg = strip_styles(msg);
		}

		// Do it!
		match *self {
			Self::Error(_) | Self::Warning(_) => eprintln!("{}", msg),
			_ => println!("{}", msg),
		}
	}
}



/// Indent.
fn indentation(indent: u64) -> String {
	if 0 < indent {
		String::from_utf8(vec![b' '; (indent * 4) as usize]).unwrap_or("".to_string())
	}
	else {
		"".to_string()
	}
}

/// Strip Styles
fn strip_styles<S> (text: S) -> String
where S: Into<String> {
	let text = strip_ansi_escapes::strip(text.into())
		.unwrap_or(Vec::new());
	std::str::from_utf8(&text)
		.unwrap_or("")
		.to_string()
}
