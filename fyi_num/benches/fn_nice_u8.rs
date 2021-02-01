/*!
# Benchmark: `fyi_num::nice_u8`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_num::NiceU8;



fn from_u8(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceU8");
	group.sample_size(30);

	for ints in [0_u8, 18_u8, 101_u8, u8::MAX].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u8>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NiceU8::from(ints)).as_str();
				});
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	from_u8,
);
criterion_main!(benches);
