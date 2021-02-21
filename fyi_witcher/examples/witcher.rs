/*!
# FYI Witcher: Filtered Find
*/

use std::{
	os::unix::ffi::OsStrExt,
	path::PathBuf,
};

/// Do it.
fn main() {
	// Search for gzipped MAN pages.
	let len: usize = fyi_witcher::Witcher::default()
		.with_filter(|p: &PathBuf| p.extension()
			.map_or(
				false,
				|e| e.as_bytes().eq_ignore_ascii_case(b"gz")
			)
		)
		.with_path("/usr/share/man")
		.build()
		.len();

	println!("There are {} .gz files in /usr/share/man.", len);
}
