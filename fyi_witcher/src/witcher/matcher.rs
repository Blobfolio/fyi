/*!
# FYI Witcher: Matcher
*/

use crate::utility;
use std::{
	convert::TryFrom,
	path::PathBuf,
};



#[derive(Debug, Copy, Clone)]
/// # Matcher Error.
pub enum WitcherMatcherError {
	/// Invalid extension.
	InvalidExt,
}



#[derive(Debug, Clone, Eq, Hash, PartialEq)]
/// # Extension Matcher.
///
/// As the name implies, `WitcherMatcher` provides a means to compare a path's
/// file extension against one or more expected extensions. It is optimized for
/// extensions with typical lengths — a dot plus 2-4 letters — but can handle
/// longer strings as well.
///
/// The ideal use case for this is matching one path against 2-4 possible
/// extensions, using a setup like:
///
/// ```
/// use fyi_witcher::WitcherMatcher;
///
/// let ext1 = WitcherMatcher::try_from(b".jpg").unwrap();
/// let ext2 = WitcherMatcher::try_from(b".png").unwrap();
///
/// let result = WitcherMatcher::try_from(PathBuf::from("/some/file/path"))
///     .map_or(false, |p| p == ext1 || p == ext2);
/// ```
pub enum WitcherMatcher {
	/// 2-char Extension, e.g. `.gz`.
	Ext2(u16),
	/// 3-char Extension, e.g. `.htm`.
	Ext3(u32),
	/// 4-char Extension, e.g. `.html`.
	Ext4(u32),
	/// Some other size extension.
	Ext(Box<[u8]>),
}

impl TryFrom<&[u8; 3]> for WitcherMatcher {
	type Error = WitcherMatcherError;

	fn try_from(src: &[u8; 3]) -> Result<Self, Self::Error> {
		if src[0] == b'.' {
			Ok(Self::Ext2(unsafe {
				lower_ext2(*(src.as_ptr().add(1).cast::<u16>()))
			}))
		}
		else {
			Err(WitcherMatcherError::InvalidExt)
		}
	}
}

impl TryFrom<&[u8; 4]> for WitcherMatcher {
	type Error = WitcherMatcherError;

	fn try_from(src: &[u8; 4]) -> Result<Self, Self::Error> {
		if src[0] == b'.' {
			Ok(Self::Ext3(unsafe {
				lower_ext3(*(src.as_ptr().cast::<u32>()))
			}))
		}
		else {
			Err(WitcherMatcherError::InvalidExt)
		}
	}
}

impl TryFrom<&[u8; 5]> for WitcherMatcher {
	type Error = WitcherMatcherError;

	fn try_from(src: &[u8; 5]) -> Result<Self, Self::Error> {
		if src[0] == b'.' {
			Ok(Self::Ext4(unsafe {
				lower_ext4(*(src.as_ptr().add(1).cast::<u32>()))
			}))
		}
		else {
			Err(WitcherMatcherError::InvalidExt)
		}
	}
}

impl TryFrom<&[u8]> for WitcherMatcher {
	type Error = WitcherMatcherError;

	fn try_from(src: &[u8]) -> Result<Self, Self::Error> {
		if src.len() < 2 || src[0] != b'.' {
			return Err(WitcherMatcherError::InvalidExt);
		}

		Ok(
			match src.len() {
				3 => Self::Ext2(unsafe {
					lower_ext2(*(src.as_ptr().add(1).cast::<u16>()))
				}),
				4 => Self::Ext3(unsafe {
					lower_ext3(*(src.as_ptr().cast::<u32>()))
				}),
				5 => Self::Ext4(unsafe {
					lower_ext4(*(src.as_ptr().add(1).cast::<u32>()))
				}),
				_ => Self::Ext(src.to_ascii_lowercase().into())
			}
		)
	}
}

impl TryFrom<&PathBuf> for WitcherMatcher {
	type Error = WitcherMatcherError;

	fn try_from(src: &PathBuf) -> Result<Self, Self::Error> {
		let path = utility::path_as_bytes(src);

		path.iter()
			.rposition(|&x| x == b'.' || x == b'/')
			.and_then(|idx| Self::try_from(&path[idx..]).ok())
			.ok_or(WitcherMatcherError::InvalidExt)
	}
}

