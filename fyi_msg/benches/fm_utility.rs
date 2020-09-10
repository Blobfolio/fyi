/*!
# Benchmark: `fyi_msg::utility`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::utility;



fn ansi_code_bold(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");

	for color in [1, 50, 100].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"ansi_code_bold({})",
				color
			)),
			color,
			|b, &color| {
				b.iter(|| utility::ansi_code_bold(color));
			}
		);
	}

	group.finish();
}

fn time_format_dd(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");

	for num in [0, 10, 100].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("time_format_dd({})", num)),
			num,
			|b, &num| {
				b.iter(|| utility::time_format_dd(num));
			}
		);
	}

	group.finish();
}

fn vec_resize_at(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");

	fn data() -> Vec<u8> {
		"access.conf.5.gz___________deb-control.5.gz___________deb-triggers.5.gz__________gitignore.5.gz_____________locale.gen.5.gz____________pear.conf.5.gz_____________term.5.gz
adduser.conf.5.gz__________deb-extra-override.5.gz____deluser.conf.5.gz__________gitmodules.5.gz____________login.defs.5.gz____________rsyncd.conf.5.gz___________terminal-colors.d.5.gz
adjtime_config.5.gz________deb-old.5.gz_______________devscripts.conf.5.gz_______gitrepository-layout.5.gz__magic.5.gz_________________scr_dump.5.gz______________terminfo.5.gz".as_bytes().to_vec()
	}

	group.bench_function("vec_resize_at(50/50)", move |b| {
		b.iter_with_setup(||
			data(), |mut v|
			utility::vec_resize_at(&mut v, 50, 50)
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	ansi_code_bold,
	time_format_dd,
	vec_resize_at,
);
criterion_main!(benches);
