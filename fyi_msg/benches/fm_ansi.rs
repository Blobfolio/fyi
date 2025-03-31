/*!
# Benchmark: `fyi_msg::ansi`
*/

use brunch::{
	Bench,
	benches,
};

use fyi_msg::{
	ansi::NoAnsi,
	Msg,
};

const PLAIN: &str = "This is an example message.           ";
const ANSI: &str = "This is an \x1b[1;91mexample\x1b[0m message.";

benches!(
	Bench::new("fyi_msg::NoAnsi(Plain)")
		.run(|| NoAnsi::<char, _>::new(PLAIN.chars()).collect::<String>()),

	Bench::new("fyi_msg::NoAnsi(Formatted)")
		.run(|| NoAnsi::<char, _>::new(ANSI.chars()).collect::<String>()),

	Bench::spacer(),

	Bench::new("fyi_msg::Msg::error(Plain)::without_ansi")
		.run(|| Msg::error(PLAIN).without_ansi()),

	Bench::new("fyi_msg::Msg::error(Formatted)::without_ansi")
		.run(|| Msg::error(ANSI).without_ansi()),
);
