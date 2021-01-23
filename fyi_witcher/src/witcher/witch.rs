/*!
# FYI Witcher: Witch
*/

use std::{
	convert::TryFrom,
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
	dev: u64,
	ino: u64,
}

impl TryFrom<&PathBuf> for Witch {
	type Error = bool;
	fn try_from(src: &PathBuf) -> Result<Self, Self::Error> {
		std::fs::metadata(src)
			.map(|meta| Self {
				dir: meta.is_dir(),
				dev: meta.dev(),
				ino: meta.ino(),
			})
			.map_err(|_| false)
	}
}

impl Eq for Witch {}

impl Hash for Witch {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write_u64(self.dev);
		state.write_u64(self.ino);
	}
}

impl PartialEq for Witch {
	fn eq(&self, other: &Self) -> bool {
		self.dev == other.dev &&
		self.ino == other.ino
	}
}

impl Witch {
	/// # Is Directory?
	pub(super) const fn is_dir(&self) -> bool { self.dir }
}
