/*!
# FYI Menu: Key Master

**Note:** This is not intended for external use and is subject to change.
*/

use crate::die;
use fyi_msg::utility::hash64;



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
	keys: [u64; MAX_KEYS],
	values: [usize; MAX_KEYS],
	len: usize,
}

/// ## Misc Methods.
///
/// Little odds and ends.
impl KeyMaster {
	#[must_use]
	#[inline]
	/// # Is Empty?
	///
	/// This returns `true` if no keys are present.
	pub const fn is_empty(&self) -> bool { self.len == 0 }

	#[must_use]
	#[inline]
	/// # Length.
	///
	/// Return the total number of keys.
	pub const fn len(&self) -> usize { self.len }
}



/// ## Manipulation.
///
/// These methods add and retrieve from the store.
impl KeyMaster {
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

		let key = hash64(key.as_bytes());
		if self.keys[0..self.len].iter().all(|x| *x != key) {
			self.keys[self.len] = key;
			self.values[self.len] = idx;
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

		let key = hash64(key.as_bytes());
		if self.keys[0..self.len].iter().any(|x| *x == key) {
			die(b"Duplicate key.");
			unreachable!();
		}

		self.keys[self.len] = key;
		self.values[self.len] = idx;
		self.len += 1;
	}

	#[must_use]
	/// # Has Key?
	///
	/// Returns `true` if the key is stored, or `false` if not.
	pub fn contains(&self, key: &str) -> bool {
		let key = hash64(key.as_bytes());
		self.keys[0..self.len].iter().any(|x| *x == key)
	}

	#[must_use]
	/// # Has Either of Two Keys?
	///
	/// This is a convenience method that checks for the existence of two keys
	/// at once, returning `true` if either are present.
	pub fn contains2(&self, short: &str, long: &str) -> bool {
		let short = hash64(short.as_bytes());
		let long = hash64(long.as_bytes());
		self.keys[0..self.len].iter().any(|x| *x == short || *x == long)
	}

	#[must_use]
	/// # Get Key's Index.
	///
	/// If a key is present, return its corresponding index; if not, `None`.
	pub fn get(&self, key: &str) -> Option<usize> {
		let key = hash64(key.as_bytes());
		self.keys[0..self.len].iter()
			.position(|x| *x == key)
			.map(|x| self.values[x])
	}

	#[must_use]
	/// # Get Either of Two Key's Index.
	///
	/// This is a convenience method that checks for the existence of two keys
	/// at once, returning the first index found, if any.
	pub fn get2(&self, short: &str, long: &str) -> Option<usize> {
		let short = hash64(short.as_bytes());
		let long = hash64(long.as_bytes());
		self.keys[0..self.len].iter()
			.position(|x| *x == short || *x == long)
			.map(|x| self.values[x])
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
