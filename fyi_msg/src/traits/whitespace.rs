/*!
# FYI Msg Traits: Generate Whitespace

This trait provides a means of slicing whitespace from a static ref. It can be
useful for quick string padding, etc.

Data is returned as either a `&[u8]` or `&str` depending on the implementation.

If you need more than 256 contiguous spaces, call the method more than once.

## Example:

```no_run
use fyi_msg::traits::WhiteSpace;

let ten_spaces = <[u8]>::whitespace(10);
assert_eq!(ten_spaces, b"          ");
```
*/



/// Generate Whitespace.
pub trait WhiteSpace {
	/// Cache the whitespace.
	const WHITES: &'static [u8; 255] = b"                                                                                                                                                                                                                                                               ";

	/// Generate Whitespace.
	fn whitespace(num: u8) -> &'static Self;
}

impl WhiteSpace for [u8] {
	/// Generate Whitespace.
	fn whitespace(num: u8) -> &'static [u8] {
		let num: usize = num as usize;
		&Self::WHITES[0..num]
	}
}

impl WhiteSpace for str {
	/// Generate Whitespace.
	fn whitespace(num: u8) -> &'static str {
		let num: usize = num as usize;
		unsafe { std::str::from_utf8_unchecked(&Self::WHITES[0..num]) }
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn whitespace() {
		_whitespace(0_u8);
		_whitespace(10_u8);
		_whitespace(100_u8);
		_whitespace(150_u8);
		_whitespace(255_u8);
	}

	fn _whitespace(num: u8) {
		assert_eq!(
			<[u8]>::whitespace(num).len(),
			num as usize,
			"{} should produce {} spaces",
			num,
			num
		);

		assert_eq!(
			str::whitespace(num).len(),
			num as usize,
			"{} should produce {} spaces",
			num,
			num
		);
	}
}
