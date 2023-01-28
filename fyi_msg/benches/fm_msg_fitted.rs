/*!
# Benchmark: `fyi_msg::Msg` Fitted
*/

use brunch::{
	Bench,
	benches,
};
use fyi_msg::Msg;

const TEXT: &str = "This is an example message.";

benches!(
	Bench::new("fyi_msg::Msg::error()::fitted(20)")
		.run_seeded(Msg::error(TEXT), |v| v.fitted(20).len()),

	Bench::new("fyi_msg::Msg::error()::fitted(40)")
		.run_seeded(Msg::error(TEXT), |v| v.fitted(40).len()),

	Bench::new("fyi_msg::Msg::error()::fitted(50)")
		.run_seeded(Msg::error(TEXT), |v| v.fitted(50).len())
);
