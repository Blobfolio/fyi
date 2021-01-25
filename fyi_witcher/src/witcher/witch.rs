/*!
# FYI Witcher: Witch
*/

use std::{
	convert::TryFrom,
	fs::DirEntry,
	hash::{
		Hash,
		Hasher,
	},
	os::unix::fs::MetadataExt,
	path::PathBuf,
};



#[derive(Debug, Copy, Clone, Default)]
/// # Witch
///
/// This is essentially a specialized take on "same-file", allowing [`Witcher`]
/// to keep track of unique paths, while also separating file and directory
/// handling.
///
/// This struct must be instantiated from [`std::convert::TryFrom`], so any
/// constructed instance will either be a file or a directory.
pub(super) struct Witch {
	dir: bool,
	hash: u64,
}

impl TryFrom<&PathBuf> for Witch {
	type Error = bool;
	fn try_from(src: &PathBuf) -> Result<Self, Self::Error> {
		std::fs::metadata(src)
			.map(|meta| Self {
				dir: meta.is_dir(),
				hash: witch_hash(meta.dev(), meta.ino()),
			})
			.map_err(|_| false)
	}
}

impl Eq for Witch {}

impl Hash for Witch {
	fn hash<H: Hasher>(&self, state: &mut H) { state.write_u64(self.hash); }
}

impl PartialEq for Witch {
	fn eq(&self, other: &Self) -> bool { self.hash == other.hash }
}

impl Witch {
	/// # Is Directory?
	pub(super) const fn is_dir(&self) -> bool { self.dir }

	#[inline]
	/// # From `DirEntry`.
	///
	/// This returns a tuple containing the witch and the path if it was
	/// importable.
	///
	/// This is merely a convenience method used to cut down on the labor of
	/// traversing [`std::fs::ReadDir`].
	pub(super) fn from_dent(dent: Result<DirEntry, std::io::Error>) -> Option<(Self, PathBuf)> {
		dent.ok().and_then(|dent| {
			let path = dent.path();
			Self::try_from(&path).ok().zip(Some(path))
		})
	}
}



/// # Witch "Hash".
///
/// This is just a quick way to (mostly) uniquely combine two u64s into a
/// single u64, halving the storage requirements and number of future hash/cmp
/// operations.
///
/// Taken from [szudzik](http://szudzik.com/ElegantPairing.pdf), but made
/// wrapping in case the numbers get really big.
const fn witch_hash(dev: u64, ino: u64) -> u64 {
	if dev < ino {
		ino.wrapping_mul(ino).wrapping_add(dev)
	}
	else {
		dev.wrapping_mul(dev)
			.wrapping_add(dev)
			.wrapping_add(ino)
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use rand::Rng;

	#[test]
	fn t_witch_hash() {
		use std::collections::HashSet;
		let mut rng = rand::thread_rng();

		// Populate a bunch of values and see what happens.
		let set: HashSet<(u64, u64)> = (0..100_000)
			.into_iter()
			.map(|_| (rng.gen(), rng.gen()))
			.collect();
		let out: Vec<u64> = set.iter().map(|(x, y)| witch_hash(*x, *y)).collect();

		assert_eq!(set.len(), out.len());
	}
}
