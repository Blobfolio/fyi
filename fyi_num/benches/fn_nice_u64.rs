/*!
# Benchmark: `fyi_num::nice_u64`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_num::NiceU64;



fn from_u64(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceU64");
	group.sample_size(30);

	for ints in [
		0_u64,
		6_489_320_013_u64,
		42_489_320_013_u64,
		1_999_999_999_999_u64,
		u64::MAX,
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u64>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NiceU64::from(ints)).as_str();
				});
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	from_u64,
);
criterion_main!(benches);
