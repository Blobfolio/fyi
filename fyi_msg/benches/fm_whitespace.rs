/*!
# Benchmark: `fyi_msg::whitespace`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



fn whitespace(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::whitespace");

	for spaces in [0, 4, 50, 100, 250].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{}", spaces)),
			spaces,
			|b, &spaces| {
				b.iter(||
					fyi_msg::whitespace(spaces)
				);
			}
		);
	}
}

criterion_group!(
	benches,
	whitespace,
);
criterion_main!(benches);
