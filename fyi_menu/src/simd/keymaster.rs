/*!
# FYI Menu: Key Master (SIMD)

The functionality is identical to that of the non-SIMD-optimized version; it is
just faster for x86-64 machines supporting SSE/AVX/etc.

**Note:** This is not intended for external use and is subject to change.
*/

use crate::{
	die,
	utility::hash_arg_key,
};
use packed_simd::{
	u16x8,
	u64x8,
};



/// The maximum number of keys allowed.
///
/// We're using a fixed-length array for storage, so we have to be considerate
/// of the space being allocated.
const MAX_KEYS: usize = 8;



#[derive(Debug, Copy, Clone, Default)]
/// `KeyMaster` is a simple, pseudo-hashmap for storing [`Argue`](crate::Argue) keys and
/// indexes. Because the maximum number of keys is highly constrained — up to
/// **8** — and the behaviors have a very narrow scope, this saves some
/// overhead versus using an actual [`std::collections::HashMap`].
pub struct KeyMaster {
	keys: u64x8,
	values: u16x8,
	len: usize,
}

impl KeyMaster {
	#[must_use]
	/// # Is Empty?
	///
	/// This returns `true` if no keys are present.
	pub const fn is_empty(&self) -> bool { self.len == 0 }

	#[must_use]
	/// # Length.
	///
	/// Return the total number of keys.
	pub const fn len(&self) -> usize { self.len }

	/// # Insert.
	///
	/// If the key is not already stored, it will be added and `true` will be
	/// returned, otherwise no action will be taken and `false` will be
	/// returned.
	///
	/// If the maximum number of keys has already been reached, an error
	/// message will be printed and the program will exit with a status code of
	/// `1`.
	pub fn insert(&mut self, key: &str, idx: usize) -> bool {
		if self.len >= MAX_KEYS {
			die(b"Too many options.");
			unreachable!();
		}

		let key = hash_arg_key(key);
		if self.keys.eq(u64x8::splat(key)).none() {
			unsafe {
				self.keys = self.keys.replace_unchecked(self.len, key);
				self.values = self.values.replace_unchecked(self.len, idx as u16);
			}
			self.len += 1;
			true
		}
		else { false }
	}

	/// # Insert (Unique).
	///
	/// This is just like '[`KeyMaster::insert`] except that if a duplicate key
	/// is submitted, it will print an error and exit with status code 1.
	pub fn insert_unique(&mut self, key: &str, idx: usize) {
		if self.len >= MAX_KEYS {
			die(b"Too many options.");
			unreachable!();
		}

		let key = hash_arg_key(key);
		if self.keys.eq(u64x8::splat(key)).any() {
			die(b"Duplicate key.");
			unreachable!();
		}

		unsafe {
			self.keys = self.keys.replace_unchecked(self.len, key);
			self.values = self.values.replace_unchecked(self.len, idx as u16);
		}
		self.len += 1;
	}

	#[must_use]
	/// # Has Key?
	///
	/// Returns `true` if the key is stored, or `false` if not.
	pub fn contains(&self, key: &str) -> bool {
		self.keys.eq(u64x8::splat(hash_arg_key(key))).any()
	}

	#[must_use]
	/// # Has Either of Two Keys?
	///
	/// This is a convenience method that checks for the existence of two keys
	/// at once, returning `true` if either are present.
	pub fn contains2(&self, short: &str, long: &str) -> bool {
		self.contains(short) || self.contains(long)
	}

	#[must_use]
	/// # Get Key's Index.
	///
	/// If a key is present, return its corresponding index; if not, `None`.
	pub fn get(&self, key: &str) -> Option<usize> {
		let res = self.keys.eq(u64x8::splat(hash_arg_key(key)));
		if res.any() {
			Some(unsafe {
				self.values.extract_unchecked(res.bitmask().trailing_zeros() as usize)
			} as usize)
		}
		else { None }
	}

	#[must_use]
	/// # Get Either of Two Key's Index.
	///
	/// This is a convenience method that checks for the existence of two keys
	/// at once, returning the first index found, if any.
	pub fn get2(&self, short: &str, long: &str) -> Option<usize> {
		self.get(short).or_else(|| self.get(long))
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_insert() {
		let mut km = KeyMaster::default();
		assert!(km.is_empty());
		assert_eq!(km.len(), 0);

		assert!(km.insert("Hello", 5));
		assert!(km.contains("Hello"));
		assert!(! km.contains("World"));
		assert_eq!(km.get("Hello"), Some(5));

		assert!(! km.insert("Hello", 5));

		assert!(km.insert("World", 10));
		assert_eq!(km.len(), 2);

		assert!(km.insert("One", 1));
		assert!(km.insert("Two", 2));
		assert!(km.insert("Three", 3));

		assert!(km.contains2("two", "Two"));
		assert!(! km.contains2("two", "three"));
		assert_eq!(km.get2("two", "Two"), Some(2));
		assert_eq!(km.get2("two", "three"), None);
	}
}
