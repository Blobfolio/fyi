use std::{
	borrow::{Borrow, Cow},
	fmt,
};



#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// Glue Types.
pub enum OxfordGlue {
	/// For.
	For,
	/// And.
	And,
	/// Nor.
	Nor,
	/// But.
	But,
	/// Or.
	Or,
	/// Yet.
	Yet,
	/// So.
	So,
}

impl AsRef<str> for OxfordGlue {
	#[inline]
	/// As ref.
	fn as_ref(&self) -> &str {
		match *self {
			OxfordGlue::For => " for ",
			OxfordGlue::And => " and ",
			OxfordGlue::Nor => " nor ",
			OxfordGlue::But => " but ",
			OxfordGlue::Or => " or ",
			OxfordGlue::Yet => " yet ",
			OxfordGlue::So => " so ",
		}
	}
}

impl AsRef<[u8]> for OxfordGlue {
	#[inline]
	/// As ref.
	fn as_ref(&self) -> &[u8] {
		match *self {
			OxfordGlue::For => b" for ",
			OxfordGlue::And => b" and ",
			OxfordGlue::Nor => b" nor ",
			OxfordGlue::But => b" but ",
			OxfordGlue::Or => b" or ",
			OxfordGlue::Yet => b" yet ",
			OxfordGlue::So => b" so ",
		}
	}
}

impl Borrow<str> for OxfordGlue {
	#[inline]
	/// As ref.
	fn borrow(&self) -> &str {
		match *self {
			OxfordGlue::For => " for ",
			OxfordGlue::And => " and ",
			OxfordGlue::Nor => " nor ",
			OxfordGlue::But => " but ",
			OxfordGlue::Or => " or ",
			OxfordGlue::Yet => " yet ",
			OxfordGlue::So => " so ",
		}
	}
}

impl Default for OxfordGlue {
	#[inline]
	/// Display.
	fn default() -> Self {
		OxfordGlue::And
	}
}

impl OxfordGlue {
	#[must_use]
	/// Conjunction.
	///
	/// This is the same as `as_ref()`, except the leading space is
	/// stripped, and the result is returned as bytes since that's
	/// what is needed for gluing anyway.
	pub fn conjunction(&self) -> &[u8] {
		match *self {
			OxfordGlue::For => b"for ",
			OxfordGlue::And => b"and ",
			OxfordGlue::Nor => b"nor ",
			OxfordGlue::But => b"but ",
			OxfordGlue::Or => b"or ",
			OxfordGlue::Yet => b"yet ",
			OxfordGlue::So => b"so ",
		}
	}
}

impl fmt::Display for OxfordGlue {
	#[inline]
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}



/// Oxford Join
///
/// This is like join, but with commas and conjunctions in the right
/// spots.
pub trait OxfordJoin {
	/// Join.
	fn oxford_join<'oj> (&'oj self, glue: &OxfordGlue) -> Cow<'oj, str>;
}

impl OxfordJoin for [&str] {
	fn oxford_join<'oj> (&'oj self, glue: &OxfordGlue) -> Cow<'oj, str> {
		match self.len() {
			0 => "".into(),
			1 => self[0].into(),
			2 => self.join(glue.as_ref()).into(),
			x => {
				// Join with a comma.
				let mut tmp: String = self.join(", ");

				// Add the conjunction.
				let len: usize = tmp.len();
				let idx: usize = len - self[x - 1].len();
				let glue = glue.conjunction();
				let glue_len: usize = glue.len();

				// Mirroring the private stuff ::insert_str() does
				// seems to yield the best performance, even if it is
				// a touch ugly.
				unsafe {
					let vec = tmp.as_mut_vec();
					vec.reserve(glue_len);
					std::ptr::copy(
						vec.as_ptr().add(idx),
						vec.as_mut_ptr().add(idx + glue_len),
						len - idx
					);
					std::ptr::copy(
						glue.as_ptr(),
						vec.as_mut_ptr().add(idx),
						glue_len
					);
					vec.set_len(len + glue_len);
				}

				tmp
			}.into()
		}
	}
}

impl OxfordJoin for [String] {
	fn oxford_join<'oj> (&'oj self, glue: &OxfordGlue) -> Cow<'oj, str> {
		match self.len() {
			0 => "".into(),
			1 => self[0].as_str().into(),
			2 => self.join(glue.as_ref()).into(),
			x => {
				// Join with a comma.
				let mut tmp: String = self.join(", ");

				// Add the conjunction.
				let len: usize = tmp.len();
				let idx: usize = len - self[x - 1].len();
				let glue = glue.conjunction();
				let glue_len: usize = glue.len();

				// Mirroring the private stuff ::insert_str() does
				// seems to yield the best performance, even if it is
				// a touch ugly.
				unsafe {
					let vec = tmp.as_mut_vec();
					vec.reserve(glue_len);
					std::ptr::copy(
						vec.as_ptr().add(idx),
						vec.as_mut_ptr().add(idx + glue_len),
						len - idx
					);
					std::ptr::copy(
						glue.as_ptr(),
						vec.as_mut_ptr().add(idx),
						glue_len
					);
					vec.set_len(len + glue_len);
				}

				tmp
			}.into()
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn oxford_join() {
		let data: [String; 5] = [
			"apples".to_string(),
			"bananas".to_string(),
			"carrots".to_string(),
			"dates".to_string(),
			"eggplants".to_string(),
		];
		let expected_and: [&str; 6] = [
			"",
			"apples",
			"apples and bananas",
			"apples, bananas, and carrots",
			"apples, bananas, carrots, and dates",
			"apples, bananas, carrots, dates, and eggplants",
		];
		let expected_or: [&str; 6] = [
			"",
			"apples",
			"apples or bananas",
			"apples, bananas, or carrots",
			"apples, bananas, carrots, or dates",
			"apples, bananas, carrots, dates, or eggplants",
		];

		for i in (0..6).into_iter() {
			assert_eq!(
				data[0..i].oxford_join(&OxfordGlue::And),
				expected_and[i]
			);
			assert_eq!(
				data[0..i].oxford_join(&OxfordGlue::Or),
				expected_or[i]
			);
		}
	}
}
