/*!
# FYI Num: "Nice" ANSI
*/

use std::{
	fmt,
	ops::Deref,
	ptr,
};



#[derive(Debug, Copy, Clone)]
/// # Nice ANSI.
///
/// This is a simple struct for generating bold, colored ANSI opening tags.
/// This is slightly faster than using `format!()` or the like.
///
/// ## Examples
///
/// ```no_run
/// use fyi_num::NiceANSI;
/// assert_eq!(
///     NiceANSI::from(199).as_str(),
///     "\x1b[1;38;5;199m"
/// );
/// ```
pub struct NiceANSI {
	inner: [u8; 13],
	len: usize,
}

impl Default for NiceANSI {
	#[inline]
	/// # Default.
	///
	/// This is a reset.
	fn default() -> Self {
		Self {
			inner: *b"\x1b[0m000000000",
			len: 4,
		}
	}
}

impl Deref for NiceANSI {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.inner[0..self.len] }
}

impl fmt::Display for NiceANSI {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl From<u8> for NiceANSI {
	fn from(src: u8) -> Self {
		if src > 99 {
			let mut out = Self {
				inner: *b"\x1b[1;38;5;000m",
				len: 13,
			};

			unsafe {
				ptr::copy_nonoverlapping(
					crate::TRIPLE.as_ptr().add(usize::from(src) * 3),
					out.inner.as_mut_ptr().add(9),
					3
				);
			}

			out
		}
		else if src > 9 {
			let mut out = Self {
				inner: *b"\x1b[1;38;5;00m0",
				len: 12,
			};

			unsafe {
				ptr::copy_nonoverlapping(
					crate::DOUBLE.as_ptr().add((usize::from(src)) << 1),
					out.inner.as_mut_ptr().add(9),
					2
				);
			}

			out
		}
		else if src > 0 {
			let mut out = Self {
				inner: *b"\x1b[1;38;5;0m00",
				len: 11,
			};

			unsafe {
				ptr::copy_nonoverlapping(
					crate::SINGLE.as_ptr().add(usize::from(src)),
					out.inner.as_mut_ptr().add(9),
					1
				);
			}

			out
		}
		else { Self::default() }
	}
}

#[allow(clippy::len_without_is_empty)] // It can't be empty.
impl NiceANSI {
	#[must_use]
	/// # Length.
	pub const fn len(&self) -> usize { self.len }

	#[must_use]
	#[inline]
	/// # As Bytes.
	pub fn as_bytes(&self) -> &[u8] { self }

	#[must_use]
	#[inline]
	/// # As Str.
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
	}
}