impl TryFrom<&str> for WitcherMatcher {
	type Error = WitcherMatcherError;

	fn try_from(src: &str) -> Result<Self, Self::Error> {
		Self::try_from(src.as_bytes())
	}
}

#[allow(clippy::len_without_is_empty)] // This is never empty.
impl WitcherMatcher {
	#[must_use]
	/// # Length.
	pub const fn len(&self) -> usize {
		match self {
			Self::Ext2(_) => 3,
			Self::Ext3(_) => 4,
			Self::Ext4(_) => 5,
			Self::Ext(x) => x.len(),
		}
	}

	#[must_use]
	/// # Is Match?
	pub fn is_match(&self, path: &PathBuf) -> bool {
		let path = utility::path_as_bytes(path);
		let p_len = path.len();
		let e_len = self.len();

		p_len > e_len &&
		Self::try_from(&path[p_len - e_len..])
			.map_or(false, |x| x.eq(self))
	}
}



/// # Lowercase Mask.
///
/// An uppercase ASCII byte can be made lowercase by BIT-ORing its value
/// against this, like `b'J' | (1 << 5) == b'j'`.
///
/// This has no effect against digits or `-` or `a-z`, so can be used here
/// without the usual range checking.
const LOWER: u8 = 1 << 5;

#[allow(clippy::missing_const_for_fn)] // Dereference isn't allowed in const.
/// # Lowercase Ext2.
///
/// This is a cheap and good-enough trick to lowercase the sort of string
/// expected for a file extension.
///
/// It might corrupt UTF-8 or non-alpha ASCII, but as we're only comparing
/// integers, string-sanity doesn't matter.
unsafe fn lower_ext2(src: u16) -> u16 {
	src | *([LOWER, LOWER].as_ptr().cast::<u16>())
}

#[allow(clippy::missing_const_for_fn)] // Dereference isn't allowed in const.
/// # Lowercase Ext3.
///
/// See notes for [`lower_ext2`] in regards to safety and limitations.
unsafe fn lower_ext3(src: u32) -> u32 {
	src | *([0, LOWER, LOWER, LOWER].as_ptr().cast::<u32>())
}

#[allow(clippy::missing_const_for_fn)] // Dereference isn't allowed in const.
/// # Lowercase Ext4.
///
/// See notes for [`lower_ext2`] in regards to safety and limitations.
unsafe fn lower_ext4(src: u32) -> u32 {
	src | *([LOWER, LOWER, LOWER, LOWER].as_ptr().cast::<u32>())
}



#[cfg(test)]
mod tests {
	use super::*;
	use criterion as _;

	#[test]
	fn t_matcher() {
		let gz = WitcherMatcher::try_from(b".gz").unwrap();
		let pl_gz = WitcherMatcher::try_from(&PathBuf::from("/test/file.gz")).unwrap();
		let pu_gz = WitcherMatcher::try_from(&PathBuf::from("/test/file.GZ")).unwrap();

		let jpg = WitcherMatcher::try_from(".jpg").unwrap();
		let pl_jpg = WitcherMatcher::try_from(&PathBuf::from("/test/file.jpg")).unwrap();
		let pu_jpg = WitcherMatcher::try_from(&PathBuf::from("/test/file.JPG")).unwrap();

		let jpeg = WitcherMatcher::try_from(".jpeg").unwrap();
		let pl_jpeg = WitcherMatcher::try_from(&PathBuf::from("/test/file.jpeg")).unwrap();
		let pu_jpeg = WitcherMatcher::try_from(&PathBuf::from("/test/file.JPeG")).unwrap();

		assert_eq!(gz, pl_gz);
		assert!(gz.is_match(&PathBuf::from("/test/file.GZ")));
		assert_eq!(gz, pu_gz);
		assert!(gz != pl_jpeg);

		assert_eq!(jpg, pl_jpg);
		assert_eq!(jpg, pu_jpg);
		assert!(jpg != pl_jpeg);

		assert_eq!(jpeg, pl_jpeg);
		assert_eq!(jpeg, pu_jpeg);
		assert!(jpeg != pl_jpg);
	}
}
