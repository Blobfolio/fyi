/*!
# Benchmark: `fyi_msg::buffer`
*/

use brunch::{
	Bench,
	benches,
};
use fyi_msg::{
	MsgBuffer,
	MsgBuffer3,
};

/// # Test data.
fn test_data() -> MsgBuffer<MsgBuffer3> {
	MsgBuffer::<MsgBuffer3>::from_raw_parts(
		"access.conf.5.gz___________deb-control.5.gz___________deb-triggers.5.gz__________gitignore.5.gz_____________locale.gen.5.gz____________pear.conf.5.gz_____________term.5.gz
adduser.conf.5.gz__________deb-extra-override.5.gz____deluser.conf.5.gz__________gitmodules.5.gz____________login.defs.5.gz____________rsyncd.conf.5.gz___________terminal-colors.d.5.gz
adjtime_config.5.gz________deb-old.5.gz_______________devscripts.conf.5.gz_______gitrepository-layout.5.gz__magic.5.gz_________________scr_dump.5.gz______________terminfo.5.gz".as_bytes().to_vec(),
		[
			0, 16,
			27, 43,
			57, 71,
		]
	)
}

benches!(
	Bench::new("fyi_msg::MsgBuffer", "extend(+6)")
		.with(|| {
			let mut msg = test_data();
			msg.extend(1, b" LOL.");
			msg
		}),

	Bench::new("fyi_msg::MsgBuffer", "replace(bigger)")
		.with(|| {
			let mut msg = test_data();
			msg.replace(1, b"Hello, my name is Bob.");
			msg
		}),

	Bench::new("fyi_msg::MsgBuffer", "replace(smaller)")
		.with(|| {
			let mut msg = test_data();
			msg.replace(1, b"!!");
			msg
		}),

	Bench::new("fyi_msg::MsgBuffer", "truncate(0)")
		.with(|| {
			let mut msg = test_data();
			msg.truncate(1, 0);
			msg
		})
);
