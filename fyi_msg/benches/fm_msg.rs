/*!
# Benchmark: `fyi_msg::Msg`
*/

use brunch::{
	Bench,
	benches,
};
use fyi_msg::{
	Msg,
	MsgKind,
};

const TEXT: &str = "This is an example message.";

benches!(
	Bench::new("fyi_msg::Msg::default()")
		.run(Msg::default),

	Bench::new("fyi_msg::Msg::plain()")
		.run(|| Msg::plain(TEXT)),

	Bench::new("fyi_msg::Msg::custom(Prefix, 199)")
		.run(|| Msg::custom("Prefix", 199, TEXT)),

	Bench::new("fyi_msg::Msg::error()")
		.run(|| Msg::error(TEXT)),

	Bench::new("fyi_msg::Msg::new(Error)")
		.run(|| Msg::new(MsgKind::Error, TEXT)),

	Bench::new("fyi_msg::MsgKind::into_msg()")
		.run(|| MsgKind::Error.into_msg(TEXT)),
);
