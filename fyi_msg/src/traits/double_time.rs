/*!
# FYI Msg Traits: `DoubleTime`

This is a very simple trait for converting a time-like `u64` value (i.e. 0–59)
into 2-digit, human-readable UTF-8, either as a borrowed `&str` or `&[u8]`
depending on the implementation.

The primary use case is compiling a "00:00:00"-like string.

Values greater than 59 are returned empty.

## Example:

```no_run
use fyi_msg::traits::DoubleTime;

let time = <[u8]>::double_digit_time(3);
assert_eq!(time, b"03");

let time = <[u8]>::double_digit_time(13);
assert_eq!(time, b"13");
```
*/



/// Double-digit Time, i.e. 00-59.
pub trait DoubleTime {
	/// Double-digit Time, i.e. 00-59.
	fn double_digit_time(num: u64) -> &'static Self;
}

impl DoubleTime for [u8] {
	/// Double-digit Time, i.e. 00-59.
	fn double_digit_time(num: u64) -> &'static [u8] {
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

impl DoubleTime for str {
	/// Double-digit Time, i.e. 00-59.
	fn double_digit_time(num: u64) -> &'static str {
		match num {
			0 => "00",
			1 => "01",
			2 => "02",
			3 => "03",
			4 => "04",
			5 => "05",
			6 => "06",
			7 => "07",
			8 => "08",
			9 => "09",
			10 => "10",
			11 => "11",
			12 => "12",
			13 => "13",
			14 => "14",
			15 => "15",
			16 => "16",
			17 => "17",
			18 => "18",
			19 => "19",
			20 => "20",
			21 => "21",
			22 => "22",
			23 => "23",
			24 => "24",
			25 => "25",
			26 => "26",
			27 => "27",
			28 => "28",
			29 => "29",
			30 => "30",
			31 => "31",
			32 => "32",
			33 => "33",
			34 => "34",
			35 => "35",
			36 => "36",
			37 => "37",
			38 => "38",
			39 => "39",
			40 => "40",
			41 => "41",
			42 => "42",
			43 => "43",
			44 => "44",
			45 => "45",
			46 => "46",
			47 => "47",
			48 => "48",
			49 => "49",
			50 => "50",
			51 => "51",
			52 => "52",
			53 => "53",
			54 => "54",
			55 => "55",
			56 => "56",
			57 => "57",
			58 => "58",
			59 => "59",
			_ => "",
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn double_digit_time() {
		_double_digit_time(0, "00");
		_double_digit_time(1, "01");
		_double_digit_time(9, "09");
		_double_digit_time(50, "50");
		_double_digit_time(60, "");
	}

	fn _double_digit_time(num: u64, expected: &str) {
		assert_eq!(
			<[u8]>::double_digit_time(num),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected.as_bytes()
		);

		assert_eq!(
			str::double_digit_time(num),
			expected,
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}
}
