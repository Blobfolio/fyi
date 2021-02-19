/*!
# FYI Witcher: Filtered Find
*/

/// Do it.
fn main() {
	// Search for gzipped MAN pages.
	let len: usize = fyi_witcher::Witcher::default()
		.with_ext(b".gz")
		.with_path("/usr/share/man")
		.build()
		.len();

	println!("There are {} .gz files in /usr/share.", len);
}
