/*!
# Benchmark: `fyi_msg::fitted`
*/

use brunch::{
	Bench,
	benches,
};

use fyi_msg::{
	fit_to_width,
	length_width,
	width,
};

const PLAIN: &str = "This is an example message.";
const PLAIN_ANSI: &str = "This \x1b[1mis\x1b[0m an example message.";
const UNICODE: &str = "This world has a Björk too.";
const UNICODE_ANSI: &str = "This world has a \x1b[1mBjörk\x1b[0m too.";

benches!(
	Bench::new("fyi_msg::width(ASCII)")
		.run(|| width(PLAIN)),

	Bench::new("fyi_msg::width(ASCII + ANSI)")
		.run(|| width(PLAIN_ANSI)),

	Bench::new("fyi_msg::width(UNICODE)")
		.run(|| width(UNICODE)),

	Bench::new("fyi_msg::width(UNICODE + ANSI)")
		.run(|| width(UNICODE_ANSI)),

	Bench::spacer(),

	Bench::new("fyi_msg::length_width(UNICODE, 22)")
		.run(|| length_width(UNICODE, 22)),

	Bench::new("fyi_msg::length_width(UNICODE + ANSI, 22)")
		.run(|| length_width(UNICODE_ANSI, 22)),

	Bench::spacer(),

	Bench::new("fyi_msg::fit_to_width(UNICODE, 22)")
		.run(|| fit_to_width(UNICODE, 22)),

	Bench::new("fyi_msg::fit_to_width(UNICODE + ANSI, 22)")
		.run(|| fit_to_width(UNICODE_ANSI, 22)),
);
