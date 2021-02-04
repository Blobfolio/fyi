/*!
# Benchmark: `fyi_msg::Msg` Fitted
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_msg::Msg;
use std::time::Duration;

const TEXT: &str = "This is an example message.";

#[cfg(feature = "fitted")]
benches!(
	Bench::new("fyi_msg::Msg::error()", "fitted(20)")
		.timed(Duration::from_secs(2))
		.with_setup(Msg::error(TEXT), |v| v.fitted(20).len()),

	Bench::new("fyi_msg::Msg::error()", "fitted(40)")
		.timed(Duration::from_secs(2))
		.with_setup(Msg::error(TEXT), |v| v.fitted(40).len()),

	Bench::new("fyi_msg::Msg::error()", "fitted(50)")
		.timed(Duration::from_secs(2))
		.with_setup(Msg::error(TEXT), |v| v.fitted(50).len())
);

#[cfg(not(feature = "fitted"))]
fn main() {
	Msg::error("This bench requires the 'fitted' feature.").die(1);
}
