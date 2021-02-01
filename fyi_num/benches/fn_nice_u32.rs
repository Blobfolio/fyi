/*!
# Benchmark: `fyi_num::nice_u32`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_num::NiceU32;



fn from_u32(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceU32");
	group.sample_size(30);

	for ints in [
		0_u32,
		100_020_u32,
		6_330_004_u32,
		57_444_000_u32,
		777_804_132_u32,
		u32::MAX,
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u32>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NiceU32::from(ints)).as_str();
				});
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	from_u32,
);
criterion_main!(benches);
