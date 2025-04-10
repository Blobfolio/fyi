/*!
# Benchmark: `fyi_msg::ansi`
*/

use brunch::{
	Bench,
	benches,
};

use fyi_msg::Msg;

const PLAIN: &str = "This is an example message.         ";
const ANSI: &str = "This is an \x1b[1;91mexample\x1b[0m message.";

benches!(
	Bench::new("fyi_msg::Msg::error(Plain)::without_ansi")
		.run_seeded(Msg::error(PLAIN), |msg| msg.without_ansi()),

	Bench::new("fyi_msg::Msg::error(Formatted)::without_ansi")
		.run_seeded(Msg::error(ANSI), |msg| msg.without_ansi()),
);
