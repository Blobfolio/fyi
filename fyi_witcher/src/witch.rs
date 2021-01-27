/*!
# FYI Witcher: Witch
*/

use ahash::AHashSet;
use crate::utility::{
	resolve_dir_entry,
	resolve_path,
};
use rayon::iter::{
	ParallelBridge,
	ParallelDrainRange,
	ParallelIterator,
};
use std::{
    fs::{
    	self,
    	ReadDir,
    },
    path::{
    	Path,
    	PathBuf,
    },
    sync::{
    	Arc,
    	Mutex,
    },
};



/// Helper: Unlock the inner Mutex, handling poisonings inasmuch as is
/// possible.
macro_rules! mutex_ptr {
	($mutex:expr) => (
		$mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
	);
}



/// # Traverse Paths Deeply.
///
/// This works just like [`Witcher`], but without the overhead of filtering.
pub fn witch<P, I>(paths: I) -> Vec<PathBuf>
where P: AsRef<Path>, I: IntoIterator<Item=P> {
	// Parse out seed paths.
	let (mut dirs, files, seen) = {
		let mut files: Vec<PathBuf> = Vec::with_capacity(2048);
		let mut seen: AHashSet<u128> = AHashSet::with_capacity(2048);

		let dirs: Vec<ReadDir> = paths.into_iter()
			.filter_map(|p| resolve_path(PathBuf::from(p.as_ref()), false))
			.filter_map(|(h, is_dir, p)| {
				// A new path.
				if seen.insert(h) {
					// A directory.
					if is_dir {
						return fs::read_dir(p).ok();
					}
					// A file.
					else { files.push(p); }
				}

				None
			})
			.collect();

		(dirs, Arc::from(Mutex::new(files)), Arc::from(Mutex::new(seen)))
	};

	// Read and read and read until we're done!
	while ! dirs.is_empty() {
		// Handle the directories we've found so far in parallel.
		dirs = dirs.par_drain(..)
			.flat_map(ParallelBridge::par_bridge)
			.filter_map(resolve_dir_entry)
			.filter_map(|(h, is_dir, p)|
				// A new path.
				if mutex_ptr!(seen).insert(h) {
					// A directory to look at on the next while.
					if is_dir { fs::read_dir(p).ok() }
					// A file.
					else {
						mutex_ptr!(files).push(p);
						None
					}
				}
				else { None }
			)
			.collect();
	}

	// De-arc, de-mutex, and return!
	Arc::<Mutex<Vec<PathBuf>>>::try_unwrap(files)
		.ok()
		.and_then(|x| x.into_inner().ok())
		.unwrap_or_default()
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_new() {
		let mut abs_dir = fs::canonicalize("tests/assets/").unwrap();
		abs_dir.push("_.txt");
		let abs_p1 = abs_dir.with_file_name("file.txt");
		let abs_p2 = abs_dir.with_file_name("is-executable.sh");
		let abs_perr = abs_dir.with_file_name("foo.bar");

		// Do a non-search search.
		let w1 = witch(&[PathBuf::from("tests/")]);
		assert!(! w1.is_empty());
		assert_eq!(w1.len(), 9);
		assert!(w1.contains(&abs_p1));
		assert!(w1.contains(&abs_p2));
		assert!(! w1.contains(&abs_perr));
	}
}
