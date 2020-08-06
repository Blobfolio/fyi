/*!
# Benchmark: `fyi_progress::utility`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_progress::utility;



fn inflect(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::utility");
	let singular = black_box("agendum");
	let plural = black_box("agenda");

	for num in [0_u64, 1_u64, 500_u64, 1000_u64, 86400_u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"inflect({}, \"agendum\", \"agenda\")",
				num
			)),
			num,
			|b, &num| {
				b.iter(||
					utility::inflect(num, singular, plural)
				);
			}
		);
	}
}

fn secs_chunks(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::utility");

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
	inflect,
	secs_chunks,
);
criterion_main!(benches);
