/*!
# Benchmark: `fyi_msg::fitted`
*/

use brunch::{
	Bench,
	benches,
};

#[cfg(feature = "fitted")] use fyi_msg::width;

// "This is an example message."
const PLAIN: &[u8] = &[84, 104, 105, 115, 32, 105, 115, 32, 97, 110, 32, 101, 120, 97, 109, 112, 108, 101, 32, 109, 101, 115, 115, 97, 103, 101, 46];
// "This \x1b[1mis\x1b[0m an example message."
const PLAIN_ANSI: &[u8] = &[84, 104, 105, 115, 32, 27, 91, 49, 109, 105, 115, 27, 91, 48, 109, 32, 97, 110, 32, 101, 120, 97, 109, 112, 108, 101, 32, 109, 101, 115, 115, 97, 103, 101, 46];
// "This world has a Björk too."
const UNICODE: &[u8] = &[84, 104, 105, 115, 32, 119, 111, 114, 108, 100, 32, 104, 97, 115, 32, 97, 32, 66, 106, 195, 182, 114, 107, 32, 116, 111, 111, 46];
// "This world has a \x1b[1mBjörk\x1b[0m too."
const UNICODE_ANSI: &[u8] = &[84, 104, 105, 115, 32, 119, 111, 114, 108, 100, 32, 104, 97, 115, 32, 97, 32, 27, 91, 49, 109, 66, 106, 195, 182, 114, 107, 27, 91, 48, 109, 32, 116, 111, 111, 46];

#[cfg(feature = "fitted")]
benches!(
	Bench::new("fyi_msg::width(ASCII)")
		.run(|| width(PLAIN)),

	Bench::new("fyi_msg::width(ASCII + ANSI)")
		.run(|| width(PLAIN_ANSI)),

	Bench::new("fyi_msg::width(UNICODE)")
		.run(|| width(UNICODE)),

	Bench::new("fyi_msg::width(UNICODE + ANSI)")
		.run(|| width(UNICODE_ANSI))
);

#[cfg(not(feature = "fitted"))]
fn main() {
	Msg::error("This bench requires the 'fitted' feature.").die(1);
}
