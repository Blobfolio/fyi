/*!
# FYI Msg Traits: `BytesSaved`

Compare an "after" size to a "before" size to find savings. After must be less
than before *and* greater than zero in order to report.

## Example:

```no_run
use fyi_msg::traits::BytesSaved;

let before: usize = 1000;
assert_eq!(before.bytes_saved(500), 500);
assert_eq!(before.bytes_saved(0), 0); // After cannot be nothing.
assert_eq!(before.bytes_saved(1200), 0); // After cannot be larger than before.
```
*/



/// Bytes Saved
pub trait BytesSaved {
	/// Bytes Saved.
	fn bytes_saved(self, after: Self) -> Self;
}

macro_rules! impl_bytes_saved {
	($type:ty) => {
		impl BytesSaved for $type {
			/// Bytes Saved.
			fn bytes_saved(self, after: Self) -> Self {
				if 0 < after && after < self {
					self - after
				}
				else { 0 }
			}
		}
	};
}

impl_bytes_saved!(usize);
impl_bytes_saved!(u32);
impl_bytes_saved!(u64);
impl_bytes_saved!(u128);



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bytes_saved() {
		_bytes_saved(0, 0, 0);
		_bytes_saved(1000, 0, 0);
		_bytes_saved(1000, 1000, 0);
		_bytes_saved(1000, 500, 500);
	}

	fn _bytes_saved(before: u64, after: u64, expected: u64) {
		assert_eq!(
			before.bytes_saved(after),
			expected,
			"{} after {} should be a savings of {}",
			after,
			before,
			expected
		);
	}
}
