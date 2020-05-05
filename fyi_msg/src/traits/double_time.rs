/*!
# FYI Msg Traits: `DoubleTime`

This is a very simple trait for converting time-like unsigned integer values
(i.e. 0–59) to 2-digit, human-readable UTF-8 strings.

The primary use case is compiling a "00:00:00"-like string.

Values greater than 59 are returned empty.

## Example:

```no_run
use fyi_msg::traits::DoubleTime;

assert_eq!(3.double_digit_time(), "03");
assert_eq!(45, "45");
```
*/



/// Double-digit Time, i.e. 00-59.
pub trait DoubleTime {
	/// Double-digit Time, i.e. 00-59.
	fn double_digit_time(self) -> &'static str;
}

macro_rules! impl_double_time {
	($type:ty) => {
		impl DoubleTime for $type {
			/// Double-digit Time, i.e. 00-59.
			fn double_digit_time(self) -> &'static str {
				match self {
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
	};
}

impl_double_time!(usize);
impl_double_time!(u8);
impl_double_time!(u16);
impl_double_time!(u32);
impl_double_time!(u64);
impl_double_time!(u128);



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
			num.double_digit_time(),
			expected,
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}
}
