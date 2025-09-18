/*!
# Benchmark: `fyi_msg::Msg`
*/

use brunch::{
	Bench,
	benches,
};
use fyi_msg::{
	AnsiColor,
	Msg,
	MsgKind,
};

const TEXT: &str = "This is an example message.";

benches!(
	Bench::new("fyi_msg::Msg::default()")
		.run(Msg::default),

	Bench::new("fyi_msg::Msg::from()")
		.run(|| Msg::from(TEXT)),

	Bench::spacer(),

	Bench::new("fyi_msg::Msg::error()")
		.run(|| Msg::error(TEXT)),

	Bench::new("fyi_msg::Msg::new(MsgKind::Error)")
		.run(|| Msg::new(MsgKind::Error, TEXT)),

	Bench::new("fyi_msg::Msg::new(Prefix)")
		.run(|| Msg::new("Prefix", TEXT)),

	Bench::new("fyi_msg::Msg::new((Prefix, 199))")
		.run(|| Msg::new(("Prefix", AnsiColor::Misc199), TEXT)),

	Bench::spacer(),

	Bench::new("fyi_msg::MsgKind::into_msg()")
		.run(|| MsgKind::Error.into_msg(TEXT)),
);
