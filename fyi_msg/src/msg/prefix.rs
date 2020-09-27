/*!
# FYI Msg: Prefix
*/

use crate::utility;
use std::{
	cmp::Ordering,
	fmt,
	hash::{
		Hash,
		Hasher,
	},
	mem,
	ops::Deref,
	ptr,
};



#[derive(Clone, Copy)]
/// # Prefix Buffer.
///
/// This is a simple fixed-array buffer to store custom prefixes for
/// [`MsgKind::Other`](`super::MsgKind::Other`). This is implemented as a custom struct in order to take
/// advantage of `Copy`, thus allowing the [`MsgKind`](`super::MsgKind`) enum to also implement
/// `Copy`.
///
/// ## Restrictions
///
/// Because the buffer is fixed at a length of `64` — including the label and
/// any ANSI formatting — this leaves 45 bytes for the label itself. Prefixes
/// exceeding this limit are silently ignored.
///
/// ## Safety
///
/// This struct is not intended to be interacted with directly. Nearly all of
/// its methods are unsafe and require sane data to function correctly.
///
/// While the buffer's length is fixed, only the "occupied" regions will
/// contain trusted, predictable data. The overflow is transmuted into `u8` from
/// [`std::mem::MaybeUninit`], so may be zeroes or may be random weirdness.
pub struct MsgPrefix {
	buf: [u8; 64],
	len: usize,
}

impl fmt::Debug for MsgPrefix {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("MsgPrefix")
			.field("buf", &self)
			.finish()
	}
}

impl Default for MsgPrefix {
	#[inline]
	fn default() -> Self {
		Self {
			buf: [0; 64],
			len: 0,
		}
	}
}

impl Deref for MsgPrefix {
	type Target = [u8];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.buf[0..self.len] }
}

impl Eq for MsgPrefix {}

impl Hash for MsgPrefix {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_bytes().hash(state);
	}
}

impl Ord for MsgPrefix {
	#[inline]
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_bytes().cmp(other.as_bytes())
	}
}

impl PartialEq for MsgPrefix {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}

impl PartialOrd for MsgPrefix {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.as_bytes().cmp(other.as_bytes()))
	}
}

/// ## Casting.
///
/// These methods provide means of converting a `MsgPrefix` into other data
/// structures.
///
/// Note: this struct can also be dereferenced to `&[u8]`.
impl MsgPrefix {
	#[must_use]
	#[inline]
	/// # As Bytes.
	///
	/// Return the prefix as a slice.
	pub fn as_bytes(&self) -> &[u8] { self }

	#[must_use]
	#[inline]
	/// # As Pointer.
	///
	/// Return a raw pointer to the slice.
	pub const fn as_ptr(&self) -> *const u8 { self.buf.as_ptr() }

	#[must_use]
	#[inline]
	/// # As Str.
	///
	/// Return the prefix as a string slice.
	///
	/// ## Safety
	///
	/// The string's UTF-8 is not validated for sanity!
	pub unsafe fn as_str(&self) -> &str { std::str::from_utf8_unchecked(self) }
}

/// ## The rest!
///
/// There isn't really a lot going on here. Haha.
impl MsgPrefix {
	#[must_use]
	/// # New Instance (Unchecked).
	///
	/// Create a new instance using the given prefix and color.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use fyi_msg::MsgPrefix;
	///
	/// let prefix = unsafe { MsgPrefix::new_unchecked(b"Hello", 199) };
	/// ```
	///
	/// ## Safety
	///
	/// The prefix must be valid UTF-8 and cannot exceed 45 bytes in length.
	pub unsafe fn new_unchecked(prefix: &[u8], color: u8) -> Self {
		let mut buf = [mem::MaybeUninit::<u8>::uninit(); 64];

		let len: usize = {
			let mut dst = buf.as_mut_ptr() as *mut u8;

			// Write the color.
			if color == 0 {
				ptr::copy_nonoverlapping(b"\x1b[0m".as_ptr(), dst, 4);
				dst = dst.add(4);
			}
			else {
				ptr::copy_nonoverlapping(b"\x1b[1;38;5;".as_ptr(), dst, 9);
				let tmp: usize = utility::write_u8(dst.add(9), color) + 9;
				ptr::write(dst.add(tmp), b'm');
				dst = dst.add(tmp + 1);
			}

			// Write the prefix.
			ptr::copy_nonoverlapping(prefix.as_ptr(), dst, prefix.len());
			dst = dst.add(prefix.len());

			// Write the closer.
			ptr::copy_nonoverlapping(b":\x1b[0m ".as_ptr(), dst, 6);

			dst.add(6) as usize - buf.as_ptr() as *const u8 as usize
		};

		// Align and return!
		Self {
			buf: mem::transmute::<_, [u8; 64]>(buf),
			len
		}
	}

	#[must_use]
	#[inline]
	/// # Is Empty.
	///
	/// Returns `true` if the prefix is empty.
	pub const fn is_empty(&self) -> bool { 0 == self.len }

	#[must_use]
	#[inline]
	/// # Length.
	///
	/// Return the length of the prefix.
	pub const fn len(&self) -> usize { self.len }
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_prefix() {
		unsafe {
			let prefix = MsgPrefix::new_unchecked(b"Hello World", 199);
			assert_eq!(prefix.deref(), b"\x1b[1;38;5;199mHello World:\x1b[0m ");
			assert_eq!(prefix.len(), 30);
			assert!(! prefix.is_empty());
		}
	}
}
