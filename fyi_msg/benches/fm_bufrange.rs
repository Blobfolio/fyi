/*!
# Benchmark: `fyi_msg::buf_range`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::{
	BufRange,
	replace_buf_range,
	utility::vec_resize_at,
};



fn b_vec_resize_at(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg");

	fn data() -> Vec<u8> {
		"access.conf.5.gz___________deb-control.5.gz___________deb-triggers.5.gz__________gitignore.5.gz_____________locale.gen.5.gz____________pear.conf.5.gz_____________term.5.gz
adduser.conf.5.gz__________deb-extra-override.5.gz____deluser.conf.5.gz__________gitmodules.5.gz____________login.defs.5.gz____________rsyncd.conf.5.gz___________terminal-colors.d.5.gz
adjtime_config.5.gz________deb-old.5.gz_______________devscripts.conf.5.gz_______gitrepository-layout.5.gz__magic.5.gz_________________scr_dump.5.gz______________terminfo.5.gz".as_bytes().to_vec()
	}

	group.bench_function("vec_resize_at()", move |b| {
		b.iter_with_setup(||
			data(), |mut v|
			vec_resize_at(&mut v, 50, 50)
		)
	});

	group.finish();
}

fn b_replace_buf_range(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg");

	fn data() -> (Vec<u8>, Vec<BufRange>) {
		(
			"access.conf.5.gz___________deb-control.5.gz___________deb-triggers.5.gz__________gitignore.5.gz_____________locale.gen.5.gz____________pear.conf.5.gz_____________term.5.gz
adduser.conf.5.gz__________deb-extra-override.5.gz____deluser.conf.5.gz__________gitmodules.5.gz____________login.defs.5.gz____________rsyncd.conf.5.gz___________terminal-colors.d.5.gz
adjtime_config.5.gz________deb-old.5.gz_______________devscripts.conf.5.gz_______gitrepository-layout.5.gz__magic.5.gz_________________scr_dump.5.gz______________terminfo.5.gz".as_bytes().to_vec(),
			vec![BufRange::new(0, 16), BufRange::new(27, 43), BufRange::new(57, 71)]
		)
	}

	group.bench_function("replace_buf_range(mid-bigger)", move |b| {
		b.iter_with_setup(||
			data(),
			|(mut v, mut br)|
			replace_buf_range(&mut v, &mut br, 1, b"Hello, my name is Bob.")
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	b_replace_buf_range,
	b_vec_resize_at,
);
criterion_main!(benches);
