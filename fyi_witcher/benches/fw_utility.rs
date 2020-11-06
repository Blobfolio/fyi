/*!
# Benchmark: `fyi_witcher::utility`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::utility;



fn fitted_range(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");
	group.sample_size(30);

	for txt in [
		"Hello World",
		"\x1b[1;31mHello\x1b[0m World",
		"Björk Guðmundsdóttir",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"fitted_range({}, 20)",
				txt,
			)),
			txt.as_bytes(),
			|b, txt| {
				b.iter(||
					utility::fitted_range(txt, 20)
				);
			}
		);
	}

	group.finish();
}

fn hms(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");
	group.sample_size(30);

	for secs in [10_u32, 113_u32, 10502_u32].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"hms_u32({})",
				secs,
			)),
			secs,
			|b, &secs| {
				b.iter(||
					utility::hms_u32(secs)
				);
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	fitted_range,
	hms,
);
criterion_main!(benches);
