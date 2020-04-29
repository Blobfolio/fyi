use num_traits::cast::AsPrimitive;
use std::borrow::Cow;



/// Bytes Saved.
pub trait MebiSaved {
	/// Byte diff.
	fn saved<N> (&self, after: N) -> u64
	where N: AsPrimitive<u64>;
}

/// Convert a number in bytes to a string with the best-fit unit.
pub trait ToMebi {
	/// Human bytes.
	fn to_mebi(&self) -> Cow<str>;
}

impl<T> MebiSaved for T
where T: AsPrimitive<u64> {
	/// Byte diff.
	fn saved<N> (&self, after: N) -> u64
	where N: AsPrimitive<u64> {
		let before: u64 = self.as_();
		let after: u64 = after.as_();
		if 0 < after && after < before {
			before - after
		}
		else {
			0
		}
	}
}

impl<T> ToMebi for T
where T: AsPrimitive<f64> {
	/// Human bytes.
	fn to_mebi(&self) -> Cow<str> {
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
				out.into()
			},
			1 => format!("{:.*}KiB", 2, bytes).into(),
			2 => format!("{:.*}MiB", 2, bytes).into(),
			3 => format!("{:.*}GiB", 2, bytes).into(),
			_ => format!("{:.*}TiB", 2, bytes).into(),
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn to_mebi() {
		assert_eq!(999u64.to_mebi(), Cow::Borrowed("999B"));
		assert_eq!(1000u64.to_mebi(), Cow::Borrowed("0.98KiB"));
		assert_eq!(12003u64.to_mebi(), Cow::Borrowed("11.72KiB"));
		assert_eq!(4887391u64.to_mebi(), Cow::Borrowed("4.66MiB"));
		assert_eq!(499288372u64.to_mebi(), Cow::Borrowed("476.16MiB"));
		assert_eq!(99389382145u64.to_mebi(), Cow::Borrowed("92.56GiB"));
	}

	#[test]
	fn saved() {
		// Negatives.
		assert_eq!(0u64.saved(500u64), 0u64);
		assert_eq!(500u64.saved(500u64), 0u64);
		assert_eq!(500u64.saved(0u64), 0u64);

		// Positives.
		assert_eq!(1000u64.saved(500u64), 500u64);
		assert_eq!(10000u64.saved(500u64), 9500u64);
	}
}
