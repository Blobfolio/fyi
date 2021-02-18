/*!
# Benchmark: `fyi_msg::Msg` Timestamps
*/

use brunch::{
	Bench,
	benches,
};
use fyi_msg::Msg;
use std::time::Duration;

const TEXT: &str = "This is an example message.";

#[cfg(feature = "timestamps")]
benches!(
	Bench::new("fyi_msg::Msg::default()", "with_timestamp(true)")
		.timed(Duration::from_secs(2))
		.with_setup(Msg::default(), |v| v.with_timestamp(true)),

	Bench::new("fyi_msg::Msg::plain()", "with_timestamp(true)")
		.timed(Duration::from_secs(2))
		.with_setup(Msg::plain(TEXT), |v| v.with_timestamp(true)),

	Bench::new("fyi_msg::Msg::error()", "with_timestamp(true)")
		.timed(Duration::from_secs(2))
		.with_setup(Msg::error(TEXT), |v| v.with_timestamp(true))
);

#[cfg(not(feature = "timestamps"))]
fn main() {
	Msg::error("This bench requires the 'timestamps' feature.").die(1);
}
