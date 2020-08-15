/*!
# Benchmark: `fyi_witcher::utility`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::utility;



fn secs_chunks(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");

	for secs in [10, 113, 10502].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"secs_chunks({})",
				secs,
			)),
			secs,
			|b, &secs| {
				b.iter(||
					utility::secs_chunks(secs)
				);
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	secs_chunks,
);
criterion_main!(benches);
