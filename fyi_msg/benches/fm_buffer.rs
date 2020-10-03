/*!
# Benchmark: `fyi_msg::toc`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::MsgBuffer3;


fn test_data() -> MsgBuffer3 {
	unsafe {
		MsgBuffer3::from_raw_parts(
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
}

fn from_raw_parts(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuffer");
	group.sample_size(50);

	group.bench_function("from_raw_parts(...)", move |b| {
		b.iter(||
			unsafe { MsgBuffer3::from_raw_parts(
				black_box(vec![97, 99, 99, 101, 115, 115, 46, 99, 111, 110, 102, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 100, 101, 98, 45, 99, 111, 110, 116, 114, 111, 108, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 100, 101, 98, 45, 116, 114, 105, 103, 103, 101, 114, 115, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 103, 105, 116, 105, 103, 110, 111, 114, 101, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 108, 111, 99, 97, 108, 101, 46, 103, 101, 110, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 112, 101, 97, 114, 46, 99, 111, 110, 102, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 116, 101, 114, 109, 46, 53, 46, 103, 122, 10, 97, 100, 100, 117, 115, 101, 114, 46, 99, 111, 110, 102, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 100, 101, 98, 45, 101, 120, 116, 114, 97, 45, 111, 118, 101, 114, 114, 105, 100, 101, 46, 53, 46, 103, 122, 95, 95, 95, 95, 100, 101, 108, 117, 115, 101, 114, 46, 99, 111, 110, 102, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 103, 105, 116, 109, 111, 100, 117, 108, 101, 115, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 108, 111, 103, 105, 110, 46, 100, 101, 102, 115, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 114, 115, 121, 110, 99, 100, 46, 99, 111, 110, 102, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 116, 101, 114, 109, 105, 110, 97, 108, 45, 99, 111, 108, 111, 114, 115, 46, 100, 46, 53, 46, 103, 122, 10, 97, 100, 106, 116, 105, 109, 101, 95, 99, 111, 110, 102, 105, 103, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 100, 101, 98, 45, 111, 108, 100, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 100, 101, 118, 115, 99, 114, 105, 112, 116, 115, 46, 99, 111, 110, 102, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 103, 105, 116, 114, 101, 112, 111, 115, 105, 116, 111, 114, 121, 45, 108, 97, 121, 111, 117, 116, 46, 53, 46, 103, 122, 95, 95, 109, 97, 103, 105, 99, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 115, 99, 114, 95, 100, 117, 109, 112, 46, 53, 46, 103, 122, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 95, 116, 101, 114, 109, 105, 110, 102, 111, 46, 53, 46, 103, 122]),
				[
					0, 16,
					27, 43,
					57, 71,
				]
			) }
		)
	});

	group.finish();
}

fn replace(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuffer");
	group.sample_size(30);

	group.bench_function("replace(bigger)", move |b| {
		b.iter_with_setup(||
			test_data(),
			|mut t|
			unsafe { t.replace_unchecked(1, b"Hello, my name is Bob.") }
		)
	});

	group.bench_function("replace(smaller)", move |b| {
		b.iter_with_setup(||
			test_data(),
			|mut t|
			unsafe { t.replace_unchecked(1, b"!!") }
		)
	});

	group.finish();
}

fn start(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuffer");
	group.sample_size(50);

	group.bench_function("start(2)", move |b| {
		b.iter_with_setup(||
			test_data(),
			|t|
			unsafe { t.start_unchecked(2) }
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	from_raw_parts,
	replace,
	start,
);
criterion_main!(benches);
