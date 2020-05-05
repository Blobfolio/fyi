/*!
# Benchmark: `fyi_msg::traits::WhiteSpace`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::WhiteSpace;



fn whitespace(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::WhiteSpace");

	for spaces in [0, 4, 50, 100, 250].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"<[u8]>::whitespace({})",
				spaces
			)),
			spaces,
			|b, &spaces| {
				b.iter(||
					<[u8]>::whitespace(spaces)
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
