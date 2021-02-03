/*!
# Benchmark: `fyi_num::nice_percent`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_num::NicePercent;



fn from_f32(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NicePercent");
	group.sample_size(30);

	for ints in [
		0_f32,
		0.1_f32,
		0.12_f32,
		0.123_f32,
		0.1234_f32,
		0.12345_f32,
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<f32>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NicePercent::from(ints)).as_str();
				});
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	from_f32,
);
criterion_main!(benches);
