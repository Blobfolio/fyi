/*!
# FYI Msg Traits: `HumanBytes`

This trait adds a `.human_bytes()` method to all `AsPrimitive<f64>` values for
producing a UTF-8 string with the most appropriate Mebi unit (B, KiB, etc.).

## Example:

```no_run
use fyi_msg::traits::HumanBytes;

assert_eq!(12003.human_bytes(), "11.72KiB".to_string());
```
*/
use num_traits::cast::AsPrimitive;



/// Human Bytes.
pub trait HumanBytes {
	/// Human Bytes.
	fn human_bytes(self) -> String;
}

impl<N> HumanBytes for N
where N: AsPrimitive<f64> {
	/// Human Bytes.
	fn human_bytes(self) -> String {
		let mut bytes: f64 = self.as_();
		let mut index: usize = 0;
		while bytes >= 1000.0 && index < 4 {
			bytes /= 1024.0;
			index += 1;
		}

		match index {
			0 => {
				let mut out: String = String::with_capacity(4);
				itoa::fmt(&mut out, bytes as usize).expect("Fucked up number.");
				out.push('B');
				out
			},
			1 => format!("{:.*}KiB", 2, bytes),
			2 => format!("{:.*}MiB", 2, bytes),
			3 => format!("{:.*}GiB", 2, bytes),
			_ => format!("{:.*}TiB", 2, bytes),
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn human_bytes() {
		_human_bytes(999, "999B");
		_human_bytes(1000, "0.98KiB");
		_human_bytes(12003, "11.72KiB");
		_human_bytes(4887391, "4.66MiB");
		_human_bytes(499288372, "476.16MiB");
		_human_bytes(99389382145, "92.56GiB");
	}

	fn _human_bytes(num: u64, expected: &str) {
		assert_eq!(
			num.human_bytes(),
			expected.to_string(),
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}
}
