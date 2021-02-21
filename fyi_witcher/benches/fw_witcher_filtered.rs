/*!
# Benchmark: `fyi_witcher::Witcher` (Filtered)
*/

use brunch::{
	Bench,
	benches,
};
use fyi_witcher::Witcher;
use std::{
	os::unix::ffi::OsStrExt,
	path::PathBuf,
};

/// # Filter Callback.
fn cb(path: &PathBuf) -> bool {
	path.extension()
		.map_or(
			false,
			|e| e.as_bytes().eq_ignore_ascii_case(b"gz")
		)
}

#[cfg(feature = "regexp")]
benches!(
	Bench::new("fyi_witcher::Witcher", "with_filter(.gz)")
		.with(|| Witcher::default().with_filter(cb).with_path("/usr/share/man").build()),

	Bench::new("fyi_witcher::Witcher", "with_regex(.gz)")
		.with(|| Witcher::default().with_regex(r"(?i).+\.gz$").with_path("/usr/share/man").build())
);

#[cfg(not(feature = "regexp"))]
benches!(
	Bench::new("fyi_witcher::Witcher", "with_filter(.gz)")
		.with(|| Witcher::default().with_filter(cb).with_path("/usr/share/man").build())
);
