/*!
# FYI Witcher Example: Walk and Count

This simply finds all files under /usr/share and reports the total.
*/

use fyi_witcher::Witcher;



/// Do it.
fn main() {
	let len: usize = Witcher::default()
		.with_path("/usr/share")
		.build()
		.len();

	println!("There are {} files in /usr/share.", len);
}
