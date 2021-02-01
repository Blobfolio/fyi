/*!
# Benchmark: `fyi_num::nice_u16`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_num::NiceU16;



fn from_u16(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceU16");
	group.sample_size(30);

	for ints in [0_u16, 18_u16, 101_u16, 1_620_u16, 40_999_u16, u16::MAX].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u16>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NiceU16::from(ints)).as_str();
				});
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	from_u16,
);
criterion_main!(benches);
