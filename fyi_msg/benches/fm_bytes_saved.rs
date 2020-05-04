/*!
# Benchmark: `fyi_msg::traits::BytesSaved`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::BytesSaved;



fn bytes_saved(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::BytesSaved");

	for pair in [
		(0_usize, 0_usize),
		(1000_usize, 1000_usize),
		(1000_usize, 500_usize),
	].iter() {
		let name: String = {
			let (a, b) = pair;
			format!(
				"{}.bytes_saved({})",
				a, b
			)
		};

		group.bench_with_input(
			BenchmarkId::from_parameter(&name),
			pair,
			|b, &(before, after)| {
				b.iter(||
					before.bytes_saved(after)
				);
			}
		);
	}
}

criterion_group!(
	benches,
	bytes_saved,
);
criterion_main!(benches);
