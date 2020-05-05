/*!
# Benchmark: `fyi_msg::traits::DoubleTime`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::DoubleTime;



fn double_digit_time(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::DoubleTime");

	for secs in [1_u8, 50_u8, 100_u8].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"{}.double_digit_time()",
				secs
			)),
			secs,
			|b, &secs| {
				b.iter(||
					secs.double_digit_time()
				);
			}
		);
	}
}

criterion_group!(
	benches,
	double_digit_time,
);
criterion_main!(benches);
