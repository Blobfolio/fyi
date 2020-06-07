/*!
# Benchmark: `fyi_progress::nice_elapsed`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_progress::NiceElapsed;



fn from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::NiceElapsed");

	for secs in [1, 50, 100, 2121, 37732, 428390].iter() {
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
