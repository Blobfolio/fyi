/*!
# Benchmark: `fyi_msg::Msg`
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_msg::{
	Msg,
	MsgKind,
};
use std::time::Duration;

const TEXT: &str = "This is an example message.";

benches!(
	Bench::new("fyi_msg::Msg", "default()")
		.timed(Duration::from_secs(2))
		.with(|| Msg::default()),

	Bench::new("fyi_msg::Msg", "plain()")
		.timed(Duration::from_secs(2))
		.with(|| Msg::plain(TEXT)),

	Bench::new("fyi_msg::Msg", "custom(Prefix, 199)")
		.timed(Duration::from_secs(2))
		.with(|| Msg::custom("Prefix", 199, TEXT)),

	Bench::new("fyi_msg::Msg", "error()")
		.timed(Duration::from_secs(2))
		.with(|| Msg::error(TEXT)),

	Bench::new("fyi_msg::Msg", "new(Error)")
		.timed(Duration::from_secs(2))
		.with(|| Msg::new(MsgKind::Error, TEXT)),

	Bench::new("fyi_msg::MsgKind", "into_msg()")
		.timed(Duration::from_secs(2))
		.with(|| MsgKind::Error.into_msg(TEXT))
);
