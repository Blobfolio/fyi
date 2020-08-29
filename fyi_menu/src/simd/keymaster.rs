/*!
# FYI: `KeyMaster` (SIMD)

This is a simple pseudo-hashmap for storing `Argue` keys and indexes. Because
we're only ever going to store a limited number of keys, a full-blown variable
`HashMap` would be overkill. This is able to simply store the keys — as `u64`
`AHash` hashes — in one fixed-length array, and the corresponding indexes in
another fixed-length array.

The maximum length is currently `8`. That might change, but will likely remain
a power of two.
*/

use crate::{
	die,
	hash_arg_key,
};
use packed_simd::u64x8;



/// The maximum number of keys allowed.
///
/// We're using a fixed-length array for storage, so we have to be considerate
/// of the space being allocated.
const MAX_KEYS: usize = 8;



#[derive(Debug, Default, Copy, Clone)]
/// Hash Map.
///
/// See the module level documentation for more details.
pub struct KeyMaster {
	keys: u64x8,
	values: u64x8,
	len: usize,
}

impl KeyMaster {
	#[must_use]
	/// Is Empty.
	pub const fn is_empty(&self) -> bool { self.len == 0 }

	#[must_use]
	/// Length.
	pub const fn len(&self) -> usize { self.len }

	/// Insert.
	///
	/// If the key is not already stored, it will be added, otherwise `false`
	/// is returned and `Argue` will error out.
	pub fn insert(&mut self, key: &str, idx: usize) -> bool {
		if self.len >= MAX_KEYS {
			die(b"Too many options.");
		}

		let key = hash_arg_key(key);
		if self.keys.eq(u64x8::splat(key)).none() {
			self.keys = self.keys.replace(self.len, key);
			self.values = self.values.replace(self.len, idx as u64);
			self.len += 1;
			true
		}
		else { false }
	}

	#[must_use]
	/// Has Key?
	///
	/// Returns `true` if the key is stored, or `false` if not.
	pub fn contains(&self, key: &str) -> bool {
		self.keys.eq(u64x8::splat(hash_arg_key(key))).any()
	}

	#[must_use]
	/// Has Key (short/long)?
	///
	/// This is the same as `contains()`, except both a short and long key are
	/// checked in a single iteration, `true` being returned if either are
	/// present.
	pub fn contains2(&self, short: &str, long: &str) -> bool {
		self.contains(short) || self.contains(long)
	}

	#[must_use]
	/// Get Key's IDX.
	///
	/// If a key is present, return its corresponding index; if not, `None`.
	pub fn get(&self, key: &str) -> Option<usize> {
		let res = self.keys.eq(u64x8::splat(hash_arg_key(key)));
		if res.any() {
			Some(res.select(self.values, u64x8::splat(0)).max_element() as usize)
		}
		else { None }
	}

	#[must_use]
	/// Get Key's IDX (short/long).
	///
	/// Same as `get()`, except both a short and long key are checked. The
	/// first matching index, if any, is returned.
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
