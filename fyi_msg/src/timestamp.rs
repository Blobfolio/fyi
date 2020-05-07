/*!
# Timestamp

Printing local datetime in Rust sucks. This is an attempt to do that one thing
as quickly as possible.

The result is a colored UTF-8 string like "[0000-00-00 00:00:00]".

## Example:

```no_run
use fyi_msg::Timestamp;

println({}, Timestamp::new());
// e.g. [2020-05-06 12:00:33]
```
*/

use std::{
	borrow::Borrow,
	fmt,
	ops::Deref,
};



#[derive(Debug, Clone, PartialEq, Hash)]
/// The Timestamp!
pub struct Timestamp(Vec<u8>);

impl AsRef<str> for Timestamp {
	#[inline]
	/// As Str.
	fn as_ref(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
	}
}

impl AsRef<[u8]> for Timestamp {
	#[inline]
	/// As Str.
	fn as_ref(&self) -> &[u8] {
		self
	}
}

impl Borrow<str> for Timestamp {
	#[inline]
	fn borrow(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
	}
}

impl Borrow<[u8]> for Timestamp {
	#[inline]
	fn borrow(&self) -> &[u8] {
		self
	}
}

impl Default for Timestamp {
	/// Default.
	fn default() -> Self {
		Timestamp::new()
	}
}

impl Deref for Timestamp {
	type Target = [u8];

	/// Deref.
	///
	/// We deref to `&[u8]` as most contexts want bytes.
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl fmt::Display for Timestamp {
	#[inline]
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}

// The fastest way to write our date bits to the buffer is by index,
// which requires some repetitive code best offloaded to a macro.
macro_rules! write_time_chunk {
	($buf:ident, $var:ident, $val:expr, $start:literal, $end:literal) => {
		$var = Timestamp::time_format_dd($val);
		if $buf[$start] != $var[0] {
			$buf[$start] = $var[0];
		}
		if $buf[$end] != $var[1] {
			$buf[$end] = $var[1];
		}
	};
}

impl Timestamp {
	#[must_use]
	/// New timestamp.
	pub fn new() -> Self {
		use chrono::{
			Datelike,
			Local,
			Timelike,
		};

		let mut buf: Vec<u8> = b"\x1B[2m[\x1B[34m2000-00-00 00:00:00\x1B[39m]\x1B[0m".to_vec();
		let now = Local::now();

		// Note: the shortcut we're taking to patch in the year will require
		// revisiting prior to 2060. Haha.
		let mut tmp;
		write_time_chunk!(buf, tmp, (now.year() as u32).saturating_sub(2000), 12, 13);
		write_time_chunk!(buf, tmp, now.month(), 15, 16);
		write_time_chunk!(buf, tmp, now.day(), 18, 19);
		write_time_chunk!(buf, tmp, now.hour(), 21, 22);
		write_time_chunk!(buf, tmp, now.minute(), 24, 25);
		write_time_chunk!(buf, tmp, now.second(), 27, 28);

		// Done!
		Self(buf)
	}

	#[must_use]
	/// Time Number to String.
	pub fn time_format_dd(num: u32) -> &'static [u8] {
		match num {
			0 => b"00",
			1 => b"01",
			2 => b"02",
			3 => b"03",
			4 => b"04",
			5 => b"05",
			6 => b"06",
			7 => b"07",
			8 => b"08",
			9 => b"09",
			10 => b"10",
			11 => b"11",
			12 => b"12",
			13 => b"13",
			14 => b"14",
			15 => b"15",
			16 => b"16",
			17 => b"17",
			18 => b"18",
			19 => b"19",
			20 => b"20",
			21 => b"21",
			22 => b"22",
			23 => b"23",
			24 => b"24",
			25 => b"25",
			26 => b"26",
			27 => b"27",
			28 => b"28",
			29 => b"29",
			30 => b"30",
			31 => b"31",
			32 => b"32",
			33 => b"33",
			34 => b"34",
			35 => b"35",
			36 => b"36",
			37 => b"37",
			38 => b"38",
			39 => b"39",
			40 => b"40",
			41 => b"41",
			42 => b"42",
			43 => b"43",
			44 => b"44",
			45 => b"45",
			46 => b"46",
			47 => b"47",
			48 => b"48",
			49 => b"49",
			50 => b"50",
			51 => b"51",
			52 => b"52",
			53 => b"53",
			54 => b"54",
			55 => b"55",
			56 => b"56",
			57 => b"57",
			58 => b"58",
			59 => b"59",
			_ => b"",
		}
	}
}



mod tests {
	use super::Timestamp;

	#[test]
	fn time_format_dd() {
		_time_format_dd(0, "00");
		_time_format_dd(1, "01");
		_time_format_dd(9, "09");
		_time_format_dd(50, "50");
		_time_format_dd(60, "");
	}

	fn _time_format_dd(num: u32, expected: &str) {
		assert_eq!(
			Timestamp::time_format_dd(num),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}
}
