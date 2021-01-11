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
/// This is a simple way to compare extensions without having to iterate over
/// and over again.
///
/// Implementing libraries are expected to pre-lowercase extensions, and use
/// either the string slice or byte slice [`WitcherMatcher::try_from`].
///
/// If there is only one desired extension, use that matcher's [`WitcherMatcher::is_match`]
/// method to see if a given path matches.
///
/// If there are multiple possible extensions, it is more efficient to use
/// [`WitcherMatcher::try_from`] against the `&PathBuf`, and then use basic
/// equality operators, like:
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
				*(src.as_ptr().add(1).cast::<u16>())
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
				*(src.as_ptr().cast::<u32>())
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
				*(src.as_ptr().add(1).cast::<u32>())
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
					*(src.as_ptr().add(1).cast::<u16>())
				}),
				4 => Self::Ext3(unsafe {
					*(src.as_ptr().cast::<u32>())
				}),
				5 => Self::Ext4(unsafe {
					*(src.as_ptr().add(1).cast::<u32>())
				}),
				_ => Self::Ext(src.into())
			}
		)
	}
}

impl TryFrom<&PathBuf> for WitcherMatcher {
	type Error = WitcherMatcherError;

	fn try_from(src: &PathBuf) -> Result<Self, Self::Error> {
		let path = utility::path_as_bytes(src);
		let len = path.len();

		if len > 3 {
			let stop = len - 8.min(len);

			// Find the dot.
			let mut idx = len - 2;
			while idx >= stop {
				if path[idx] == b'.' {
					return Self::try_from(path[idx..].to_ascii_lowercase().as_slice());
				}

				idx -= 1;
			}
		}

		Err(WitcherMatcherError::InvalidExt)
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
		Self::try_from(path[p_len - e_len..].to_ascii_lowercase().as_slice())
			.map_or(false, |x| x.eq(self))
	}
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
