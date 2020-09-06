/*!
# Benchmark: `fyi_witcher::nice_elapsed`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::NiceElapsed;



fn from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::NiceElapsed");

	for secs in [1_u32, 50, 100, 2121, 37732, 428390].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"from({})",
				secs
			)),
			secs,
			|b, &secs| { b.iter(|| NiceElapsed::from(secs)); }
		);
	}

	group.finish();
}

criterion_group!(
	benches,
	from,
);
criterion_main!(benches);
