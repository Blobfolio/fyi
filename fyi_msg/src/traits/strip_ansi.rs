/*!
# FYI Msg Traits: Strip ANSI

This trait is provides a wrapper around a simple Regular Expression search/
replace — courtesy of the `regex` crate — to strip out ANSI formatting codes
from a UTF-8 string. The result is either a `Cow<str>` or `Cow<[u8]>`
depending on the source type (`&str` or `&[u8]` respectively).

## Example:

```no_run
use fyi_msg::traits::StripAnsi;

assert_eq!("\x1B[1mBOLD\x1B[0m".strip_ansi(), "BOLD");
```
*/

use std::borrow::Cow;

lazy_static::lazy_static! {
	/// Regex: [u8] ANSI
	pub static ref STRIPPER_BYTES: regex::bytes::Regex =
		regex::bytes::Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
		.unwrap();

	/// Regex: str ANSI
	pub static ref STRIPPER: regex::Regex =
		regex::Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
		.unwrap();
}



/// Remove ANSI codes.
pub trait StripAnsi<'strip> {
	/// Trait return type.
	type Target;

	/// Stripped ANSI.
	fn strip_ansi(&'strip self) -> Self::Target;
}

impl<'strip> StripAnsi<'strip> for [u8] {
	/// Trait return type.
	type Target = Cow<'strip, [u8]>;

	/// Strip ANSI.
	fn strip_ansi(&'strip self) -> Self::Target {
		STRIPPER_BYTES.replace_all(self, regex::bytes::NoExpand(b""))
	}
}

impl<'strip> StripAnsi<'strip> for str {
	/// Trait return type.
	type Target = Cow<'strip, str>;

	/// Strip ANSI.
	fn strip_ansi(&'strip self) -> Self::Target {
		STRIPPER.replace_all(self, "")
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn strip_ansi() {
		_strip_ansi("Normal", "Normal");
		_strip_ansi("\x1B[1mBOLD\x1B[0m", "BOLD");
		_strip_ansi("\x1B[1;96mBjörk\x1B[0m Guðmundsdóttir", "Björk Guðmundsdóttir");
	}

	fn _strip_ansi(text: &str, expected: &str) {
		assert_eq!(
			text.strip_ansi().as_ref(),
			expected,
			"{} should be equivalent to {:?}",
			text,
			expected
		);

		// Convert to bytes to test bytes.
		assert_eq!(
			text.as_bytes().strip_ansi().as_ref(),
			expected.as_bytes(),
			"{:?} should be equivalent to {:?}",
			text.as_bytes(),
			expected.as_bytes()
		);
	}
}
