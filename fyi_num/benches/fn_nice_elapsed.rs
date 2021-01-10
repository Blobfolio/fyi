/*!
# Benchmark: `fyi_num::nice_elapsed`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_num::NiceElapsed;



fn from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::NiceElapsed");
	group.sample_size(30);

	for secs in [1_u32, 50, 100, 2121, 37732, 428390].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"from::<u32>::({})",
				secs
			)),
			secs,
			|b, &secs| {
				b.iter(|| {
					let _ = black_box(NiceElapsed::from(secs));
				});
			}
		);
	}

	group.finish();
}

fn hms(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceElapsed");
	group.sample_size(30);

	for secs in [10_u32, 113_u32, 10502_u32].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"hms({})",
				secs,
			)),
			secs,
			|b, &secs| {
				b.iter(|| {
					let _ = black_box(NiceElapsed::hms(secs));
				});
			}
		);
	}

	group.finish();
}

criterion_group!(
	benches,
	from,
	hms,
);
criterion_main!(benches);
