/*!
# Benchmark: `fyi_msg::buffer`
*/

use brunch::{
	Bench,
	benches,
};
use fyi_msg::{
	MsgBuffer,
	BUFFER3,
};

/// # Test data.
fn test_data() -> MsgBuffer<BUFFER3> {
	MsgBuffer::<BUFFER3>::from_raw_parts(
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
	Bench::new("fyi_msg::MsgBuffer", "len(2)")
		.with(|| test_data().len(2)),

	Bench::new("fyi_msg::MsgBuffer", "range(2)")
		.with(|| test_data().range(2))
);
