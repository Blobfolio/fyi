/*!
# FYI Menu: Key Master

**Note:** This is not intended for external use and is subject to change.
*/

use crate::{
	die,
	utility::hash_arg_key,
};
use std::{
	hash::{
		Hash,
		Hasher,
	},
	ops::Deref,
};



/// The maximum number of keys allowed.
///
/// We're using a fixed-length array for storage, so we have to be considerate
/// of the space being allocated.
const MAX_KEYS: usize = 8;



#[derive(Debug, Default, Copy, Clone)]
/// A `KeyEntry` is a key/idx pairing, where the key is a `u64` hash of the
/// original string value, and the index is the `usize` corresponding to the
/// key's position in the full [`Argue`](crate::Argue) set.
struct KeyEntry {
	pub(crate) hash: u64,
	pub(crate) idx: usize,
}

impl Deref for KeyEntry {
	type Target = u64;
	fn deref(&self) -> &Self::Target { &self.hash }
}

impl Hash for KeyEntry {
	fn hash<H: Hasher>(&self, state: &mut H) { self.hash.hash(state); }
}

impl Eq for KeyEntry {}

impl PartialEq<u64> for KeyEntry {
	fn eq(&self, other: &u64) -> bool { self.hash == *other }
}

impl PartialEq for KeyEntry {
	fn eq(&self, other: &Self) -> bool { self.hash == other.hash }
}

impl KeyEntry {
	/// # New Instance.
	///
	/// Generate a new `KeyEntry` for a given string at index `idx`.
	pub(crate) fn new(key: &str, idx: usize) -> Self {
		Self {
			hash: hash_arg_key(key),
			idx
		}
	}
}



#[derive(Debug, Copy, Clone, Default)]
/// `KeyMaster` is a simple, pseudo-hashmap for storing [`Argue`](crate::Argue) keys and
/// indexes. Because the maximum number of keys is highly constrained — up to
/// **8** — and the behaviors have a very narrow scope, this saves some
/// overhead versus using an actual [`std::collections::HashMap`].
pub struct KeyMaster {
	keys: [KeyEntry; MAX_KEYS],
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
		}

		let key = KeyEntry::new(key, idx);
		if self.keys[0..self.len].iter().all(|x| x.ne(&key)) {
			self.keys[self.len] = key;
			self.len += 1;
			true
		}
		else { false }
	}

	#[must_use]
	/// # Has Key?
	///
	/// Returns `true` if the key is stored, or `false` if not.
	pub fn contains(&self, key: &str) -> bool {
		let key = hash_arg_key(key);
		self.keys[0..self.len].iter().any(|x| x.hash == key)
	}

	#[must_use]
	/// # Has Either of Two Keys?
	///
	/// This is a convenience method that checks for the existence of two keys
	/// at once, returning `true` if either are present.
	pub fn contains2(&self, short: &str, long: &str) -> bool {
		let short = hash_arg_key(short);
		let long = hash_arg_key(long);
		self.keys[0..self.len].iter().any(|x| x.hash == short || x.hash == long)
	}

	#[must_use]
	/// # Get Key's Index.
	///
	/// If a key is present, return its corresponding index; if not, `None`.
	pub fn get(&self, key: &str) -> Option<usize> {
		let key = hash_arg_key(key);
		self.keys[0..self.len].iter()
			.find_map(|x|
				if x.hash == key { Some(x.idx) }
				else { None }
			)
	}

	#[must_use]
	/// # Get Either of Two Key's Index.
	///
	/// This is a convenience method that checks for the existence of two keys
	/// at once, returning the first index found, if any.
	pub fn get2(&self, short: &str, long: &str) -> Option<usize> {
		let short = hash_arg_key(short);
		let long = hash_arg_key(long);
		self.keys[0..self.len].iter()
			.find_map(|x|
				if x.hash == short || x.hash == long { Some(x.idx) }
				else { None }
			)
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
