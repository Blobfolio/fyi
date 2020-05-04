/*!
# Benchmark: `fyi_msg::traits::HumanBytes`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::HumanBytes;



fn human_bytes(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::HumanBytes");

	for bytes in [999_u64, 12003_u64, 499288372_u64, 99389382145_u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"{}.human_bytes()",
				bytes
			)),
			bytes,
			|b, &bytes| {
				b.iter(||
					bytes.human_bytes()
				);
			}
		);
	}
}

criterion_group!(
	benches,
	human_bytes,
);
criterion_main!(benches);
