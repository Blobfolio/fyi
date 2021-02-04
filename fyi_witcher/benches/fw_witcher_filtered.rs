/*!
# Benchmark: `fyi_witcher::Witcher` (Filtered)
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_witcher::Witcher;
use std::{
	ffi::OsStr,
	path::PathBuf,
};

/// # Filter Callback.
fn cb(path: &PathBuf) -> bool {
	let bytes: &[u8] = unsafe { &*(path.as_os_str() as *const OsStr as *const [u8]) };
	bytes.len() > 3 && bytes[bytes.len()-3..].eq_ignore_ascii_case(b".gz")
}

#[cfg(feature = "regexp")]
benches!(
	Bench::new("fyi_witcher::Witcher", "with_ext(.gz)")
		.with(|| Witcher::default().with_ext(b".gz").with_path("/usr/share/man").build()),

	Bench::new("fyi_witcher::Witcher", "with_filter(.gz)")
		.with(|| Witcher::default().with_filter(cb).with_path("/usr/share/man").build()),

	Bench::new("fyi_witcher::Witcher", "with_regex(.gz)")
		.with(|| Witcher::default().with_regex(r"(?i).+\.gz$").with_path("/usr/share/man").build())
);

#[cfg(not(feature = "regexp"))]
benches!(
	Bench::new("fyi_witcher::Witcher", "with_ext(.gz)")
		.with(|| Witcher::default().with_ext(b".gz").with_path("/usr/share/man").build()),

	Bench::new("fyi_witcher::Witcher", "with_filter(.gz)")
		.with(|| Witcher::default().with_filter(cb).with_path("/usr/share/man").build())
);
